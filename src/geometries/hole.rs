use nalgebra_glm::{Vec2, Vec3};

use crate::{geometry::Geometry, iso_surface::IsoSurface};

pub struct Hole;

impl Geometry for Hole {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(p.x, p.y, self.z(p))
    }
    fn du(&self) -> impl Geometry { HoleDu {} }
    fn dv(&self) -> impl Geometry { HoleDv {} }
}

impl Hole {
    pub fn new() -> Hole {
        Hole {}        
    }
    pub fn z(&self, p: &Vec2) -> f32 {
        1.0 / p.norm_squared()
    }
}

impl IsoSurface for Hole {
    fn iso_level(&self, position: &Vec3) -> f32 {
        self.z(&position.xy()) - position.z
    }
}

// First order partial derivatives
struct HoleDu;

// first order derivatives
impl Geometry for HoleDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(
            1.0,
            0.0,
            -(2.0 * p.x) / (p.x * p.x + p.y * p.y).powi(2),
        )
    }
    fn du(&self) -> impl Geometry { HoleDuDu {} }
    fn dv(&self) -> impl Geometry { HoleDuDv {} }
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
    fn du(&self) -> impl Geometry { HoleDuDv {} } // order of diffrentiation does not matter
    fn dv(&self) -> impl Geometry { HoleDvDv {} }
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
