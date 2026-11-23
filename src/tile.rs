use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Tile {
    pub a: u8,
    pub b: u8,
}

impl Tile {
    pub fn new(x: u8, y: u8) -> Self {
        if x <= y {
            Tile { a: x, b: y }
        } else {
            Tile { a: y, b: x }
        }
    }
    pub fn is_double(self) -> bool {
        self.a == self.b
    }
    pub fn pips(self) -> u8 {
        self.a + self.b
    }
    pub fn has(self, n: u8) -> bool {
        self.a == n || self.b == n
    }
    pub fn other(self, n: u8) -> u8 {
        if self.a == n { self.b } else { self.a }
    }
    pub fn index(self) -> u8 {
        self.b * (self.b + 1) / 2 + self.a
    }
    pub fn from_index(i: u8) -> Tile {
        let mut b = 0;
        while (b + 1) * (b + 2) / 2 <= i {
            b += 1;
        }
        Tile {
            a: i - b * (b + 1) / 2,
            b,
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}|{}]", self.a, self.b)
    }
}

pub fn full_set() -> Vec<Tile> {
    (0..28).map(Tile::from_index).collect()
}
