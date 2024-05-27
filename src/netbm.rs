use std::{fs::File, io::{Write, self}};

use crate::buffer::Buffer;

pub fn save_pgm(filename: &str, buffer: &Buffer) -> io::Result<()> {
    let mut w = File::create(filename)?;
    writeln!(&mut w, "P5 {} {} {}", buffer.resolution.0, buffer.resolution.1, 255)?;
    w.write_all(&buffer.pixels)
}