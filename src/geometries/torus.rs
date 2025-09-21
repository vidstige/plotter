use nalgebra_glm::{Mat2x2, Vec2, Vec3};

use crate::{
    geometry::{DifferentiableGeometry, Geometry},
    sdf::SDF,
};

pub struct Torus {
    pub radius_minor: f32,
    pub radius_major: f32,
}

impl Torus {
    pub fn new(radius_minor: f32, radius_major: f32) -> Self {
        Torus { radius_minor, radius_major }
    }
}

impl Geometry for Torus {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let ring = self.radius_major + self.radius_minor * cos_u;
        let r = self.radius_minor;
        Vec3::new(
            ring * cos_v,  //
            ring * sin_v,  //
            r * sin_u,     //
        )
    }
}

impl DifferentiableGeometry for Torus {
    fn du(&self) -> impl DifferentiableGeometry {
        TorusDu { radius_minor: self.radius_minor, radius_major: self.radius_major }
    }

    fn dv(&self) -> impl DifferentiableGeometry {
        TorusDv { radius_minor: self.radius_minor, radius_major: self.radius_major }
    }

    fn metric(&self, p: &Vec2) -> Mat2x2 {
        let u = p.x;
        let cos_u = u.cos();
        //let sin_u = u.sin();
        let ring = self.radius_major + self.radius_minor * cos_u;
        let r = self.radius_minor;
        Mat2x2::new(
            r * r, 0.0,       // E, F
            0.0, ring * ring, // F, G
        )
    }
}

impl SDF for Torus {
    fn sdf(&self, position: &Vec3) -> f32 {
        let x = position.x;
        let y = position.y;
        let z = position.z;
        let xy_len = (x * x + y * y).sqrt();
        ((xy_len - self.radius_major).powi(2) + z * z).sqrt() - self.radius_minor
    }
}

// First derivatives
pub struct TorusDu {
    pub radius_minor: f32,
    pub radius_major: f32,
}

impl Geometry for TorusDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let d_ring = -self.radius_minor * sin_u;
        Vec3::new(
            d_ring * cos_v, //
            d_ring * sin_v, //
            self.radius_minor * cos_u, //
        )
    }
}

impl DifferentiableGeometry for TorusDu {
    fn du(&self) -> impl DifferentiableGeometry {
        TorusDuDu { radius_minor: self.radius_minor, radius_major: self.radius_major }
    }

    fn dv(&self) -> impl DifferentiableGeometry {
        TorusDuDv { radius_minor: self.radius_minor, radius_major: self.radius_major }
    }
}

pub struct TorusDv {
    pub radius_minor: f32,
    pub radius_major: f32,
}

impl Geometry for TorusDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let ring = self.radius_major + self.radius_minor * cos_u;
        Vec3::new(
            -ring * sin_v, //
            ring * cos_v,  //
            0.0,           //
        )
    }
}

impl DifferentiableGeometry for TorusDv {
    fn du(&self) -> impl DifferentiableGeometry {
        TorusDuDv { radius_minor: self.radius_minor, radius_major: self.radius_major }
    }
    fn dv(&self) -> impl DifferentiableGeometry {
        TorusDvDv { radius_minor: self.radius_minor, radius_major: self.radius_major }
    }
}

// Second derivatives
pub struct TorusDuDu {
    pub radius_minor: f32,
    pub radius_major: f32,
}

impl Geometry for TorusDuDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let d2_ring = -self.radius_minor * cos_u;
        Vec3::new(
            d2_ring * cos_v, //
            d2_ring * sin_v, //
            -self.radius_minor * sin_u, //
        )
    }
}
impl DifferentiableGeometry for TorusDuDu {}

pub struct TorusDvDv {
    pub radius_minor: f32,
    pub radius_major: f32,
}

impl Geometry for TorusDvDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let ring = self.radius_major + self.radius_minor * cos_u;
        Vec3::new(
            -ring * cos_v, //
            -ring * sin_v, //
            0.0,           //
        )
    }
}
impl DifferentiableGeometry for TorusDvDv {}

pub struct TorusDuDv {
    pub radius_minor: f32,
    pub radius_major: f32,
}

impl Geometry for TorusDuDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let sin_u = u.sin();
        //let cos_u = u.cos();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let d_ring = -self.radius_minor * sin_u;
        Vec3::new(
            -d_ring * sin_v, //
            d_ring * cos_v,  //
            0.0,             //
        )
    }
}
impl DifferentiableGeometry for TorusDuDv {}
