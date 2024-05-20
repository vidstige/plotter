use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::{Data, Command, Position};

const A4_PORTRAIT: (i32, i32, i32, i32) = (0, 0, 210, 297);

struct Canvas {
    data: Data,
}

impl Canvas {
    fn new() -> Canvas {
        Canvas { data: Data::new() }
    }
    fn move_to(&mut self, x: i32, y: i32) {
        self.data.append(Command::Move(Position::Absolute, (x, y).into()));
    }
    
    fn line_to(&mut self, x: i32, y: i32) {
        self.data.append(Command::Line(Position::Relative, (x, y).into()));
    }
    
    fn close(&mut self) {
        self.data.append(Command::Close);
    }
}

fn main() {
    let mut canvas = Canvas::new();
    canvas.move_to(10, 10);
    canvas.line_to(0, 50);
    canvas.line_to(50, 0);
    canvas.line_to(0, -50);
    canvas.close();

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("d", canvas.data);

    let document = Document::new()
        .set("viewBox", A4_PORTRAIT)
        .add(path);

    svg::save("image.svg", &document).unwrap();
}
