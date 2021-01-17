use crate::world::{Coord, World};
use std::{cmp, default::Default, iter::Iterator};

pub struct View {
    orig: Coord,
    offset: (f64, f64), // [0, 1)
    cell_dim: (f64, f64),
    viewport_dim: (usize, usize),
    zoom: f64,
}

impl View {
    pub fn new() -> Self {
        View {
            orig: (0, 0),
            offset: (0., 0.),
            cell_dim: (10., 10.),
            viewport_dim: (0, 0),
            zoom: 1.,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.viewport_dim = (width, height);
    }

    pub fn trans(&mut self, pdx: isize, pdy: isize) {
        let dx = pdx as f64 / (self.cell_dim.0 * self.zoom) + self.offset.0;
        let dy = pdy as f64 / (self.cell_dim.1 * self.zoom) + self.offset.1;
        let dxf = dx.floor();
        let dyf = dy.floor();

        self.orig.0 = self.orig.0.wrapping_add(dxf as isize);
        self.orig.1 = self.orig.1.wrapping_add(dyf as isize);
        self.offset.0 = (dx - dxf).abs();
        self.offset.1 = (dy - dyf).abs();
    }

    pub fn scale(&mut self, factor: f64, center: (isize, isize)) {
        self.trans(-center.0, -center.1);
        self.zoom *= factor;
        self.trans(center.0, center.0);
    }

    pub fn gridlines_x(&self) -> impl Iterator<Item = usize> {
        let m = self.zoom * self.cell_dim.0;
        let b = (self.offset.0 * m) as usize;
        let n = (self.viewport_dim.0 as f64 / m) as usize;
        let m = m as usize;
        (0..=n).map(move |i| i * m + b)
    }

    pub fn gridlines_y(&self) -> impl Iterator<Item = usize> {
        let m = self.zoom * self.cell_dim.1;
        let b = (self.offset.1 * m) as usize;
        let n = (self.viewport_dim.1 as f64 / m) as usize;
        let m = m as usize;
        (0..=n).map(move |i| i * m + b)
    }

    /// Given a `World`, returns rectangles to draw in form (x, y, w, h).
    pub fn rects<'a>(
        &'a self,
        w: &'a World,
    ) -> impl Iterator<Item = (usize, usize, usize, usize)> + 'a {
        let xm = (self.zoom * self.cell_dim.0) as isize;
        let xo = (self.offset.0 * self.zoom * self.cell_dim.0) as isize;

        let ym = (self.zoom * self.cell_dim.1) as isize;
        let yo = (self.offset.1 * self.zoom * self.cell_dim.1) as isize;

        w.into_iter()
            .filter(move |&&c| self.in_view(c))
            .map(move |(x, y)| {
                let xa = (x - self.orig.0) * xm - xo;
                let ya = (y - self.orig.0) * ym - yo;
                let xb = cmp::min(self.viewport_dim.0 as isize, xa + xm);
                let yb = cmp::min(self.viewport_dim.1 as isize, ya + ym);
                let xa = cmp::max(0, xa);
                let ya = cmp::max(0, ya);
                (
                    xa as usize,
                    ya as usize,
                    (xb - xa) as usize,
                    (yb - ya) as usize,
                )
            })
    }

    pub fn in_view(&self, (x, y): Coord) -> bool {
        let w = (self.viewport_dim.0 as f64 / (self.zoom * self.cell_dim.0)) as isize;
        let w = self.orig.0.wrapping_add(w);
        let h = (self.viewport_dim.1 as f64 / (self.zoom * self.cell_dim.1)) as isize;
        let h = self.orig.1.wrapping_add(h);

        (x >= self.orig.0 || x <= w) && (y >= self.orig.1 || y <= h)
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}
