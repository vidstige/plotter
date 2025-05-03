use std::{ops::{Sub, Add}, io::{self, Write}, collections::VecDeque, f32::consts::TAU};

use plotter::{fields::Spiral, geometries::gaussian::Gaussian, geometry::{acceleration, compute_gamma}, integrate::implicit_euler, sdf::SDF, raytracer::{backproject, trace}};
use plotter::geometries::{sphere::Sphere, hole::Hole};
use plotter::geometry::Geometry;
use plotter::resolution::Resolution;
use plotter::polyline::Polyline2;

use rand::{distributions::Distribution, rngs::ThreadRng};
use nalgebra_glm::{Vec2, Vec3, look_at, project, Vec4, perspective};
use rand_distr::{StandardNormal, Uniform};
use tiny_skia::{Pixmap, PathBuilder, Paint, Stroke, Transform, Color};

fn contains(resolution: &Resolution, point: &Vec2) -> bool {
    point.x >= 0.0 && point.x < resolution.width as f32 && point.y >= 0.0 && point.y < resolution.height as f32
}

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

fn main() -> io::Result<()> {
    let resolution = Resolution::new(506, 253);

    let mut rng = rand::thread_rng();
    let distribution = StandardNormal {};
    let field = Spiral::new(Vec2::zeros());

    let eye = Vec3::new(-2.5, -2.5, -1.5);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.1), &Vec3::new(0.0, 0.0, 1.0));
    let near = 0.1;
    let far = 10.0;
    let projection = perspective(resolution.aspect_ratio(), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(0.0, 0.0, resolution.width as f32, resolution.height as f32);

    let geometry = Gaussian::new();
    //let geometry = Sphere::new();

    let mut output = io::stdout().lock();

    let positions: Vec<_> = (0..256)
        .map(|_| sample_vec2(&distribution, &mut rng))
        .filter(|position| position.magnitude_squared() > 0.3*0.3)
        .collect();
    //let semicircle = Uniform::new(0.0 + 0.2, TAU - 0.2);
    //let positions: Vec<_> = (0..256).map(|_| sample_vec2(&semicircle, &mut rng)).collect();
    let mut particles: Vec<_> = positions.iter().map(|p| Particle {
        position: Vec2::new(p.x, p.y),
        velocity: Vec2::new(1.0, 0.0),
        //velocity:  1.0 / field.at(p).magnitude_squared() * field.at(p),
        //velocity: 0.1 * sample_vec2(&distribution, &mut rng),
    }).collect();
    let mut traces: Vec<VecDeque<Vec3>> = particles.iter().map(|_| VecDeque::new()).collect();
    let fps = 25.0;
    let dt = 0.2 / fps;
    for frame in 0..512 {
        let mut polylines = Vec::new();
        for (index, particle) in particles.iter_mut().enumerate() {
            // take integration step
            (particle.position, particle.velocity) = implicit_euler(&geometry, &particle.position, &particle.velocity, dt);

            // project world cordinate into screen cordinate
            let world = geometry.evaluate(&particle.position);
            let screen = project(&world, &model, &projection, viewport);

            traces[index].push_back(screen);
            if traces[index].len() > 10 {
                traces[index].pop_front();
            }
        }

        // draw traces
        if frame < 10 {
            continue;
        }
        for particle_trace in &traces {
            let mut polyline = Polyline2::new();
            for screen in particle_trace {
                if contains(&resolution, &screen.xy()) {
                    // back project and ray trace to find occlusions
                    let ray = backproject(&screen.xy(), &model, &projection, viewport);
                    if let Some(intersection) = trace(&ray, &geometry, near, far) {
                        let traced_screen = project(&intersection, &model, &projection, viewport);
                        // handle occlusions
                        if screen.z - traced_screen.z < 0.0001 {
                            polyline.add(screen.xy());
                        }
                    }
                }
            }
            polylines.push(polyline);
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
        //pixmap.save_png("image.png")?;
        output.write_all(pixmap.data())?;
    }

    Ok(())
}