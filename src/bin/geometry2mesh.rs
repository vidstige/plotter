use std::{
    f32::consts::TAU,
    fs::File,
    io::{self, BufWriter, Write},
};

use nalgebra_glm::{Vec2, Vec3};
use plotter::{
    geometries::{heightmap::Heightmap, hole::Hole},
    mesh2::Mesh2,
    mesh3::{FaceVertex, Mesh3},
};

const OUTPUT_PATH: &str = "hole-heightmap.obj";
const INNER_RADIUS: f32 = 0.35355338;
const OUTER_RADIUS: f32 = 3.0;
const RADIAL_STEPS: usize = 32;
const ANGULAR_STEPS: usize = 96;

fn build_annulus() -> Mesh2 {
    let mut vertices = Vec::new();
    let mut quads = Vec::new();

    for ring in 0..=RADIAL_STEPS {
        let t = ring as f32 / RADIAL_STEPS as f32;
        let radius = INNER_RADIUS + t * (OUTER_RADIUS - INNER_RADIUS);
        for segment in 0..ANGULAR_STEPS {
            let angle = TAU * segment as f32 / ANGULAR_STEPS as f32;
            let uv = Vec2::new(radius * angle.cos(), radius * angle.sin());
            vertices.push(uv);
        }
    }

    for ring in 0..RADIAL_STEPS {
        let base = ring * ANGULAR_STEPS;
        let next = (ring + 1) * ANGULAR_STEPS;
        for segment in 0..ANGULAR_STEPS {
            let a = base + segment;
            let b = base + (segment + 1) % ANGULAR_STEPS;
            let c = next + segment;
            let d = next + (segment + 1) % ANGULAR_STEPS;
            quads.push([a, c, d, b]);
        }
    }

    Mesh2 { vertices, quads }
}

fn sample_heightmap(mesh: Mesh2, heightmap: &impl Heightmap) -> Mesh3 {
    let vertices = mesh
        .vertices
        .into_iter()
        .map(|uv| Vec3::new(uv.x, uv.y, heightmap.z(&uv)))
        .collect();
    let faces = mesh
        .quads
        .into_iter()
        .map(|quad| quad.into_iter().map(|vertex| FaceVertex { vertex, normal: None }).collect())
        .collect();
    Mesh3 { vertices, normals: Vec::new(), faces }
}

fn write_obj(path: &str, mesh: &Mesh3) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    for vertex in &mesh.vertices {
        writeln!(writer, "v {} {} {}", vertex.x, vertex.y, vertex.z)?;
    }
    for face in &mesh.faces {
        write!(writer, "f")?;
        for vertex in face {
            write!(writer, " {}", vertex.vertex + 1)?;
        }
        writeln!(writer)?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let hole = Hole::new();
    let uv_mesh = build_annulus();
    let mesh = sample_heightmap(uv_mesh, &hole);
    write_obj(OUTPUT_PATH, &mesh)?;
    println!("wrote: {OUTPUT_PATH}");
    println!("saved {OUTPUT_PATH} with {} vertices and {} faces", mesh.vertices.len(), mesh.faces.len());
    Ok(())
}
