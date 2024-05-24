use paper::{ViewBox, A4_PORTRAIT, Paper};
use polyline::{Point2, Vector2, Polyline};
use rand::Rng;
use rand::rngs::ThreadRng;

mod paper;
mod polyline;

fn pad(view_box: ViewBox, pad: i32) -> ViewBox {
    let (x, y, w, h) = view_box;
    (x + pad, y + pad, w - 2 * pad, h - 2 * pad)
}

fn random_in(rng: &mut ThreadRng, view_box: &ViewBox) -> Point2 {
    Point2::new(
        rng.gen::<f32>() * (view_box.2 - view_box.0) as f32 + view_box.0 as f32,
        rng.gen::<f32>() * (view_box.3 - view_box.1) as f32 + view_box.1 as f32,
    )
}

fn contains(view_box: &ViewBox, point: &Point2) -> bool {
    let (x, y, w, h) = view_box;
    point.x > *x as f32 && point.y > *y as f32 && point.x < (x + w) as f32 && point.y < (y + h) as f32
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
    
fn main() {
    let mut paper = Paper::new();
    // compute drawing area
    let area = pad(A4_PORTRAIT, 20);

    let mut rng = rand::thread_rng();
    let field = Spiral::new(Point2::new(0.5 * 210.0, 0.5 * 297.0));
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
        paper.add(&polyline);
    }
    
    paper.save("image.svg");
}
