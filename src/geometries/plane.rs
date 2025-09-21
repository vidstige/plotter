use nalgebra_glm::{Mat2x2, Vec2, Vec3};

use crate::geometries::zero::Zero;
use crate::geometry::DifferentiableGeometry;
use crate::{geometry::Geometry, sdf::SDF};

pub struct Plane;
impl Plane {
    pub fn new() -> Plane {
        Plane {}
    }
}

impl Geometry for Plane {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(u, v, 0.0)
    }
}

impl DifferentiableGeometry for Plane {
    fn du(&self) -> impl DifferentiableGeometry {
        PlaneDu {}
    }
    fn dv(&self) -> impl DifferentiableGeometry {
        PlaneDv {}
    }
    fn metric(&self, _p: &Vec2) -> Mat2x2 {
        // override metric tensor with analytical expression
        Mat2x2::new(
            1.0, 0.0, //
            0.0, 1.0, //
        )
    }
}

impl SDF for Plane {
    fn sdf(&self, position: &Vec3) -> f32 {
        position.z
    }
}

// first derivatives of sphere
struct PlaneDu;
impl Geometry for PlaneDu {
    fn evaluate(&self, _p: &Vec2) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

impl DifferentiableGeometry for PlaneDu {
    fn du(&self) -> impl DifferentiableGeometry { Zero }
    fn dv(&self) -> impl DifferentiableGeometry { Zero }
}

struct PlaneDv;
impl Geometry for PlaneDv {
    fn evaluate(&self, _p: &Vec2) -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }
}
impl DifferentiableGeometry for PlaneDv {
    fn du(&self) -> impl DifferentiableGeometry { Zero }
    fn dv(&self) -> impl DifferentiableGeometry { Zero }
}
