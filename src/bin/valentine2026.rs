use std::io::{self, ErrorKind, Write};

use plotter::resolution::Resolution;
use tiny_skia::{Color, Pixmap};

const FRAME_COUNT: usize = 256;
const FPS: f32 = 30.0;

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, message.into())
}

fn parse_time_arg(raw: &str) -> io::Result<f32> {
    raw.parse::<f32>()
        .map_err(|_| invalid_input(format!("invalid time value: {raw}")))
}

fn parse_args() -> io::Result<Option<f32>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Ok(None);
    }

    match args.as_slice() {
        [flag, value] if flag == "-t" || flag == "--time" => parse_time_arg(value).map(Some),
        [value] if value.starts_with("--time=") => {
            let Some(raw) = value.strip_prefix("--time=") else {
                return Err(invalid_input("failed to parse --time argument"));
            };
            parse_time_arg(raw).map(Some)
        }
        _ => Err(invalid_input("usage: valentine2026 [-t|--time <seconds>]")),
    }
}

fn render_frame(pixmap: &mut Pixmap, _time: f32) {
    pixmap.fill(Color::WHITE);
}

fn main() -> io::Result<()> {
    let time = parse_args()?;
    let resolution = Resolution::new(720, 720);
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    let mut output = io::stdout().lock();

    if let Some(time) = time {
        render_frame(&mut pixmap, time);
        output.write_all(pixmap.data())?;
        output.flush()?;
        return Ok(());
    }

    for frame in 0..FRAME_COUNT {
        let time = frame as f32 / FPS;
        render_frame(&mut pixmap, time);
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
