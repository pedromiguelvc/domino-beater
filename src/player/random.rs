//! Baseline bot: plays a uniformly random legal move and chooses starters at random.

use rand::{rngs::StdRng, Rng};

use super::{Player, PlayerView, StarterChoice};
use crate::game::Move;

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

    fn choose_starter(&mut self, _view: &PlayerView) -> StarterChoice {
        if self.rng.gen_bool(0.5) {
            StarterChoice::Myself
        } else {
            StarterChoice::Partner
        }
    }
}
