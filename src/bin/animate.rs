use std::{ops::{Sub, Add}, io::{self, Write}, collections::VecDeque, f32::consts::TAU};

use plotter::{fields::Spiral, geometry::compute_gamma, iso_surface::IsoSurface, raytracer::{backproject, trace}};
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

// Compute acceleration: a^k = Γ^k_ij v^i v^j
// This is the solution to the geodesic equation
fn acceleration(geometry: &impl Geometry, position: &Vec2, velocity: &Vec2) -> Vec2 {
    let gamma = compute_gamma(geometry, position);
    let mut a = Vec2::zeros();
    let u = velocity.as_slice();
    // tensor sum
    for k in 0..2 {
        for i in 0..2 {
            for j in 0..2 {
                a.as_mut_slice()[k] += -gamma[k][i][j] * u[i] * u[j];
            }
        }
    }
    a
}

struct Particle {
    position: Vec2,
    velocity: Vec2,
}

// step functions
fn euler(geometry: &impl Geometry, position: &Vec2, velocity: &Vec2, dt: f32) -> (Vec2, Vec2) {
    let a = acceleration(geometry, position, &velocity);
    (position + velocity * dt, velocity + a * dt)
}

fn verlet(geometry: &impl Geometry, position: &Vec2, velocity: &Vec2, dt: f32) -> (Vec2, Vec2) {
    let a = acceleration(geometry, position, velocity);
    let new_position = position + velocity * dt + a * (dt * dt * 0.5);
    let new_a = acceleration(geometry, &new_position, &velocity);
    // TODO: acceleration could be stored to save time next frame
    let new_velocity = velocity + (a + new_a) * (dt * 0.5);
    (new_position, new_velocity)
}

// Fixed-point iteration (from ChatGPT)
fn implicit_euler(geometry: &impl Geometry, position: &Vec2, velocity: &Vec2, dt: f32) -> (Vec2, Vec2) {
    // Initial guess using explicit Euler
    let x = *position;
    let v = *velocity;
    let mut x_next = x + dt * v;
    let mut v_next = v;
    for _ in 0..10 {
        let acc = acceleration(geometry, position, &velocity);

        let v_new = v - dt * acc;
        let x_new = x + dt * v_new;

        let dx = x_new - x_next;
        let dv = v_new - v_next;

        x_next = x_new;
        v_next = v_new;
        if dx.norm_squared() + dv.norm_squared() < 1e-9 {
            break;
        }
    }
    (x_next, v_next)
}

fn main() -> io::Result<()> {
    let resolution = Resolution::new(506, 253);

    let mut rng = rand::thread_rng();
    let distribution = StandardNormal {};
    let field = Spiral::new(Vec2::zeros());

    let eye = Vec3::new(-2.5, -2.5, -1.5);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    let near = 0.1;
    let far = 10.0;
    let projection = perspective(resolution.aspect_ratio(), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(0.0, 0.0, resolution.width as f32, resolution.height as f32);

    //let geometry = Hole::new();
    let geometry = Sphere::new();

    //let mut output = File::create(std::path::Path::new("output.raw"))?;
    let mut output = io::stdout().lock();

    //let positions: Vec<_> = (0..256)
    //    .map(|_| sample_vec2(&distribution, &mut rng))
    //    .filter(|position| position.magnitude_squared() > 0.3*0.3)
    //    .collect();
    let semicircle = Uniform::new(0.0 + 0.1, TAU - 0.1);
    let positions: Vec<_> = (0..256).map(|_| sample_vec2(&semicircle, &mut rng)).collect();
    let mut particles: Vec<_> = positions.iter().map(|p| Particle {
        position: Vec2::new(p.x, p.y),
        //velocity: 0.15 / field.at(p).magnitude_squared() * field.at(p),
        velocity: Vec2::new(0.0, 0.1),
    }).collect();
    let mut traces: Vec<VecDeque<Vec3>> = particles.iter().map(|_| VecDeque::new()).collect();
    let fps = 25.0;
    let dt = 1.0 / fps;
    for frame in 0..512 {
        let mut polylines = Vec::new();
        for (index, particle) in particles.iter_mut().enumerate() {
            // euler
            //(particle.position, particle.velocity) = euler(&geometry, &particle.position, &particle.velocity, dt);

            // verlet
            //(particle.position, particle.velocity) = verlet(&geometry, &particle.position, &particle.velocity, dt);

            // implicit euler
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