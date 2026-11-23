use crate::tile::Tile;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Hand(pub u32);

impl Hand {
    pub fn from_tiles(tiles: &[Tile]) -> Hand {
        let mut h = Hand::default();
        for t in tiles {
            h.insert(*t);
        }
        h
    }
    pub fn contains(self, t: Tile) -> bool {
        (self.0 >> t.index()) & 1 == 1
    }
    pub fn insert(&mut self, t: Tile) {
        self.0 |= 1 << t.index();
    }
    pub fn remove(&mut self, t: Tile) {
        self.0 &= !(1 << t.index());
    }
    pub fn len(self) -> u32 {
        self.0.count_ones()
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub fn iter(self) -> impl Iterator<Item = Tile> {
        (0..28u8)
            .filter(move |&i| (self.0 >> i) & 1 == 1)
            .map(Tile::from_index)
    }
    pub fn pips(self) -> u32 {
        self.iter().map(|t| t.pips() as u32).sum()
    }
    pub fn doubles(self) -> u32 {
        self.iter().filter(|t| t.is_double()).count() as u32
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for t in self.iter() {
            write!(f, "{} ", t)?;
        }
        Ok(())
    }
}
