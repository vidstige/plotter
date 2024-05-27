pub type Resolution = (i32, i32);

pub fn aspect_ratio(resolution: Resolution) -> f32 {
    resolution.0 as f32 / resolution.1 as f32
}
fn area(resolution: Resolution) -> usize {
    let (width, height) = resolution;
    (width * height) as usize
}

pub struct Buffer {
    pub resolution: Resolution,
    pub pixels: Vec<u8>,
}

impl Buffer {
    fn new(resolution: Resolution) -> Buffer {
        Buffer { resolution, pixels: vec![0; area(resolution)]}
    }
}

pub fn pixel(target: &mut Buffer, x: i32, y: i32, gray: u8) {
    if x < 0 || x >= target.resolution.0 || y < 0 || y >= target.resolution.1 {
        return;
    }
    let stride = target.resolution.0;
    let index = (x + y * stride) as usize;
    target.pixels[index] = gray;
}

pub fn gray(intensity: f32) -> u8 {
    (intensity.clamp(0.0, 1.0) * 255.0) as u8
}
