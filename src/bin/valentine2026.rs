use std::io::{self, ErrorKind, Write};

use nalgebra_glm::{dot, identity, look_at, perspective, Mat4x4, Vec2, Vec3, Vec4};
use plotter::camera::Camera;
use plotter::fields::Spiral;
use plotter::geometries::hole::Hole;
use plotter::polyline::Polyline2;
use plotter::resolution::Resolution;
use plotter::skia_utils::draw_polylines;
use plotter::uv2xy::reproject;
use rand::distributions::Distribution;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::StandardNormal;
use tiny_skia::{Color, Paint, Pixmap, Stroke, Transform};

const FRAME_COUNT: usize = 256;
const FPS: f32 = 30.0;
const CAMERA_SWITCH_SECONDS: f32 = 2.0;
const TRACE_COUNT: usize = 240;
const TRACE_LENGTH: usize = 18;
const TRACE_STEP: f32 = 0.06;
const FLOW_SPEED: f32 = 0.25;
const NEAR: f32 = 0.1;
const FAR: f32 = 10.0;

struct Theme<'a> {
    paint: Paint<'a>,
    stroke: Stroke,
    background: Color,
}

#[derive(Copy, Clone)]
struct CameraSegment {
    eye_from: Vec3,
    eye_to: Vec3,
    target_from: Vec3,
    target_to: Vec3,
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, message.into())
}

fn parse_time_arg(raw: &str) -> io::Result<f32> {
    raw.parse::<f32>()
        .map_err(|_| invalid_input(format!("invalid time value: {raw}")))
}

fn parse_args() -> io::Result<Option<f32>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Ok(None);
    }

    match args.as_slice() {
        [flag, value] if flag == "-t" || flag == "--time" => parse_time_arg(value).map(Some),
        [value] if value.starts_with("--time=") => {
            let Some(raw) = value.strip_prefix("--time=") else {
                return Err(invalid_input("failed to parse --time argument"));
            };
            parse_time_arg(raw).map(Some)
        }
        _ => Err(invalid_input("usage: valentine2026 [-t|--time <seconds>]")),
    }
}

fn black_and_white<'a>() -> Theme<'a> {
    let mut paint = Paint::default();
    paint.set_color(Color::BLACK);
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = 1.8;

    Theme {
        paint,
        stroke,
        background: Color::WHITE,
    }
}

fn initialize_camera(resolution: &Resolution) -> Camera {
    let projection = perspective(resolution.aspect_ratio(), 45.0_f32.to_radians(), NEAR, FAR);
    let viewport = Vec4::new(0.0, 0.0, resolution.width as f32, resolution.height as f32);
    Camera {
        projection,
        model: identity(),
        viewport,
    }
}

fn smoothstep(t: f32) -> f32 {
    let clamped = t.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}

fn lerp_vec3(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    from * (1.0 - t) + to * t
}

fn slerp_vec3(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    let from_len = from.norm();
    let to_len = to.norm();
    if from_len < 1.0e-5 || to_len < 1.0e-5 {
        return lerp_vec3(from, to, t);
    }

    let from_dir = from / from_len;
    let to_dir = to / to_len;
    let cos_theta = dot(&from_dir, &to_dir).clamp(-1.0, 1.0);

    let direction = if cos_theta > 0.9995 || cos_theta < -0.9995 {
        lerp_vec3(from_dir, to_dir, t).normalize()
    } else {
        let theta = cos_theta.acos();
        let inv_sin_theta = 1.0 / theta.sin();
        let w0 = ((1.0 - t) * theta).sin() * inv_sin_theta;
        let w1 = (t * theta).sin() * inv_sin_theta;
        from_dir * w0 + to_dir * w1
    };

    let radius = from_len * (1.0 - t) + to_len * t;
    direction * radius
}

fn seeded_rng(key: u64) -> StdRng {
    let seed = key
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0xD1B5_4A32_D192_ED03);
    StdRng::seed_from_u64(seed)
}

fn random_eye_point(key: u64) -> Vec3 {
    let mut rng = seeded_rng(key ^ 0xE61A_01F5_42D0_A117);
    let radius = rng.gen_range(2.4..3.2);
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let height = rng.gen_range(-2.2..-1.1);
    Vec3::new(radius * angle.cos(), radius * angle.sin(), height)
}

fn random_target_point(key: u64) -> Vec3 {
    let mut rng = seeded_rng(key ^ 0xA9C7_2E11_D0E4_4D21);
    let radius = rng.gen_range(0.05..0.30);
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let height = rng.gen_range(1.0..1.5);
    Vec3::new(radius * angle.cos(), radius * angle.sin(), height)
}

fn camera_segment(segment: usize) -> CameraSegment {
    let scene_key = segment as u64;
    let mut rng = seeded_rng(scene_key ^ 0x47AA_BF0E_3E8C_91D3);

    let eye_center = random_eye_point(scene_key);
    let eye_delta = Vec3::new(
        rng.gen_range(-0.32..0.32),
        rng.gen_range(-0.32..0.32),
        rng.gen_range(-0.18..0.18),
    );

    let target_center = random_target_point(scene_key);
    let target_delta = Vec3::new(
        rng.gen_range(-0.06..0.06),
        rng.gen_range(-0.06..0.06),
        rng.gen_range(-0.05..0.05),
    );

    CameraSegment {
        eye_from: eye_center - 0.5 * eye_delta,
        eye_to: eye_center + 0.5 * eye_delta,
        target_from: target_center - 0.5 * target_delta,
        target_to: target_center + 0.5 * target_delta,
    }
}

fn camera_at(time: f32) -> Mat4x4 {
    let segment_time = (time / CAMERA_SWITCH_SECONDS).max(0.0);
    let segment = segment_time.floor() as usize;
    let t = smoothstep(segment_time.fract());
    let path = camera_segment(segment);
    let eye = lerp_vec3(path.eye_from, path.eye_to, t);
    let target = slerp_vec3(path.target_from, path.target_to, t);

    look_at(&eye, &target, &Vec3::new(0.0, 0.0, 1.0))
}

fn sample_vec2<D: Distribution<f32>>(distribution: &D, rng: &mut StdRng) -> Vec2 {
    Vec2::new(distribution.sample(rng), distribution.sample(rng))
}

fn generate_start_positions() -> Vec<Vec2> {
    let mut rng = StdRng::seed_from_u64(20260214);
    let distribution = StandardNormal {};
    let mut positions = Vec::with_capacity(TRACE_COUNT);
    while positions.len() < TRACE_COUNT {
        let p = 1.2 * sample_vec2(&distribution, &mut rng);
        let r2 = p.magnitude_squared();
        if r2 > 0.55 * 0.55 && r2 < 2.4 * 2.4 {
            positions.push(p);
        }
    }
    positions
}

fn rotate_around_center(point: &Vec2, center: &Vec2, angle: f32) -> Vec2 {
    let rel = point - center;
    let rotated = Vec2::new(
        rel.x * angle.cos() - rel.y * angle.sin(),
        rel.x * angle.sin() + rel.y * angle.cos(),
    );
    rotated + center
}

fn trace_field(field: &Spiral, position: &Vec2, n: usize, dt: f32) -> Polyline2 {
    let mut points = Vec::with_capacity(n);
    let mut p = *position;
    for _ in 0..n {
        points.push(p);
        p += field.at(&p) * dt;
    }
    Polyline2 { points }
}

fn render_frame(
    pixmap: &mut Pixmap,
    resolution: &Resolution,
    time: f32,
    geometry: &Hole,
    field: &Spiral,
    base_positions: &[Vec2],
    camera: &mut Camera,
    theme: &Theme<'_>,
) {
    camera.model = camera_at(time);

    let moved_positions: Vec<_> = base_positions
        .iter()
        .map(|p| rotate_around_center(p, &Vec2::new(0.0, 0.0), FLOW_SPEED * time))
        .collect();
    let uv_polylines: Vec<_> = moved_positions
        .iter()
        .map(|p| trace_field(field, p, TRACE_LENGTH, TRACE_STEP))
        .collect();

    let mut polylines = Vec::new();
    for uv_polyline in &uv_polylines {
        polylines.extend(reproject(
            uv_polyline,
            geometry,
            camera,
            (0, 0, resolution.width as i32, resolution.height as i32),
            NEAR,
            FAR,
        ));
    }

    pixmap.fill(theme.background);
    draw_polylines(
        pixmap,
        &polylines,
        &theme.paint,
        &theme.stroke,
        Transform::identity(),
    );
}

fn main() -> io::Result<()> {
    let time = parse_args()?;
    let resolution = Resolution::new(720, 720);
    let mut camera = initialize_camera(&resolution);
    let geometry = Hole::new();
    let field = Spiral::new(Vec2::new(0.0, 0.0));
    let base_positions = generate_start_positions();
    let theme = black_and_white();
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    let mut output = io::stdout().lock();

    if let Some(time) = time {
        render_frame(
            &mut pixmap,
            &resolution,
            time,
            &geometry,
            &field,
            &base_positions,
            &mut camera,
            &theme,
        );
        output.write_all(pixmap.data())?;
        output.flush()?;
        return Ok(());
    }

    for frame in 0..FRAME_COUNT {
        let time = frame as f32 / FPS;
        render_frame(
            &mut pixmap,
            &resolution,
            time,
            &geometry,
            &field,
            &base_positions,
            &mut camera,
            &theme,
        );
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
