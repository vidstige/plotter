use std::cmp::Ordering;
use std::f32::consts::TAU;
use std::io::{self, ErrorKind, Write};

use nalgebra_glm::{cross, identity, look_at, perspective, Mat4x4, Vec2, Vec3, Vec4};
use plotter::audio_sync::AudioAnalysis;
use plotter::camera::Camera;
use plotter::fields::Spiral;
use plotter::geometries::hole::Hole;
use plotter::geometries::pulse::Pulse;
use plotter::geometries::sum::Sum;
use plotter::lerp::lerp;
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

const FRAME_COUNT: usize = 2048;
const FPS: f32 = 30.0;
const CAMERA_SWITCH_EVENTS: usize = 1;
const BEATS_PER_CAMERA_SWITCH: usize = 4;
const TRACE_COUNT: usize = 240;
const TRACE_LENGTH: usize = 18;
const TRACE_STEP: f32 = 0.06;
const FLOW_SPEED: f32 = 0.10;
const TRACE_INNER_RADIUS: f32 = 0.55;
const TRACE_OUTER_RADIUS: f32 = 4.0;
const TRACE_RADIAL_CDF_SAMPLES: usize = 8192;
const NEAR: f32 = 0.1;
const FAR: f32 = 10.0;
const PULSE_AMPLITUDE: f32 = 0.2;
const PULSE_SIGMA: f32 = 0.8;
const PULSE_SPEED: f32 = 16.0;
const PULSE_LAMBDA: f32 = 0.2;
const PULSE_CYCLES: f32 = 0.4;
const PULSE_BEAT_PHASE_OFFSET: f32 = 2.0 / (PULSE_SIGMA * PULSE_SPEED);

struct Theme<'a> {
    paint: Paint<'a>,
    stroke: Stroke,
    background: Color,
}

#[derive(Copy, Clone)]
enum CameraSegment {
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

fn seeded_rng(key: u64) -> StdRng {
    let seed = key
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0xD1B5_4A32_D192_ED03);
    StdRng::seed_from_u64(seed)
}

fn edge_camera_model_at(scene_key: u64, time: f32, duration: f32) -> Mat4x4 {
    let mut rng = seeded_rng(scene_key ^ 0x47AA_BF0E_3E8C_91D3);
    const EYE_RADIUS_MIN: f32 = 2.0;
    const EYE_RADIUS_MAX: f32 = 3.8;
    const EDGE_EYE_STEP: f32 = 0.5;
    let duration = duration.max(1.0e-4);
    let t = (time / duration).clamp(0.0, 1.0);

    let eye_dir = sample_on_circle(&mut rng);
    let eye_radius = rng.gen_range(EYE_RADIUS_MIN..EYE_RADIUS_MAX);
    let eye_from = Vec3::new(
        eye_radius * eye_dir.x,
        eye_radius * eye_dir.y,
        rng.gen_range(-1.9..-0.2),
    );

    let eye_step_dir = sample_on_circle(&mut rng);
    let eye_to = eye_from + Vec3::new(
        EDGE_EYE_STEP * eye_step_dir.x,
        EDGE_EYE_STEP * eye_step_dir.y,
        0.0,
    );

    let target_from = Vec3::new(
        rng.gen_range(-0.10..0.10),
        rng.gen_range(-0.10..0.10),
        rng.gen_range(0.80..1.00),
    );
    let target_to = Vec3::new(
        rng.gen_range(-0.10..0.10),
        rng.gen_range(-0.30..0.10),
        rng.gen_range(0.80..1.00),
    );

    let eye = lerp(eye_from, eye_to, t);
    let target = lerp(target_from, target_to, t);
    let up = Vec3::new(0.0, 0.0, 1.0);
    look_at(&eye, &target, &up)
}

fn follow_camera_model_at(scene_key: u64, time: f32, duration: f32) -> Mat4x4 {
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
    let duration = duration.max(1.0e-4);
    let t = (time / duration).clamp(0.0, 1.0);

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

    let eye = lerp(eye_from, eye_to, t);
    let target = lerp(target_from, target_to, t);
    let up = Vec3::new(0.0, 0.0, 1.0);
    look_at(&eye, &target, &up)
}

fn camera_segment(segment: usize, allow_follow: bool) -> CameraSegment {
    let scene_key = segment as u64;
    if !allow_follow {
        CameraSegment::Edge
    } else {
        let mut rng = seeded_rng(scene_key ^ 0x68F6_2B44_17C0_DA93);
        if rng.gen_bool(0.5) {
            CameraSegment::Edge
        } else {
            CameraSegment::Follow
        }
    }
}

fn camera_model_at(segment: CameraSegment, scene_key: u64, time: f32, duration: f32) -> Mat4x4 {
    match segment {
        CameraSegment::Edge => edge_camera_model_at(scene_key, time, duration),
        CameraSegment::Follow => follow_camera_model_at(scene_key, time, duration),
    }
}

fn is_beat_time(time: f32, beat_times: &[f32]) -> bool {
    beat_times
        .iter()
        .any(|beat_time| (*beat_time - time).abs() < 1.0e-4)
}

fn build_camera_segments(events: &[f32], beat_times: &[f32]) -> Vec<(f32, CameraSegment)> {
    let mut segments = Vec::new();
    let mut start = 0.0;
    let mut segment_index = 0usize;

    loop {
        let allow_follow = !(segment_index > 0 && is_beat_time(start, beat_times));
        let segment = camera_segment(segment_index, allow_follow);
        segments.push((start, segment));

        let next_start_index = (segment_index + 1) * CAMERA_SWITCH_EVENTS - 1;
        let Some(next_start) = events.get(next_start_index).copied() else {
            break;
        };
        start = next_start;
        segment_index += 1;
    }

    segments
}

fn camera_at(time: f32, camera_segments: &[(f32, CameraSegment)]) -> Mat4x4 {
    let time = time.max(0.0);
    let segment_count = camera_segments.len();
    if segment_count == 0 {
        return camera_model_at(CameraSegment::Edge, 0, time, 2.0);
    }

    let active_segment = camera_segments.partition_point(|(start, _)| *start <= time);
    let segment_index = active_segment.saturating_sub(1);
    let (start, segment) = camera_segments[segment_index];
    let end = camera_segments
        .get(segment_index + 1)
        .map(|(next_start, _)| *next_start)
        .unwrap_or(start + 2.0);
    let duration = (end - start).max(1.0e-4);
    let local_time = (time - start).max(0.0);
    camera_model_at(segment, segment_index as u64, local_time, duration)
}

fn build_camera_events(beat_times: &[f32], claps: &[f32]) -> Vec<f32> {
    let mut events: Vec<f32> = beat_times
        .iter()
        .enumerate()
        .filter(|(index, _)| index % BEATS_PER_CAMERA_SWITCH == 0)
        .map(|(_, beat_time)| *beat_time)
        .filter(|time| time.is_finite())
        .collect();
    events.extend(claps.iter().copied().filter(|time| time.is_finite()));
    events.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    events.dedup_by(|a, b| (*a - *b).abs() < 1.0e-4);
    events
}

fn build_beat_times(raw_beat_times: &[f32]) -> Vec<f32> {
    let mut times: Vec<f32> = raw_beat_times
        .iter()
        .copied()
        .filter(|time| time.is_finite())
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    times.dedup_by(|a, b| (*a - *b).abs() < 1.0e-4);
    times
}

fn pulse_time(time: f32, beat_times: &[f32]) -> f32 {
    if beat_times.is_empty() {
        return time + PULSE_BEAT_PHASE_OFFSET;
    }

    let beat_count = beat_times.partition_point(|beat_time| *beat_time <= time);
    let time_since_beat = if beat_count == 0 {
        // Allow negative local time before the first beat so pulse is pre-rolled into the hit.
        time - beat_times[0]
    } else {
        time - beat_times[beat_count - 1]
    };

    time_since_beat + PULSE_BEAT_PHASE_OFFSET
}

fn geometry_at(time: f32, beat_times: &[f32]) -> Sum<Hole, Pulse> {
    Sum::new(
        Hole::new(),
        Pulse {
            amplitude: PULSE_AMPLITUDE,
            sigma: PULSE_SIGMA,
            c: PULSE_SPEED,
            lambda: PULSE_LAMBDA,
            cycles: PULSE_CYCLES,
            t: pulse_time(time, beat_times),
        },
    )
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

fn hole_radial_weight(radius: f32) -> f32 {
    radius * (1.0 + 4.0 / radius.powi(6)).sqrt()
}

fn build_hole_radial_cdf(inner_radius: f32, outer_radius: f32, samples: usize) -> (Vec<f32>, Vec<f32>) {
    let sample_count = samples.max(2);
    let mut radii = Vec::with_capacity(sample_count);
    let mut cdf = Vec::with_capacity(sample_count);
    let dr = (outer_radius - inner_radius) / (sample_count as f32 - 1.0);

    for i in 0..sample_count {
        let radius = inner_radius + i as f32 * dr;
        radii.push(radius);
    }

    cdf.push(0.0);
    for i in 1..sample_count {
        let r0 = radii[i - 1];
        let r1 = radii[i];
        let w0 = hole_radial_weight(r0);
        let w1 = hole_radial_weight(r1);
        let area = 0.5 * (w0 + w1) * (r1 - r0);
        cdf.push(cdf[i - 1] + area);
    }

    let total = *cdf.last().unwrap_or(&0.0);
    if total > 0.0 {
        for value in &mut cdf {
            *value /= total;
        }
    } else {
        for (i, value) in cdf.iter_mut().enumerate() {
            *value = i as f32 / (sample_count as f32 - 1.0);
        }
    }

    (radii, cdf)
}

fn sample_radius_from_cdf(u: f32, radii: &[f32], cdf: &[f32]) -> f32 {
    if radii.is_empty() || cdf.is_empty() {
        return TRACE_INNER_RADIUS;
    }

    let idx = cdf.partition_point(|value| *value < u);
    if idx == 0 {
        return radii[0];
    }
    if idx >= radii.len() {
        return *radii.last().unwrap_or(&TRACE_OUTER_RADIUS);
    }

    let c0 = cdf[idx - 1];
    let c1 = cdf[idx];
    let r0 = radii[idx - 1];
    let r1 = radii[idx];
    if (c1 - c0).abs() <= 1.0e-8 {
        return r0;
    }
    let t = (u - c0) / (c1 - c0);
    r0 + t * (r1 - r0)
}

fn generate_start_positions() -> Vec<Vec2> {
    let mut rng = StdRng::seed_from_u64(20260214);
    let (radii, cdf) = build_hole_radial_cdf(
        TRACE_INNER_RADIUS,
        TRACE_OUTER_RADIUS,
        TRACE_RADIAL_CDF_SAMPLES,
    );
    let mut positions = Vec::with_capacity(TRACE_COUNT);
    for _ in 0..TRACE_COUNT {
        let radius = sample_radius_from_cdf(rng.gen::<f32>(), &radii, &cdf);
        let angle = rng.gen_range(0.0..TAU);
        positions.push(Vec2::new(radius * angle.cos(), radius * angle.sin()));
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
    geometry: &Sum<Hole, Pulse>,
    field: &Spiral,
    base_positions: &[Vec2],
    camera: &Camera,
    theme: &Theme<'_>,
) {
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
    let beat_times = build_beat_times(audio.beats());
    let camera_events = build_camera_events(&beat_times, audio.onsets());
    let camera_segments = build_camera_segments(&camera_events, &beat_times);
    let resolution = Resolution::new(720, 720);
    let mut camera = initialize_camera(&resolution);
    let field = Spiral::new(Vec2::new(0.0, 0.0));
    let base_positions = generate_start_positions();
    let theme = black_and_white();
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    let mut output = io::stdout().lock();

    if let Some(time) = time {
        camera.model = camera_at(time, &camera_segments);
        let geometry = geometry_at(time, &beat_times);
        render_frame(
            &mut pixmap,
            &resolution,
            &geometry,
            &field,
            &base_positions,
            &camera,
            &theme,
        );
        output.write_all(pixmap.data())?;
        output.flush()?;
        return Ok(());
    }

    for frame in 0..FRAME_COUNT {
        let time = frame as f32 / FPS;
        camera.model = camera_at(time, &camera_segments);
        let geometry = geometry_at(time, &beat_times);
        render_frame(
            &mut pixmap,
            &resolution,
            &geometry,
            &field,
            &base_positions,
            &camera,
            &theme,
        );
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
