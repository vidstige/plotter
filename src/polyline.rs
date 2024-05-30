use std::ops::Sub;

//use nalgebra_glm::Vec2;
use nalgebra_glm::TVec;

#[derive(Clone)]
pub struct Polyline<const N: usize> {
    pub points: Vec<TVec<f32, N>>,
}

impl<const N: usize> Polyline<N> {
    pub fn new() -> Polyline<N> {
        Polyline { points: Vec::new() }
    }
    pub fn add(&mut self, point: TVec<f32, N>) {
        self.points.push(point);
    }
    pub fn length(&self) -> f32 {
        let mut length = 0.0;
        for i in 1..self.points.len() + 1 {
            let pi = self.points[i % self.points.len()];
            let pj = self.points[i - 1];
            length += pi.sub(&pj).norm();
        }
        length
    }
}

pub type Polyline2 = Polyline<2>;
pub type Polyline3 = Polyline<3>;
