use std::{ops::{Sub, AddAssign, Add}, f32::INFINITY};

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
    let direction = Vec3::new(screen.x, screen.y, 1.0);
    let direction = unproject(&direction, &model, &projection, viewport).normalize();
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

fn trace<S: Surface>(ray: &Ray, surface: &S) -> Option<Vec3> {
    let step = 0.05;
    for i in 0..64 {
        let t = i as f32 * step;
        let p = ray.at(t);
        if surface.at(&p) < 0.0 {
            // we've entered the surface
            return Some(p);
        }
    }
    None
}

fn main() {
    let mut paper = Paper::new(A4_LANDSCAPE);
    // compute drawing area
    let area = pad(paper.view_box, 20);

    let mut rng = rand::thread_rng();
    let field = Spiral::new(Vec2::zeros());
    let eye = Vec3::new(-1.2, -1.2, -0.3);
    let model = look_at(&eye, &Vec3::new(0.0, 0.0, 0.8), &Vec3::new(0.0, 0.0, 1.0));
    let projection = perspective(viewbox_aspect(paper.view_box), 90.0_f32.to_radians(), 0.1, 10.0);
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
            let screen = project(&world, &model, &projection, viewport).xy();
            // back project and ray trace to find occlusions
            let ray = backproject(&screen, &model, &projection, viewport);
            let intersection = trace(&ray, &hole).unwrap_or(Vec3::new(10000.0, 10000.0, 10000.0));
            // threshold related to step in trace function
            let threshold = INFINITY;
            // clip against drawing area
            if contains(&area, &screen) {
                // handle occlusions
                if intersection.sub(&world).magnitude() < threshold {
                    polyline.add(screen);
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
