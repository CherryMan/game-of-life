use std::{
    collections::{btree_set, BTreeSet},
    default::Default,
};

pub type Num = isize;
pub type Coord = (Num, Num);

#[derive(Default)]
pub struct World {
    curr: BTreeSet<Coord>,
    next: BTreeSet<Coord>,
}

impl World {
    pub fn new() -> World {
        World::default()
    }

    pub fn reset(&mut self) {
        self.curr.clear();
    }

    pub fn set(&mut self, x: Num, y: Num) {
        self.curr.insert((x, y));
    }

    pub fn unset(&mut self, x: Num, y: Num) {
        self.curr.remove(&(x, y));
    }

    pub fn is_set(&self, x: Num, y: Num) -> bool {
        self.curr.contains(&(x, y))
    }

    pub fn step(&mut self) {
        self.next.clear();

        for c in self.curr.iter() {
            if self.count_neighbours(*c) == 2 {
                self.next.insert(*c);
            }

            for dy in -1..=1 {
                for dx in -1..=1 {
                    let c = (c.0.wrapping_add(dx), c.1.wrapping_add(dy));
                    if self.count_neighbours(c) == 3 {
                        self.next.insert(c);
                    }
                }
            }
        }

        std::mem::swap(&mut self.curr, &mut self.next);
    }

    fn count_neighbours(&self, (x, y): Coord) -> usize {
        const DELTAS: [Coord; 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        let mut r = 0;
        for (dx, dy) in DELTAS.iter() {
            r += self.is_set(x.wrapping_add(*dx), y.wrapping_add(*dy)) as usize;
        }
        r
    }
}

impl<'a> IntoIterator for &'a World {
    type Item = &'a Coord;
    type IntoIter = btree_set::Iter<'a, Coord>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.curr).iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lone_block() {
        let mut w = World::new();
        w.set(0, 0);

        assert!(w.into_iter().count() == 1);
        assert!(w.is_set(0, 0));

        w.step();
        assert!(w.into_iter().count() == 0);
    }

    #[test]
    fn blinker() {
        let mut w = World::new();
        w.set(0, -1);
        w.set(0, 0);
        w.set(0, 1);

        assert!(w.into_iter().count() == 3);
        assert!([(0, -1), (0, 0), (0, 1)]
            .iter()
            .all(|(x, y)| w.is_set(*x, *y)));

        w.step();

        assert!(w.into_iter().count() == 3);
        assert!([(-1, 0), (0, 0), (1, 0)]
            .iter()
            .all(|(x, y)| w.is_set(*x, *y)));

        w.step();
        assert!(w.into_iter().count() == 3);
        assert!([(0, -1), (0, 0), (0, 1)]
            .iter()
            .all(|(x, y)| w.is_set(*x, *y)));
    }

    #[test]
    fn glider() {
        let mut w = World::new();
        w.set(1, 0);
        w.set(2, 1);
        w.set(0, 2);
        w.set(1, 2);
        w.set(2, 2);

        assert!(w.into_iter().count() == 5);
        assert!([(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]
            .iter()
            .all(|(x, y)| w.is_set(*x, *y)));

        w.step();
        assert!(w.into_iter().count() == 5);
        assert!([(0, 1), (2, 1), (1, 2), (2, 2), (1, 3)]
            .iter()
            .all(|(x, y)| w.is_set(*x, *y)));

        w.step();
        assert!(w.into_iter().count() == 5);
        assert!([(2, 1), (0, 2), (2, 2), (1, 3), (2, 3)]
            .iter()
            .all(|(x, y)| w.is_set(*x, *y)));

        w.step();
        assert!(w.into_iter().count() == 5);
        assert!([(1, 1), (2, 2), (3, 2), (1, 3), (2, 3)]
            .iter()
            .all(|(x, y)| w.is_set(*x, *y)));

        w.step();
        assert!(w.into_iter().count() == 5);
        assert!([(2, 1), (3, 2), (1, 3), (2, 3), (3, 3)]
            .iter()
            .all(|(x, y)| w.is_set(*x, *y)));
    }
}
