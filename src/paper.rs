use std::collections::HashSet;
use std::io;
use std::ops::Sub;

use nalgebra_glm::Vec2;
use svg::node::element::Group;
use svg::{Document, Node};

use crate::polyline::Polyline2;

pub type ViewBox = (i32, i32, i32, i32);
pub const A4_PORTRAIT: ViewBox = (0, 0, 210, 297);
pub const A4_LANDSCAPE: ViewBox = (0, 0, 297, 210);

pub fn viewbox_aspect(view_box: ViewBox) -> f32 {
    view_box.2 as f32 / view_box.3 as f32
}

pub fn pad(view_box: ViewBox, pad: i32) -> ViewBox {
    let (x, y, w, h) = view_box;
    (x + pad, y + pad, w - 2 * pad, h - 2 * pad)
}

pub fn contains(view_box: &ViewBox, point: &Vec2) -> bool {
    let (x, y, w, h) = view_box;
    point.x > *x as f32 && point.y > *y as f32 && point.x < (x + w) as f32 && point.y < (y + h) as f32
}



pub struct Paper {
    pub view_box: ViewBox,
    pub pen: f32,
    polylines: Vec<Polyline2>,
}

fn as_node(polyline: &Polyline2) -> String {
    let points: Vec<_> = polyline.points.iter().map(|p| (p.x, p.y)).map(|(x, y)| format!("{x} {y}")).collect();
    points.join(" ")
}

impl Paper {
    pub fn new(view_box: ViewBox, pen: f32) -> Paper {
        Paper { view_box, pen, polylines: Vec::new() }
    }

    pub fn add(&mut self, polyline: Polyline2) {
        self.polylines.push(polyline);
    }

    fn distance_to(&self, point: Vec2, index: usize) -> f32 {
        if let Some(start) = self.polylines[index].points.first() {
            point.sub(start).norm()
        } else {
            f32::INFINITY
        }
    }

    // total travel length for pen
    pub fn length(&self) -> f32 {
        let mut len = 0.0;
        let mut pen = Vec2::new(0.0, self.view_box.2 as f32);
        for polyline in &self.polylines {
            // distance from pen to first point
            len += pen.sub(polyline.points.first().unwrap_or(&pen)).norm();
            len += polyline.length();
            pen = *polyline.points.last().unwrap_or(&pen);
        }
        len
    }

    // re-orders poly-lines for faster plotting
    pub fn optimize(&mut self) {
        // Simple greedy algorithm for the travelling salesmen

        // start at idrawpenplotter home (top right)
        let mut current = Vec2::new(0.0, self.view_box.2 as f32);
        let mut unvisited: HashSet<usize> = HashSet::from_iter(0..self.polylines.len());
        let mut path = Vec::new();
        while !unvisited.is_empty() {
            // find shortest distance to polyline start
            let distances: Vec<_> = unvisited
                .iter()
                .map(|index| (self.distance_to(current, *index), *index))
                .collect();
            let (_, index) = distances.iter().min_by(|(a, _), (b, _)| a.total_cmp(b)).unwrap();
            unvisited.remove(index);
            // update path
            path.push(*index);
            // move current point to end (or do nothing for empty polylines)
            current = *self.polylines[*index].points.last().unwrap_or(&current);
        }

        self.polylines = path
            .iter()
            .map(|index| self.polylines[*index].clone())
            .collect();
    }

    pub fn save(self, filename: &str) -> io::Result<()> {
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
