//! Bot-only simulation harness. Plays many matches and reports event frequencies.

use std::fmt;

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    match_player::{play_match, Counters},
    player::{random::RandomPlayer, Player},
};

#[derive(Debug, Default)]
pub struct SimReport {
    pub matches: u64,
    /// Matches that hit the deal limit without a winner.
    pub aborted: u64,
    pub team_wins: [u64; 2],
    pub total_points: [u64; 2],
    pub total_deals: u64,
    pub counters: Counters,
}

/// Plays `matches` bot-only matches with four random players, seeded by `seed`.
pub fn run(matches: u64, seed: u64) -> SimReport {
    let mut master = StdRng::seed_from_u64(seed);
    let mut report = SimReport::default();

    for _ in 0..matches {
        let mut players: Vec<Box<dyn Player>> = (0..4)
            .map(|_| {
                let rng = StdRng::seed_from_u64(master.r#gen());
                Box::new(RandomPlayer::new(rng)) as Box<dyn Player>
            })
            .collect();

        let result = play_match(&mut players, &mut master, &mut report.counters);
        report.matches += 1;
        report.total_deals += result.deals;
        report.total_points[0] += u64::from(result.scores[0]);
        report.total_points[1] += u64::from(result.scores[1]);
        match result.winner {
            Some(team) => report.team_wins[team] += 1,
            None => report.aborted += 1,
        }
    }
    report
}

fn pct(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        100.0 * part as f64 / whole as f64
    }
}

impl fmt::Display for SimReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = &self.counters;
        let decided = self.matches - self.aborted;
        writeln!(f, "Matches:            {}", self.matches)?;
        writeln!(f, "Aborted (no winner): {}", self.aborted)?;
        writeln!(
            f,
            "Team A wins:        {} ({:.1}%)",
            self.team_wins[0],
            pct(self.team_wins[0], decided)
        )?;
        writeln!(
            f,
            "Team B wins:        {} ({:.1}%)",
            self.team_wins[1],
            pct(self.team_wins[1], decided)
        )?;
        writeln!(
            f,
            "Average deals/match: {:.2}",
            self.total_deals as f64 / self.matches.max(1) as f64
        )?;
        writeln!(
            f,
            "Average points/match: A {:.2}, B {:.2}",
            self.total_points[0] as f64 / self.matches.max(1) as f64,
            self.total_points[1] as f64 / self.matches.max(1) as f64
        )?;
        writeln!(f, "--- Event frequencies (% of deals) ---")?;
        let d = c.deals;
        writeln!(f, "Four-double redeals:  {} ({:.3}%)", c.four_double_redeals, pct(c.four_double_redeals, d))?;
        writeln!(f, "Five-double awards:   {} ({:.3}%)", c.five_double_awards, pct(c.five_double_awards, d))?;
        writeln!(f, "Six-double awards:    {} ({:.3}%)", c.six_double_awards, pct(c.six_double_awards, d))?;
        writeln!(f, "Blocked games:        {} ({:.2}%)", c.blocked, pct(c.blocked, d))?;
        writeln!(f, "  decided by pips:    {}", c.blocked_awards)?;
        writeln!(f, "  tied (redeal):      {}", c.ties)?;
        writeln!(f, "--- Round finishes (played rounds) ---")?;
        let r = c.rounds_played;
        for base in 1..=4usize {
            writeln!(
                f,
                "Base {base} finish:        {} ({:.2}%)",
                c.base_awards[base],
                pct(c.base_awards[base], r)
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_run_completes_every_match() {
        let report = run(200, 7);
        assert_eq!(report.aborted, 0);
        assert_eq!(report.team_wins[0] + report.team_wins[1], report.matches);
    }

    #[test]
    fn same_seed_gives_same_report() {
        let a = run(50, 99);
        let b = run(50, 99);
        assert_eq!(a.team_wins, b.team_wins);
        assert_eq!(a.total_points, b.total_points);
    }
}
