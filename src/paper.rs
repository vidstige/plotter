use std::cmp::Ordering;
use std::io;

use svg::node::element::Group;
use svg::{Document, Node};

use crate::polyline::Polyline;

pub type ViewBox = (i32, i32, i32, i32);
pub const A4_PORTRAIT: ViewBox = (0, 0, 210, 297);
pub const A4_LANDSCAPE: ViewBox = (0, 0, 297, 210);

pub fn viewbox_aspect(view_box: ViewBox) -> f32 {
    view_box.2 as f32 / view_box.3 as f32
}


pub struct Paper {
    pub view_box: ViewBox,
    pub pen: f32,
    polylines: Vec<crate::Polyline>,
}

fn as_node(polyline: &Polyline) -> String {
    let points: Vec<_> = polyline.points.iter().map(|p| (p.x, p.y)).map(|(x, y)| format!("{x} {y}")).collect();
    points.join(" ")
}

fn compare_polylines(a: &Polyline, b: &Polyline) -> Ordering {
    if a.points.len() == 0 {
        return Ordering::Less;
    }
    if b.points.len() == 0 {
        return Ordering::Greater;
    }
    let a0 = a.points.first().unwrap();
    let b0 = b.points.first().unwrap();
    (a0.x, a0.y).partial_cmp(&(b0.x, b0.y)).unwrap()
}

impl Paper {
    pub fn new(view_box: ViewBox, pen: f32) -> Paper {
        Paper { view_box, pen, polylines: Vec::new() }
    }

    pub(crate) fn add(&mut self, polyline: crate::Polyline) {
        self.polylines.push(polyline);
    }

    // re-orders poly-lines for faster plotting
    pub(crate) fn optimize(&mut self) {
        self.polylines.sort_by(compare_polylines);
    }

    pub(crate) fn save(self, filename: &str) -> io::Result<()> {
        let mut group = Group::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", self.pen);
        for polyline in self.polylines {
            group.append(svg::node::element::Polyline::new().set("points", as_node(&polyline)));
        }
        let document = Document::new()
            .set("width", format!("{}mm", self.view_box.2))
            .set("height", format!("{}mm", self.view_box.3))
            .set("viewBox", self.view_box)
            .add(group);

        svg::save(filename, &document)
    }
}
