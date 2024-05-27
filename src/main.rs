use std::{ops::{Sub, AddAssign, Add}, io::{self}};

use eq::{linesearch, newton_raphson};
use nalgebra_glm::{Vec2, Vec3, look_at, project, Vec4, perspective, unproject, Mat4};

use buffer::{Buffer, aspect_ratio, gray, pixel};
use paper::{ViewBox, Paper, A4_LANDSCAPE, viewbox_aspect};
use polyline::Polyline;

use rand::distributions::Distribution;
use statrs::distribution::Normal;

mod eq;
mod buffer;
mod netbm;
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
    let world = unproject(&Vec3::new(screen.x, screen.y, 1.0), &model, &projection, viewport);
    // recover eye position
    let model_inverse = model.try_inverse().unwrap();
    let eye = model_inverse.column(3).xyz();

    Ray{ origin: eye, direction: world.sub(eye).normalize() }
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
                pixel(target, x, y, gray(p.z / 10.0));
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

fn main() -> io::Result<()> {
    let mut paper = Paper::new(A4_LANDSCAPE, 0.5);

    // compute drawing area
    let area = pad(paper.view_box, 20);

    let mut rng = rand::thread_rng();
    let distribution = Normal::new(0.0, 1.0).unwrap();
    let field = Spiral::new(Vec2::zeros());
    let eye = Vec3::new(-1.8, -1.8, -0.8);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    let near = 0.1;
    let far = 10.0;
    let projection = perspective(viewbox_aspect(paper.view_box), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    let hole = Hole::new();
    for _ in 0..1024 {
        let mut polyline = Polyline::new();

        let mut p = Vec2::new(
            distribution.sample(&mut rng) as f32,
            distribution.sample(&mut rng) as f32,
        );
        for _ in 0..5 {
            // evaluate surface at x, y
            let z = hole.z(&p);
            let world = Vec3::new(p.x, p.y, z);
            // project world cordinate into screen cordinate
            let screen = project(&world, &model, &projection, viewport);
            // clip against drawing area
            if contains(&area, &screen.xy()) {
                // back project and ray trace to find occlusions
                let ray = backproject(&screen.xy(), &model, &projection, viewport);
                if let Some(intersection) = trace(&ray, &hole, near, far) {
                    let traced_screen = project(&intersection, &model, &projection, viewport);
                    // handle occlusions
                    if (traced_screen.z - screen.z).abs() < 0.001 {
                        polyline.add(screen.xy());
                    }
                }
            }

            // step forward
            let delta = field.at(p);
            let norm = delta.norm();
            let step = 0.1;
            p.add_assign(delta.scale(step / norm));
            
        }
        paper.add(polyline);
    }
    
    paper.optimize();
    paper.save("image.svg")?;
    Ok(())
}