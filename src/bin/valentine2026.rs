use std::io::{self, ErrorKind, Write};

use nalgebra_glm::{identity, look_at, perspective, Mat4x4, Vec2, Vec3, Vec4};
use plotter::camera::Camera;
use plotter::fields::Spiral;
use plotter::geometries::hole::Hole;
use plotter::polyline::Polyline2;
use plotter::resolution::Resolution;
use plotter::skia_utils::draw_polylines;
use plotter::uv2xy::reproject;
use rand::distributions::Distribution;
use rand::rngs::StdRng;
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
struct CameraPath {
    radius: f32,
    base_angle: f32,
    angular_speed: f32,
    z_base: f32,
    z_amplitude: f32,
    z_speed: f32,
    target_radius: f32,
    target_speed: f32,
    target_z: f32,
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

fn camera_paths() -> [CameraPath; 5] {
    [
        CameraPath {
            radius: 2.7,
            base_angle: 0.2,
            angular_speed: 0.10,
            z_base: -1.8,
            z_amplitude: 0.12,
            z_speed: 0.60,
            target_radius: 0.22,
            target_speed: 0.22,
            target_z: 1.25,
        },
        CameraPath {
            radius: 2.4,
            base_angle: 1.4,
            angular_speed: 0.08,
            z_base: -1.4,
            z_amplitude: 0.15,
            z_speed: 0.50,
            target_radius: 0.16,
            target_speed: 0.18,
            target_z: 1.05,
        },
        CameraPath {
            radius: 3.1,
            base_angle: 2.7,
            angular_speed: 0.07,
            z_base: -2.1,
            z_amplitude: 0.08,
            z_speed: 0.42,
            target_radius: 0.20,
            target_speed: 0.14,
            target_z: 1.45,
        },
        CameraPath {
            radius: 2.2,
            base_angle: 4.0,
            angular_speed: 0.12,
            z_base: -1.1,
            z_amplitude: 0.10,
            z_speed: 0.55,
            target_radius: 0.10,
            target_speed: 0.16,
            target_z: 0.95,
        },
        CameraPath {
            radius: 2.9,
            base_angle: 5.1,
            angular_speed: 0.09,
            z_base: -1.9,
            z_amplitude: 0.10,
            z_speed: 0.48,
            target_radius: 0.24,
            target_speed: 0.20,
            target_z: 1.30,
        },
    ]
}

fn camera_at(time: f32) -> Mat4x4 {
    let paths = camera_paths();
    let segment = (time / CAMERA_SWITCH_SECONDS).floor().max(0.0) as usize;
    let path = paths[segment % paths.len()];

    let angle = path.base_angle + path.angular_speed * time;
    let eye = Vec3::new(
        path.radius * angle.cos(),
        path.radius * angle.sin(),
        path.z_base + path.z_amplitude * (path.z_speed * time).sin(),
    );

    let target_angle = path.base_angle * 0.7 + path.target_speed * time;
    let target = Vec3::new(
        path.target_radius * target_angle.cos(),
        path.target_radius * target_angle.sin(),
        path.target_z,
    );

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
