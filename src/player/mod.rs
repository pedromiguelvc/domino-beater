//! Player interface. A player sees only its own hand and the public board.

use crate::{board::Board, game::Move, hand::Hand, Seat};

pub mod random;

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

    /// Chooses which member of the deciding team opens the next round.
    /// `eligible` holds the seats that hold a double; it always has two members
    /// here, because the engine does not ask when only one member is eligible.
    /// Returns one of the seats in `eligible`.
    fn choose_starter(&mut self, view: &PlayerView, eligible: &[Seat]) -> Seat;
}
