use std::ops::Sub;

use nalgebra_glm::Vec2;

fn cross2(vector: Vec2) -> Vec2 {
    Vec2::new(-vector.y, vector.x)
}

pub struct Spiral {
    center: Vec2,
}
impl Spiral {
    pub fn new(center: Vec2) -> Spiral {
        Spiral { center }
    }
    pub fn at(&self, p: &Vec2) -> Vec2 {
        cross2(p.sub(&self.center))
    }
}
