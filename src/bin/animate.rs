use std::{io::{self, Write}, collections::VecDeque};

use plotter::{camera::Camera, geometries::gaussian::Gaussian, integrate::implicit_euler, uv2xy::reproject};
use plotter::resolution::Resolution;
use plotter::polyline::Polyline2;

use rand::{distributions::Distribution, rngs::ThreadRng};
use nalgebra_glm::{Vec2, Vec3, look_at, Vec4, perspective};
use rand_distr::{StandardNormal, Uniform};
use tiny_skia::{Pixmap, PathBuilder, Paint, Stroke, Transform, Color};

fn sample_vec2<D: Distribution<f32>>(distribution: &D, rng: &mut ThreadRng) -> Vec2 {
    Vec2::new(
        distribution.sample(rng),
        distribution.sample(rng),
    )
}

struct Particle {
    position: Vec2,
    velocity: Vec2,
}

fn setup_gaussian(resolution: &Resolution) -> (Gaussian, Camera) {
    let eye = Vec3::new(-2.6, -2.6, -1.5);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.4), &Vec3::new(0.0, 0.0, 1.0));
    let near = 0.1;
    let far = 10.0;
    let projection = perspective(resolution.aspect_ratio(), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(0.0, 0.0, resolution.width as f32, resolution.height as f32);
    let camera = Camera { projection, model, viewport};

    let geometry = Gaussian::new();
    (geometry, camera)
}

fn draw_polyline(pixmap: &mut Pixmap, polyline: Polyline2, paint: &Paint, stroke: &Stroke) {
    let mut pb = PathBuilder::new();
    for (index, point) in polyline.points.iter().enumerate() {
        if index == 0 {
            pb.move_to(point.x, point.y);
        } else {
            pb.line_to(point.x, point.y);
        }
    }
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn main() -> io::Result<()> {
    let resolution = Resolution::new(720, 720);
    let (geometry, camera) = setup_gaussian(&resolution);
    let near = 0.1;
    let far = 10.0;

    let mut output = io::stdout().lock();

    let mut rng = rand::thread_rng();
    let distribution = StandardNormal {};

    let positions: Vec<_> = (0..1024)
        .map(|_| 2.0 * sample_vec2(&distribution, &mut rng))
        //.filter(|position| position.magnitude_squared() > 0.3*0.3)
        .collect();
    //let semicircle = Uniform::new(0.0 + 0.2, TAU - 0.2);
    //let positions: Vec<_> = (0..256).map(|_| sample_vec2(&semicircle, &mut rng)).collect();
    let mut particles: Vec<_> = positions.iter().map(|p| Particle {
        position: Vec2::new(p.x, p.y),
        velocity: Vec2::new(1.0, 0.0),
        //velocity:  1.0 / field.at(p).magnitude_squared() * field.at(p),
        //velocity: 0.1 * sample_vec2(&distribution, &mut rng),
    }).collect();
    let mut traces: Vec<VecDeque<Vec2>> = particles.iter().map(|_| VecDeque::new()).collect();
    let trace_length = 24;
    let fps = 30.0;
    let dt = 0.4 / fps;
    for frame in 0..256 {
        let mut polylines = Vec::new();
        for (index, particle) in particles.iter_mut().enumerate() {
            // take integration step
            (particle.position, particle.velocity) = implicit_euler(&geometry, &particle.position, &particle.velocity, dt);

            traces[index].push_back(particle.position);
            if traces[index].len() > trace_length {
                traces[index].pop_front();
            }
        }

        // draw traces
        if frame < trace_length {
            continue;
        }
        for particle_trace in &traces {
            let uv_polyline = Polyline2 { points: particle_trace.iter().step_by(2).copied().collect() };
            let polyline = reproject(&uv_polyline, &geometry, &camera, (0, 0, resolution.width as i32, resolution.height as i32), near, far);
            polylines.extend(polyline);
        }

        // render to pixmap
        let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
        
        let color = Color::from_rgba8(255, 180, 220, 0xff);
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;

        let mut stroke = Stroke::default();
        stroke.width = 2.0;

        for polyline in polylines {
            draw_polyline(&mut pixmap, polyline, &paint, &stroke);
        }
        //pixmap.save_png("image.png")?;
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
