use std::{io, ops::AddAssign};

use nalgebra_glm::{cross2d, look_at, perspective, project, Vec2, Vec3, Vec4};
use plotter::{fields::Spiral, geometries::hole::Hole, geometry::Geometry, integrate::{euler, implicit_euler}, paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE}, polyline::Polyline2, raytracer::{backproject, trace}};
use rand_distr::{Distribution, Normal};

fn contains(view_box: &ViewBox, point: &Vec2) -> bool {
    let (x, y, w, h) = view_box;
    point.x > *x as f32 && point.y > *y as f32 && point.x < (x + w) as f32 && point.y < (y + h) as f32
}

struct Particle {
    position: Vec2,
    velocity: Vec2,
}

fn main() -> io::Result<()> {
    // set up paper
    let mut paper = Paper::new(A4_LANDSCAPE, 0.5);
    let area = pad(paper.view_box, 20);

    // set up pseudo random generator
    let mut rng = rand::thread_rng();
    let distribution = Normal::new(0.0, 1.0).unwrap();

    // set up uv-field
    let field = Spiral::new(Vec2::zeros());

    // set up 3D geometry
    let geometry = Hole::new();

    // set up 3D camera
    let eye = Vec3::new(-2.5, -2.5, -1.5);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    let near = 0.1;
    let far = 10.0;
    let projection = perspective(viewbox_aspect(paper.view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);

    let n = 64;
    for i in 0..n {
        let mut polyline = Polyline2::new();

        /*let position = Vec2::new(
            distribution.sample(&mut rng) as f32,
            distribution.sample(&mut rng) as f32,
        );

        let mut particle = Particle {
            position,
            velocity: field.at(&position),
        };*/

        let size = 2.5;
        let mut particle = Particle {
            position: Vec2::new(-size, size * 2.0 * (i as f32 / n as f32 - 0.5)),
            velocity: Vec2::new(1.0, 0.0),
        };

        for _ in 0..50 {
            // evaluate surface at x, y
            let world = geometry.evaluate(&particle.position);
            // project world cordinate into screen cordinate
            let screen = project(&world, &model, &projection, viewport);

            if contains(&area, &screen.xy()) {
                polyline.add(screen.xy());

                /*// back project and ray trace to find occlusions
                let ray = backproject(&screen.xy(), &model, &projection, viewport);
                if let Some(intersection) = trace(&ray, &geometry, near, far) {
                    let traced_screen = project(&intersection, &model, &projection, viewport);
                    // handle occlusions
                    if screen.z - traced_screen.z < 0.0001 {
                        polyline.add(screen.xy());
                    }
                }*/
            }

            // step forward
            let dt = 0.1;
            (particle.position, particle.velocity) = implicit_euler(&geometry, &particle.position, &particle.velocity, dt);
            //(particle.position, particle.velocity) = euler(&geometry, &particle.position, &particle.velocity, dt);
            //particle.position += particle.velocity * dt;
        }
        paper.add(polyline);
    }

    paper.optimize();
    paper.save("output.svg")?;

    Ok(())
}