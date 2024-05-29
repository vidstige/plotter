use std::ops::Sub;

use nalgebra_glm::Vec2;

#[derive(Clone)]
pub struct Polyline2 {
    pub points: Vec<Vec2>,
}

impl Polyline2 {
    pub fn new() -> Polyline2 {
        Polyline2 { points: Vec::new() }
    }
    pub fn add(&mut self, point: Vec2) {
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