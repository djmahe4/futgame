# FutGame - Rust Football Simulator

A fast, performant CLI football (soccer) simulator reimagined in Rust, inspired by OpenFootManager architecture, with xT (Expected Threat) integration and the original xG gameplay.

## How to Play FutGame

### Single Player
```bash
cargo run
```
Choose team → difficulty → play vs AI.

### Multiplayer (TCP)
**Host:**
```bash
cargo run -- --host
```
**Client (on another machine or terminal):**
```bash
cargo run -- --join 127.0.0.1
```
You can also use the interactive menu at startup — press **H** to host or **J** to join.

### Screenshots / Example Output

```
╔═══════════════════════════════════╗
║       ⚽  FutGame  ⚽              ║
║   Rust CLI Football Simulator      ║
╚═══════════════════════════════════╝

Game Mode:
  (S)ingle Player
  (H)ost Multiplayer
  (J)oin Multiplayer

▶ Your ball! 12 mins
Current pos: 5
Options: 6, 7, 8, 9
Move to position: 9
  [state] turn=12 ball_zone=penalty_area | move 5 → 9
  [xT] Vinicius Junior chose zone 9 (tactic boost 0.04)
  💬 He's gone past two defenders!

◀ Computer's ball! 15 mins
Options: 5, 6, 7, g
Guess where they'll move: 6
  [state] turn=15 ball_zone=midfield | move 7 → 6
  ↔ Real Madrid gain possession

╔══════════════════════════════════════╗
║  Real Madrid         1 - 0 Bayern    ║
╚══════════════════════════════════════╝
  ⚽ Vinicius Junior (Real Madrid) 38'
```

## Controls
- Enter zone number (0-8, or `g`) when prompted
- Guess opponent's zone when defending
- Type exactly as shown — validation loops will re-prompt on errors

## Features
- Configurable turn duration (10s–60s)
- xG + xT engine with probability scaling
- Role-constrained movement
- AI levels up to Insane (dual guess)
- Deterministic multiplayer with shared RNG seed

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

## Recent Improvements (Copilot Refactor)

- Halftime logic fixed to properly include the full 45th minute before break.
- All event probabilities now scale correctly with turn_duration_secs for balanced matches at any granularity (10s–60s).
- Input validation loops added for moves and guesses (no more silent defaults on typos).
- Team selection menu is now deterministic (sorted alphabetically).
- Squads restored to original 11 starters + appended generic substitutes (18+ players per team).
- Substitutes now correctly initialize xG values to prevent runtime errors.
- Lightweight xT influence and tactical logging added for better coach visibility.
- AI difficulty levels implemented (Easy to Insane with dual-guess logic on Insane).

## Artifacts & Test Results
- All tests pass (`cargo test`): 3 doc-tests in config.rs (halftime_turn, total_turns, turn_to_minute)
- Single-player game verified: balanced scorelines at both 60s and 10s turn durations
- Multiplayer tested locally (two terminals): host and client produced identical tactical logs using shared RNG seed
- Rendering verified identical on host and client side
- No substitute/xG panics: all 18-player squads initialize xG correctly
- Input validation re-prompts on invalid zone entries
- Halftime triggers correctly after full first half (turn ≥ halftime_turn)
- Team menu is deterministic (alphabetically sorted)

## Advanced Commands: 'm' (Move Players) & 'p' (Pause Insights)

### 'p' — Pause & Insights (max 2 per match)
At any input prompt, type **`p`** to pause and display a full coach-level snapshot:

```
⏸  Pause 1/2 — showing insights…

╔══════════════════════════════════════════════════════════════╗
║                  📊  COACH INSIGHTS                         ║
╚══════════════════════════════════════════════════════════════╝
  Real Madrid 1-0 Bayern  [45']  pauses: 1/2

  xT Heatmap  (col: own-goal→penalty box | row: L/C/R)
  ┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
 L │  .0.001  │  :0.008  │  -0.025  │  =0.060  │  #0.150  │  @0.320  │
 C │  .0.002  │  :0.012  │  -0.035  │  =0.080  │  #0.200  │  @0.420  │
 R │  .0.001  │  :0.008  │  -0.025  │  =0.060  │  #0.150  │  @0.320  │
  └──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘
    col:  0-GK      1-Own     2-O.Mid   3-A.Mid   4-Final   5-Pen.box
    key: .=low  :=low-med  -=med  ==med-high  #=high  @=max

  ┌──────┬────────────────────────┬───────────────┬──────────────┬────────────────────┐
  │ Pos  │ Name                   │ xG (cumul.)   │ xT (cumul.)  │ World pos (105×68) │
  ├──────┼────────────────────────┼───────────────┼──────────────┼────────────────────┤
  │ 9    │ Vinicius Junior        │        0.2620 │       0.0348 │ ( 82.0, 25.0)m      │
  │ 0    │ Benzema                │        0.2580 │       0.0348 │ ( 82.0, 43.0)m      │
  │ 7    │ Modric                 │        0.1510 │       0.0348 │ ( 65.0, 15.0)m      │
  └──────────────────────────────────────────────────────────────────────────────────────┘
```

The insights view is also shown automatically at **half-time** and **full-time**.

### 'm' — Reposition Players (role-limited, up to 2 per turn)
At any input prompt, type **`m`** to access the movement menu:

```
▶ Your ball! 32 mins
Current pos: 5
Options: 3, 4, 5, 6, 7  |  [m]=move players  [p]=pause insights
Move to position (or 'm'/'p'): m

  Player positions (enter 'from to' to reposition, or blank to skip):
    [g]  Courtois              (Goalkeeper  ) @ (  5.0, 34.0)m
    [5]  Kroos                 (Midfielder  ) @ ( 50.0, 22.0)m
    [7]  Modric                (Midfielder  ) @ ( 65.0, 15.0)m
    [9]  Vinicius Junior       (Striker     ) @ ( 82.0, 25.0)m
  Reposition [from to] or blank to done: 5 7
  ✓ Moved 5 → 7 (65.0,15.0)m
  Reposition [from to] or blank to done:

Move to position (or 'm'/'p'): 7
  [xT] Kroos chose zone 7 (tactic boost 1.10)
  [state] turn=32 ball_zone=7 | move 5 → 7
```

Movement to a higher-xT zone gives the team a small `total_xt` boost.

### Dribble (same-position trigger)
Choose the **same position you're in** to attempt a dribble:
```
Move to position: 9
  [xT] Vinicius Junior chose zone 9 (tactic boost 1.30)
  🔥 Brilliant dribble! Beat the defender!
  [state] turn=58 ball_zone=9 | dribble → beat defender!
```
On failure:
```
  😤 Dribble failed — lost the ball!
  ↔ Bayern gain possession
```

### Path-Based Interception
Passes through defensive zones carry a small interception risk:
```
  ⚡ Intercepted in zone 3!
  ↔ Bayern gain possession
  [state] turn=21 ball_zone=0 | ⚡ intercepted by player 5
```

## Floating-Point Pitch System (105×68m)
Every position key maps to a real-world coordinate:

| Key | Role         |  x (m) |  y (m) |
|-----|-------------|--------|--------|
|  g  | Goalkeeper  |   5.0  |  34.0  |
|  1  | Left Back   |  20.0  |  13.0  |
|  2  | Right Back  |  20.0  |  55.0  |
|  3  | CDM-L       |  32.0  |  20.0  |
|  4  | CDM-R       |  32.0  |  48.0  |
|  5  | CM-L        |  50.0  |  22.0  |
|  6  | CM-R        |  50.0  |  46.0  |
|  7  | AMF-L / LW  |  65.0  |  15.0  |
|  8  | AMF-R / RW  |  65.0  |  53.0  |
|  9  | Striker-L   |  82.0  |  25.0  |
|  0  | Striker-R   |  82.0  |  43.0  |

World positions are updated when you use 'm' to reposition players, and shown in the insights overlay.
