use crate::resolution::Resolution;

pub struct Buffer {
    pub resolution: Resolution,
    pub pixels: Vec<u8>,
}

impl Buffer {
    fn new(resolution: Resolution) -> Buffer {
        Buffer { resolution, pixels: vec![0; resolution.area()]}
    }
}

pub fn pixel(target: &mut Buffer, x: i32, y: i32, gray: u8) {
    if x < 0 || x >= target.resolution.width || y < 0 || y >= target.resolution.height {
        return;
    }
    let stride = target.resolution.width;
    let index = (x + y * stride) as usize;
    target.pixels[index] = gray;
}

pub fn gray(intensity: f32) -> u8 {
    (intensity.clamp(0.0, 1.0) * 255.0) as u8
}
