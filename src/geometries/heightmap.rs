use nalgebra_glm::{Vec2, Vec3};

use crate::geometry::Geometry;

pub trait Heightmap {
    fn z(&self, p: &Vec2) -> f32;
}

impl<T: Heightmap> Geometry for T {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(p.x, p.y, self.z(&p))
    }
}
