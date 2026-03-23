use std::{
    collections::{HashMap, HashSet},
    env, io,
    path::PathBuf,
};

use nalgebra_glm::{look_at, perspective, Vec2, Vec3, Vec4};
use plotter::{
    camera::Camera,
    duration_extras::format_duration,
    mesh3::Mesh3,
    mesh3_io::load_obj,
    paper::{pad, viewbox_aspect, Paper, ViewBox, A4_LANDSCAPE},
    polyline::Polyline2,
    time_estimator::Estimator,
};

const BARY_EPSILON: f32 = 1e-4;
const DEPTH_EPSILON: f32 = 0.001;
const MIN_SCREEN_LENGTH: f32 = 2.0;
const MAX_EDGE_DEPTH: usize = 8;

#[derive(Clone, Copy)]
struct ProjectedTriangle {
    points: [Vec3; 3],
    min: Vec2,
    max: Vec2,
}

#[derive(Default)]
struct VertexVisibility {
    nearest: Vec<usize>,
}

#[derive(Clone, Copy)]
struct EdgePoint {
    world: Vec3,
    screen: Option<Vec3>,
    visible: bool,
}

fn setup_camera(view_box: ViewBox, area: ViewBox) -> Camera {
    let eye = Vec3::new(-2.7, -1.8, -2.0);
    let target = Vec3::new(0.0, 0.0, 1.6);
    let model = look_at(&eye, &target, &Vec3::new(0.0, 0.0, 1.0));
    let projection = perspective(viewbox_aspect(view_box), 45.0_f32.to_radians(), 0.1, 4.0);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    Camera { projection, model, viewport }
}

fn mesh_edges(mesh: &Mesh3) -> HashSet<(usize, usize)> {
    let mut edges = HashSet::new();
    for face in &mesh.faces {
        for i in 0..face.len() {
            let a = face[i].vertex;
            let b = face[(i + 1) % face.len()].vertex;
            let key = if a < b { (a, b) } else { (b, a) };
            edges.insert(key);
        }
    }
    edges
}

fn triangulate(
    mesh: &Mesh3,
    projected: &[Option<Vec3>],
) -> (Vec<ProjectedTriangle>, HashMap<(usize, usize), Vec<usize>>) {
    let mut triangles = Vec::new();
    let mut edge_triangles = HashMap::new();

    for face in &mesh.faces {
        for i in 1..face.len().saturating_sub(1) {
            let vertices = [face[0].vertex, face[i].vertex, face[i + 1].vertex];
            let [Some(a), Some(b), Some(c)] = [
                projected[vertices[0]],
                projected[vertices[1]],
                projected[vertices[2]],
            ] else {
                continue;
            };
            let points = [a, b, c];
            let triangle = ProjectedTriangle {
                points,
                min: Vec2::new(
                    points[0].x.min(points[1].x).min(points[2].x),
                    points[0].y.min(points[1].y).min(points[2].y),
                ),
                max: Vec2::new(
                    points[0].x.max(points[1].x).max(points[2].x),
                    points[0].y.max(points[1].y).max(points[2].y),
                ),
            };
            let triangle_index = triangles.len();
            triangles.push(triangle);

            for edge in [(vertices[0], vertices[1]), (vertices[1], vertices[2]), (vertices[2], vertices[0])] {
                let key = if edge.0 < edge.1 { edge } else { (edge.1, edge.0) };
                edge_triangles.entry(key).or_insert_with(Vec::new).push(triangle_index);
            }
        }
    }

    (triangles, edge_triangles)
}

fn barycentric(point: Vec2, triangle: &ProjectedTriangle) -> Option<[f32; 3]> {
    let a = triangle.points[0].xy();
    let b = triangle.points[1].xy();
    let c = triangle.points[2].xy();
    let v0 = b - a;
    let v1 = c - a;
    let v2 = point - a;
    let det = v0.x * v1.y - v1.x * v0.y;
    if det.abs() < BARY_EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;
    let u = (v2.x * v1.y - v1.x * v2.y) * inv_det;
    let v = (v0.x * v2.y - v2.x * v0.y) * inv_det;
    let w = 1.0 - u - v;
    if u < -BARY_EPSILON || v < -BARY_EPSILON || w < -BARY_EPSILON {
        return None;
    }
    Some([w, u, v])
}

fn project_vertices(mesh: &Mesh3, camera: &Camera) -> Vec<Option<Vec3>> {
    mesh.vertices
        .iter()
        .map(|vertex| {
            let screen = camera.project(*vertex);
            (screen.z > 0.0).then_some(screen)
        })
        .collect()
}

fn vertex_visibility(vertex: Vec3, triangles: &[ProjectedTriangle]) -> VertexVisibility {
    let mut nearest_depth = f32::INFINITY;
    let mut nearest = Vec::new();

    for (index, triangle) in triangles.iter().enumerate() {
        if vertex.x < triangle.min.x
            || vertex.x > triangle.max.x
            || vertex.y < triangle.min.y
            || vertex.y > triangle.max.y
        {
            continue;
        }
        let Some(weights) = barycentric(vertex.xy(), triangle) else {
            continue;
        };
        let depth = weights[0] * triangle.points[0].z
            + weights[1] * triangle.points[1].z
            + weights[2] * triangle.points[2].z;

        if depth < nearest_depth - DEPTH_EPSILON {
            nearest_depth = depth;
            nearest.clear();
            nearest.push(index);
        } else if (depth - nearest_depth).abs() <= DEPTH_EPSILON {
            nearest.push(index);
        }
    }

    VertexVisibility { nearest }
}

fn visible_vertices(projected: &[Option<Vec3>], triangles: &[ProjectedTriangle]) -> Vec<VertexVisibility> {
    projected
        .iter()
        .map(|vertex| vertex.map(|point| vertex_visibility(point, triangles)).unwrap_or_default())
        .collect()
}

fn point_visible(point: Vec3, triangles: &[ProjectedTriangle], adjacent: &[usize]) -> bool {
    let nearest = vertex_visibility(point, triangles);
    adjacent.iter().any(|triangle| nearest.nearest.contains(triangle))
}

fn classify_point(world: Vec3, camera: &Camera, triangles: &[ProjectedTriangle], adjacent: &[usize]) -> EdgePoint {
    let screen = camera.project(world);
    let screen = (screen.z > 0.0).then_some(screen);
    let visible = screen.map(|screen| point_visible(screen, triangles, adjacent)).unwrap_or(false);
    EdgePoint { world, screen, visible }
}

fn push_segment(polylines: &mut Vec<Polyline2>, a: Vec2, b: Vec2) {
    let mut polyline = Polyline2::new();
    polyline.add(a);
    polyline.add(b);
    polylines.push(polyline);
}

fn subdivide_edge(
    start: EdgePoint,
    end: EdgePoint,
    camera: &Camera,
    triangles: &[ProjectedTriangle],
    adjacent: &[usize],
    depth: usize,
    polylines: &mut Vec<Polyline2>,
) {
    let screen_length = match (start.screen, end.screen) {
        (Some(a), Some(b)) => (b.xy() - a.xy()).norm(),
        _ => 0.0,
    };
    let midpoint = classify_point(0.5 * (start.world + end.world), camera, triangles, adjacent);

    if depth >= MAX_EDGE_DEPTH || screen_length <= MIN_SCREEN_LENGTH {
        if start.visible && midpoint.visible {
            push_segment(polylines, start.screen.unwrap().xy(), midpoint.screen.unwrap().xy());
        }
        if midpoint.visible && end.visible {
            push_segment(polylines, midpoint.screen.unwrap().xy(), end.screen.unwrap().xy());
        }
        return;
    }

    if start.visible == midpoint.visible && midpoint.visible == end.visible {
        if start.visible {
            push_segment(polylines, start.screen.unwrap().xy(), end.screen.unwrap().xy());
        }
        return;
    }

    subdivide_edge(start, midpoint, camera, triangles, adjacent, depth + 1, polylines);
    subdivide_edge(midpoint, end, camera, triangles, adjacent, depth + 1, polylines);
}

fn edge_polylines(
    mesh: &Mesh3,
    edge: (usize, usize),
    camera: &Camera,
    triangles: &[ProjectedTriangle],
    edge_triangles: &HashMap<(usize, usize), Vec<usize>>,
    visibility: &[VertexVisibility],
    projected: &[Option<Vec3>],
) -> Vec<Polyline2> {
    let Some(adjacent) = edge_triangles.get(&edge) else {
        return Vec::new();
    };

    let start = EdgePoint {
        world: mesh.vertices[edge.0],
        screen: projected[edge.0],
        visible: adjacent.iter().any(|triangle| visibility[edge.0].nearest.contains(triangle)),
    };
    let end = EdgePoint {
        world: mesh.vertices[edge.1],
        screen: projected[edge.1],
        visible: adjacent.iter().any(|triangle| visibility[edge.1].nearest.contains(triangle)),
    };

    let mut polylines = Vec::new();
    subdivide_edge(start, end, camera, triangles, adjacent, 0, &mut polylines);
    polylines
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
    let camera = setup_camera(paper.view_box, area);
    let projected = project_vertices(&mesh, &camera);
    let (triangles, edge_triangles) = triangulate(&mesh, &projected);
    let visibility = visible_vertices(&projected, &triangles);

    for edge in mesh_edges(&mesh) {
        for polyline in edge_polylines(&mesh, edge, &camera, &triangles, &edge_triangles, &visibility, &projected) {
            paper.add(polyline);
        }
    }

    paper.optimize();
    let (dl, ml) = paper.length();
    println!("draw: {dl} mm, move: {ml} mm");
    let output_path = args.output_path.to_str().unwrap();
    paper.save(output_path)?;
    println!("wrote: {output_path}");

    let estimator = Estimator::best();
    let duration = estimator.estimate(&paper, 2000.0, 8000.0);
    println!("Estimated time: {}", format_duration(duration));

    Ok(())
}
