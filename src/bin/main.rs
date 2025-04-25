use std::{f32::consts::TAU, io};

use nalgebra_glm::{cross2d, look_at, perspective, project, Vec2, Vec3, Vec4};
use plotter::{fields::Spiral, geometries::hole::Hole, geometry::Geometry, integrate::verlet, paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE}, polyline::Polyline2, raytracer::{backproject, trace}};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};
use svg::node::element::path::Position;

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
        // random positions
        /*let position = Vec2::new(
            distribution.sample(&mut rng) as f32,
            distribution.sample(&mut rng) as f32,
        );

        let mut particle = Particle {
            position,
            velocity: field.at(&position),
        };*/

        // paralell lines
        let size = 1.5;
        let mut particle = Particle {
            position: Vec2::new(-size, size * 2.0 * (i as f32 / n as f32 - 0.5)),
            velocity: Vec2::new(1.0, 0.0),
        };

        // inward
        /*let theta = TAU * (i as f32 / n as f32);
        let position = 2.0 * Vec2::new(theta.cos(), theta.sin());
        let velocity = -cross2(position) - 2.0 * position;
        let mut particle = Particle {position, velocity};*/

        // integrate
        let mut uv_polyline = Polyline2::new();
        for _ in 0..20 {
            uv_polyline.add(particle.position);
            let dt = 0.1;
            (particle.position, particle.velocity) = verlet(&geometry, &particle.position, &particle.velocity, dt);
        }

        // project & etc
        let mut polyline = Polyline2::new();
        for position in uv_polyline.points {
             // evaluate surface at u, v
             let world = geometry.evaluate(&position);
             // project world cordinate into screen cordinate
             let screen = project(&world, &model, &projection, viewport);

             // clip against screen
             if contains(&area, &screen.xy()) {
                 polyline.add(screen.xy());

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
        paper.add(polyline);
    }

    paper.optimize();
    paper.save("output.svg")?;

    Ok(())
}