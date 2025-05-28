use nalgebra_glm::{cross, normalize, Mat2x2, Vec2, Vec3};

struct DerivativeNotImplemented {

}
impl Geometry for DerivativeNotImplemented {
    fn evaluate(&self, p: &Vec2) -> Vec3 {
        todo!()
    }
}
impl DifferentiableGeometry for DerivativeNotImplemented {
    fn du(&self) -> impl DifferentiableGeometry { DerivativeNotImplemented {} }
    fn dv(&self) -> impl DifferentiableGeometry { DerivativeNotImplemented {} }
}

pub trait Geometry {
    // maps a point on the surface p=(u,v) to a point in space (x, y, z)
    fn evaluate(&self, p: &Vec2) -> Vec3;
}

pub trait DifferentiableGeometry: Geometry {
    // returns the partial derivative (d/du) for the geometry
    fn du(&self) -> impl DifferentiableGeometry {
        DerivativeNotImplemented {}
    }
    // returns the partial derivative (d/dv) for the geometry
    fn dv(&self) -> impl DifferentiableGeometry {
        DerivativeNotImplemented {}
    }

    // evaluates metric tensor at p using derivatives dot product
    // can be overriden with analytic expression
    fn metric(&self, p: &Vec2) -> Mat2x2 {
        let du = self.du().evaluate(p);
        let dv = self.dv().evaluate(p);
        Mat2x2::new(
            du.dot(&du), du.dot(&dv),
            dv.dot(&du), dv.dot(&dv),
        )
    }
    /// Computes the normal vector at a given UV coordinate for any DifferentiableGeometry.
    fn normal(&self, uv: &Vec2) -> Vec3 {
        let du = self.du().evaluate(uv);
        let dv = self.dv().evaluate(uv);
        normalize(&cross(&du, &dv))
    }
}

// return Christoffel symbols with index k, i, j
pub fn compute_gamma(geometry: &impl DifferentiableGeometry, p: &Vec2) -> [[[f32; 2]; 2]; 2] {
    let metric = geometry.metric(p);
    let maybe_inverse_metric = metric.try_inverse();
    if maybe_inverse_metric.is_none() {
        println!("could not invert {:?}", metric);
    }
    let inverse_metric = maybe_inverse_metric.unwrap();
    // compute all second order partial derivatives
    let d2: [[Vec3; 2]; 2] = [ 
        [geometry.du().du().evaluate(p), geometry.du().dv().evaluate(p)],
        [geometry.dv().du().evaluate(p), geometry.dv().dv().evaluate(p)],
    ];
    // compute first order partial derivatives
    let d: [Vec3; 2] = [geometry.du().evaluate(p), geometry.dv().evaluate(p)];

    // compute tensor product gamma^k_ij = (d²R/du^i du^j) * (dR/du^l) * (g^-1)^lk
    // the index l is thus summed over
    let mut tmp = [[[0.0; 2]; 2]; 2];
    for k in 0..2 {
        for i in 0..2 {
            for j in 0..2 {
                for l in 0..2 {
                    tmp[k][i][j] += d2[i][j].dot(&d[l]) * inverse_metric[(l, k)];
                }
            }
        }
    }
    tmp
}

// Compute acceleration: a^k = Γ^k_ij v^i v^j
// This is the solution to the geodesic equation
pub fn acceleration(geometry: &impl DifferentiableGeometry, position: &Vec2, velocity: &Vec2) -> Vec2 {
    let gamma = compute_gamma(geometry, position);
    let mut a = Vec2::zeros();
    // tensor sum
    for k in 0..2 {
        for i in 0..2 {
            for j in 0..2 {
                a[k] += -gamma[k][i][j] * velocity[i] * velocity[j];
            }
        }
    }
    a
}
