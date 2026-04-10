// === ENHANCED: Floating-Point Position System (105x68m) + 'm' Per-Guess Movements + 'p' Pause + Dribble/Interception + Insights Viz ===
// === UPDATED: Step 6 - Lightweight GameState + Tactical Rendering ===

/// Lightweight per-turn snapshot of a single player's position.
#[derive(Clone, Debug)]
pub struct PlayerState {
    pub id: usize,
    pub name: String,
    /// Zone index (0 = GK area, 1-4 = defence, 5-8 = midfield, 9-10 = attack).
    /// Matches `position_index()` from `xg.rs`.
    pub zone: u8,
    pub role: String,
}

/// Where the ball is and (optionally) who holds it.
#[derive(Clone, Debug)]
pub struct BallState {
    pub zone: u8,
    pub possessed_by: Option<usize>,
}

/// Lightweight summary of the most significant thing that happened this turn.
#[derive(Clone, Debug)]
pub enum TurnEvent {
    Move { player_id: usize, from: u8, to: u8 },
    Shot { player_id: usize, success: bool },
    Foul { player_id: usize },
    Dribble { player_id: usize, success: bool },
    Interception { defender_id: usize },
}

/// Full lightweight snapshot of the game after one turn resolves.
#[derive(Clone, Debug)]
pub struct GameState {
    pub turn: u32,
    pub players: Vec<PlayerState>,
    pub ball: BallState,
    pub last_event: Option<TurnEvent>,
}
