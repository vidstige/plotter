use std::{ops::{Sub, AddAssign, Add}, f32::INFINITY, io::{self, Write}, fs};

use nalgebra_glm::{Vec2, Vec3, look_at, project, Vec4, perspective, unproject, Mat4};
use paper::{ViewBox, Paper, A4_LANDSCAPE, viewbox_aspect};
use polyline::Polyline;
use rand::Rng;

mod paper;
mod polyline;

fn pad(view_box: ViewBox, pad: i32) -> ViewBox {
    let (x, y, w, h) = view_box;
    (x + pad, y + pad, w - 2 * pad, h - 2 * pad)
}

fn contains(view_box: &ViewBox, point: &Vec2) -> bool {
    let (x, y, w, h) = view_box;
    point.x > *x as f32 && point.y > *y as f32 && point.x < (x + w) as f32 && point.y < (y + h) as f32
}

fn cross2(vector: Vec2) -> Vec2 {
    Vec2::new(-vector.y, vector.x)
}

struct Spiral {
    center: Vec2,
}
impl Spiral {
    fn new(center: Vec2) -> Spiral {
        Spiral { center }
    }
    fn at(&self, p: Vec2) -> Vec2 {
        cross2(p.sub(&self.center))
    }
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn at(&self, t: f32) -> Vec3 {
        self.origin.add(self.direction.scale(t))
    }
}

fn backproject(screen: &Vec2, model: &Mat4, projection: &Mat4, viewport: Vec4) -> Ray {
    let direction = unproject(&Vec3::new(screen.x, screen.y, 1.0), &model, &projection, viewport).normalize();
    // recover eye position
    let model_inverse = model.try_inverse().unwrap();
    let eye = model_inverse.column(3).xyz();
    Ray{ origin: eye, direction }
}

trait Surface {
    fn at(&self, position: &Vec3) -> f32;
}

struct Hole {
}

impl Hole {
    fn new() -> Hole {
        Hole {}        
    }
    fn z(&self, p: &Vec2) -> f32 {
        1.0 / (p.x*p.x + p.y*p.y)
    }
}

impl Surface for Hole {
    fn at(&self, position: &Vec3) -> f32 {
        self.z(&position.xy()) - position.z
    }
}

fn newton_raphson<F: Fn(f32) -> f32>(f: F, x0: f32) -> Option<f32> {
    let epsilon = 0.01; // for numerical diffrentiation
    let tol = 0.01; // for considering roots
    let mut x = x0; 
    
    for _ in 0..10 {
        // compute df/dt using forward diffrentiation
        let dfdt = (f(x + epsilon) - f(x)) / epsilon;
        if dfdt.abs() < 0.001 {
            break;
        }
        x = x - f(x) / dfdt;
        // exit early if root found
        if f(x).abs() < tol {
            break;
        }
    }
    // if we're close enough a root was found
    (f(x).abs() < tol).then_some(x)
}

fn linesearch<F: Fn(f32) -> f32>(f: F, lo: f32, hi: f32, steps: usize) -> Option<(f32, f32)> {
    let step_length = (hi - lo) / steps as f32;
    let mut x0 = lo;
    let mut f0 = f(x0);
    for step in 0..steps {
        let x = step as f32 * step_length;
        let fx = f(x);
        if (f0 < 0.0) != (fx < 0.0) {
            // root range found!
            return Some((x0, x));
        }
        x0 = x;
        f0 = fx;
    }
    None
}

fn trace<S: Surface>(ray: &Ray, surface: &S, lo: f32, hi: f32) -> Option<Vec3> {
    // first linesearch to find rough estimate
    let f = |t| surface.at(&ray.at(t));
    if let Some((lo, hi)) = linesearch(f, lo, hi, 10) {
        // fine tune with newton_raphson
        if let Some(t) = newton_raphson(f, 0.5 * (hi + lo)) {
            return Some(ray.at(t));
        }
    }
    None
}

type Resolution = (i32, i32);

fn aspect_ratio(resolution: Resolution) -> f32 {
    resolution.0 as f32 / resolution.1 as f32
}
fn area(resolution: Resolution) -> usize {
    let (width, height) = resolution;
    (width * height) as usize
}

struct Buffer {
    resolution: Resolution,
    pixels: Vec<u8>,
}

impl Buffer {
    fn new(resolution: Resolution) -> Buffer {
        Buffer { resolution, pixels: vec![0; area(resolution) * 4]}
    }
}

type Color = [u8; 4];

fn pixel(target: &mut Buffer, x: i32, y: i32, color: &Color) {
    let (stride, _) = target.resolution;
    let index = ((x + y * stride) * 4) as usize;
    target.pixels[index..index + color.len()].copy_from_slice(color);
}

fn gray(intensity: f32) -> Color {
    let gray = (intensity.clamp(0.0, 1.0) * 255.0) as u8;
    [gray, gray, gray, 0xff]
}

fn render(target: &mut Buffer) {
    let eye = Vec3::new(-1.2, -1.2, -0.3);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    let projection = perspective(aspect_ratio(target.resolution), 90.0_f32.to_radians(), 0.1, 10.0);
    let viewport = Vec4::new(0.0, 0.0, target.resolution.0 as f32, target.resolution.1 as f32);
    let hole = Hole::new();

    let (width, height) = target.resolution;
    for y in 0..height {
        for x in 0..width {
            let screen = Vec2::new(x as f32, y as f32);
            let ray = backproject(&screen, &model, &projection, viewport);
            if let Some(p) = trace(&ray, &hole, 0.1, 10.0) {
                pixel(target, x, y, &gray(p.z / 10.0));
            }
        }
    }
}

/*
fn main() -> io::Result<()>{
    let resolution = (297, 210);
    let mut buffer = Buffer::new(resolution);
    render(&mut buffer);
    fs::write("output.raw", &buffer.pixels)?;
    //std::io::stdout().write_all(&buffer.pixels)?;
    Ok(())
}
*/

fn main() {
    let mut paper = Paper::new(A4_LANDSCAPE);
    // compute drawing area
    let area = pad(paper.view_box, 20);

    let mut rng = rand::thread_rng();
    let field = Spiral::new(Vec2::zeros());
    let eye = Vec3::new(-1.2, -1.2, -0.3);
    let near = 0.1;
    let far = 10.0;
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    let projection = perspective(viewbox_aspect(paper.view_box), 90.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let hole = Hole::new();
    for _ in 0..1024 {
        let mut polyline = Polyline::new();
        let mut p = Vec2::new((rng.gen::<f32>() - 0.5) * 4.0, (rng.gen::<f32>() - 0.5) * 4.0);
        for _ in 0..5 {
            // evaluate surface at x, y
            let z = hole.z(&p);
            let world = Vec3::new(p.x, p.y, z);
            // project world cordinate into screen cordinate
            let screen = project(&world, &model, &projection, viewport);
            // back project and ray trace to find occlusions
            let ray = backproject(&screen.xy(), &model, &projection, viewport);
            let intersection = trace(&ray, &hole, near, far).unwrap_or(Vec3::new(-1000.0, -1000.0, -1000.0));
            let traced_screen = project(&intersection, &model, &projection, viewport);
            // clip against drawing area
            if contains(&area, &screen.xy()) {
                // handle occlusions
                if traced_screen.z < screen.z {
                    polyline.add(screen.xy());
                }
            }

            // step forward
            let delta = field.at(p);
            let norm = delta.norm();
            let step = 0.1;
            p.add_assign(delta.scale(step / norm));
            
        }
        paper.add(&polyline);
    }
    
    paper.save("image.svg");
}