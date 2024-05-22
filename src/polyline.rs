
#[derive(Copy, Clone)]
pub struct Point2 {    
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Point2 {
        Point2 { x, y }
    }
    pub fn minus(&self, other: &Point2) -> Vector2 {
        Vector2::new(self.x - other.x, self.y - other.y)
    }

    pub fn add(&self, delta: Vector2) -> Point2 {
        Point2 { x: self.x + delta.x, y: self.y + delta.y }
    }
}

pub struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }
    pub fn cross(&self) -> Vector2 {
        Vector2 { x: -self.y, y: self.x }
    }
    pub fn norm2(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    pub fn norm(&self) -> f32 {
        self.norm2().sqrt()
    }
    pub fn scale(&self, k: f32) -> Vector2 {
        Vector2 { x: self.x * k, y: self.y * k }
    }
    pub fn add(&self, vector: Vector2) -> Vector2 {
        Vector2 { x: self.x + vector.x, y: self.y + vector.y }
    }
}

pub struct Polyline {
    pub points: Vec<Point2>,
}
impl Polyline {
    pub fn new() -> Polyline {
        Polyline { points: Vec::new() }
    }
    pub fn add(&mut self, point: Point2) {
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