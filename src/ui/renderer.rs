// === UPDATED: Step 6 - Lightweight GameState + Tactical Rendering ===

use crate::state::{GameState, TurnEvent};

/// Print a minimal one-line tactical summary for the current turn.
/// Called after every `step_turn` to log ball position and last event.
pub fn render_tactical(state: &GameState) {
    let event_desc = match state.last_event {
        Some(TurnEvent::Move { from, to, .. }) => format!("move {} → {}", from, to),
        Some(TurnEvent::Shot { success: true, .. }) => "shot → ⚽ GOAL".to_string(),
        Some(TurnEvent::Shot { success: false, .. }) => "shot → blocked/missed".to_string(),
        Some(TurnEvent::Foul { .. }) => "foul committed".to_string(),
        None => "no event".to_string(),
    };
    println!("  [state] turn={} ball_zone={} | {}", state.turn, state.ball.zone, event_desc);
}
