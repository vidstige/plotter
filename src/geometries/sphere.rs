use nalgebra_glm::{Vec2, Vec3, Mat2x2};

use crate::{geometry::Geometry, iso_surface::IsoSurface};


pub struct Sphere {

}
impl Sphere {
    pub fn new() -> Sphere {
        Sphere { }
    }
}

// second derivatives of sphere
struct SphereDuDu {

}
impl Geometry for SphereDuDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        return Vec3::new(
           -v.cos() * u.sin(),
           -v.sin() * u.sin(),
           -u.cos(),
        )
    }
}

struct SphereDvDv {

}
impl Geometry for SphereDvDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        return Vec3::new(
          -v.cos() * u.sin(),
          -v.sin() * u.sin(),
          0.0,
        )
    }
}

struct SphereDuDv {

}

impl Geometry for SphereDuDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        return Vec3::new(
            -v.sin() * u.cos(),
            v.cos() * u.cos(),
            0.0,
        )
    }
}

// first derivatives of sphere
struct SphereDu {

}
impl Geometry for SphereDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        return Vec3::new(
            v.cos() * u.cos(),
            v.sin() * u.cos(),
            -u.sin(),
        )
    }

    fn du(&self) -> impl Geometry {
        SphereDuDu {}
    }

    fn dv(&self) -> impl Geometry {
        SphereDuDv {}
    }
}

struct SphereDv {

}
impl Geometry for SphereDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        return Vec3::new(
            -v.sin() * u.sin(),
            v.cos() * u.sin(),
            0.0,
        )
    }

    fn du(&self) -> impl Geometry {
        // Order of derivation does not matter, so just reuse (d/du)(d/dv)
        SphereDuDv {}
    }

    fn dv(&self) -> impl Geometry {
        SphereDvDv {}
    }
}

impl Geometry for Sphere {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (u, v) = (p.x, p.y);
        Vec3::new(
            v.cos() * u.sin(),
            v.sin() * u.sin(),
            u.cos(),
        )
    }
    fn du(&self) -> impl Geometry {
        SphereDu {}
    }
    fn dv(&self) -> impl Geometry {
        SphereDv {}
    }
    fn metric(&self, p: &Vec2) -> Mat2x2 {
        // override metric tensor with analytical expression
        return Mat2x2::new(
            1.0, 0.0,
            0.0, p.y.sin().powi(2)
        )
    }
}

impl IsoSurface for Sphere {
    fn iso_level(&self, position: &Vec3) -> f32 {
        position.norm() - 1.0
    }
}