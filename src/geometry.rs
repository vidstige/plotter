use nalgebra_glm::{Vec3, Vec2, Mat2x2};

struct DerivativeNotImplemented {

}
impl Geometry for DerivativeNotImplemented {
    fn evaluate(&self, _p: &Vec2) -> Vec3 {
        todo!()
    }
    fn du(&self) -> impl Geometry { DerivativeNotImplemented {} }
    fn dv(&self) -> impl Geometry { DerivativeNotImplemented {} }
}

pub trait Geometry {
    // maps a point on the surface p=(u,v) to a point in space (x, y, z)
    fn evaluate(&self, p: &Vec2) -> Vec3;

    // returns the partial derivative (d/du) for the geometry
    fn du(&self) -> impl Geometry {
        DerivativeNotImplemented {}
    }
    // returns the partial derivative (d/dv) for the geometry
    fn dv(&self) -> impl Geometry {
        DerivativeNotImplemented {}
    }

    // evaluates metric tensor at p using derivatives dot product
    // can be overriden with analytic expression
    fn metric(&self, p: &Vec2) -> Mat2x2 {
        let du = self.du().evaluate(p);
        let dv = self.dv().evaluate(p);
        Mat2x2::new(
            du.dot(&du), du.dot(&dv),
            du.dot(&dv), dv.dot(&dv),
        )
    }
}