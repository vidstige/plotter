use nalgebra_glm::{Mat2x2, Vec2, Vec3};

use crate::{geometry::Geometry, sdf::SDF};

pub struct Torus {
    pub r: f32, // minor radius
    pub R: f32, // major radius
}

impl Torus {
    pub fn new(r: f32, R: f32) -> Self {
        Torus { r, R }
    }
}

impl Geometry for Torus {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let ring = self.R + self.r * cos_u;
        Vec3::new(
            ring * cos_v,
            ring * sin_v,
            self.r * sin_u,
        )
    }

    fn du(&self) -> impl Geometry {
        TorusDu { r: self.r, R: self.R }
    }

    fn dv(&self) -> impl Geometry {
        TorusDv { r: self.r, R: self.R }
    }

    fn metric(&self, p: &Vec2) -> Mat2x2 {
        let u = p.x;
        let cos_u = u.cos();
        let sin_u = u.sin();
        let ring = self.R + self.r * cos_u;
        Mat2x2::new(
            self.r * self.r,                   // E
            0.0,                                // F
            0.0,
            ring * ring                         // G
        )
    }
}

impl SDF for Torus {
    fn iso_level(&self, position: &Vec3) -> f32 {
        let x = position.x;
        let y = position.y;
        let z = position.z;
        let xy_len = (x * x + y * y).sqrt();
        ((xy_len - self.R).powi(2) + z * z).sqrt() - self.r
    }
}

// First derivatives
pub struct TorusDu {
    pub r: f32,
    pub R: f32,
}

impl Geometry for TorusDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let d_ring = -self.r * sin_u;
        Vec3::new(
            d_ring * cos_v,
            d_ring * sin_v,
            self.r * cos_u,
        )
    }

    fn du(&self) -> impl Geometry {
        TorusDuDu { r: self.r, R: self.R }
    }

    fn dv(&self) -> impl Geometry {
        TorusDuDv { r: self.r, R: self.R }
    }
}

pub struct TorusDv {
    pub r: f32,
    pub R: f32,
}

impl Geometry for TorusDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let ring = self.R + self.r * cos_u;
        Vec3::new(
            -ring * sin_v,
            ring * cos_v,
            0.0,
        )
    }

    fn du(&self) -> impl Geometry {
        TorusDuDv { r: self.r, R: self.R }
    }

    fn dv(&self) -> impl Geometry {
        TorusDvDv { r: self.r, R: self.R }
    }
}

// Second derivatives
pub struct TorusDuDu {
    pub r: f32,
    pub R: f32,
}

impl Geometry for TorusDuDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let d2_ring = -self.r * cos_u;
        Vec3::new(
            d2_ring * cos_v,
            d2_ring * sin_v,
            -self.r * sin_u,
        )
    }
}

pub struct TorusDvDv {
    pub r: f32,
    pub R: f32,
}

impl Geometry for TorusDvDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let cos_u = u.cos();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let ring = self.R + self.r * cos_u;
        Vec3::new(
            -ring * cos_v,
            -ring * sin_v,
            0.0,
        )
    }
}

pub struct TorusDuDv {
    pub r: f32,
    pub R: f32,
}

impl Geometry for TorusDuDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        let sin_u = u.sin();
        let cos_u = u.cos();
        let cos_v = v.cos();
        let sin_v = v.sin();
        let d_ring = -self.r * sin_u;
        Vec3::new(
            -d_ring * sin_v,
            d_ring * cos_v,
            0.0,
        )
    }
}
