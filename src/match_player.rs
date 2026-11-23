//! Match loop: dealing, special hands, bonus accumulation, starter selection,
//! and the first-to-six win condition.

use rand::{rngs::StdRng, seq::SliceRandom};

use crate::{
    board::Board,
    game::{team_of, Outcome, Round},
    hand::Hand,
    player::{Player, PlayerView},
    tile::{full_set, Tile},
    Seat,
};

/// A match ends when a team reaches this score.
pub const WINNING_SCORE: u32 = 6;

/// Safety guard on deals per match. This is a code guard, not a game rule.
pub const MAX_DEALS: u64 = 1_000_000;

/// Event counters accumulated across matches.
#[derive(Debug, Default, Clone)]
pub struct Counters {
    /// Every deal, including special-hand and tie redeals.
    pub deals: u64,
    /// Deals that reached play.
    pub rounds_played: u64,
    pub four_double_redeals: u64,
    pub five_double_awards: u64,
    pub six_double_awards: u64,
    pub blocked: u64,
    pub ties: u64,
    /// Round wins by base points; index is the base value (1 to 4).
    pub base_awards: [u64; 5],
    /// Blocked games decided by pip count (not ties).
    pub blocked_awards: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchResult {
    /// Winning team index, or `None` if the deal guard was reached.
    pub winner: Option<usize>,
    pub scores: [u32; 2],
    pub deals: u64,
}

/// Who opens the next round.
#[derive(Debug, Clone, Copy)]
enum Next {
    /// Highest double starts and must play it (first round, after a tie or a 4-double redeal).
    Forced,
    /// The team that won the previous award chooses the starter.
    Winner(usize),
}

/// Plays one match to completion.
pub fn play_match(
    players: &mut [Box<dyn Player>],
    rng: &mut StdRng,
    counters: &mut Counters,
) -> MatchResult {
    let mut scores = [0u32; 2];
    let mut bonus = 0u32;
    let mut next = Next::Forced;
    let mut deals = 0u64;

    loop {
        if deals >= MAX_DEALS {
            return MatchResult { winner: None, scores, deals };
        }
        deals += 1;
        counters.deals += 1;
        let hands = deal(rng);

        // Special hands, checked on every deal before play.
        let (max_seat, max_doubles) = (0..4)
            .map(|s| (s, hands[s].doubles()))
            .max_by_key(|&(_, d)| d)
            .expect("four seats");

        if max_doubles >= 5 {
            let base = if max_doubles == 6 {
                counters.six_double_awards += 1;
                2
            } else {
                counters.five_double_awards += 1;
                1
            };
            let team = team_of(max_seat);
            scores[team] += base + bonus;
            bonus = 0;
            next = Next::Winner(team);
            if scores[team] >= WINNING_SCORE {
                return result(team, scores, deals);
            }
            continue;
        }
        if max_doubles == 4 {
            counters.four_double_redeals += 1;
            bonus += 1;
            next = Next::Forced;
            continue;
        }

        let (starter, forced) = match next {
            Next::Forced => {
                let (seat, tile) = highest_double(&hands);
                (seat, Some(tile))
            }
            Next::Winner(team) => (choose_next_starter(players, team, &hands), None),
        };

        counters.rounds_played += 1;
        let mut round = Round::new(hands, starter, forced);
        let outcome = loop {
            let seat = round.turn;
            let legal = round.legal_moves(seat);
            if legal.is_empty() {
                if let Some(o) = round.pass() {
                    break o;
                }
            } else {
                let view = round.view(seat);
                let index = players[seat].choose_move(&view, &legal);
                assert!(index < legal.len(), "bot chose an index outside the legal moves");
                if let Some(o) = round.apply(legal[index]) {
                    break o;
                }
            }
        };

        match outcome {
            Outcome::Win { seat, base, .. } => {
                counters.base_awards[base as usize] += 1;
                let team = team_of(seat);
                scores[team] += base + bonus;
                bonus = 0;
                next = Next::Winner(team);
                if scores[team] >= WINNING_SCORE {
                    return result(team, scores, deals);
                }
            }
            Outcome::Blocked => {
                counters.blocked += 1;
                let team_pips = [
                    round.hands[0].pips() + round.hands[2].pips(),
                    round.hands[1].pips() + round.hands[3].pips(),
                ];
                if team_pips[0] == team_pips[1] {
                    counters.ties += 1;
                    bonus += 1;
                    next = Next::Forced;
                } else {
                    let team = if team_pips[0] < team_pips[1] { 0 } else { 1 };
                    counters.blocked_awards += 1;
                    scores[team] += 1 + bonus;
                    bonus = 0;
                    next = Next::Winner(team);
                    if scores[team] >= WINNING_SCORE {
                        return result(team, scores, deals);
                    }
                }
            }
        }
    }
}

fn result(team: usize, scores: [u32; 2], deals: u64) -> MatchResult {
    MatchResult { winner: Some(team), scores, deals }
}

/// Shuffles the 28 tiles and deals 6 to each seat. The 4 boneyard tiles are not dealt.
fn deal(rng: &mut StdRng) -> [Hand; 4] {
    let mut tiles = full_set();
    tiles.shuffle(rng);
    let mut hands = [Hand::default(); 4];
    for (i, tile) in tiles.iter().take(24).enumerate() {
        hands[i / 6].insert(*tile);
    }
    hands
}

/// The seat holding the highest double, and that double.
fn highest_double(hands: &[Hand; 4]) -> (Seat, Tile) {
    for v in (0..=6u8).rev() {
        let tile = Tile::new(v, v);
        if let Some(seat) = (0..4).find(|&s| hands[s].contains(tile)) {
            return (seat, tile);
        }
    }
    unreachable!("a double-six set always contains a double")
}

/// Starter rule:
/// - The winning team starts. If only one of its members holds a double, that member starts.
/// - If both hold a double, the team decides between them. The first eligible seat is asked
///   to decide, and the human prompt is the choice shown to that player.
/// - If neither member holds a double, the same rule applies to the opposing team.
///
/// Because the boneyard holds only 4 of the 7 doubles, at least one team always has a double.
fn choose_next_starter(players: &mut [Box<dyn Player>], team: usize, hands: &[Hand; 4]) -> Seat {
    let eligible_in = |t: usize| -> Vec<Seat> {
        [t, t + 2].into_iter().filter(|&s| hands[s].doubles() > 0).collect()
    };

    let mut eligible = eligible_in(team);
    if eligible.is_empty() {
        eligible = eligible_in(1 - team);
    }
    assert!(!eligible.is_empty(), "some team always holds a double");

    if eligible.len() == 1 {
        return eligible[0];
    }

    let decider = eligible[0];
    let view = PlayerView {
        seat: decider,
        hand: hands[decider],
        board: Board::default(),
    };
    let chosen = players[decider].choose_starter(&view, &eligible);
    assert!(eligible.contains(&chosen), "starter must be an eligible seat");
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{game::Move, player::PlayerView as View, tile::Tile as T};

    /// Test player: records whether it was asked, and always picks `pick`.
    struct Scripted {
        pick: Seat,
        asked: std::rc::Rc<std::cell::Cell<bool>>,
    }

    impl Player for Scripted {
        fn choose_move(&mut self, _v: &View, _legal: &[Move]) -> usize {
            0
        }
        fn choose_starter(&mut self, _v: &View, _eligible: &[Seat]) -> Seat {
            self.asked.set(true);
            self.pick
        }
    }

    fn scripted(pick: Seat) -> (Box<dyn Player>, std::rc::Rc<std::cell::Cell<bool>>) {
        let asked = std::rc::Rc::new(std::cell::Cell::new(false));
        (Box::new(Scripted { pick, asked: asked.clone() }), asked)
    }

    fn hands_with_doubles(seats: &[Seat]) -> [Hand; 4] {
        let mut hands = [Hand::from_tiles(&[T::new(0, 1)]); 4];
        for &s in seats {
            hands[s] = Hand::from_tiles(&[T::new(6, 6)]);
        }
        hands
    }

    #[test]
    fn single_eligible_member_starts_without_a_choice() {
        let (p0, asked0) = scripted(0);
        let (p1, _) = scripted(1);
        let mut players = [p0, p1, scripted(2).0, scripted(3).0];
        let hands = hands_with_doubles(&[2]);
        assert_eq!(choose_next_starter(&mut players, 0, &hands), 2);
        assert!(!asked0.get());
    }

    #[test]
    fn two_eligible_members_are_asked_and_the_choice_is_used() {
        let (p0, asked0) = scripted(2);
        let mut players = [p0, scripted(1).0, scripted(2).0, scripted(3).0];
        let hands = hands_with_doubles(&[0, 2]);
        assert_eq!(choose_next_starter(&mut players, 0, &hands), 2);
        assert!(asked0.get());
    }

    #[test]
    fn if_winning_team_has_no_double_the_opposing_team_decides() {
        let mut players = [scripted(0).0, scripted(1).0, scripted(2).0, scripted(3).0];
        let hands = hands_with_doubles(&[1]);
        assert_eq!(choose_next_starter(&mut players, 0, &hands), 1);
    }
}
