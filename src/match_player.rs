//! Match loop: dealing, special hands, bonus accumulation, starter selection,
//! and the first-to-six win condition.

use rand::{rngs::StdRng, seq::SliceRandom};

use crate::{
    board::Board,
    game::{team_of, Outcome, Round},
    hand::Hand,
    player::{Player, StarterChoice},
    tile::{full_set, Tile},
    Seat,
};

/// A match ends when a team reaches this score.
pub const WINNING_SCORE: u32 = 6;

/// Safety limit on deals per match, so a degenerate bot cannot loop forever.
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
    /// Winning team index, or `None` if the deal limit was reached.
    pub winner: Option<usize>,
    pub scores: [u32; 2],
    pub deals: u64,
}

/// Who opens the next round.
#[derive(Debug, Clone, Copy)]
enum Next {
    /// Highest double starts and must play it (first round, after a tie or a 4-double redeal).
    Forced,
    /// The given seat's team chooses the starter (after a win, a special hand, or a blocked game).
    Chooser(Seat),
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
            next = Next::Chooser(max_seat);
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
            Next::Chooser(chooser) => (choose_next_starter(players, chooser, &hands), None),
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
                next = Next::Chooser(seat);
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
                    // The winning-team member with fewer pips chooses; ties go to the lower seat.
                    let low = team;
                    let high = team + 2;
                    let chooser = if round.hands[low].pips() <= round.hands[high].pips() {
                        low
                    } else {
                        high
                    };
                    next = Next::Chooser(chooser);
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

/// The chooser's team picks the starter if it holds a double; otherwise the other team
/// picks, beginning with the seat after the original chooser. The chosen player starts
/// if they hold a double; otherwise the partner starts.
fn choose_next_starter(players: &mut [Box<dyn Player>], chooser: Seat, hands: &[Hand; 4]) -> Seat {
    let team_has_double = |team: usize| hands[team].doubles() + hands[team + 2].doubles() > 0;

    let chooser = if team_has_double(team_of(chooser)) {
        chooser
    } else {
        (chooser + 1) % 4
    };
    debug_assert!(team_has_double(team_of(chooser)), "some team always holds a double");

    let partner = (chooser + 2) % 4;
    let view = crate::player::PlayerView {
        seat: chooser,
        hand: hands[chooser],
        board: Board::default(),
    };
    let target = match players[chooser].choose_starter(&view) {
        StarterChoice::Myself => chooser,
        StarterChoice::Partner => partner,
    };
    if hands[target].doubles() > 0 {
        target
    } else if target == chooser {
        partner
    } else {
        chooser
    }
}
