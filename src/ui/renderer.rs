// === ENHANCED: Floating-Point Position System (105x68m) + 'm' Per-Guess Movements + 'p' Pause + Dribble/Interception + Insights Viz ===
// === UPDATED: Step 6 - Lightweight GameState + Tactical Rendering ===

use std::collections::HashMap;

use crate::pitch::{pos_to_world, Position};
use crate::state::{GameState, TurnEvent};
use crate::team::Team;
use crate::xt::get_zone_xt;

/// Print a minimal one-line tactical summary for the current turn.
/// Called after every `step_turn` to log ball position and last event.
pub fn render_tactical(state: &GameState) {
    let event_desc = match state.last_event {
        Some(TurnEvent::Move { from, to, .. }) => format!("move {} → {}", from, to),
        Some(TurnEvent::Shot { success: true, .. }) => "shot → ⚽ GOAL".to_string(),
        Some(TurnEvent::Shot { success: false, .. }) => "shot → blocked/missed".to_string(),
        Some(TurnEvent::Foul { .. }) => "foul committed".to_string(),
        Some(TurnEvent::Dribble { success: true, .. }) => "dribble → beat defender!".to_string(),
        Some(TurnEvent::Dribble { success: false, .. }) => "dribble → lost possession".to_string(),
        Some(TurnEvent::Interception { defender_id, .. }) => {
            format!("⚡ intercepted by player {}", defender_id)
        }
        None => "no event".to_string(),
    };
    println!("  [state] turn={} ball_zone={} | {}", state.turn, state.ball.zone, event_desc);
}

/// Full coach-level insights snapshot.
/// Shown at halftime, fulltime, and on 'p' pause.
///
/// Displays:
///  - 6×3 ASCII xT heatmap with gradient  `. : - = # @`
///  - Player xT/xG summary table
///  - Floating-point position overlay (world coords on 105×68 pitch)
///  - Movement trajectory hints
///  - Pauses used counter
pub fn render_insights(
    team1: &Team,
    team2: &Team,
    score1: u32,
    score2: u32,
    minute: u32,
    pauses_used: u32,
    world_positions: &HashMap<String, Position>,
) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                  📊  COACH INSIGHTS                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("  {} {}-{} {}  [{}']  pauses: {}/2",
        team1.name, score1, score2, team2.name, minute, pauses_used);
    println!();

    // --- 6×3 xT heatmap ---
    println!("  xT Heatmap  (col: own-goal→penalty box | row: L/C/R)");
    println!("  ┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐");
    let row_labels = ["L", "C", "R"];
    for row in 0..3usize {
        print!(" {} │", row_labels[row]);
        for col in 0..6usize {
            let xt = get_zone_xt(col, row);
            let ch = xt_to_char(xt);
            print!("  {:^6}  │", format!("{}{:.3}", ch, xt));
        }
        println!();
    }
    println!("  └──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘");
    println!("    col:  0-GK      1-Own     2-O.Mid   3-A.Mid   4-Final   5-Pen.box");
    println!("    key: .=low  :=low-med  -=med  ==med-high  #=high  @=max");
    println!();

    // --- Player position table ---
    println!("  ┌──────┬────────────────────────┬───────────────┬──────────────┬────────────────────┐");
    println!("  │ Pos  │ Name                   │ xG (cumul.)   │ xT (cumul.)  │ World pos (105×68) │");
    println!("  ├──────┼────────────────────────┼───────────────┼──────────────┼────────────────────┤");

    let pos_keys = ["g","1","2","3","4","5","6","7","8","9","0"];

    for &pk in &pos_keys {
        // Team 1 player at this position
        if let Some(p) = team1.player_at_pos(pk) {
            let xg_val = team1.xg_values.get(pk).copied().unwrap_or(0.0);
            let wp = world_positions.get(pk).copied().unwrap_or_else(|| pos_to_world(pk));
            println!("  │ {:4} │ {:22} │ {:13.4} │ {:12.4} │ ({:5.1},{:5.1})m      │",
                pk, truncate(&p.name, 22), xg_val, team1.total_xt / 11.0_f32.max(1.0), wp.x, wp.y);
        }
    }
    println!("  ├──────┴────────────────────────┴───────────────┴──────────────┴────────────────────┤");
    println!("  │  Team 2: {}", team2.name);
    for &pk in &pos_keys {
        if let Some(p) = team2.player_at_pos(pk) {
            let xg_val = team2.xg_values.get(pk).copied().unwrap_or(0.0);
            let wp = world_positions.get(pk).copied().unwrap_or_else(|| pos_to_world(pk));
            println!("  │ {:4} │ {:22} │ {:13.4} │ {:12.4} │ ({:5.1},{:5.1})m      │",
                pk, truncate(&p.name, 22), xg_val, team2.total_xt / 11.0_f32.max(1.0), wp.x, wp.y);
        }
    }
    println!("  └──────────────────────────────────────────────────────────────────────────────────────┘");
    println!();
}

/// Map an xT value to a gradient character for the ASCII heatmap.
fn xt_to_char(xt: f32) -> char {
    if xt < 0.005 { '.' }
    else if xt < 0.020 { ':' }
    else if xt < 0.060 { '-' }
    else if xt < 0.150 { '=' }
    else if xt < 0.300 { '#' }
    else { '@' }
}

/// Truncate a string to at most `max_len` characters.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}
