use svg::Document;
use svg::node::element::Polyline;

const A4_PORTRAIT: (i32, i32, i32, i32) = (0, 0, 210, 297);

fn main() {
    let polyline = Polyline::new()
        .set("points", "10,10, 20,30, 40,30")
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1);
    

    let document = Document::new()
        .set("viewBox", A4_PORTRAIT)
        .add(polyline);

    svg::save("image.svg", &document).unwrap();
}
