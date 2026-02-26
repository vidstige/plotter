use nalgebra_glm::{Vec2, Vec3};

use crate::sdf::SDF;

use super::heightmap::Heightmap;

pub struct Sum<A, B> {
    pub a: A,
    pub b: B,
}

impl<A, B> Sum<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> Heightmap for Sum<A, B>
where
    A: Heightmap,
    B: Heightmap,
{
    fn z(&self, p: &Vec2) -> f32 {
        self.a.z(p) + self.b.z(p)
    }
}

impl<A, B> SDF for Sum<A, B>
where
    A: Heightmap,
    B: Heightmap,
{
    fn sdf(&self, position: &Vec3) -> f32 {
        self.z(&position.xy()) - position.z
    }
}
