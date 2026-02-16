use std::io::{self, ErrorKind, Write};

use nalgebra_glm::{cross, dot, identity, look_at, perspective, Mat4x4, Vec2, Vec3, Vec4};
use plotter::audio_sync::AudioAnalysis;
use plotter::camera::Camera;
use plotter::fields::Spiral;
use plotter::geometries::hole::Hole;
use plotter::geometry::{DifferentiableGeometry, Geometry};
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

const FRAME_COUNT: usize = 1024;
const FPS: f32 = 30.0;
const CAMERA_SWITCH_BEATS: usize = 1;
const TRACE_COUNT: usize = 240;
const TRACE_LENGTH: usize = 18;
const TRACE_STEP: f32 = 0.06;
const FLOW_SPEED: f32 = 0.10;
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

#[derive(Copy, Clone)]
enum CameraStyle {
    Edge,
    Follow,
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

fn polar_to_vec3(radius: f32, angle: f32, z: f32) -> Vec3 {
    Vec3::new(radius * angle.cos(), radius * angle.sin(), z)
}

fn choose_camera_style(scene_key: u64) -> CameraStyle {
    let mut rng = seeded_rng(scene_key ^ 0x68F6_2B44_17C0_DA93);
    if rng.gen_bool(0.5) {
        CameraStyle::Edge
    } else {
        CameraStyle::Follow
    }
}

fn random_edge_eye_point(key: u64) -> Vec3 {
    let mut rng = seeded_rng(key ^ 0xE61A_01F5_42D0_A117);
    let radius = rng.gen_range(2.4..3.2);
    let angle = sample_circle_angle(&mut rng);
    let height = rng.gen_range(-2.2..-1.1);
    polar_to_vec3(radius, angle, height)
}

fn random_edge_target_point(key: u64) -> Vec3 {
    let mut rng = seeded_rng(key ^ 0xA9C7_2E11_D0E4_4D21);
    let radius = rng.gen_range(0.05..0.30);
    let angle = sample_circle_angle(&mut rng);
    let height = rng.gen_range(1.0..1.5);
    polar_to_vec3(radius, angle, height)
}

fn edge_segment(scene_key: u64) -> CameraSegment {
    let mut rng = seeded_rng(scene_key ^ 0x47AA_BF0E_3E8C_91D3);
    let direction = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
    let eye_center = random_edge_eye_point(scene_key);
    let eye_radius = (eye_center.x * eye_center.x + eye_center.y * eye_center.y).sqrt();
    let eye_angle0 = eye_center.y.atan2(eye_center.x);
    let eye_angle_delta = direction * rng.gen_range(0.10..0.28);
    let eye_z0 = eye_center.z;
    let eye_z1 = eye_z0 + rng.gen_range(-0.08..0.08);
    let eye_from = polar_to_vec3(eye_radius, eye_angle0, eye_z0);
    let eye_to = polar_to_vec3(eye_radius, eye_angle0 + eye_angle_delta, eye_z1);

    let target_center = random_edge_target_point(scene_key);
    let target_radius = (target_center.x * target_center.x + target_center.y * target_center.y).sqrt();
    let target_angle_offset = rng.gen_range(-0.22..0.22);
    let target_z0 = target_center.z;
    let target_z1 = target_z0 + rng.gen_range(-0.05..0.05);
    let target_from = polar_to_vec3(target_radius, eye_angle0 + target_angle_offset, target_z0);
    let target_to = polar_to_vec3(
        target_radius,
        eye_angle0 + eye_angle_delta + target_angle_offset,
        target_z1,
    );

    CameraSegment {
        eye_from,
        eye_to,
        target_from,
        target_to,
    }
}

fn follow_along_segment(scene_key: u64) -> CameraSegment {
    let mut rng = seeded_rng(scene_key ^ 0x9327_9A11_2B4F_E55C);
    let direction = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
    const FOLLOW_ANGLE_DELTA_MIN: f32 = 0.24;
    const FOLLOW_ANGLE_DELTA_MAX: f32 = 0.44;
    const FOLLOW_EYE_RADIUS_MIN: f32 = 1.6;
    const FOLLOW_EYE_RADIUS_MAX: f32 = 2.2;
    const FOLLOW_EYE_Z: f32 = -0.15;
    const FOLLOW_LOOK_DISTANCE_MIN: f32 = 1.8;
    const FOLLOW_LOOK_DISTANCE_MAX: f32 = 2.5;
    const FOLLOW_DOWNWARD_WEIGHT: f32 = 0.55;

    let center = Vec2::new(0.0, 0.0);
    let eye_dir0 = sample_on_circle(&mut rng);
    let eye_angle_delta = direction * rng.gen_range(FOLLOW_ANGLE_DELTA_MIN..FOLLOW_ANGLE_DELTA_MAX);
    let eye_dir1 = rotate_around_center(&eye_dir0, &center, eye_angle_delta);

    let eye_radius = rng.gen_range(FOLLOW_EYE_RADIUS_MIN..FOLLOW_EYE_RADIUS_MAX);
    let eye_z0 = FOLLOW_EYE_Z;
    let eye_z1 = FOLLOW_EYE_Z;
    let eye_from = Vec3::new(eye_radius * eye_dir0.x, eye_radius * eye_dir0.y, eye_z0);
    let eye_to = Vec3::new(eye_radius * eye_dir1.x, eye_radius * eye_dir1.y, eye_z1);

    let look_distance = rng.gen_range(FOLLOW_LOOK_DISTANCE_MIN..FOLLOW_LOOK_DISTANCE_MAX);
    let target_from = tangential_target_from_eye(
        eye_from,
        direction,
        look_distance,
        FOLLOW_DOWNWARD_WEIGHT,
    );
    let target_to =
        tangential_target_from_eye(eye_to, direction, look_distance, FOLLOW_DOWNWARD_WEIGHT);

    CameraSegment {
        eye_from,
        eye_to,
        target_from,
        target_to,
    }
}

fn camera_segment(segment: usize) -> CameraSegment {
    let scene_key = segment as u64;
    match choose_camera_style(scene_key) {
        CameraStyle::Edge => edge_segment(scene_key),
        CameraStyle::Follow => follow_along_segment(scene_key),
    }
}

fn surface_normal(geometry: &impl DifferentiableGeometry, uv: &Vec2) -> Vec3 {
    let du = geometry.du().evaluate(&uv);
    let dv = geometry.dv().evaluate(&uv);
    let normal = cross(&du, &dv);
    if normal.magnitude_squared() < 1.0e-6 {
        return Vec3::new(0.0, 0.0, 1.0);
    }

    let mut up = normal.normalize();
    if up.z < 0.0 {
        up = -up;
    }
    up
}

fn camera_segment_from_beats(time: f32, beats: &[f32]) -> usize {
    let beat_count = beats.partition_point(|beat| *beat <= time);
    beat_count / CAMERA_SWITCH_BEATS
}

fn camera_segment_start_time(segment: usize, beats: &[f32]) -> f32 {
    if segment == 0 {
        return 0.0;
    }
    let index = segment * CAMERA_SWITCH_BEATS - 1;
    beats
        .get(index)
        .copied()
        .unwrap_or_else(|| beats.last().copied().unwrap_or(0.0))
}

fn camera_segment_end_time(segment: usize, beats: &[f32], start: f32) -> f32 {
    let index = (segment + 1) * CAMERA_SWITCH_BEATS - 1;
    beats.get(index).copied().unwrap_or(start + 2.0)
}

fn camera_at(time: f32, geometry: &impl DifferentiableGeometry, beats: &[f32]) -> Mat4x4 {
    let time = time.max(0.0);
    let segment = camera_segment_from_beats(time, beats);
    let start = camera_segment_start_time(segment, beats);
    let end = camera_segment_end_time(segment, beats, start);
    let duration = (end - start).max(1.0e-4);
    let t = ((time - start) / duration).clamp(0.0, 1.0);
    let path = camera_segment(segment);
    let eye = lerp_vec3(path.eye_from, path.eye_to, t);
    let target = slerp_vec3(path.target_from, path.target_to, t);
    let mut uv = target.xy();
    // Avoid the singularity exactly at the center of the hole.
    let min_r = 0.12_f32;
    if uv.magnitude_squared() < min_r * min_r {
        uv = Vec2::new(min_r, 0.0);
    }
    let up = surface_normal(geometry, &uv);

    look_at(&eye, &target, &up)
}

fn sample_vec2<D: Distribution<f32>>(distribution: &D, rng: &mut StdRng) -> Vec2 {
    Vec2::new(distribution.sample(rng), distribution.sample(rng))
}

fn sample_on_circle(rng: &mut StdRng) -> Vec2 {
    let distribution = StandardNormal {};
    loop {
        let p = sample_vec2(&distribution, rng);
        if p.magnitude_squared() > 1.0e-6 {
            return p.normalize();
        }
    }
}

fn sample_circle_angle(rng: &mut StdRng) -> f32 {
    let p = sample_on_circle(rng);
    p.y.atan2(p.x)
}

fn tangential_target_from_eye(
    eye: Vec3,
    direction: f32,
    distance: f32,
    downward_weight: f32,
) -> Vec3 {
    let up = Vec3::new(0.0, 0.0, 1.0);
    let radial = Vec3::new(eye.x, eye.y, 0.0);
    let tangent = direction * cross(&up, &radial);
    if tangent.norm() > 1.0e-6 && radial.norm() > 1.0e-6 {
        // In this scene setup, looking "down" toward the surface means +Z.
        let forward = (tangent.normalize() + downward_weight * up).normalize();
        eye + distance * forward
    } else {
        eye + distance * Vec3::new(1.0, 0.0, 0.2).normalize()
    }
}

fn generate_start_positions() -> Vec<Vec2> {
    let mut rng = StdRng::seed_from_u64(20260214);
    let distribution = StandardNormal {};
    let mut positions = Vec::with_capacity(TRACE_COUNT);
    while positions.len() < TRACE_COUNT {
        let p = 1.65 * sample_vec2(&distribution, &mut rng);
        let r2 = p.magnitude_squared();
        if r2 > 0.55 * 0.55 && r2 < 3.2 * 3.2 {
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
    audio: &AudioAnalysis,
    field: &Spiral,
    base_positions: &[Vec2],
    camera: &mut Camera,
    theme: &Theme<'_>,
) {
    camera.model = camera_at(time, geometry, audio.beats());

    // Keep line seeds static for now (disable flow-based advection).
    let moved_positions: Vec<_> = base_positions.to_vec();
    // let moved_positions: Vec<_> = base_positions
    //     .iter()
    //     .map(|p| rotate_around_center(p, &Vec2::new(0.0, 0.0), FLOW_SPEED * time))
    //     .collect();
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
    let audio = AudioAnalysis::load_dat_file("every_breath_you_take.dat")?;
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
            &audio,
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
            &audio,
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
