use nalgebra_glm::{Vec2, Vec3};
use crate::geometry::Geometry;
use crate::sdf::SDF;
use crate::lerp::lerp;

/// A surface blending two geometries A and B with interpolation factor t ∈ [0, 1]
pub struct Blend<A, B> {
    pub a: A,
    pub b: B,
    pub t: f32,
}

impl<A, B> Blend<A, B> {
    pub fn new(a: A, b: B, t: f32) -> Self {
        Self { a, b, t }
    }
}

impl<A, B> SDF for Blend<A, B>
where
    A: SDF,
    B: SDF,
{
    fn iso_level(&self, position: &Vec3) -> f32 {
        let a_level = self.a.iso_level(position);
        let b_level = self.b.iso_level(position);
        lerp(a_level, b_level, self.t)
    }
}

impl<A, B> Geometry for Blend<A, B>
where
    A: Geometry,
    B: Geometry,
{
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let va = self.a.evaluate(p);
        let vb = self.b.evaluate(p);
        lerp(va, vb, self.t)
    }

    fn du(&self) -> impl Geometry {
        Blend {
            a: self.a.du(),
            b: self.b.du(),
            t: self.t,
        }
    }

    fn dv(&self) -> impl Geometry {
        Blend {
            a: self.a.dv(),
            b: self.b.dv(),
            t: self.t,
        }
    }
}
