use domino_beater::sim;

/// Usage: `domino-beater [MATCHES]` (default 10000). Runs a bot-only simulation.
fn main() {
    let matches = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    let report = sim::run(matches, 42);
    print!("{report}");
}
