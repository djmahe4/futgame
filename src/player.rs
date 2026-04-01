// === OpenFootManager-inspired ===
use rand::Rng;

#[derive(Debug, Clone)]
pub enum PlayerRole {
    // Goalkeepers
    Goalkeeper, SweeperKeeper, ShotStopper, BallPlayingGoalkeeper,
    // Centre Backs
    CentreBack, BallPlayingCentreBack, NoNonsenseCentreBack, Stopper, CoverDefender, Libero,
    // Full Backs
    FullBack, WingBack, InvertedFullBack, CompleteWingBack, DefensiveFullBack,
    // Defensive Midfielders
    DefensiveMidfielder, AnchorMan, HalfBack, DeepLyingPlaymaker, BallWinningMidfielder, Regista,
    // Central Midfielders
    CentralMidfielder, BoxToBoxMidfielder, AdvancedPlaymaker, RoamingPlaymaker, Carrilero, Mezzala,
    // Attacking Midfielders
    AttackingMidfielder, ShadowStriker, Enganche, Trequartista, SecondStriker,
    // Wingers
    Winger, InvertedWinger, InsideForward, WidePlaymaker, DefensiveWinger,
    // Strikers
    Striker, AdvancedForward, Poacher, TargetMan, CompleteForward, FalseNine,
    PressingForward, DeepLyingForward,
    // Extra roles
    WideTargetMan, Raumdeuter, ShadowForward, InsidePlaymaker, FalseFulBack,
    InvertedWingBack, PressingMidfielder, HoldingMidfielder, CreativePlaymaker,
    SupportStriker, WideForward, AttackingWingBack,
}

impl std::fmt::Display for PlayerRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub position_key: String,
    pub index: usize,
    pub shooting: f32,
    pub passing: f32,
    pub defending: f32,
    pub pace: f32,
    pub stamina: f32,
    pub current_stamina: f32,
    pub yellow_cards: u8,
    pub red_card: bool,
    pub role: PlayerRole,
}

pub fn role_to_xt_multiplier(role: &PlayerRole) -> f32 {
    match role {
        PlayerRole::Goalkeeper | PlayerRole::ShotStopper => 0.4,
        PlayerRole::SweeperKeeper | PlayerRole::BallPlayingGoalkeeper => 0.6,
        PlayerRole::CentreBack | PlayerRole::NoNonsenseCentreBack | PlayerRole::Stopper => 0.5,
        PlayerRole::BallPlayingCentreBack | PlayerRole::Libero | PlayerRole::CoverDefender => 0.65,
        PlayerRole::FullBack | PlayerRole::DefensiveFullBack => 0.6,
        PlayerRole::WingBack | PlayerRole::CompleteWingBack | PlayerRole::AttackingWingBack => 0.85,
        PlayerRole::InvertedFullBack | PlayerRole::FalseFulBack | PlayerRole::InvertedWingBack => 0.80,
        PlayerRole::DefensiveMidfielder | PlayerRole::AnchorMan | PlayerRole::HoldingMidfielder => 0.55,
        PlayerRole::HalfBack | PlayerRole::BallWinningMidfielder | PlayerRole::PressingMidfielder => 0.65,
        PlayerRole::DeepLyingPlaymaker | PlayerRole::Regista | PlayerRole::InsidePlaymaker => 0.85,
        PlayerRole::CentralMidfielder | PlayerRole::Carrilero => 0.75,
        PlayerRole::BoxToBoxMidfielder | PlayerRole::RoamingPlaymaker => 0.85,
        PlayerRole::AdvancedPlaymaker | PlayerRole::CreativePlaymaker | PlayerRole::Mezzala => 0.90,
        PlayerRole::AttackingMidfielder | PlayerRole::Enganche | PlayerRole::Trequartista => 0.95,
        PlayerRole::ShadowStriker | PlayerRole::SecondStriker | PlayerRole::ShadowForward => 0.95,
        PlayerRole::Winger | PlayerRole::DefensiveWinger | PlayerRole::WideForward => 0.85,
        PlayerRole::InvertedWinger | PlayerRole::InsideForward | PlayerRole::WidePlaymaker => 0.90,
        PlayerRole::WideTargetMan | PlayerRole::Raumdeuter => 0.90,
        PlayerRole::Striker | PlayerRole::AdvancedForward | PlayerRole::CompleteForward => 1.0,
        PlayerRole::Poacher | PlayerRole::TargetMan | PlayerRole::SupportStriker => 1.0,
        PlayerRole::FalseNine | PlayerRole::DeepLyingForward => 0.95,
        PlayerRole::PressingForward => 1.0,
    }
}

pub fn stamina_penalty(player: &Player) -> f32 {
    let ratio = player.current_stamina / player.stamina.max(1.0);
    ratio.clamp(0.5, 1.0)
}

pub fn apply_fatigue(player: &mut Player, minutes: u8) {
    let decay = 0.3 * minutes as f32;
    player.current_stamina = (player.current_stamina - decay).max(0.0);
}

fn default_role_for_index(index: usize) -> PlayerRole {
    match index {
        0 => PlayerRole::Goalkeeper,
        1 | 2 | 3 | 4 => PlayerRole::CentreBack,
        5 | 6 | 7 | 8 => PlayerRole::CentralMidfielder,
        _ => PlayerRole::Striker,
    }
}

pub fn new_player(name: String, position_key: String, index: usize, rng: &mut impl Rng) -> Player {
    let shooting: f32 = rng.gen_range(40.0..90.0);
    let passing: f32 = rng.gen_range(40.0..90.0);
    let defending: f32 = rng.gen_range(40.0..90.0);
    let pace: f32 = rng.gen_range(50.0..95.0);
    let stamina: f32 = rng.gen_range(60.0..95.0);
    Player {
        name,
        position_key,
        index,
        shooting,
        passing,
        defending,
        pace,
        stamina,
        current_stamina: stamina,
        yellow_cards: 0,
        red_card: false,
        role: default_role_for_index(index),
    }
}
