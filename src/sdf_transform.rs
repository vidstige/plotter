use distance_transform::{dt2d, BoolGrid};
use tiny_skia::Pixmap;

use crate::{field::Field, resolution::Resolution};

/// Builds a signed distance `Field` from a binary alpha mask.
pub fn sdf_from_pixmap(pixmap: &Pixmap) -> Field<f32> {
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;

    let mut mask = BoolGrid::new(width, height);
    let mut inverse_mask = BoolGrid::new(width, height);

    for (idx, pixel) in pixmap.pixels().iter().enumerate() {
        let inside = pixel.alpha() > 127;
        let x = idx % width;
        let y = idx / width;
        mask.set(x, y, inside);
        inverse_mask.set(x, y, !inside);
    }

    let outside = dt2d(&mask);
    let inside = dt2d(&inverse_mask);

    let mut values = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let outside_dist = outside.get_unchecked(x, y).sqrt() as f32;
            let inside_dist = inside.get_unchecked(x, y).sqrt() as f32;
            values.push(outside_dist - inside_dist);
        }
    }

    Field {
        resolution: Resolution::new(pixmap.width(), pixmap.height()),
        values,
    }
}
