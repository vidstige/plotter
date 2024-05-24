use std::ops::{Sub, AddAssign};

use nalgebra_glm::{Vec2, Vec3, look_at, project, Vec4, Mat4};
use paper::{ViewBox, A4_PORTRAIT, Paper};
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
    
fn main() {
    let mut paper = Paper::new();
    // compute drawing area
    let area = pad(A4_PORTRAIT, 20);

    let mut rng = rand::thread_rng();
    let field = Spiral::new(Vec2::zeros());
    let max_step = 0.05;
    let projection = look_at(&Vec3::new(-0.8, -0.8, 0.0), &Vec3::zeros(), &Vec3::new(0.0, 0.0, 1.0));
    let viewport = Vec4::new(area.0 as f32, area.1 as f32, area.2 as f32, area.3 as f32);
    for _ in 0..256 {
        let mut polyline = Polyline::new();
        let mut p = Vec2::new((rng.gen::<f32>() - 0.5) * 4.0, (rng.gen::<f32>() - 0.5) * 4.0);
        for _ in 0..10 {
            // evaluate surface at x, y
            let z = 1.0 / (p.x*p.x + p.y*p.y);
            let world = Vec3::new(p.x, p.y, z);
            // project world cordinate into screen cordinate (and discard z)
            let screen = project(&world, &Mat4::identity(), &projection, viewport).xy();
            if contains(&area, &screen) {
                polyline.add(screen);
            }

            let delta = field.at(p);
            let norm = delta.norm();
            let step = norm.min(max_step);
            p.add_assign(delta.scale(step / norm));
            
        }
        paper.add(&polyline);
    }
    
    paper.save("image.svg");
}
