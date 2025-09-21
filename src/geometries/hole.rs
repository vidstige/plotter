use nalgebra_glm::{Vec2, Vec3};

use crate::{
    geometry::{DifferentiableGeometry, Geometry},
    sdf::SDF,
};

use super::heightmap::Heightmap;

pub struct Hole;

impl DifferentiableGeometry for Hole {
    fn du(&self) -> impl DifferentiableGeometry { HoleDu }
    fn dv(&self) -> impl DifferentiableGeometry { HoleDv }
}

impl Hole {
    pub fn new() -> Hole { Hole }
}

impl Heightmap for Hole {
    fn z(&self, p: &Vec2) -> f32 {
        1.0 / p.norm_squared()
    }
}

impl SDF for Hole {
    fn sdf(&self, position: &Vec3) -> f32 {
        self.z(&position.xy()) - position.z
    }
}

// First order partial derivatives
struct HoleDu;

// first order derivatives
impl Geometry for HoleDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(1.0, 0.0, -(2.0 * p.x) / (p.x * p.x + p.y * p.y).powi(2))
    }
}

impl DifferentiableGeometry for HoleDu {
    fn du(&self) -> impl DifferentiableGeometry { HoleDuDu }
    fn dv(&self) -> impl DifferentiableGeometry { HoleDuDv }
}

struct HoleDv;

impl Geometry for HoleDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(
            0.0,
            1.0,
            -(2.0 * p.y) / (p.x * p.x + p.y * p.y).powi(2),
        )
    }
}

impl DifferentiableGeometry for HoleDv {
    fn du(&self) -> impl DifferentiableGeometry { HoleDuDv } // order of diffrentiation does not matter
    fn dv(&self) -> impl DifferentiableGeometry { HoleDvDv }
}

// second order partial derivatives
struct HoleDuDu;
impl Geometry for HoleDuDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(
            0.0,
            0.0,
            (8.0 * p.x * p.x) / (p.x*p.x + p.y*p.y).powi(3) - 2.0 / (p.x*p.x + p.y*p.y).powi(2)
        )
    }
}
impl DifferentiableGeometry for HoleDuDu {}

struct HoleDvDv;
impl Geometry for HoleDvDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(
            0.0,
            0.0,
            (8.0 * p.y * p.y) / (p.x*p.x + p.y*p.y).powi(3) - 2.0 / (p.x*p.x + p.y*p.y).powi(2)
        )
    }
}
impl DifferentiableGeometry for HoleDvDv {}

struct HoleDuDv;
impl Geometry for HoleDuDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(
            0.0,
            0.0,
            (8.0 * p.x * p.y) / (p.x*p.x + p.y*p.y).powi(3)
        )
    }
}
impl DifferentiableGeometry for HoleDuDv {}
