mod wasm;
pub use wasm::*;

use std::collections::{btree_set, BTreeSet};

type Coord = (u16, u16);

type Set = BTreeSet<Coord>;

pub struct Game {
    gen: usize,
    curr: Set,
    next: Set,
    width: u16,
    height: u16,
}

impl Game {
    pub fn new(width: u16, height: u16) -> Game {
        Game {
            gen: 0,
            curr: Set::new(),
            next: Set::new(),
            width,
            height,
        }
    }

    pub fn set(&mut self, x: u16, y: u16) {
        if x < self.width && y < self.height {
            self.curr.insert((x, y));
        }
    }

    pub fn step(&mut self) {
        self.next.clear();

        for c in self.curr.iter() {
            match self.neighbours(*c) {
                2 | 3 => {
                    self.next.insert(*c);
                }
                _ => {}
            }

            for c in self.iter(*c) {
                if self.neighbours(c) == 3 {
                    self.next.insert(c);
                }
            }
        }

        std::mem::swap(&mut self.curr, &mut self.next);
        self.gen += 1;
    }

    fn neighbours(&self, c: Coord) -> usize {
        self.iter(c).filter(|x| self.curr.contains(x)).count()
    }

    fn iter(&self, (x, y): Coord) -> impl std::iter::Iterator<Item = Coord> {
        // Bounds checking since x, y are unsigned.
        let i = if x > 0 { x - 1 } else { 0 };
        let j = if y > 0 { y - 1 } else { 0 };
        let mi = std::cmp::min(x + 1, self.width - 1);
        let mj = std::cmp::min(y + 1, self.height - 1);

        (i..=mi)
            .flat_map(move |i| (j..=mj).map(move |j| (i, j)))
            .filter(move |(i, j)| *i != x || *j != y)
    }
}

impl<'a> IntoIterator for &'a Game {
    type Item = &'a Coord;
    type IntoIter = btree_set::Iter<'a, Coord>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.curr).into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        let g = Game::new(1, 1);
        assert!(g.iter((0, 0)).count() == 0);

        let g = Game::new(2, 2);
        let mut it = g.iter((0, 0));
        assert!(it.next() == Some((0, 1)));
        assert!(it.next() == Some((1, 0)));
        assert!(it.next() == Some((1, 1)));
        assert!(it.next() == None);

        let g = Game::new(3, 3);
        let mut it = g.iter((1, 1));
        assert!(it.next() == Some((0, 0)));
        assert!(it.next() == Some((0, 1)));
        assert!(it.next() == Some((0, 2)));
        assert!(it.next() == Some((1, 0)));
        assert!(it.next() == Some((1, 2)));
        assert!(it.next() == Some((2, 0)));
        assert!(it.next() == Some((2, 1)));
        assert!(it.next() == Some((2, 2)));
        assert!(it.next() == None);
    }

    #[test]
    fn lone_block() {
        let mut g = Game::new(1, 1);
        g.set(0, 0);

        let it = g.into_iter();
        assert!([(0, 0)].iter().eq(it));

        g.step();
        let it = g.into_iter();
        assert!(it.count() == 0);
    }

    #[test]
    fn triplet() {
        let mut g = Game::new(3, 3);
        g.set(1, 0);
        g.set(1, 1);
        g.set(1, 2);

        let it = g.into_iter();
        assert!([(1, 0), (1, 1), (1, 2)].iter().eq(it));

        g.step();
        let it = g.into_iter();
        assert!([(0, 1), (1, 1), (2, 1)].iter().eq(it));

        g.step();
        let it = g.into_iter();
        assert!([(1, 0), (1, 1), (1, 2)].iter().eq(it));
    }
}
