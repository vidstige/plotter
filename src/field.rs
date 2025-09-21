use std::ops::{Add, Index, Mul};

use crate::resolution::Resolution;

pub struct Field<T> {
    pub resolution: Resolution,
    pub values: Vec<T>,
}

impl<T> Field<T> {
    pub fn width(&self) -> usize { self.resolution.width as usize }
    pub fn height(&self) -> usize { self.resolution.height as usize }
}

impl<T> Index<(usize, usize)> for Field<T> {
    type Output = T;
    fn index<'a>(&'a self, (x, y): (usize, usize)) -> &'a T {
        &self.values[x + y * self.width()]
    }
}

impl<T: Clone> Field<T> {
    pub fn from_buffer(resolution: Resolution, buffer: &Vec<T>) -> Field<T> {
        Field { resolution, values: buffer.to_owned() }
    }
    pub fn fill(resolution: Resolution, value: T) -> Field<T> {
        let area = resolution.area();
        Field { resolution, values: vec![value; area] }
    }
}

impl<T> Add for Field<T>
where
    T: Add<Output = T>,
{
    type Output = Field<T>;

    fn add(self, rhs: Field<T>) -> Field<T> {
        let Field { resolution, values } = self;
        let Field { resolution: rhs_resolution, values: rhs_values } = rhs;

        assert_same_resolution(&resolution, &rhs_resolution);
        combine_fields(resolution, values, rhs_values, |lhs, rhs| lhs + rhs)
    }
}

impl<'a, T> Add<&'a Field<T>> for Field<T>
where
    T: Add<Output = T> + Clone,
{
    type Output = Field<T>;

    fn add(self, rhs: &'a Field<T>) -> Field<T> {
        let Field { resolution, values } = self;

        assert_same_resolution(&resolution, &rhs.resolution);
        combine_fields(resolution, values, rhs.values.iter().cloned(), |lhs, rhs| lhs + rhs)
    }
}

impl<T, S> Mul<S> for Field<T>
where
    T: Mul<S, Output = T>,
    S: Clone,
{
    type Output = Field<T>;

    fn mul(self, rhs: S) -> Field<T> {
        let Field { resolution, values } = self;
        scale_field(resolution, values, rhs)
    }
}

impl<'a, T, S> Mul<S> for &'a Field<T>
where
    T: Mul<S, Output = T> + Clone,
    S: Clone,
{
    type Output = Field<T>;

    fn mul(self, rhs: S) -> Field<T> {
        scale_field(self.resolution.clone(), self.values.iter().cloned(), rhs)
    }
}

fn assert_same_resolution(lhs: &Resolution, rhs: &Resolution) {
    assert_eq!(lhs.width, rhs.width, "field width mismatch");
    assert_eq!(lhs.height, rhs.height, "field height mismatch");
}

fn combine_fields<T, L, R, F>(resolution: Resolution, lhs: L, rhs: R, mut f: F) -> Field<T>
where
    L: IntoIterator<Item = T>,
    R: IntoIterator<Item = T>,
    F: FnMut(T, T) -> T,
{
    let lhs_iter = lhs.into_iter();
    let rhs_iter = rhs.into_iter();
    Field { resolution, values: lhs_iter.zip(rhs_iter).map(|(l, r)| f(l, r)).collect() }
}

fn scale_field<T, S, I>(resolution: Resolution, values: I, scalar: S) -> Field<T>
where
    I: IntoIterator<Item = T>,
    S: Clone,
    T: Mul<S, Output = T>,
{
    let iter = values.into_iter();
    Field { resolution, values: iter.map(|value| value * scalar.clone()).collect() }
}
