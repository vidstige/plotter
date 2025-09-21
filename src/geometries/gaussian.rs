use crate::{
    geometry::{DifferentiableGeometry, Geometry},
    sdf::SDF,
};
use nalgebra_glm::{Vec2, Vec3};

use super::heightmap::Heightmap;

pub struct Gaussian;
impl Gaussian {
    pub fn new() -> Self { Self }
}

impl Heightmap for Gaussian {
    fn z(&self, p: &Vec2) -> f32 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        (-r2).exp()
    }
}

impl DifferentiableGeometry for Gaussian {
    fn du(&self) -> impl DifferentiableGeometry { GaussianDu }
    fn dv(&self) -> impl DifferentiableGeometry { GaussianDv }
}

impl SDF for Gaussian {
    fn sdf(&self, position: &Vec3) -> f32 {
        self.z(&position.xy()) - position.z
    }
}

struct GaussianDu;
impl Geometry for GaussianDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        let dz = -2.0 * u * (-r2).exp();
        Vec3::new(1.0, 0.0, dz)
    }
}
impl DifferentiableGeometry for GaussianDu {
    fn du(&self) -> impl DifferentiableGeometry { GaussianDuDu }
    fn dv(&self) -> impl DifferentiableGeometry { GaussianDuDv }
}

struct GaussianDv;
impl Geometry for GaussianDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        let dz = -2.0 * v * (-r2).exp();
        Vec3::new(0.0, 1.0, dz)
    }
}
impl DifferentiableGeometry for GaussianDv {
    // Order of derivation does not matter, so just reuse (d/du)(d/dv)
    fn du(&self) -> impl DifferentiableGeometry { GaussianDuDv }
    fn dv(&self) -> impl DifferentiableGeometry { GaussianDvDv }
}

struct GaussianDuDu;
impl Geometry for GaussianDuDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        let e = (-r2).exp();
        let dz = e * (4.0 * u * u - 2.0);
        Vec3::new(0.0, 0.0, dz)
    }
}
impl DifferentiableGeometry for GaussianDuDu {}

struct GaussianDvDv;
impl Geometry for GaussianDvDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        let e = (-r2).exp();
        let dz = e * (4.0 * v * v - 2.0);
        Vec3::new(0.0, 0.0, dz)
    }
}
impl DifferentiableGeometry for GaussianDvDv {}

struct GaussianDuDv;
impl Geometry for GaussianDuDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        let dz = 4.0 * u * v * (-r2).exp();
        Vec3::new(0.0, 0.0, dz)
    }
}
impl DifferentiableGeometry for GaussianDuDv {}
