use nalgebra_glm::{Vec2, Vec3};

use crate::geometry::Geometry;


pub struct Hole {
}

impl Geometry for Hole {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(p.x, p.y, self.z(p))
    }
}

impl Hole {
    pub fn new() -> Hole {
        Hole {}        
    }
    pub fn z(&self, p: &Vec2) -> f32 {
        1.0 / p.norm_squared()
    }
}
