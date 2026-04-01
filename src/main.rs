use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

use clap::Parser;
use colored::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use futgame::config::GameConfig;
use futgame::events::GameEvent;
use futgame::simulation::{step_minute, MatchState};
use futgame::tactics::{Formation, Tactic};
use futgame::team::{new_team, Team};
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

fn get_commentary(evt: &GameEvent, commentary: &[String], rng: &mut impl Rng) -> Option<String> {
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
    let mut rng = SmallRng::from_entropy();
    let commentary = load_commentary();
    let names_db = load_names();

    // Build the match configuration from CLI args
    let config = GameConfig::with_turn_duration(args.turn_duration);

    println!("{}", "╔═══════════════════════════════════╗".bright_green().bold());
    println!("{}", "║       ⚽  FutGame  ⚽              ║".bright_green().bold());
    println!("{}", "║   Rust CLI Football Simulator      ║".bright_green().bold());
    println!("{}", "╚═══════════════════════════════════╝".bright_green().bold());

    if args.simple {
        println!("{}", "Mode: Simple (xG only, no xT)".yellow());
    } else {
        println!("{}", "Mode: Full (xG + xT)".green());
    }
    println!("⏱  Turn timing: {}", config.describe().cyan());

    let team_names: Vec<String> = names_db.keys().cloned().collect();
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
    let human_team_is_t1 = true;

    println!("\n{}", "🏁 KICK OFF!".bright_yellow().bold());

    let total_turns = config.total_turns();
    let halftime_turn = config.halftime_turn();
    let mut halftime_shown = false;
    let mut prev_minute = 0u32;

    for turn in 0..total_turns {
        let minute = config.turn_to_minute(turn);
        state.minute = minute;

        // Half-time: show once when we first reach minute 45
        if !halftime_shown && turn >= halftime_turn {
            show_halftime(&state, &team1, &team2);
            prompt("Press Enter for second half...");
            println!("{}", "🏁 SECOND HALF!".bright_yellow().bold());
            halftime_shown = true;
        }

        // Print a minute marker whenever the displayed minute advances to a multiple of 5
        if minute != prev_minute && minute % 5 == 0 {
            print!("{} ", format!("[{}']", minute).dimmed());
            io::stdout().flush().unwrap();
        }
        prev_minute = minute;

        let is_human_turn = (human_team_is_t1 && state.team1_has_ball)
            || (!human_team_is_t1 && !state.team1_has_ball);

        let (human_pos, human_guess) = if is_human_turn {
            let adj = adjacent_positions(state.prev_pos.as_deref().unwrap_or("g"));
            println!("\n{} Your ball! {} mins", "▶".green(), minute);
            println!("Current pos: {}", state.prev_pos.as_deref().unwrap_or("g").yellow());
            println!("Options: {}", adj.join(", ").cyan());
            let chosen = prompt("Move to position: ");
            let valid = adj.contains(&chosen.as_str());
            let mv = if valid { chosen } else { adj[0].to_string() };
            (Some(mv), None)
        } else {
            let adj = adjacent_positions(state.prev_pos.as_deref().unwrap_or("g"));
            println!("\n{} Computer's ball! {} mins", "◀".red(), minute);
            println!("Options: {}", adj.join(", ").cyan());
            let guess = prompt("Guess where they'll move: ");
            (None, Some(guess))
        };

        let evts = step_minute(&mut state, &mut team1, &mut team2, human_pos, human_guess, &mut rng, args.simple);

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
                _ => {}
            }
        }
    }

    show_fulltime(&state, &team1, &team2);
}
