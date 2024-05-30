use std::{ops::{Sub, AddAssign, Add}, io::{self}};

use eq::{linesearch, newton_raphson};
use nalgebra_glm::{Vec2, Vec3, look_at, project, Vec4, perspective, unproject, Mat4};

use polyline::Polyline2;

use rand::distributions::Distribution;
use resolution::Resolution;
use statrs::distribution::Normal;
use tiny_skia::{Pixmap, PathBuilder, Paint, Stroke, Transform};

mod resolution;
mod eq;
mod buffer;
mod netbm;
mod paper;
mod polyline;

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

fn contains(resolution: &Resolution, point: &Vec2) -> bool {
    point.x >= 0.0 && point.x < resolution.width as f32 && point.y >= 0.0 && point.y < resolution.height as f32
}

fn main() -> io::Result<()> {
    let resolution = Resolution::new(320, 200);

    let mut rng = rand::thread_rng();
    let distribution = Normal::new(0.0, 1.0).unwrap();
    let field = Spiral::new(Vec2::zeros());
    let eye = Vec3::new(-2.5, -2.5, -1.5);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    let near = 0.1;
    let far = 10.0;
    let projection = perspective(resolution.aspect_ratio(), 45.0_f32.to_radians(), near, far);
    let viewport = Vec4::new(0.0, 0.0, resolution.width as f32, resolution.height as f32);
    let hole = Hole::new();
    let mut polylines = Vec::new();
    for _ in 0..1024 {
        let mut polyline = Polyline2::new();

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
            
            if contains(&resolution, &screen.xy()) {
                // back project and ray trace to find occlusions
                let ray = backproject(&screen.xy(), &model, &projection, viewport);
                if let Some(intersection) = trace(&ray, &hole, near, far) {
                    let traced_screen = project(&intersection, &model, &projection, viewport);
                    // handle occlusions
                    if screen.z - traced_screen.z < 0.0001 {
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
        polylines.push(polyline);
    }

    // render to pixmap
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    
    let mut paint = Paint::default();
    paint.set_color_rgba8(210, 2, 180, 0xff);
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = 1.0;

    for polyline in polylines {
        let mut pb = PathBuilder::new();
        for (index, point) in polyline.points.iter().enumerate() {
            if index == 0 {
                pb.move_to(point.x, point.y);
            } else {
                pb.line_to(point.x, point.y);
            }
            
        }
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
    pixmap.save_png("image.png")?;

    Ok(())
}