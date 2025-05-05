use std::{f32::consts::TAU, io};

use nalgebra_glm::{look_at, perspective, Vec2, Vec3, Vec4};
use plotter::{camera::Camera, geometries::torus::Torus, gridlines::generate_grid, paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE}, polyline::Polyline2, uv2xy::reproject};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};

struct Particle {
    position: Vec2,
    velocity: Vec2,
}

fn setup_torus(view_box: ViewBox, area: ViewBox) -> (Torus, Camera, Vec<Polyline2>) {
    let geometry = Torus::new(0.5, 1.0);

    let eye = Vec3::new(-2.2, -2.2, -1.2);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.5), &Vec3::new(0.0, 0.0, 1.0));

    let near = 0.1;
    let far = 4.0;
    let projection = perspective(viewbox_aspect(view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let camera = Camera { projection, model, viewport };

    let uv_polylines = generate_grid((0.0, TAU), (0.0, TAU), 32, 128);

    (geometry, camera, uv_polylines)
}

fn main() -> io::Result<()> {
    // set up paper
    let mut paper = Paper::new(A4_LANDSCAPE, 0.5);
    let area = pad(paper.view_box, 8);

    // set up pseudo random generator
    //let mut rng = rand::thread_rng();
    //let distribution = Normal::new(0.0, 1.0).unwrap();

    // set up uv-field
    //let field = Spiral::new(Vec2::zeros());

    // set up 3D geometry
    //let geometry = Hole::new();
    //let geometry = Gaussian::new();

    // set up 3D camera
    // hole & gaussian view
    //let eye = Vec3::new(-1.8, -1.8, -0.8);
    //let model = look_at(&eye, &Vec3::new(0.0, 0.0, 1.3), &Vec3::new(0.0, 0.0, 1.0));
    // top-view
    //let eye = Vec3::new(0.0, 0.0, -2.5);
    //let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.0), &Vec3::new(0.0, 1.0, 0.0));

    let near = 0.1;
    let far = 10.0;
    /*
    let projection = perspective(viewbox_aspect(paper.view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let camera = Camera { projection, model, viewport };*/

    let (geometry, camera, uv_polylines) = setup_torus(paper.view_box, area);

    for uv_polyline in uv_polylines {
        for polyline in reproject(&uv_polyline, &geometry, &camera, area, near, far) {
            paper.add(polyline);
        }
    }
    
    // integrate
    /*let mut uv_polyline = Polyline2::new();
    for _ in 0..20 {
        uv_polyline.add(particle.position);
        let dt = 0.1;
        (particle.position, particle.velocity) = verlet(&geometry, &particle.position, &particle.velocity, dt);
    }*/
    
    paper.optimize();
    let (dl, ml) = paper.length();
    println!("draw: {dl} mm, move: {ml} mm");
    paper.save("output.svg")?;

    Ok(())
}