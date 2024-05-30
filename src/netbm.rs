use std::{fs::File, io::{Write, self}};

use crate::buffer::Buffer;

pub fn save_pgm(filename: &str, buffer: &Buffer) -> io::Result<()> {
    let mut w = File::create(filename)?;
    writeln!(&mut w, "P5 {} {} {}", buffer.resolution.width, buffer.resolution.height, 255)?;
    w.write_all(&buffer.pixels)
}