//! Player interface. A player sees only its own hand and the public board.

use crate::{board::Board, game::Move, hand::Hand, Seat};

pub mod random;

/// The starter choice made by the team that wins a round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarterChoice {
    Myself,
    Partner,
}

/// Information available to one seat. It excludes the other hands by design.
#[derive(Debug, Clone)]
pub struct PlayerView {
    pub seat: Seat,
    pub hand: Hand,
    pub board: Board,
}

pub trait Player {
    /// Returns the index into `legal` of the chosen move. `legal` is never empty.
    fn choose_move(&mut self, view: &PlayerView, legal: &[Move]) -> usize;

    /// Chooses between this player and its partner to open the next round.
    fn choose_starter(&mut self, view: &PlayerView) -> StarterChoice;
}
