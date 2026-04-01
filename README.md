# FutGame - Rust Football Simulator

A fast, performant CLI football (soccer) simulator reimagined in Rust, inspired by OpenFootManager architecture, with xT (Expected Threat) integration and the original xG gameplay.

## Quick Start

```bash
cargo run
```

Or with options:
```bash
cargo run -- --simple          # Original xG-only mode (no xT)
cargo run -- --team1 "Brazil" --team2 "Argentina"
```

## Architecture

- `src/xg.rs` - Original xG logic (exact port from Python)
- `src/xt.rs` - Expected Threat grid and xT→xG modifier
- `src/pitch.rs` - Pitch zones
- `src/player.rs` - Player struct and 60+ roles
- `src/team.rs` - Team and formation management
- `src/events.rs` - 25 game event types
- `src/tactics.rs` - Formation and tactic modifiers
- `src/simulation.rs` - Markov-chain match engine

## Gameplay

The game simulates a football match using position numbers:
- `g` = Goalkeeper (xG: 0.01)
- `1-4` = Defenders (xG: 0.05)
- `5-8` = Midfielders (xG: 0.15)
- `9,0` = Attackers (xG: 0.25)

When your team has the ball, pick a position to pass/move to. The computer guesses where you'll go. If it guesses right, you lose possession.

## xT Integration

Expected Threat (xT) models how dangerous each zone of the pitch is. Higher xT zones (penalty area = ~0.35) provide a small bonus to xG calculations. This enriches buildup play without changing the core xG shot resolution.

## Building

```bash
cargo build --release
cargo test
```
