use nalgebra_glm::{Vec2, Vec3};
use crate::{geometry::Geometry, sdf::SDF};

pub struct Gaussian;
impl Gaussian {
    pub fn new() -> Self { Self }
    pub fn z(&self, p: &Vec2) -> f32 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        (-r2).exp()
    }
}

impl Geometry for Gaussian {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(p.x, p.y, self.z(p))
    }

    fn du(&self) -> impl Geometry { GaussianDu }
    fn dv(&self) -> impl Geometry { GaussianDv }
}

impl SDF for Gaussian {
    fn iso_level(&self, position: &Vec3) -> f32 {
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

    fn du(&self) -> impl Geometry { GaussianDuDu }
    fn dv(&self) -> impl Geometry { GaussianDuDv }
}

struct GaussianDv;
impl Geometry for GaussianDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        let dz = -2.0 * v * (-r2).exp();
        Vec3::new(0.0, 1.0, dz)
    }

    // Order of derivation does not matter, so just reuse (d/du)(d/dv)
    fn du(&self) -> impl Geometry { GaussianDuDv }
    fn dv(&self) -> impl Geometry { GaussianDvDv }
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

struct GaussianDuDv;
impl Geometry for GaussianDuDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let r2 = u * u + v * v;
        let dz = 4.0 * u * v * (-r2).exp();
        Vec3::new(0.0, 0.0, dz)
    }
}
