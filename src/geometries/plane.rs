use nalgebra_glm::{Vec2, Vec3, Mat2x2};

use crate::{geometry::Geometry, iso_surface::IsoSurface};
use crate::geometries::zero::Zero;

pub struct Plane;
impl Plane {
    pub fn new() -> Plane {
        Plane { }
    }
}

impl Geometry for Plane {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(u, v, 0.0)
    }
    fn du(&self) -> impl Geometry {
        PlaneDu {}
    }
    fn dv(&self) -> impl Geometry {
        PlaneDv {}
    }
    fn metric(&self, p: &Vec2) -> Mat2x2 {
        // override metric tensor with analytical expression
        return Mat2x2::new(
            1.0, 0.0,
            0.0, 1.0,
        )
    }
}

impl IsoSurface for Plane {
    fn iso_level(&self, position: &Vec3) -> f32 {
        position.z
    }
}

// first derivatives of sphere
struct PlaneDu;
impl Geometry for PlaneDu {
    fn evaluate(&self, _p: &Vec2) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
    fn du(&self) -> impl Geometry { Zero }
    fn dv(&self) -> impl Geometry { Zero }
}

struct PlaneDv;
impl Geometry for PlaneDv {
    fn evaluate(&self, _p: &Vec2) -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }
    fn du(&self) -> impl Geometry { Zero }
    fn dv(&self) -> impl Geometry { Zero }
}
