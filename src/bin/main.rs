use std::{f32::consts::TAU, io};

use nalgebra_glm::{look_at, perspective, Mat2x2, Vec2, Vec3, Vec4};
use plotter::{camera::Camera, fields::cross2, geometries::{gaussian::Gaussian, torus::Torus}, geometry::{DifferentiableGeometry}, gridlines::generate_grid, integrate::euler, paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE}, polyline::Polyline2, uv2xy::reproject};
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};


fn simulate(
    geometry: &impl DifferentiableGeometry,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    dt: f32,
    n: usize,
) -> Vec<Polyline2> {
    let mut uv_polylines = Vec::new();
    for (position, velocity) in positions.iter().zip(velocities.iter()) {
        let mut position = position.clone();
        let mut velocity = velocity.clone();
        let mut uv_polyline = Polyline2::new();
        for _ in 0..n {
            uv_polyline.add(position);
            (position, velocity) = euler(geometry, &position, &velocity, dt);
        }
        uv_polylines.push(uv_polyline);
    }
    uv_polylines
}

fn setup_torus(view_box: ViewBox, area: ViewBox, rng: &mut impl Rng) -> (Torus, Camera, Vec<Polyline2>) {
    let geometry = Torus::new(0.5, 1.0);

    let eye = Vec3::new(-2.2, -2.2, -1.2);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.5), &Vec3::new(0.0, 0.0, 1.0));

    let near = 0.1;
    let far = 4.0;
    let projection = perspective(viewbox_aspect(view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let camera = Camera { projection, model, viewport };

    let a: f32 = 0.0;
    let rotation = Mat2x2::new(
        a.cos(), -a.sin(),
        a.sin(), a.cos(),
    );

    let uv_polylines: Vec<_> = generate_grid((0.0, TAU), (0.0, TAU), 48, 128)
        .iter()
        .map(|line| line.transform(&rotation))
        .collect();

    (geometry, camera, uv_polylines)
}

fn sample_vec2<D: Distribution<f32>>(distribution: &D, rng: &mut impl Rng) -> Vec2 {
    Vec2::new(
        distribution.sample(rng),
        distribution.sample(rng),
    )
}

fn setup_gaussian(view_box: ViewBox, area: ViewBox, rng: &mut impl Rng) -> (Gaussian, Camera, Vec<Polyline2>) {
    let geometry = Gaussian;

    let eye = Vec3::new(-1.8, -1.8, -1.2);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 1.0), &Vec3::new(0.0, 0.0, 1.0));

    let near = 0.1;
    let far = 4.0;
    let projection = perspective(viewbox_aspect(view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let camera = Camera { projection, model, viewport };

    // set up initial positions and velocities
    let distribution = Normal::new(0.0, 1.0).unwrap();
    let n = 100;
    let positions: Vec<_> = (0..n)
        .map(|i| TAU * i as f32 / n as f32)
        .map(|a| {
            let r = 2.0;
            Vec2::new(r * a.cos(), r * a.sin())
        })
        .collect();
    let velocities: Vec<_> = positions
        .iter()
        .map(|p| 0.5 * cross2(*p).normalize() - p)
        .map(|p| p + 0.125 * sample_vec2(&distribution, rng))
        .collect();

    let uv_polylines = simulate(&geometry, positions, velocities, 0.05, 32);

    (geometry, camera, uv_polylines)
}

fn main() -> io::Result<()> {
    // set up paper
    let mut paper = Paper::new(A4_LANDSCAPE, 0.5);
    let area = pad(paper.view_box, 8);

    // set up pseudo random generator
    let mut rng = rand::rngs::StdRng::seed_from_u64(17);

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

    let (geometry, camera, uv_polylines) = setup_torus(paper.view_box, area, &mut rng);
    //let (geometry, camera, uv_polylines) = setup_gaussian(paper.view_box, area, &mut rng);

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