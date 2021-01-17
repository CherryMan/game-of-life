use crate::world::Coord;
use std::{cmp, default::Default, iter, iter::Iterator};

pub struct View {
    orig: Coord,
    offset: (f64, f64), // [0, 1)
    viewport_dim: (usize, usize),
    zoom: f64,
}

impl View {
    pub fn new() -> Self {
        View {
            orig: (0, 0),
            offset: (0., 0.),
            viewport_dim: (0, 0),
            zoom: 10.,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.viewport_dim = (width, height);
    }

    pub fn trans(&mut self, pdx: isize, pdy: isize) {
        let dx = -pdx as f64 / self.zoom + self.offset.0;
        let dy = -pdy as f64 / self.zoom + self.offset.1;
        let dxf = dx.floor();
        let dyf = dy.floor();

        self.orig.0 = self.orig.0.wrapping_add(dxf as isize);
        self.orig.1 = self.orig.1.wrapping_add(dyf as isize);
        self.offset.0 = (dx - dxf).abs();
        self.offset.1 = (dy - dyf).abs();
    }

    pub fn update_scale<F: FnOnce(f64) -> f64>(&mut self, f: F, center: (isize, isize)) {
        self.set_scale(f(self.zoom), center);
    }

    pub fn set_scale(&mut self, scale: f64, center: (isize, isize)) {
        self.trans(-center.0, -center.1);
        self.zoom = scale;
        self.trans(center.0, center.1);
    }

    pub fn gridlines_x(&self) -> impl Iterator<Item = usize> {
        let mut i = -1;
        let zoom = self.zoom;
        let offset = self.offset.0;
        let dim = self.viewport_dim.0;
        iter::from_fn(move || {
            i += 1;
            let r = (zoom * (i as f64 - offset)) as usize;
            if r <= dim {
                Some(r)
            } else {
                None
            }
        })
    }

    pub fn gridlines_y(&self) -> impl Iterator<Item = usize> {
        let mut i = -1;
        let zoom = self.zoom;
        let offset = self.offset.1;
        let dim = self.viewport_dim.1;
        iter::from_fn(move || {
            i += 1;
            let r = (zoom * (i as f64 - offset)) as usize;
            if r <= dim {
                Some(r)
            } else {
                None
            }
        })
    }

    /// Given an iterator over `Coord`s, returns rectangles to draw in form (x, y, w, h).
    pub fn rects<'a, T: Iterator<Item = &'a Coord> + 'a>(
        &'a self,
        cs: T,
    ) -> impl Iterator<Item = (usize, usize, usize, usize)> + 'a {
        cs.filter_map(move |(x, y)| {
            let xa = (x.wrapping_sub(self.orig.0) as f64 - self.offset.0) * self.zoom;
            let ya = (y.wrapping_sub(self.orig.1) as f64 - self.offset.1) * self.zoom;
            let xb = xa + self.zoom;
            let yb = ya + self.zoom;

            let xa = xa.max(0.) as usize;
            let ya = ya.max(0.) as usize;

            if xa >= self.viewport_dim.0 || ya >= self.viewport_dim.1 || xb <= 0. || yb <= 0. {
                None
            } else {
                let xb = cmp::min(self.viewport_dim.0, xb as usize);
                let yb = cmp::min(self.viewport_dim.1, yb as usize);
                Some((xa, ya, xb - xa, yb - ya))
            }
        })
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}
