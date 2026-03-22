use std::{collections::HashMap, env, io, path::PathBuf};

use nalgebra_glm::{cross, dot, look_at, perspective, Vec2, Vec3, Vec4};
use plotter::{
    camera::Camera,
    duration_extras::format_duration,
    mesh3::Mesh3,
    mesh3_io::load_obj,
    paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE},
    polyline::Polyline2,
    time_estimator::Estimator,
};

fn setup_camera(view_box: ViewBox, area: ViewBox) -> (Camera, Vec3) {
    let eye = Vec3::new(-2.7, -1.8, -2.0);
    let target = Vec3::new(0.0, 0.0, 1.6);
    let model = look_at(&eye, &target, &Vec3::new(0.0, 0.0, 1.0));
    let projection = perspective(viewbox_aspect(view_box), 45.0_f32.to_radians(), 0.1, 4.0);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    (Camera { projection, model, viewport }, eye)
}

fn face_faces_camera(mesh: &Mesh3, face: &[plotter::mesh3::FaceVertex], eye: &Vec3) -> bool {
    let a = mesh.vertices[face[0].vertex];
    let b = mesh.vertices[face[1].vertex];
    let c = mesh.vertices[face[2].vertex];
    let normal = cross(&(b - a), &(c - a));
    let center = face
        .iter()
        .map(|vertex| mesh.vertices[vertex.vertex])
        .fold(Vec3::new(0.0, 0.0, 0.0), |sum, vertex| sum + vertex)
        / face.len() as f32;
    dot(&normal, &(*eye - center)) < 0.0
}

fn project(world: Vec3, camera: &Camera) -> Option<Vec2> {
    let clip = camera.projection * camera.model * Vec4::new(world.x, world.y, world.z, 1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let ndc = clip.xyz() / clip.w;
    if ndc.z < -1.0 || ndc.z > 1.0 {
        return None;
    }
    Some(Vec2::new(
        camera.viewport.x + camera.viewport.z * (ndc.x + 1.0) * 0.5,
        camera.viewport.y + camera.viewport.w * (ndc.y + 1.0) * 0.5,
    ))
}

fn mesh_edges(mesh: &Mesh3, eye: &Vec3) -> HashMap<(usize, usize), (usize, bool)> {
    let mut edges = HashMap::new();
    for face in &mesh.faces {
        let front = face_faces_camera(mesh, face, eye);
        for i in 0..face.len() {
            let a = face[i].vertex;
            let b = face[(i + 1) % face.len()].vertex;
            let key = if a < b { (a, b) } else { (b, a) };
            edges
                .entry(key)
                .and_modify(|edge: &mut (usize, bool)| {
                    edge.0 += 1;
                    edge.1 |= front;
                })
                .or_insert((1, front));
        }
    }
    edges
}

fn edge_polyline(mesh: &Mesh3, edge: (usize, usize), camera: &Camera) -> Option<Polyline2> {
    let a = project(mesh.vertices[edge.0], camera)?;
    let b = project(mesh.vertices[edge.1], camera)?;
    let mut polyline = Polyline2::new();
    polyline.add(a);
    polyline.add(b);
    Some(polyline)
}

struct Args {
    input_path: PathBuf,
    output_path: PathBuf,
}

fn parse_args() -> io::Result<Args> {
    let mut input_path = None;
    let mut output_path = None;
    let mut args = env::args_os().skip(1);

    while let Some(arg) = args.next() {
        if arg == "-o" || arg == "--output" {
            let path = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing path after -o/--output")
            })?;
            output_path = Some(PathBuf::from(path));
            continue;
        }
        if input_path.is_none() {
            input_path = Some(PathBuf::from(arg));
            continue;
        }
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "usage: mesh <input.obj> [-o output.svg]"));
    }

    let input_path =
        input_path.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "usage: mesh <input.obj> [-o output.svg]"))?;
    let output_path = output_path.unwrap_or_else(|| input_path.with_extension("svg"));
    Ok(Args { input_path, output_path })
}

fn main() -> io::Result<()> {
    let args = parse_args()?;
    let mesh = load_obj(&args.input_path)?;
    let mut paper = Paper::new(A4_LANDSCAPE, 0.5);
    let area = pad(paper.view_box, 8);
    let (camera, eye) = setup_camera(paper.view_box, area);

    for (edge, (count, front)) in mesh_edges(&mesh, &eye) {
        if front || count == 1 {
            if let Some(polyline) = edge_polyline(&mesh, edge, &camera) {
                paper.add(polyline);
            }
        }
    }

    paper.optimize();
    let (dl, ml) = paper.length();
    println!("draw: {dl} mm, move: {ml} mm");
    paper.save(args.output_path.to_str().unwrap())?;

    let estimator = Estimator::best();
    let duration = estimator.estimate(&paper, 2000.0, 8000.0);
    println!("Estimated time: {}", format_duration(duration));

    Ok(())
}
