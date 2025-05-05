use nalgebra_glm::{Vec2, Vec3};

use crate::{geometry::Geometry, sdf::SDF};

#[derive(Clone)]
pub struct Pulse {
    omega: f32,
    sigma: f32,
    lambda: f32,
    r0: f32,
    c: f32,
    t: f32,
}

impl Pulse {
    pub fn new(omega: f32, sigma: f32, lambda: f32, r0: f32, c: f32, t: f32) -> Self {
        Self { omega, sigma, lambda, r0, c, t }
    }

    pub fn z(&self, p: &Vec2) -> f32 {
        let r = p.norm();
        let s = (r - self.r0 - self.c * self.t).powi(2);
        (self.omega * r).sin()
            * (-self.sigma * self.sigma * s).exp()
            * (-self.lambda * r).exp()
    }
}

impl Geometry for Pulse {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        Vec3::new(p.x, p.y, self.z(p))
    }

    fn du(&self) -> impl Geometry {
        PulseDu { pulse: self.clone() }
    }

    fn dv(&self) -> impl Geometry {
        PulseDv { pulse: self.clone() }
    }
}

impl SDF for Pulse {
    fn sdf(&self, position: &Vec3) -> f32 {
        self.z(&position.xy()) - position.z
    }
}

// ===== First-order derivatives =====

#[derive(Clone)]
struct PulseDu {
    pulse: Pulse,
}

#[derive(Clone)]
struct PulseDv {
    pulse: Pulse,
}

impl Geometry for PulseDu {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let r = p.norm();
        let dr_dx = p.x / r;

        let s = r - self.pulse.r0 - self.pulse.c * self.pulse.t;
        let exp1 = (-self.pulse.sigma * self.pulse.sigma * s * s).exp();
        let exp2 = (-self.pulse.lambda * r).exp();
        let sin_term = (self.pulse.omega * r).sin();
        let cos_term = (self.pulse.omega * r).cos();

        let A = self.pulse.omega * cos_term
            - 2.0 * self.pulse.sigma * self.pulse.sigma * s * sin_term
            - self.pulse.lambda * sin_term;

        let dz_dx = A * exp1 * exp2 * dr_dx;
        Vec3::new(1.0, 0.0, dz_dx)
    }

    fn du(&self) -> impl Geometry {
        PulseD2 { pulse: self.pulse.clone(), axis: Axis::U }
    }

    fn dv(&self) -> impl Geometry {
        PulseD2 { pulse: self.pulse.clone(), axis: Axis::UV }
    }
}

impl Geometry for PulseDv {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let r = p.norm();
        let dr_dy = p.y / r;

        let s = r - self.pulse.r0 - self.pulse.c * self.pulse.t;
        let exp1 = (-self.pulse.sigma * self.pulse.sigma * s * s).exp();
        let exp2 = (-self.pulse.lambda * r).exp();
        let sin_term = (self.pulse.omega * r).sin();
        let cos_term = (self.pulse.omega * r).cos();

        let A = self.pulse.omega * cos_term
            - 2.0 * self.pulse.sigma * self.pulse.sigma * s * sin_term
            - self.pulse.lambda * sin_term;

        let dz_dy = A * exp1 * exp2 * dr_dy;
        Vec3::new(0.0, 1.0, dz_dy)
    }

    fn du(&self) -> impl Geometry {
        PulseD2 { pulse: self.pulse.clone(), axis: Axis::UV }
    }

    fn dv(&self) -> impl Geometry {
        PulseD2 { pulse: self.pulse.clone(), axis: Axis::V }
    }
}

// ===== Second-order derivatives =====
enum Axis {
    U,
    V,
    UV,
}

struct PulseD2 {
    pulse: Pulse,
    axis: Axis,
}

impl Geometry for PulseD2 {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        let (x, y) = (p.x, p.y);
        let r = p.norm();
        let r2 = r * r;

        let (dr_d1, dr_d2, d2r_d1d2) = match self.axis {
            Axis::U => {
                let dx = x / r;
                let d2x = (r2 - x * x) / (r2 * r);
                (dx, dx, d2x)
            }
            Axis::V => {
                let dy = y / r;
                let d2y = (r2 - y * y) / (r2 * r);
                (dy, dy, d2y)
            }
            Axis::UV => {
                let dx = x / r;
                let dy = y / r;
                let dxy = -x * y / (r2 * r);
                (dx, dy, dxy)
            }
        };

        let s = r - self.pulse.r0 - self.pulse.c * self.pulse.t;
        let exp1 = (-self.pulse.sigma * self.pulse.sigma * s * s).exp();
        let exp2 = (-self.pulse.lambda * r).exp();
        let sin_term = (self.pulse.omega * r).sin();
        let cos_term = (self.pulse.omega * r).cos();
        let A = self.pulse.omega * cos_term
            - 2.0 * self.pulse.sigma * self.pulse.sigma * s * sin_term
            - self.pulse.lambda * sin_term;
        let B = exp1 * exp2;

        let dA_dr = -self.pulse.omega * self.pulse.omega * sin_term
            - 2.0 * self.pulse.sigma * self.pulse.sigma * sin_term
            - 2.0 * self.pulse.sigma * self.pulse.sigma * s * self.pulse.omega * cos_term
            - self.pulse.lambda * self.pulse.omega * cos_term
            + self.pulse.lambda * 2.0 * self.pulse.sigma * self.pulse.sigma * s * sin_term
            + self.pulse.lambda * self.pulse.lambda * sin_term;

        let d2z = (dA_dr * dr_d1 * dr_d2 + A * d2r_d1d2) * B;

        Vec3::new(0.0, 0.0, d2z)
    }
}
