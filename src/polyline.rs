use nalgebra_glm::Vec2;

#[derive(Clone)]
pub struct Polyline {
    pub points: Vec<Vec2>,
}

impl Polyline {
    pub fn new() -> Polyline {
        Polyline { points: Vec::new() }
    }
    pub fn add(&mut self, point: Vec2) {
        self.points.push(point);
    }
    pub fn length(&self) -> f32 {
        let mut length = 0.0;
        for i in 1..self.points.len() + 1 {
            let dx = self.points[i % self.points.len()].x - self.points[i - 1].x;
            let dy = self.points[i % self.points.len()].y - self.points[i - 1].y;
            length += (dx * dx + dy * dy).sqrt();
        }
        length
    }
}