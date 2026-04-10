// === ENHANCED: Intelligent Defender Positioning + Role-Specific Interpolation + Formation-Aware Resets + Multiplayer Sync ===
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
    println!("  ┌──────┬────────────────────────┬───────────────┬──────────────────────┬────────────────────┐");
    println!("  │ Pos  │ Name                   │ xG pos (cum.) │ Team xT avg (per pl) │ World pos (105×68) │");
    println!("  ├──────┼────────────────────────┼───────────────┼──────────────────────┼────────────────────┤");

    let pos_keys = ["g","1","2","3","4","5","6","7","8","9","0"];
    let t1_xt_avg = team1.total_xt / 11.0_f32.max(1.0);
    let t2_xt_avg = team2.total_xt / 11.0_f32.max(1.0);

    for &pk in &pos_keys {
        // Team 1 player at this position
        if let Some(p) = team1.player_at_pos(pk) {
            let xg_val = team1.xg_values.get(pk).copied().unwrap_or(0.0);
            let wp = world_positions.get(pk).copied().unwrap_or_else(|| pos_to_world(pk));
            println!("  │ {:4} │ {:22} │ {:13.4} │ {:20.4} │ ({:5.1},{:5.1})m      │",
                pk, truncate(&p.name, 22), xg_val, t1_xt_avg, wp.x, wp.y);
        }
    }
    println!("  ├──────┴────────────────────────┴───────────────┴──────────────────────┴────────────────────┤");
    println!("  │  Team 2: {}", team2.name);
    for &pk in &pos_keys {
        if let Some(p) = team2.player_at_pos(pk) {
            let xg_val = team2.xg_values.get(pk).copied().unwrap_or(0.0);
            let wp = world_positions.get(pk).copied().unwrap_or_else(|| pos_to_world(pk));
            println!("  │ {:4} │ {:22} │ {:13.4} │ {:20.4} │ ({:5.1},{:5.1})m      │",
                pk, truncate(&p.name, 22), xg_val, t2_xt_avg, wp.x, wp.y);
        }
    }
    println!("  └────────────────────────────────────────────────────────────────────────────────────────────┘");
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

/// Render a compact per-turn movement visualization.
///
/// Shows a 6-column × 3-row ASCII grid with:
///  - `B` = ball position
///  - `D` = defender position (drops deep → marked in leftmost cols)
///  - `·` = empty zone
/// Plus a line noting whether defenders are deep.
pub fn render_movement_viz(
    ball_pos_key: &str,
    world_positions: &HashMap<String, Position>,
    defenders_deep: bool,
) {
    // Map each pos_key to its (col, row) in the 6×3 grid.
    let pos_keys = ["g", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];
    // Defender position keys (indices 0-4 in 4-4-2)
    let defender_keys: &[&str] = &["g", "1", "2", "3", "4"];

    // Build occupancy grid  [col][row] -> char
    let mut grid = [[' '; 3]; 6];

    for &pk in &pos_keys {
        let wp = world_positions.get(pk).copied().unwrap_or_else(|| pos_to_world(pk));
        // Map world pos to grid col/row
        let col = ((wp.x / 105.0) * 5.0).round() as usize;
        let row = ((wp.y / 68.0) * 2.0).round() as usize;
        let col = col.min(5);
        let row = row.min(2);
        let ch = if pk == ball_pos_key {
            'B'
        } else if defender_keys.contains(&pk) {
            if grid[col][row] == ' ' { 'D' } else { 'd' }
        } else {
            if grid[col][row] == ' ' { 'p' } else { '+' }
        };
        if grid[col][row] == ' ' {
            grid[col][row] = ch;
        }
    }

    // Print grid header
    println!("  ┌────────────────────────────────────┐");
    println!("  │  Pitch [own-goal → penalty box]    │");
    println!("  ├──────┬──────┬──────┬──────┬──────┬──────┤");
    let row_labels = ["L", "C", "R"];
    for row in 0..3usize {
        print!("  │");
        for col in 0..6usize {
            let ch = if grid[col][row] == ' ' { '·' } else { grid[col][row] };
            print!("  {}   │", ch);
        }
        println!(" {}", row_labels[row]);
    }
    println!("  └──────┴──────┴──────┴──────┴──────┴──────┘");
    println!("     c0    c1    c2  | c3    c4    c5");
    println!("    [own half]       | [attacking half]");
    if defenders_deep {
        println!("  ⬇  Defenders dropped deep (ball in own half)");
    } else {
        println!("  ↑  Defenders holding line (mid-block)");
    }
    println!("  Legend: B=ball  D=defender  p=player  ·=empty");
}
