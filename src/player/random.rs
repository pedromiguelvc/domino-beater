//! Baseline bot: plays a uniformly random legal move and picks starters at random.

use rand::{rngs::StdRng, Rng};

use super::{Player, PlayerView};
use crate::{game::Move, Seat};

pub struct RandomPlayer {
    rng: StdRng,
}

impl RandomPlayer {
    pub fn new(rng: StdRng) -> Self {
        RandomPlayer { rng }
    }
}

impl Player for RandomPlayer {
    fn choose_move(&mut self, _view: &PlayerView, legal: &[Move]) -> usize {
        self.rng.gen_range(0..legal.len())
    }

    fn choose_starter(&mut self, _view: &PlayerView, eligible: &[Seat]) -> Seat {
        eligible[self.rng.gen_range(0..eligible.len())]
    }
}
