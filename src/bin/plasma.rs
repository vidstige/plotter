use std::io::{self, Write};

use nalgebra_glm::Vec3;
use plotter::{resolution::Resolution, simplex::simplex3};
use tiny_skia::{Pixmap, PremultipliedColorU8};

fn gray(intensity: f32) -> PremultipliedColorU8 {
    let i = (intensity * 256.0) as u8;
    PremultipliedColorU8::from_rgba(i, i, i, 255).unwrap()
}

fn main() -> io::Result<()> {    
    let mut output = io::stdout().lock();

    let resolution = Resolution::new(720, 720);
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();

    for i in 0..pixmap.height() {
        for j in 0..pixmap.width() {
            let x = j as f32;
            let y = i as f32;
            let p = Vec3::new(x, y, 0.0);
            let intensity = simplex3(&p);
            let index = (i * pixmap.width() + j) as usize;
            let color = gray(intensity);
            pixmap.pixels_mut()[index] = color;
        }
    }

    output.write_all(pixmap.data())?;
    output.flush()?;

    Ok(())
}
