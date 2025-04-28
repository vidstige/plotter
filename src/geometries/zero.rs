use nalgebra_glm::{Vec2, Vec3};

use crate::geometry::Geometry;

pub(crate) struct Zero;
impl Geometry for Zero {
    fn evaluate(&self, _p: &Vec2) -> Vec3 { Vec3::zeros() }
    fn du(&self) -> impl Geometry { Self }
    fn dv(&self) -> impl Geometry { Self }
}