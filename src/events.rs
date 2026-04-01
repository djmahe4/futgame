// === OpenFootManager-inspired ===

#[derive(Debug, Clone)]
pub struct GoalScorer {
    pub name: String,
    pub team_name: String,
    pub minute: u32,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    KickOff,
    PassSuccess { from: usize, to: usize, xt_gain: f32 },
    PassFail { from: usize },
    Dribble { player: usize, xt_gain: f32 },
    DribbleFail { player: usize },
    Tackle { defender: usize, attacker: usize },
    TackleFoul { defender: usize, attacker: usize },
    Cross { player: usize, success: bool },
    Header { player: usize },
    ShotOnTarget { player: usize, xg: f32 },
    ShotOffTarget { player: usize, xg: f32 },
    Goal { player: usize, team_name: String, minute: u32, xg: f32, xt: f32 },
    Save { goalkeeper: usize, xg: f32 },
    Miss { player: usize, xg: f32 },
    YellowCard { player: usize, team_name: String, minute: u32 },
    RedCard { player: usize, team_name: String, minute: u32, reason: String },
    Substitution { out: usize, in_: usize },
    Corner { team_name: String },
    ThrowIn { team_name: String },
    FreeKick { team_name: String, position: String },
    Offside { player: usize },
    PossessionChange { new_team: String },
    HalfTime,
    FullTime,
    Injury { player: usize, team_name: String, minute: u32 },
}
