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
    group: Group,
}

fn as_node(polyline: &Polyline) -> String {
    let points: Vec<_> = polyline.points.iter().map(|p| (p.x, p.y)).map(|(x, y)| format!("{x} {y}")).collect();
    points.join(" ")
}

impl Paper {
    pub fn new(view_box: ViewBox) -> Paper {
        let group = Group::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1);
        Paper { view_box, group }
    }

    pub(crate) fn add(&mut self, polyline: &crate::Polyline) {
        self.group.append(svg::node::element::Polyline::new().set("points", as_node(&polyline)))
    }

    pub(crate) fn save(self, filename: &str) -> io::Result<()> {
        let document = Document::new()
            .set("width", format!("{}mm", self.view_box.2))
            .set("height", format!("{}mm", self.view_box.3))
            .set("viewBox", self.view_box)
            .add(self.group);

        svg::save(filename, &document)
    }
}
