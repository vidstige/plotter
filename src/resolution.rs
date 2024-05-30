pub struct Resolution {
    pub width: i32,
    pub height: i32,
}

impl Resolution {
    pub fn new(width: i32, height: i32) -> Resolution {
        Resolution { width, height }
    }

    pub fn aspect_ratio(self) -> f32 {
        self.width as f32 / self.height as f32
    }
    pub fn area(self) -> usize {
        (self.width * self.height) as usize
    }
}
