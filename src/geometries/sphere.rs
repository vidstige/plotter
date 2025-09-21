use nalgebra_glm::{Mat2x2, Vec2, Vec3};

use crate::{
    geometry::{DifferentiableGeometry, Geometry},
    sdf::SDF,
};

pub struct Sphere;
impl Sphere {
    pub fn new() -> Sphere { Sphere }
}

impl Geometry for Sphere {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(
            v.cos() * u.sin(), //
            v.sin() * u.sin(), //
            u.cos(),           //
        )
    }
}

impl DifferentiableGeometry for Sphere {
    fn du(&self) -> impl DifferentiableGeometry { SphereDu }
    fn dv(&self) -> impl DifferentiableGeometry { SphereDv }
    fn metric(&self, p: &Vec2) -> Mat2x2 {
        // override metric tensor with analytical expression
        Mat2x2::new(
            1.0, 0.0,               //
            0.0, p.x.sin().powi(2), //
        )
    }
}

impl SDF for Sphere {
    fn sdf(&self, position: &Vec3) -> f32 {
        position.norm() - 1.0
    }
}

// first derivatives of sphere
struct SphereDu;
impl Geometry for SphereDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(
            v.cos() * u.cos(), //
            v.sin() * u.cos(), //
            -u.sin(),          //
        )
    }
}

impl DifferentiableGeometry for SphereDu {
    fn du(&self) -> impl DifferentiableGeometry { SphereDuDu }
    fn dv(&self) -> impl DifferentiableGeometry { SphereDuDv }
}

struct SphereDv;
impl Geometry for SphereDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(
            -v.sin() * u.sin(), //
            v.cos() * u.sin(),  //
            0.0,                //
        )
    }
}

impl DifferentiableGeometry for SphereDv {
    // Order of derivation does not matter, so just reuse (d/du)(d/dv)
    fn du(&self) -> impl DifferentiableGeometry { SphereDuDv }
    fn dv(&self) -> impl DifferentiableGeometry { SphereDvDv }
}

// second derivatives of sphere
struct SphereDuDu;
impl Geometry for SphereDuDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(
            -v.cos() * u.sin(), //
            -v.sin() * u.sin(), //
            -u.cos(),           //
        )
    }
}
impl DifferentiableGeometry for SphereDuDu {}

struct SphereDvDv;
impl Geometry for SphereDvDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(
            -v.cos() * u.sin(), //
            -v.sin() * u.sin(), //
            0.0,                //
        )
    }
}
impl DifferentiableGeometry for SphereDvDv {}

struct SphereDuDv;

impl Geometry for SphereDuDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(
            -v.sin() * u.cos(), //
            v.cos() * u.cos(),  //
            0.0,                //
        )
    }
}
impl DifferentiableGeometry for SphereDuDv {}
