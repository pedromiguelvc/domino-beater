use crate::tile::Tile;
use std::{collections::VecDeque, fmt};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Debug, Default)]
pub struct Board {
    pub line: VecDeque<(u8, u8)>,
}
impl Board {
    pub fn is_empty(&self) -> bool {
        self.line.is_empty()
    }
    pub fn ends(&self) -> Option<(u8, u8)> {
        Some((self.line.front()?.0, self.line.back()?.1))
    }
    pub fn can_play(&self, t: Tile, side: Side) -> bool {
        match self.ends() {
            None => true,
            Some((l, r)) => t.has(if side == Side::Left { l } else { r }),
        }
    }
    pub fn play(&mut self, t: Tile, side: Side) {
        match self.ends() {
            None => self.line.push_back((t.a, t.b)),
            Some((l, r)) => match side {
                Side::Left => {
                    let n = t.other(l);
                    self.line.push_front((n, l));
                }
                Side::Right => {
                    let n = t.other(r);
                    self.line.push_back((r, n));
                }
            },
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.line.is_empty() {
            return write!(f, "(empty)");
        }
        for (l, r) in &self.line {
            write!(f, "[{}|{}]", l, r)?;
        }
        Ok(())
    }
}
