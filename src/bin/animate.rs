use std::
    io::{self, Write}
;

use plotter::fields::Spiral;
use plotter::polyline::Polyline2;
use plotter::{geometries::hole::Hole, skia_utils::draw_polylines};
use plotter::resolution::Resolution;
use plotter::{
    camera::Camera,
    lerp::lerp,
    uv2xy::reproject,
};

use nalgebra_glm::{identity, look_at, perspective, Mat4x4, Vec2, Vec3, Vec4};
use rand::{distributions::Distribution, rngs::ThreadRng};
use rand_distr::StandardNormal;
use tiny_skia::{Color, Paint, Pixmap, Stroke, Transform};

fn sample_vec2<D: Distribution<f32>>(distribution: &D, rng: &mut ThreadRng) -> Vec2 {
    Vec2::new(distribution.sample(rng), distribution.sample(rng))
}

struct Particle {
    position: Vec2,
    velocity: Vec2,
}

fn initialize_camera(resolution: &Resolution) -> Camera {
    let near = 0.1;
    let far = 10.0;
    let projection = perspective(resolution.aspect_ratio(), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(0.0, 0.0, resolution.width as f32, resolution.height as f32);
    Camera { projection, model: identity(), viewport }
}

fn camera_at(t: f32) -> Mat4x4 {
    let t_ = lerp(0.1, 0.2, t);
    let eye = Vec3::new(2.6 * t_.cos(), 2.6 * t_.sin(), -1.5);
    look_at(&eye, &Vec3::new(0.0, 0.0, 0.4), &Vec3::new(0.0, 0.0, 1.0))
}

fn trace_field(field: &Spiral, position: &Vec2, n: usize, dt: f32) -> Polyline2 {
    let mut points = Vec::new();
    let mut p = position.clone();
    for _ in 0..n {
        points.push(p);
        p += field.at(&p) * dt;
    }
    Polyline2 { points }
}

struct Theme<'a> {
    paint: Paint<'a>,
    stroke: Stroke,
    background: Color,
}

fn black_and_white<'a>() -> Theme<'a> {
    let color = Color::BLACK;
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = 2.0;

    let background = Color::WHITE;
    Theme {paint, stroke, background}
}

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let resolution = Resolution::new(720, 720);
    
    let mut camera = initialize_camera(&resolution);

    /*let mut geometry = Pulse {
        amplitude: 0.2,
        sigma: 0.8,
        c: 16.0,
        cycles: 0.4,
        lambda: 0.2,
        t: 0.0,
    };*/
    let geometry = Hole::new();
    let uv_field = Spiral::new(Vec2::new(0.0, 0.0));

    let near = 0.1;
    let far = 10.0;

    let mut rng = rand::thread_rng();
    let distribution = StandardNormal {};

    // set up color
    let theme = black_and_white();

    let positions: Vec<_> = (0..1024)
        .map(|_| 2.0 * sample_vec2(&distribution, &mut rng))
        .filter(|position| position.magnitude_squared() > 0.3*0.3)
        .collect();
    let trace_length = 16;

    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    let fps = 30.0;
    let dt = 0.4 / fps;
    for frame in 0..256 {
        let t = frame as f32 / 256 as f32;

        camera.model = camera_at(t);

        // uv_polylines
        let uv_polylines: Vec<_> = positions
            .iter()
            .map(|p| trace_field(&uv_field, p, trace_length, 0.1))
            .collect();

        // draw traces
        let mut polylines = Vec::new();
        for uv_polyline in &uv_polylines {
            let polyline = reproject(
                uv_polyline,
                &geometry,
                &camera,
                (0, 0, resolution.width as i32, resolution.height as i32),
                near,
                far,
            );
            polylines.extend(polyline);
        }

        // render to pixmap
        pixmap.fill(theme.background);
        draw_polylines(&mut pixmap, &polylines, &theme.paint, &theme.stroke, Transform::identity());

        //pixmap.save_png("image.png")?;
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
