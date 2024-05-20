use rand::Rng;
use rand::rngs::ThreadRng;
use svg::{Document, Node};
use svg::node::element::Group;

const A4_PORTRAIT: (i32, i32, i32, i32) = (0, 0, 210, 297);


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
}

struct Spiral {
    center: Point2,
}

impl Spiral {
    fn new(center: Point2) -> Spiral {
        Spiral { center }
    }
    fn at(&self, p: Point2) -> Vector2 {
        p.minus(&self.center).cross()
    }
}

fn random_in(rng: &mut ThreadRng, view_box: &(i32, i32, i32, i32)) -> Point2 {
    Point2::new(
        rng.gen::<f32>() * (view_box.2 - view_box.0) as f32 + view_box.0 as f32,
        rng.gen::<f32>() * (view_box.3 - view_box.1) as f32 + view_box.1 as f32,
    )
}

fn main() {
    let field = Spiral::new(Point2::new(0.5 * 210.0, 0.5 * 297.0));

    let mut group = Group::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1);
    
    //let polyline = Polyline::new().set("points", "10,10, 20,30, 40,30");
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let mut polyline = Polyline::new();
        let mut point = random_in(&mut rng, &A4_PORTRAIT);
        while polyline.length() < 5.0 {
            polyline.add(point);
            let delta = field.at(point);
            
            point = point.add(delta);
        }
        group.append(svg::node::element::Polyline::new().set("points", as_node(&polyline)));
    }

    let document = Document::new()
        .set("viewBox", A4_PORTRAIT)
        .add(group);

    svg::save("image.svg", &document).unwrap();
}
