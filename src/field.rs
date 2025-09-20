use std::ops::Index;

use crate::resolution::Resolution;

pub struct Field<T> {
    pub resolution: Resolution,
    pub values: Vec<T>,
}

impl<T> Field<T> {
    pub fn width(&self) -> usize { return self.resolution.width as usize }
    pub fn height(&self) -> usize { return self.resolution.height as usize }
}

impl<T> Index<(usize, usize)> for Field<T> {
    type Output = T;
    fn index<'a>(&'a self, (x, y): (usize, usize)) -> &'a T {
        &self.values[x + y * self.width() ]
    }
}

impl<T: Clone> Field<T> {
    pub fn from_buffer(resolution: Resolution, buffer: &Vec<T>) -> Field<T> {
        Field {
            resolution,
            values: buffer.to_owned(),
        }
    }
    pub fn fill(resolution: Resolution, value: T) -> Field<T> {
        let area = resolution.area();
        Field {
            resolution,
            values: vec![value; area]
        }
    }
}