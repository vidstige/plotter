use std::{io, ops::AddAssign};

use nalgebra_glm::{look_at, perspective, project, Vec2, Vec3, Vec4};
use plotter::{fields::Spiral, geometries::hole::Hole, paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE}, polyline::Polyline2, raytracer::{backproject, trace}};
use rand_distr::{Distribution, Normal};

fn contains(view_box: &ViewBox, point: &Vec2) -> bool {
    let (x, y, w, h) = view_box;
    point.x > *x as f32 && point.y > *y as f32 && point.x < (x + w) as f32 && point.y < (y + h) as f32
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

    for _ in 0..512 {
        let mut polyline = Polyline2::new();

        let mut p = Vec2::new(
            distribution.sample(&mut rng) as f32,
            distribution.sample(&mut rng) as f32,
        );
        for _ in 0..5 {
            // evaluate surface at x, y
            let z = geometry.z(&p);
            let world = Vec3::new(p.x, p.y, z);
            // project world cordinate into screen cordinate
            let screen = project(&world, &model, &projection, viewport);
            
            if contains(&area, &screen.xy()) {
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

            // step forward
            let delta = field.at(&p);
            let norm = delta.norm();
            let step = 0.1;
            p.add_assign(delta.scale(step / norm));
        }
        paper.add(polyline);
    }

    paper.save("output.svg")?;

    Ok(())
}