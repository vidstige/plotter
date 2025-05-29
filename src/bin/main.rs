use std::{collections::HashSet, f32::consts::TAU, io, time::Duration};

use nalgebra_glm::{look_at, perspective, vec2, Vec2, Vec3, Vec4};
use plotter::{camera::Camera, fields::cross2, geometries::{gaussian::Gaussian, hole::Hole, torus::Torus}, geometry::{DifferentiableGeometry, Geometry}, gridlines::generate_grid, integrate::euler, mesh2::Mesh2, paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE}, polyline::Polyline2, remeshing::{extract_quad_mesh, initialize_orientation_field, initialize_position_field, optimize_orientation_field, optimize_position_field, QuadMesh}, time_estimator::Estimator, uv2xy::reproject};
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

fn setup_torus(view_box: ViewBox, area: ViewBox, rng: &mut impl Rng) -> (Torus, Camera) {
    let geometry = Torus::new(0.5, 1.0);

    let eye = Vec3::new(-2.2, -2.2, -1.2);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.5), &Vec3::new(0.0, 0.0, 1.0));

    let near = 0.1;
    let far = 4.0;
    let projection = perspective(viewbox_aspect(view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let camera = Camera { projection, model, viewport };

    (geometry, camera)
}

fn sample_vec2<D: Distribution<f32>>(distribution: &D, rng: &mut impl Rng) -> Vec2 {
    Vec2::new(
        distribution.sample(rng),
        distribution.sample(rng),
    )
}

fn setup_gaussian(view_box: ViewBox, area: ViewBox, rng: &mut impl Rng) -> (Gaussian, Camera) {
    let geometry = Gaussian;

    let eye = Vec3::new(-1.8, -1.8, -1.2);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 1.0), &Vec3::new(0.0, 0.0, 1.0));

    let near = 0.1;
    let far = 4.0;
    let projection = perspective(viewbox_aspect(view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let camera = Camera { projection, model, viewport };
    (geometry, camera)
}

fn setup_hole(view_box: ViewBox, area: ViewBox) -> (Hole, Camera, Vec<Polyline2>) {
    let geometry = Hole::new();

    let eye = Vec3::new(-2.7, -1.8, -2.0);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 1.6), &Vec3::new(0.0, 0.0, 1.0));

    let near = 0.1;
    let far = 4.0;
    let projection = perspective(viewbox_aspect(view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let camera = Camera { projection, model, viewport };
    
    let size = 3.0;
    let uv_polylines = generate_grid((-size, size), (-size, size), 32, 256);

    (geometry, camera, uv_polylines)
}

fn simulate_abc(geometry: &impl DifferentiableGeometry, rng: &mut impl Rng) -> Vec<Polyline2> {
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

    simulate(geometry, positions, velocities, 0.05, 32)
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    let hours = minutes / 60;
    let minutes = minutes % 60;
    if hours > 0 {
        return format!("{hours}h {minutes}m {seconds}s");
    }
    format!("{minutes}m {seconds}s")
}

fn debug_field(camera: &Camera, positions: &[Vec3], field: &[Vec3], length: f32) -> Vec<Polyline2> {
    let mut uv_polylines = Vec::new();
    for (position, vector) in positions.iter().zip(field.iter()) {
        let mut uv_polyline = Polyline2::new();
        uv_polyline.add(camera.project(*position).xy());
        uv_polyline.add(camera.project(*position + length * *vector).xy());
        uv_polylines.push(uv_polyline);
    }
    uv_polylines
}

fn to_uvlines(mesh: &Mesh2) -> Vec<Polyline2> {
    let all_edges = mesh.edges();
    let edges: HashSet<_> = all_edges.iter().collect();
    // TODO: Find chains
    edges.iter().map(|edge| {
        let start = mesh.vertices[edge[0]];
        let end = mesh.vertices[edge[1]];
        Polyline2::segment(start, end)
    }).collect()
}

pub fn edges(mesh: &QuadMesh) -> Vec<[usize; 2]> {
    let mut edges = HashSet::new();
    for quad in &mesh.quads {
        for i in 0..quad.len() {
            let next = (i + 1) % 4;
            edges.insert([quad[i], quad[next]]);
        }
    }
    edges.into_iter().collect()
}

fn debug_mesh3(camera: &Camera, mesh: &QuadMesh) -> Vec<Polyline2> {
    let edges = edges(mesh);
    edges.iter().map(|&[a, b]| {
        let start = mesh.vertices[a];
        let end = mesh.vertices[b];
        Polyline2::segment(camera.project(start).xy(), camera.project(end).xy())
    }).collect()
}

fn main() -> io::Result<()> {
    // set up paper
    let mut paper = Paper::new(A4_LANDSCAPE, 0.5);
    let area = pad(paper.view_box, 8);

    // set up pseudo random generator
    let mut rng = rand::rngs::StdRng::seed_from_u64(17);

    let near = 0.1;
    let far = 10.0;

    //let (geometry, camera) = setup_torus(paper.view_box, area, &mut rng);
    let (geometry, camera) = setup_gaussian(paper.view_box, area, &mut rng);
    //let (geometry, camera) = setup_hole(paper.view_box, area);
    
    //let uv_polylines = generate_grid((-2.0, 2.0), (-2.0, 2.0), 64, 256);
    let mesh = Mesh2::from_grid(16, 16, vec2(-2.0, -2.0), vec2(2.0, 2.0));
    let mut orientation_field = initialize_orientation_field(&geometry, &mesh.vertices);
    optimize_orientation_field(&geometry, &mesh, &mut orientation_field, 20);

    let rho = 0.3;
    let mut position_field = initialize_position_field(&geometry, &mesh.vertices, &orientation_field, rho);
    optimize_position_field(&geometry, &mesh, &orientation_field, &mut position_field, rho, 5);

    /*let vertices: Vec<_> = mesh.vertices.iter()
        .map(|v| geometry.evaluate(v))
        .collect();
    let screen_lines = debug_field(&camera, &vertices, &position_field, 0.1);
    for polyline in screen_lines {
        paper.add(polyline);
    }*/
    let quad_mesh = extract_quad_mesh(&geometry, &mesh, &position_field, &orientation_field, rho, rho * 0.3);
    let screen_lines = debug_mesh3(&camera, &quad_mesh);
    for polyline in screen_lines {
        paper.add(polyline);
    }

    /*let uv_polylines = to_uvlines(&mesh);
    for uv_polyline in uv_polylines {
        for polyline in reproject(&uv_polyline, &geometry, &camera, area, near, far) {
            paper.add(polyline);
        }
    }*/
    
    paper.optimize();
    let (dl, ml) = paper.length();
    println!("draw: {dl} mm, move: {ml} mm");
    paper.save("output.svg")?;

    // estimate plotting time
    let estimator = Estimator::best();
    let duration = estimator.estimate(&paper, 2000.0, 8000.0);
    println!("Estimated time: {}", format_duration(duration));

    Ok(())
}