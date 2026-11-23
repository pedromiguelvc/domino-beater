//! Round engine: legal moves, move application, and round outcomes.

use crate::{
    board::{Board, Side},
    hand::Hand,
    player::PlayerView,
    tile::Tile,
    Seat,
};

/// A tile placed on one end of the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub tile: Tile,
    pub side: Side,
}

/// How a round ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The player in `seat` emptied their hand; `base` excludes the accumulated bonus.
    Win { seat: Seat, base: u32, last: Tile },
    /// Four consecutive passes; the caller decides the pip-count result.
    Blocked,
}

/// The team of a seat. Team A is seats 0 and 2, team B is seats 1 and 3.
pub fn team_of(seat: Seat) -> usize {
    seat % 2
}

/// Base points for the tile that empties a hand.
///
/// `fits_both` is true when the tile could have been played on either end;
/// `ends_equal` is true when both open ends show the same number.
pub fn base_points(last: Tile, fits_both: bool, ends_equal: bool) -> u32 {
    match (fits_both, last.is_double()) {
        (true, true) => 4,
        (true, false) if ends_equal => 1,
        (true, false) => 3,
        (false, true) => 2,
        (false, false) => 1,
    }
}

/// The state of one round, from the deal to its outcome.
#[derive(Debug, Clone)]
pub struct Round {
    pub hands: [Hand; 4],
    pub board: Board,
    pub turn: Seat,
    pub consecutive_passes: u32,
    /// `Some(tile)` forces the opening move to that tile.
    /// `None` allows any double as the opening move.
    pub forced_opening: Option<Tile>,
}

impl Round {
    pub fn new(hands: [Hand; 4], starter: Seat, forced_opening: Option<Tile>) -> Self {
        Round {
            hands,
            board: Board::default(),
            turn: starter,
            consecutive_passes: 0,
            forced_opening,
        }
    }

    /// Legal moves for `seat`. Only the seat whose turn it is may move.
    ///
    /// When both ends show the same number, the two sides are equivalent,
    /// so only the left side is offered.
    pub fn legal_moves(&self, seat: Seat) -> Vec<Move> {
        let hand = self.hands[seat];
        let mut moves = Vec::new();

        match self.board.ends() {
            None => {
                for tile in hand.iter() {
                    let allowed = match self.forced_opening {
                        Some(forced) => tile == forced,
                        None => tile.is_double(),
                    };
                    if allowed {
                        moves.push(Move { tile, side: Side::Left });
                    }
                }
            }
            Some((left, right)) => {
                for tile in hand.iter() {
                    if tile.has(left) {
                        moves.push(Move { tile, side: Side::Left });
                    }
                    if right != left && tile.has(right) {
                        moves.push(Move { tile, side: Side::Right });
                    }
                }
            }
        }
        moves
    }

    /// Plays `m` for the current player. Returns the outcome if the round ends.
    pub fn apply(&mut self, m: Move) -> Option<Outcome> {
        let seat = self.turn;
        debug_assert!(self.legal_moves(seat).contains(&m), "illegal move {m:?}");

        let (fits_both, ends_equal) = self.fit_info(m.tile);
        self.hands[seat].remove(m.tile);
        self.board.play(m.tile, m.side);
        self.consecutive_passes = 0;

        if self.hands[seat].is_empty() {
            return Some(Outcome::Win {
                seat,
                base: base_points(m.tile, fits_both, ends_equal),
                last: m.tile,
            });
        }
        self.turn = (seat + 1) % 4;
        None
    }

    /// The current player passes. Returns `Blocked` on the fourth consecutive pass.
    pub fn pass(&mut self) -> Option<Outcome> {
        debug_assert!(self.legal_moves(self.turn).is_empty(), "pass while a move exists");
        self.consecutive_passes += 1;
        if self.consecutive_passes == 4 {
            return Some(Outcome::Blocked);
        }
        self.turn = (self.turn + 1) % 4;
        None
    }

    /// What `seat` is allowed to know: its own hand and the public board.
    pub fn view(&self, seat: Seat) -> PlayerView {
        PlayerView {
            seat,
            hand: self.hands[seat],
            board: self.board.clone(),
        }
    }

    /// Returns (fits on both ends, ends are equal), evaluated before `t` is played.
    fn fit_info(&self, t: Tile) -> (bool, bool) {
        match self.board.ends() {
            None => (false, false),
            Some((left, right)) => (t.has(left) && t.has(right), left == right),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(a: u8, b: u8) -> Tile {
        Tile::new(a, b)
    }

    #[test]
    fn base_points_table() {
        // Fits one end.
        assert_eq!(base_points(t(2, 5), false, false), 1);
        assert_eq!(base_points(t(3, 3), false, false), 2);
        // Fits both ends, ends differ.
        assert_eq!(base_points(t(2, 5), true, false), 3);
        assert_eq!(base_points(t(3, 3), true, false), 4);
        // Fits both ends, ends equal: a non-double still scores only 1.
        assert_eq!(base_points(t(2, 5), true, true), 1);
        assert_eq!(base_points(t(3, 3), true, true), 4);
    }

    #[test]
    fn forced_opening_allows_only_that_tile() {
        let hands = [
            Hand::from_tiles(&[t(6, 6), t(5, 5), t(0, 1)]),
            Hand::default(),
            Hand::default(),
            Hand::default(),
        ];
        let round = Round::new(hands, 0, Some(t(6, 6)));
        assert_eq!(
            round.legal_moves(0),
            vec![Move { tile: t(6, 6), side: Side::Left }]
        );
    }

    #[test]
    fn equal_ends_offer_only_one_side() {
        let hands = [
            Hand::from_tiles(&[t(3, 4)]),
            Hand::default(),
            Hand::default(),
            Hand::default(),
        ];
        let mut round = Round::new(hands, 0, None);
        round.board.play(t(3, 3), Side::Left); // ends are now 3 and 3
        assert_eq!(
            round.legal_moves(0),
            vec![Move { tile: t(3, 4), side: Side::Left }]
        );
    }

    #[test]
    fn emptying_hand_reports_win_with_base_points() {
        let hands = [
            Hand::from_tiles(&[t(2, 5)]),
            Hand::from_tiles(&[t(0, 0)]),
            Hand::default(),
            Hand::default(),
        ];
        let mut round = Round::new(hands, 0, None);
        round.board.play(t(2, 2), Side::Left); // ends 2 and 2
        let outcome = round.apply(Move { tile: t(2, 5), side: Side::Left });
        // [2|5] fits only the left end (both ends show 2), so it scores the one-end base of 1.
        assert_eq!(
            outcome,
            Some(Outcome::Win { seat: 0, base: 1, last: t(2, 5) })
        );
    }
}
