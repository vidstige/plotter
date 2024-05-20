use rand::Rng;
use rand::rngs::ThreadRng;
use svg::{Document, Node};
use svg::node::element::Group;

type ViewBox = (i32, i32, i32, i32);
const A4_PORTRAIT: ViewBox = (0, 0, 210, 297);


fn pad(view_box: ViewBox, pad: i32) -> ViewBox {
    let (x, y, w, h) = view_box;
    (x + pad, y + pad, w - 2 * pad, h - 2 * pad)
}

struct Polyline {
    points: Vec<Point2>,
}
impl Polyline {
    fn new() -> Polyline {
        Polyline { points: Vec::new() }
    }
    fn add(&mut self, point: Point2) {
        self.points.push(point);
    }
    fn length(&self) -> f32 {
        let mut length = 0.0;
        for i in 1..self.points.len() + 1 {
            let dx = self.points[i % self.points.len()].x - self.points[i - 1].x;
            let dy = self.points[i % self.points.len()].y - self.points[i - 1].y;
            length += (dx * dx + dy * dy).sqrt();
        }
        length
    }
}

fn as_node(polyline: &Polyline) -> String {
    let points: Vec<_> = polyline.points.iter().map(|p| (p.x, p.y)).map(|(x, y)| format!("{x} {y}")).collect();
    points.join(" ")
}

#[derive(Copy, Clone)]
struct Point2 {    
    x: f32,
    y: f32,
}
impl Point2 {
    fn new(x: f32, y: f32) -> Point2 {
        Point2 { x, y }
    }
    fn minus(&self, other: &Point2) -> Vector2 {
        Vector2::new(self.x - other.x, self.y - other.y)
    }

    fn add(&self, delta: Vector2) -> Point2 {
        Point2 { x: self.x + delta.x, y: self.y + delta.y }
    }
}

struct Vector2 {
    x: f32,
    y: f32,
}
impl Vector2 {
    fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }
    fn cross(&self) -> Vector2 {
        Vector2 { x: -self.y, y: self.x }
    }
    fn norm2(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    fn norm(&self) -> f32 {
        self.norm2().sqrt()
    }

    fn scale(&self, k: f32) -> Vector2 {
        Vector2 { x: self.x * k, y: self.y * k }
    }

    fn add(&self, vector: Vector2) -> Vector2 {
        Vector2 { x: self.x + vector.x, y: self.y + vector.y }
    }
}

struct Spiral {
    center: Point2,
}

impl Spiral {
    fn new(center: Point2) -> Spiral {
        Spiral { center }
    }
    fn at(&self, p: Point2) -> Vector2 {
        p.minus(&self.center).cross().scale(3.14).add(p.minus(&self.center))
    }
}

fn random_in(rng: &mut ThreadRng, view_box: &(i32, i32, i32, i32)) -> Point2 {
    Point2::new(
        rng.gen::<f32>() * (view_box.2 - view_box.0) as f32 + view_box.0 as f32,
        rng.gen::<f32>() * (view_box.3 - view_box.1) as f32 + view_box.1 as f32,
    )
}

fn contains(view_box: &ViewBox, point: &Point2) -> bool {
    let (x, y, w, h) = view_box;
    point.x > *x as f32 && point.y > *y as f32 && point.x < (x + w) as f32 && point.y < (y + h) as f32
}

fn main() {
    let field = Spiral::new(Point2::new(0.5 * 210.0, 0.5 * 297.0));

    let mut group = Group::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1);
    
    // compute drawing area
    let area = pad(A4_PORTRAIT, 20);
    let mut rng = rand::thread_rng();
    let max_step = 2.0;
    for _ in 0..256 {
        let mut polyline = Polyline::new();
        let mut point = random_in(&mut rng, &area);
        while polyline.length() < 64.0 {
            if !contains(&area, &point) {
                break;
            }
            polyline.add(point);
            let delta = field.at(point);
            let norm = delta.norm();
            let step = norm.min(max_step);
            point = point.add(delta.scale(step / norm));
            
        }
        group.append(svg::node::element::Polyline::new().set("points", as_node(&polyline)));
    }

    let document = Document::new()
        .set("viewBox", A4_PORTRAIT)
        .add(group);

    svg::save("image.svg", &document).unwrap();
}
