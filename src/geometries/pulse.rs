use nalgebra_glm::{Vec2, Vec3};

use crate::{geometry::Geometry, sdf::SDF};

#[derive(Clone)]
pub struct Pulse {
    pub amplitude: f32,         // Amplitude
    pub sigma: f32,     // Envelope sharpness
    pub c: f32,         // Propagation speed
    pub lambda: f32,    // Exponential decay
    pub cycles: f32,    // Number of sine cycles within the envelope
    pub t: f32,         // Time
}

impl Pulse {
    pub fn z(&self, p: &Vec2) -> f32 {
        let r = p.norm();
        let omega = self.cycles * std::f32::consts::TAU * self.sigma;
        let r0 = -2.0 / self.sigma;
        let rt = r - r0 - self.c * self.t;
        self.amplitude * (omega * r).sin()
             * (-self.sigma * self.sigma * rt * rt).exp()
             * (-self.lambda * r).exp()
    }
}

impl SDF for Pulse {
    fn sdf(&self, position: &Vec3) -> f32 {
        self.z(&position.xy()) - position.z
    }
}

impl Geometry for Pulse {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(p.x, p.y, self.z(&p))
    }
}