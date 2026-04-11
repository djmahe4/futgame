// === ENHANCED: Intelligent Defender Positioning + Role Interpolation + Formation Resets + Ball Tracking + Multiplayer Sync ===
// === ENHANCED: Intelligent Defender Positioning + Role-Specific Interpolation + Formation-Aware Resets + Multiplayer Sync ===
// === ENHANCED: Floating-Point Position System (105x68m) + 'm' Per-Guess Movements + 'p' Pause + Dribble/Interception + Insights Viz ===

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

use clap::Parser;
use colored::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use futgame::config::{Difficulty, GameConfig};
use futgame::events::GameEvent;
use futgame::network::{self, NetworkMode, DEFAULT_PORT};
use futgame::pitch::{pos_to_world, Position};
use futgame::simulation::{kickoff_world_positions, move_ball_with_action, step_turn, MatchState};
use futgame::tactics::{Formation, Tactic};
use futgame::team::{new_team, Team};
use futgame::ui::renderer::{render_insights, render_movement_viz, render_tactical};
use futgame::xg::adjacent_positions;

#[derive(Parser, Debug)]
#[command(name = "futgame", about = "CLI Football Simulator")]
struct Args {
    #[arg(long, help = "Simple mode: original xG only, no xT")]
    simple: bool,
    #[arg(long, default_value = "")]
    team1: String,
    #[arg(long, default_value = "")]
    team2: String,
    /// Seconds each turn represents on the match clock.
    /// Default: 60 (1 turn = 1 minute, 90 turns total).
    /// Example: --turn-duration 30 means 2 turns = 1 minute → 180 turns total.
    #[arg(long, default_value_t = 60, value_name = "SECS")]
    turn_duration: u32,
    /// AI difficulty level: easy, medium, hard, insane (default: easy).
    #[arg(long, default_value = "easy", value_name = "LEVEL")]
    difficulty: String,
    /// Host a multiplayer game on the given port (default 8080).
    #[arg(long, value_name = "PORT")]
    host: Option<Option<u16>>,
    /// Join a multiplayer game at the given IP address (default port 8080).
    #[arg(long, value_name = "IP")]
    join: Option<String>,
}

fn load_names() -> HashMap<String, Vec<String>> {
    let paths = ["data/names.json", "names.json", "../names.json"];
    for path in &paths {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<String>>>(&contents) {
                return map;
            }
        }
    }
    HashMap::new()
}

fn load_commentary() -> Vec<String> {
    let paths = ["data/desc.txt", "desc.txt", "../desc.txt"];
    for path in &paths {
        if let Ok(contents) = fs::read_to_string(path) {
            return contents.lines().map(|l| l.to_string()).collect();
        }
    }
    vec!["What a moment!".to_string()]
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn pick_formation() -> Formation {
    println!("{}", "Choose formation:".cyan().bold());
    println!("  1) 4-4-2");
    println!("  2) 4-3-3");
    println!("  3) 4-2-3-1");
    println!("  4) 3-5-2");
    println!("  5) 5-3-2");
    let choice = prompt("Enter 1-5: ");
    match choice.trim() {
        "1" => Formation::F442,
        "2" => Formation::F433,
        "3" => Formation::F4231,
        "4" => Formation::F352,
        "5" => Formation::F532,
        _ => Formation::F442,
    }
}

fn pick_tactic() -> Tactic {
    println!("{}", "Choose tactic:".cyan().bold());
    println!("  1) Attacking");
    println!("  2) Defensive");
    println!("  3) Possession");
    println!("  4) Counter");
    println!("  5) Pressing");
    let choice = prompt("Enter 1-5: ");
    match choice.trim() {
        "1" => Tactic::Attacking,
        "2" => Tactic::Defensive,
        "3" => Tactic::Possession,
        "4" => Tactic::Counter,
        "5" => Tactic::Pressing,
        _ => Tactic::Possession,
    }
}

fn show_formation_ascii(team: &Team) {
    println!("{}", format!("=== {} Formation: {} ===", team.name, team.formation).green().bold());
    let positions = ["0", "9", "7", "8", "5", "6", "3", "4", "1", "2", "g"];
    for pos in &positions {
        if let Some(p) = team.player_at_pos(pos) {
            let zone = futgame::pitch::zone_name_for_pos(pos);
            println!("  [{:>2}] {:30} ({})", pos.yellow(), p.name.cyan(), zone.dimmed());
        }
    }
    println!();
}

fn show_scorecard(state: &MatchState, team1: &Team, team2: &Team) {
    println!("\n{}", "╔══════════════════════════════════════╗".bright_blue());
    println!("{}", format!("║  {:15} {:3} - {:3} {:15}  ║",
        team1.name, state.score1, state.score2, team2.name).bright_blue());
    println!("{}", "╚══════════════════════════════════════╝".bright_blue());
    for gs in &state.goal_scorers {
        println!("  ⚽ {} ({}) {}'", gs.name.yellow(), gs.team_name.cyan(), gs.minute);
    }
}

fn show_halftime(state: &MatchState, team1: &Team, team2: &Team) {
    println!("\n{}", "====== HALF TIME ======".yellow().bold());
    show_scorecard(state, team1, team2);
    println!("Possession: {} {:.0}% / {} {:.0}%",
        team1.name, 100.0 * state.poss1 as f32 / (state.poss1 + state.poss2 + 1) as f32,
        team2.name, 100.0 * state.poss2 as f32 / (state.poss1 + state.poss2 + 1) as f32,
    );
}

fn show_fulltime(state: &MatchState, team1: &Team, team2: &Team) {
    println!("\n{}", "╔═══════════════════════════════════════════╗".bright_green().bold());
    println!("{}", "║              FULL TIME                     ║".bright_green().bold());
    println!("{}", "╚═══════════════════════════════════════════╝".bright_green().bold());

    println!("\n{}", format!("  {:20} {:3} - {:3} {:20}",
        team1.name, state.score1, state.score2, team2.name).bright_white().bold());

    println!("\n{}", "Goal Scorers:".yellow());
    for gs in &state.goal_scorers {
        println!("  ⚽ {} ({}) {}' ", gs.name.yellow(), gs.team_name.dimmed(), gs.minute);
    }

    println!("\n{}", "Statistics:".cyan().bold());
    let total_poss = (state.poss1 + state.poss2) as f32;
    println!("  Possession:  {} {:.1}% | {:.1}% {}",
        team1.name, 100.0 * state.poss1 as f32 / total_poss.max(1.0),
        100.0 * state.poss2 as f32 / total_poss.max(1.0), team2.name);
    println!("  Shots:       {} {:3} | {:3} {}", team1.name, team1.shots, team2.shots, team2.name);
    println!("  On Target:   {} {:3} | {:3} {}", team1.name, team1.shots_on_target, team2.shots_on_target, team2.name);
    println!("  xG:          {} {:.2} | {:.2} {}", team1.name, team1.total_xg, team2.total_xg, team2.name);
    println!("  xT:          {} {:.3} | {:.3} {}", team1.name, team1.total_xt, team2.total_xt, team2.name);

    println!("\n{}", format!("{} xG by position:", team1.name).cyan());
    let mut pos_keys: Vec<(&String, &f32)> = team1.xg_values.iter().collect();
    pos_keys.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in &pos_keys {
        print!("  [{}]={:.3}  ", k.yellow(), v);
    }
    println!();

    println!("\n{}", format!("{} xG by position:", team2.name).cyan());
    let mut pos_keys2: Vec<(&String, &f32)> = team2.xg_values.iter().collect();
    pos_keys2.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in &pos_keys2 {
        print!("  [{}]={:.3}  ", k.yellow(), v);
    }
    println!();

    let t1_yellows: u8 = team1.players.iter().map(|p| p.yellow_cards).sum();
    let t2_yellows: u8 = team2.players.iter().map(|p| p.yellow_cards).sum();
    let t1_reds: usize = team1.players.iter().filter(|p| p.red_card).count();
    let t2_reds: usize = team2.players.iter().filter(|p| p.red_card).count();
    println!("\n{}", "Cards:".cyan());
    println!("  {} - Yellow: {} Red: {}", team1.name, t1_yellows, t1_reds);
    println!("  {} - Yellow: {} Red: {}", team2.name, t2_yellows, t2_reds);
}

// Commentary file layout (1-based line numbers from desc.txt):
//   Lines  1-16 → saves  (idx 0-15)
//   Lines 17-24 → misses (idx 16-23)
//   Lines 25-49 → goals  (idx 24+)
const COMMENTARY_SAVES_START: usize = 0;
const COMMENTARY_SAVES_END: usize = 15;
const COMMENTARY_MISSES_START: usize = 16;
const COMMENTARY_MISSES_END: usize = 23;
const COMMENTARY_GOALS_START: usize = 24;

/// Inline commentary for dribble and interception events.
/// Chosen randomly so each event feels distinct without needing a desc.txt update.
const DRIBBLE_SUCCESS_LINES: &[&str] = &[
    "Silky skills! The defender is left for dead!",
    "Incredible footwork — he dances past the challenge!",
    "Nutmeg! The crowd goes wild!",
    "You can't stop him today — pure class!",
    "One touch, two touch — he's through on goal!",
];
const DRIBBLE_FAIL_LINES: &[&str] = &[
    "Too ambitious! The defender reads it perfectly.",
    "Dispossessed — the ball is quickly recycled.",
    "The gamble doesn't pay off this time.",
    "Clumsy touch — the defender steals it!",
];
const INTERCEPTION_LINES: &[&str] = &[
    "Brilliant defensive read — the pass is cut out!",
    "The defender anticipated that perfectly!",
    "Interception! The move breaks down.",
    "Sharp positioning — the line is held!",
    "No way through — the backline stays disciplined.",
];

fn get_commentary(evt: &GameEvent, commentary: &[String], rng: &mut impl Rng) -> Option<String> {
    // Handle dribble/interception with inline commentary (no desc.txt section needed)
    match evt {
        GameEvent::Dribble { .. } => {
            let idx = rng.gen_range(0..DRIBBLE_SUCCESS_LINES.len());
            return Some(DRIBBLE_SUCCESS_LINES[idx].to_string());
        }
        GameEvent::DribbleFail { .. } => {
            let idx = rng.gen_range(0..DRIBBLE_FAIL_LINES.len());
            return Some(DRIBBLE_FAIL_LINES[idx].to_string());
        }
        GameEvent::Interception { .. } => {
            let idx = rng.gen_range(0..INTERCEPTION_LINES.len());
            return Some(INTERCEPTION_LINES[idx].to_string());
        }
        _ => {}
    }

    let (start, end) = match evt {
        GameEvent::Save { .. } => (COMMENTARY_SAVES_START, COMMENTARY_SAVES_END.min(commentary.len().saturating_sub(1))),
        GameEvent::Miss { .. } => (COMMENTARY_MISSES_START, COMMENTARY_MISSES_END.min(commentary.len().saturating_sub(1))),
        GameEvent::Goal { .. } => (COMMENTARY_GOALS_START, commentary.len().saturating_sub(1)),
        _ => return None,
    };
    if start >= commentary.len() {
        return None;
    }
    // Collect non-empty lines in the range to avoid blank commentary
    let candidates: Vec<&String> = commentary[start..=end.max(start)]
        .iter()
        .filter(|l| !l.trim().is_empty())
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let idx = rng.gen_range(0..candidates.len());
    candidates.get(idx).map(|s| (*s).clone())
}

fn main() {
    let args = Args::parse();
    let commentary = load_commentary();
    let names_db = load_names();

    // Build the match configuration from CLI args
    let mut config = GameConfig::with_turn_duration(args.turn_duration);
    let difficulty = Difficulty::from_str(&args.difficulty);
    config.difficulty = difficulty;

    println!("{}", "╔═══════════════════════════════════╗".bright_green().bold());
    println!("{}", "║       ⚽  FutGame  ⚽              ║".bright_green().bold());
    println!("{}", "║   Rust CLI Football Simulator      ║".bright_green().bold());
    println!("{}", "╚═══════════════════════════════════╝".bright_green().bold());

    // Determine network mode from CLI flags first, then interactive menu.
    let network_mode: NetworkMode = if args.join.is_some() {
        let ip = args.join.as_deref().unwrap_or("127.0.0.1");
        NetworkMode::Client(ip.to_string(), DEFAULT_PORT)
    } else if args.host.is_some() {
        let port = args.host.flatten().unwrap_or(DEFAULT_PORT);
        NetworkMode::Host(port)
    } else {
        println!("\n{}", "Game Mode:".cyan().bold());
        println!("  (S)ingle Player");
        println!("  (H)ost Multiplayer");
        println!("  (J)oin Multiplayer");
        let mode_choice = prompt("Choose mode [S/H/J]: ").to_uppercase();
        match mode_choice.trim() {
            "H" => {
                let port_str = prompt(&format!("Port to host on [{}]: ", DEFAULT_PORT));
                let port: u16 = port_str.trim().parse().unwrap_or(DEFAULT_PORT);
                NetworkMode::Host(port)
            }
            "J" => {
                let ip = prompt("Host IP address [127.0.0.1]: ");
                let ip = if ip.trim().is_empty() { "127.0.0.1".to_string() } else { ip.trim().to_string() };
                let port_str = prompt(&format!("Port [{}]: ", DEFAULT_PORT));
                let port: u16 = port_str.trim().parse().unwrap_or(DEFAULT_PORT);
                NetworkMode::Client(ip, port)
            }
            _ => NetworkMode::SinglePlayer,
        }
    };

    // Establish multiplayer session (if needed) and determine shared RNG seed.
    let (mut mp_session, rng_seed) = match &network_mode {
        NetworkMode::Host(port) => {
            match network::host_game(*port) {
                Ok((session, seed)) => (Some(session), seed),
                Err(e) => {
                    eprintln!("{} {}", "Failed to host game:".red(), e);
                    std::process::exit(1);
                }
            }
        }
        NetworkMode::Client(ip, port) => {
            match network::join_game(ip, *port) {
                Ok((session, seed)) => (Some(session), seed),
                Err(e) => {
                    eprintln!("{} {}", "Failed to join game:".red(), e);
                    std::process::exit(1);
                }
            }
        }
        NetworkMode::SinglePlayer => {
            // Use entropy-based seed for single-player.
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42);
            (None, seed)
        }
    };

    // Seed the RNG deterministically so both host and client are in sync.
    let mut rng = SmallRng::seed_from_u64(rng_seed);

    if args.simple {
        println!("{}", "Mode: Simple (xG only, no xT)".yellow());
    } else {
        println!("{}", "Mode: Full (xG + xT)".green());
    }
    println!("⏱  Turn timing: {}", config.describe().cyan());
    println!("🤖 AI Difficulty: {:?}", config.difficulty);
    match &network_mode {
        NetworkMode::SinglePlayer => println!("🎮 Mode: Single Player"),
        NetworkMode::Host(p) => println!("🌐 Mode: Multiplayer Host (port {})", p),
        NetworkMode::Client(ip, p) => println!("🌐 Mode: Multiplayer Client ({}:{})", ip, p),
    }

    let mut team_names: Vec<String> = names_db.keys().cloned().collect();
    team_names.sort();
    if team_names.is_empty() {
        eprintln!("{}", "Error: no teams found. Ensure names.json is in data/ or current directory.".red());
        std::process::exit(1);
    }

    let t1_name = if args.team1.is_empty() {
        println!("\nAvailable teams:");
        for (i, tn) in team_names.iter().enumerate() {
            println!("  {}) {}", i + 1, tn);
        }
        let ch = prompt("Pick team 1 (number or name): ");
        if let Ok(n) = ch.parse::<usize>() {
            team_names.get(n.saturating_sub(1)).cloned().unwrap_or_else(|| team_names[0].clone())
        } else {
            ch
        }
    } else {
        args.team1.clone()
    };

    let t2_name = if args.team2.is_empty() {
        println!("\nAvailable teams:");
        for (i, tn) in team_names.iter().enumerate() {
            println!("  {}) {}", i + 1, tn);
        }
        let ch = prompt("Pick team 2 (number or name): ");
        if let Ok(n) = ch.parse::<usize>() {
            team_names.get(n.saturating_sub(1)).cloned().unwrap_or_else(|| team_names[team_names.len() - 1].clone())
        } else {
            ch
        }
    } else {
        args.team2.clone()
    };

    let toss = prompt("\nToss - call H or T: ").to_uppercase();
    let coin: bool = rng.gen();
    let t1_kicks_off = if toss == "H" { coin } else { !coin };
    if t1_kicks_off {
        println!("{}", format!("{} won the toss and kicks off!", t1_name).green());
    } else {
        println!("{}", format!("{} won the toss and kicks off!", t2_name).green());
    }

    println!("\n{}", format!("=== {} Setup ===", t1_name).cyan().bold());
    let f1 = pick_formation();
    let tac1 = pick_tactic();

    println!("\n{}", format!("=== {} Setup ===", t2_name).cyan().bold());
    let f2 = pick_formation();
    let tac2 = pick_tactic();

    let t1_players = names_db.get(&t1_name).cloned().unwrap_or_default();
    let t2_players = names_db.get(&t2_name).cloned().unwrap_or_default();

    let mut team1 = new_team(t1_name.clone(), t1_players, f1, tac1, &mut rng);
    let mut team2 = new_team(t2_name.clone(), t2_players, f2, tac2, &mut rng);

    show_formation_ascii(&team1);
    show_formation_ascii(&team2);

    prompt("Press Enter to kick off...");

    let mut state = MatchState::new();
    state.team1_has_ball = t1_kicks_off;
    // Clients control team2; host (and single-player) control team1.
    let human_team_is_t1 = !matches!(&network_mode, NetworkMode::Client(_, _));

    println!("\n{}", "🏁 KICK OFF!".bright_yellow().bold());
    println!("{}", "Tips: enter 'm' to reposition players | 'p' to pause & view insights (max 2)".dimmed());

    let total_turns = config.total_turns();
    let halftime_turn = config.halftime_turn();
    let mut halftime_shown = false;
    let mut prev_minute = 0u32;
    // Track floating-point world positions for each pos-key (updated on 'm' commands) + ball.
    let mut world_positions: HashMap<String, Position> = kickoff_world_positions();
    // Ball world position — starts at centre circle, updated each turn.
    let mut ball_world_pos: Position = Position { x: 52.5, y: 34.0 };
    let mut pauses_used: u32 = 0;

    for turn in 0..total_turns {
        let minute = config.turn_to_minute(turn);
        state.minute = minute;

        // Print a minute marker whenever the displayed minute advances to a multiple of 5
        if minute != prev_minute && minute % 5 == 0 {
            print!("{} ", format!("[{}']", minute).dimmed());
            io::stdout().flush().unwrap();
        }
        prev_minute = minute;

        let is_human_turn = (human_team_is_t1 && state.team1_has_ball)
            || (!human_team_is_t1 && !state.team1_has_ball);

        // Collect player movements ('m' command) for this turn.
        let mut turn_movements: Vec<(String, String)> = Vec::new();
        let mut turn_paused = false;

        let (human_pos, human_guess) = if is_human_turn {
            let adj = adjacent_positions(state.prev_pos.as_deref().unwrap_or("g"));
            println!("\n{} Your ball! {} mins", "▶".green(), minute);
            println!("Current pos: {}", state.prev_pos.as_deref().unwrap_or("g").yellow());
            println!("Options: {}  |  [m]=move players  [p]=pause insights", adj.join(", ").cyan());

            // Collect optional 'm'/'p' commands before the position choice.
            loop {
                let raw = prompt("Move to position (or 'm'/'p'): ");
                match raw.trim() {
                    "p" => {
                        if pauses_used < 2 {
                            pauses_used += 1;
                            turn_paused = true;
                            println!("{}", format!("⏸  Pause {}/2 — showing insights…", pauses_used).yellow());
                            render_insights(&team1, &team2, state.score1, state.score2, minute, pauses_used, &world_positions);
                            prompt("Press Enter to continue...");
                        } else {
                            println!("{}", "No pauses remaining (max 2 used).".red());
                        }
                    }
                    "m" => {
                        // Show player list and allow role-limited repositioning.
                        let pos_keys = ["g","1","2","3","4","5","6","7","8","9","0"];
                        println!("{}", "  Player positions (enter 'from to' to reposition, or blank to skip):".cyan());
                        let attacking_team = if human_team_is_t1 { &team1 } else { &team2 };
                        for &pk in &pos_keys {
                            if let Some(p) = attacking_team.player_at_pos(pk) {
                                let wp = world_positions.get(pk).copied().unwrap_or_else(|| pos_to_world(pk));
                                println!("    [{pk}] {name:22} ({role:12}) @ ({x:.1},{y:.1})m",
                                    pk=pk, name=&p.name, role=format!("{:?}", p.role),
                                    x=wp.x, y=wp.y);
                            }
                        }
                        let max_moves = 2usize; // max repositioning per turn
                        let mut moves_done = 0;
                        loop {
                            if moves_done >= max_moves { break; }
                            let mv_input = prompt("  Reposition [from to] or blank to done: ");
                            let parts: Vec<&str> = mv_input.split_whitespace().collect();
                            if parts.is_empty() { break; }
                            if parts.len() != 2 {
                                println!("{}", "  Enter two position keys, e.g. '5 7'".red());
                                continue;
                            }
                            let (from_pk, to_pk) = (parts[0], parts[1]);
                            let valid_pks = ["g","1","2","3","4","5","6","7","8","9","0"];
                            if !valid_pks.contains(&from_pk) || !valid_pks.contains(&to_pk) {
                                println!("{}", "  Invalid position key.".red());
                                continue;
                            }
                            // Update floating-point world position for the player at from_pk
                            let target_world = pos_to_world(to_pk);
                            world_positions.insert(from_pk.to_string(), target_world);
                            turn_movements.push((from_pk.to_string(), to_pk.to_string()));
                            println!("  ✓ Moved {} → {} ({:.1},{:.1})m",
                                from_pk, to_pk, target_world.x, target_world.y);
                            moves_done += 1;
                        }
                    }
                    s if adj.contains(&s) => {
                        let mv = s.to_string();

                        // In multiplayer: send our move + movements + pause flag.
                        if let Some(ref mut session) = mp_session {
                            let movements_enc: Vec<String> = turn_movements.iter()
                                .map(|(f, t)| format!("{}:{}", f, t))
                                .collect();
                            let msg = futgame::network::NetMessage {
                                turn,
                                move_zone: mv.clone(),
                                guess_zone: None,
                                movements: movements_enc,
                                pause: turn_paused,
                                ball_x: ball_world_pos.x,
                                ball_y: ball_world_pos.y,
                            };
                            if let Err(e) = session.send(&msg) {
                                eprintln!("Send error: {}", e);
                            }
                        }

                        break (Some(mv), None);
                    }
                    _ => {
                        println!("{}", "Invalid input. Choose from options above, or 'm'/'p'.".red());
                    }
                }
            }
        } else {
            let adj = adjacent_positions(state.prev_pos.as_deref().unwrap_or("g"));
            println!("\n{} Computer's ball! {} mins", "◀".red(), minute);
            println!("Options: {}  |  [m]=move players  [p]=pause", adj.join(", ").cyan());

            // Allow 'p' or 'm' before guessing
            loop {
                let raw = prompt("Guess (or 'm'/'p'): ");
                match raw.trim() {
                    "p" => {
                        if pauses_used < 2 {
                            pauses_used += 1;
                            turn_paused = true;
                            println!("{}", format!("⏸  Pause {}/2 — showing insights…", pauses_used).yellow());
                            render_insights(&team1, &team2, state.score1, state.score2, minute, pauses_used, &world_positions);
                            prompt("Press Enter to continue...");
                        } else {
                            println!("{}", "No pauses remaining.".red());
                        }
                    }
                    "m" => {
                        // Defensive repositioning (same UI as offensive)
                        let pos_keys = ["g","1","2","3","4","5","6","7","8","9","0"];
                        println!("{}", "  Player positions:".cyan());
                        let defending_team = if human_team_is_t1 { &team1 } else { &team2 };
                        for &pk in &pos_keys {
                            if let Some(p) = defending_team.player_at_pos(pk) {
                                let wp = world_positions.get(pk).copied().unwrap_or_else(|| pos_to_world(pk));
                                println!("    [{pk}] {name:22} @ ({x:.1},{y:.1})m",
                                    pk=pk, name=&p.name, x=wp.x, y=wp.y);
                            }
                        }
                        let mv_input = prompt("  Reposition [from to] or blank to skip: ");
                        let parts: Vec<&str> = mv_input.split_whitespace().collect();
                        if parts.len() == 2 {
                            let target_world = pos_to_world(parts[1]);
                            world_positions.insert(parts[0].to_string(), target_world);
                            turn_movements.push((parts[0].to_string(), parts[1].to_string()));
                        }
                    }
                    // In multiplayer: receive the opponent's actual move; in single-player: prompt for guess.
                    s if mp_session.is_some() || adj.contains(&s) => {
                        let result = if let Some(ref mut session) = mp_session {
                            match session.recv() {
                                Some(msg) => {
                                    println!("  [net] Opponent moved to zone {}", msg.move_zone);
                                    // Decode their movements
                                    for enc in &msg.movements {
                                        let parts: Vec<&str> = enc.splitn(2, ':').collect();
                                        if parts.len() == 2 {
                                            turn_movements.push((parts[0].to_string(), parts[1].to_string()));
                                        }
                                    }
                                    (None, Some(msg.move_zone))
                                }
                                None => {
                                    futgame::network::on_disconnect();
                                    // Use a placeholder to break the outer loop
                                    (None, None)
                                }
                            }
                        } else {
                            let g = s.to_string();
                            if adj.contains(&g.as_str()) {
                                (None, Some(g))
                            } else {
                                println!("{}", "Invalid position. Please guess one of the options above.".red());
                                continue;
                            }
                        };
                        if result.0.is_none() && result.1.is_none() {
                            // disconnect
                            break (None, None);
                        }
                        break result;
                    }
                    _ => {
                        println!("{}", "Invalid input.".red());
                    }
                }
            }
        };

        let movements_slice: Vec<(String, String)> = turn_movements;
        let (game_state, evts) = step_turn(
            &mut state, &mut team1, &mut team2,
            human_pos, human_guess,
            &mut rng,
            args.simple, config.turn_duration_secs, config.difficulty,
            &movements_slice,
        );

        // Formation-aware reset after a goal (kick-off world positions)
        if state.reset_needed {
            world_positions = kickoff_world_positions();
            ball_world_pos = Position { x: 52.5, y: 34.0 };
        } else {
            // Update ball world position based on the chosen zone
            let carrier_pos = pos_to_world(state.prev_pos.as_deref().unwrap_or("g"));
            ball_world_pos = move_ball_with_action(ball_world_pos, carrier_pos, "pass");
        }

        // Half-time: show once after the turn that completes minute 45
        if !halftime_shown && turn >= halftime_turn {
            show_halftime(&state, &team1, &team2);
            render_insights(&team1, &team2, state.score1, state.score2, 45, pauses_used, &world_positions);
            world_positions = kickoff_world_positions(); // reset for second half
            ball_world_pos = Position { x: 52.5, y: 34.0 };
            prompt("Press Enter for second half...");
            println!("{}", "🏁 SECOND HALF!".bright_yellow().bold());
            halftime_shown = true;
        }

        render_tactical(&game_state);
        // Per-turn movement visualization with defender intelligence and ball position
        render_movement_viz(
            ball_world_pos,
            &world_positions,
            state.defenders_deep,
        );

        for evt in &evts {
            if let Some(line) = get_commentary(evt, &commentary, &mut rng) {
                println!("  💬 {}", line.italic());
            }
            match evt {
                GameEvent::Goal { team_name, minute, .. } => {
                    println!("\n  {}", format!("⚽ GOAL! {} scores in minute {}!", team_name, minute).bright_yellow().bold());
                    show_scorecard(&state, &team1, &team2);
                }
                GameEvent::YellowCard { player, team_name, minute } => {
                    println!("  {} Yellow card! Player {} of {} in {}' ", "🟨".yellow(), player, team_name, minute);
                }
                GameEvent::RedCard { player, team_name, minute, reason } => {
                    println!("  {} Red card! Player {} of {} in {}' ({})", "🟥".red(), player, team_name, minute, reason);
                }
                GameEvent::PossessionChange { new_team } => {
                    println!("  {} {} gain possession", "↔".cyan(), new_team);
                }
                GameEvent::Save { .. } => {
                    println!("  🧤 Great save!");
                }
                GameEvent::Miss { .. } => {
                    println!("  😬 Off target!");
                }
                GameEvent::Dribble { .. } => {
                    println!("  🔥 {}", "Brilliant dribble! Beat the defender!".bright_green());
                }
                GameEvent::DribbleFail { .. } => {
                    println!("  😤 {}", "Dribble failed — lost the ball!".red());
                }
                GameEvent::Interception { zone, .. } => {
                    println!("  ⚡ {}", format!("Intercepted in zone {}!", zone).bright_red());
                }
                _ => {}
            }
        }
    }

    // Full-time insights
    render_insights(&team1, &team2, state.score1, state.score2, 90, pauses_used, &world_positions);
    show_fulltime(&state, &team1, &team2);
}
