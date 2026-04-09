// === UPDATED: Step 7 - Minimal Multiplayer Skeleton + Final Polish ===

/// Selects the networking mode for the current session.
pub enum NetworkMode {
    /// Standard local single-player game (default).
    SinglePlayer,
    /// Host a multiplayer game (WIP — not yet implemented).
    Host,
    /// Connect to a hosted multiplayer game (WIP — not yet implemented).
    Client,
}

/// Wire message exchanged between host and client (skeleton only).
pub struct NetMessage {
    pub turn: u32,
    pub move_zone: u8,
    pub guess_zone: Option<u8>,
}
