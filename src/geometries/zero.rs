use nalgebra_glm::{Vec2, Vec3};

use crate::geometry::{DifferentiableGeometry, Geometry};

pub(crate) struct Zero;
impl Geometry for Zero {
    fn evaluate(&self, _p: &Vec2) -> Vec3 { Vec3::zeros() }
}

impl DifferentiableGeometry for Zero {
    fn du(&self) -> impl DifferentiableGeometry { Self }
    fn dv(&self) -> impl DifferentiableGeometry { Self }
}