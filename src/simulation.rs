// === ENHANCED: Intelligent Defender Positioning + Role-Specific Interpolation + Formation-Aware Resets + Multiplayer Sync ===
// === ENHANCED: Floating-Point Position System (105x68m) + 'm' Per-Guess Movements + 'p' Pause + Dribble/Interception + Insights Viz ===
// === UPDATED: Step 5 - AI Difficulty + Role Movement Constraints ===
// === UPDATED: Step 6 - Lightweight GameState + Tactical Rendering ===
// === OpenFootManager-inspired ===
// === Original xG Core ===
// === xT Layer (New) ===

use std::collections::HashMap;
use rand::Rng;

use crate::config::Difficulty;
use crate::events::{GameEvent, GoalScorer};
use crate::pitch::{get_path_zones, pos_to_world, Position};
use crate::player::PlayerRole;
use crate::state::{BallState, GameState, PlayerState, TurnEvent};
use crate::team::Team;
use crate::xg::{adjacent_positions, base_xg, def_xg, determine_outcome, position_index};
use crate::xt::{get_zone_xt, position_to_zone, xt_to_xg_modifier};
use crate::tactics::tactic_xt_multiplier;

pub struct MatchState {
    pub minute: u32,
    pub team1_has_ball: bool,
    pub score1: u32,
    pub score2: u32,
    pub goal_scorers: Vec<GoalScorer>,
    pub events: Vec<GameEvent>,
    pub poss1: u32,
    pub poss2: u32,
    pub current_zone: (usize, usize),
    pub prev_pos: Option<String>,
    pub options: Vec<String>,
    pub home_advantage: f32,
    /// True when the defending team has dropped into their own half (ball in cols 0-2).
    pub defenders_deep: bool,
    /// Set to true by step_turn when a goal is scored — callers should reset world positions.
    pub reset_needed: bool,
}

impl MatchState {
    pub fn new() -> Self {
        MatchState {
            minute: 0,
            team1_has_ball: true,
            score1: 0,
            score2: 0,
            goal_scorers: Vec::new(),
            events: Vec::new(),
            poss1: 0,
            poss2: 0,
            current_zone: (2, 1), // midfield central in the new 6×3 grid
            prev_pos: None,
            options: crate::xg::adjacent_positions("g").iter().map(|s| s.to_string()).collect(),
            home_advantage: 0.05,
            defenders_deep: false,
            reset_needed: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Role-specific helpers (interpolation, favourite zones, movement speed)
// ---------------------------------------------------------------------------

/// Return the preferred (col, row) zone on the 6×3 xT grid for a given role.
/// Used by `role_interpolate` to bias movement toward the role's natural area.
pub fn role_favourite_zone(role: &PlayerRole) -> (usize, usize) {
    match role {
        PlayerRole::Goalkeeper | PlayerRole::ShotStopper
        | PlayerRole::BallPlayingGoalkeeper | PlayerRole::SweeperKeeper => (0, 1),

        PlayerRole::CentreBack | PlayerRole::NoNonsenseCentreBack
        | PlayerRole::Stopper | PlayerRole::CoverDefender => (1, 1),

        PlayerRole::BallPlayingCentreBack | PlayerRole::Libero => (1, 1),

        PlayerRole::FullBack | PlayerRole::DefensiveFullBack => (1, 0),
        PlayerRole::WingBack | PlayerRole::CompleteWingBack | PlayerRole::AttackingWingBack => (2, 0),
        PlayerRole::InvertedFullBack | PlayerRole::FalseFullBack | PlayerRole::InvertedWingBack => (2, 1),

        PlayerRole::DefensiveMidfielder | PlayerRole::AnchorMan
        | PlayerRole::HoldingMidfielder | PlayerRole::HalfBack => (2, 1),
        PlayerRole::BallWinningMidfielder | PlayerRole::PressingMidfielder => (2, 1),
        PlayerRole::DeepLyingPlaymaker | PlayerRole::Regista | PlayerRole::InsidePlaymaker => (2, 1),

        PlayerRole::CentralMidfielder | PlayerRole::Carrilero => (3, 1),
        PlayerRole::BoxToBoxMidfielder | PlayerRole::RoamingPlaymaker => (3, 1),
        PlayerRole::AdvancedPlaymaker | PlayerRole::CreativePlaymaker | PlayerRole::Mezzala => (3, 1),

        PlayerRole::AttackingMidfielder | PlayerRole::Enganche | PlayerRole::Trequartista => (4, 1),
        PlayerRole::ShadowStriker | PlayerRole::SecondStriker | PlayerRole::ShadowForward => (4, 1),

        PlayerRole::Winger | PlayerRole::DefensiveWinger | PlayerRole::WideForward => (3, 0),
        PlayerRole::InvertedWinger | PlayerRole::InsideForward | PlayerRole::WidePlaymaker => (4, 0),
        PlayerRole::WideTargetMan | PlayerRole::Raumdeuter => (4, 0),

        PlayerRole::Striker | PlayerRole::AdvancedForward | PlayerRole::CompleteForward => (5, 1),
        PlayerRole::Poacher | PlayerRole::TargetMan | PlayerRole::SupportStriker => (5, 1),
        PlayerRole::FalseNine | PlayerRole::DeepLyingForward => (4, 1),
        PlayerRole::PressingForward => (4, 1),
    }
}

/// Movement speed factor (zones per turn) for each role.
/// Values in range 0.3–1.0; a value of 1.0 means the player can move a full zone each turn.
pub fn role_movement_speed(role: &PlayerRole) -> f32 {
    match role {
        PlayerRole::Goalkeeper | PlayerRole::ShotStopper => 0.3,
        PlayerRole::BallPlayingGoalkeeper | PlayerRole::SweeperKeeper => 0.4,

        PlayerRole::CentreBack | PlayerRole::NoNonsenseCentreBack | PlayerRole::Stopper => 0.5,
        PlayerRole::BallPlayingCentreBack | PlayerRole::CoverDefender | PlayerRole::Libero => 0.6,

        PlayerRole::FullBack | PlayerRole::DefensiveFullBack => 0.7,
        PlayerRole::WingBack | PlayerRole::CompleteWingBack | PlayerRole::AttackingWingBack => 0.85,
        PlayerRole::InvertedFullBack | PlayerRole::FalseFullBack | PlayerRole::InvertedWingBack => 0.75,

        PlayerRole::DefensiveMidfielder | PlayerRole::AnchorMan | PlayerRole::HoldingMidfielder => 0.6,
        PlayerRole::HalfBack | PlayerRole::BallWinningMidfielder | PlayerRole::PressingMidfielder => 0.7,
        PlayerRole::DeepLyingPlaymaker | PlayerRole::Regista | PlayerRole::InsidePlaymaker => 0.7,

        PlayerRole::CentralMidfielder | PlayerRole::Carrilero => 0.8,
        PlayerRole::BoxToBoxMidfielder | PlayerRole::RoamingPlaymaker => 0.9,
        PlayerRole::AdvancedPlaymaker | PlayerRole::CreativePlaymaker | PlayerRole::Mezzala => 0.85,

        PlayerRole::AttackingMidfielder | PlayerRole::Enganche | PlayerRole::Trequartista => 0.8,
        PlayerRole::ShadowStriker | PlayerRole::SecondStriker | PlayerRole::ShadowForward => 0.85,

        PlayerRole::Winger | PlayerRole::DefensiveWinger | PlayerRole::WideForward => 0.9,
        PlayerRole::InvertedWinger | PlayerRole::InsideForward | PlayerRole::WidePlaymaker => 0.9,
        PlayerRole::WideTargetMan | PlayerRole::Raumdeuter => 0.85,

        PlayerRole::Striker | PlayerRole::AdvancedForward | PlayerRole::CompleteForward => 0.95,
        PlayerRole::Poacher | PlayerRole::TargetMan => 0.85,
        PlayerRole::SupportStriker => 0.9,
        PlayerRole::FalseNine | PlayerRole::DeepLyingForward => 0.8,
        PlayerRole::PressingForward => 0.9,
    }
}

/// Move a player from `current` toward `target` at role-appropriate speed.
/// Each turn's maximum distance is `speed × 17.5m` (one zone ≈ 17.5m on a 105m pitch).
/// Biases final direction toward the role's favourite zone if the target is the same as current.
pub fn role_interpolate(
    current: Position,
    target: Position,
    role: &PlayerRole,
    _turn_duration_secs: u32,
) -> Position {
    let speed = role_movement_speed(role);
    let max_dist = speed * 17.5_f32; // metres per turn

    let dx = target.x - current.x;
    let dy = target.y - current.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= 0.01 {
        return current; // already at target
    }
    if dist <= max_dist {
        return Position {
            x: target.x.clamp(0.0, 105.0),
            y: target.y.clamp(0.0, 68.0),
        };
    }
    let scale = max_dist / dist;
    Position {
        x: (current.x + dx * scale).clamp(0.0, 105.0),
        y: (current.y + dy * scale).clamp(0.0, 68.0),
    }
}

/// Reset all player world positions to the standard kickoff layout (based on `pos_to_world`).
/// Called by main.rs after goals and at match/half kick-off.
pub fn kickoff_world_positions() -> HashMap<String, Position> {
    let mut m = HashMap::new();
    for &pk in &["g", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] {
        m.insert(pk.to_string(), pos_to_world(pk));
    }
    m
}

// ---------------------------------------------------------------------------
// Step 5 helpers
// ---------------------------------------------------------------------------

/// Maximum number of zones a player can move per turn based on their role.
/// Strikers and Midfielders are more mobile (2); Defenders/GKs stay conservative (1).
fn max_movement(role: &str) -> u8 {
    match role {
        "Striker" | "Midfielder" => 2,
        "Defender" => 1,
        _ => 1,
    }
}

/// AI picks a zone to move to from `adj`, biased toward higher-xT zones
/// based on the current difficulty level.
fn ai_pick_move(adj: &[String], difficulty: Difficulty, rng: &mut impl Rng) -> String {
    match difficulty {
        Difficulty::Easy => adj[rng.gen_range(0..adj.len())].clone(),
        Difficulty::Medium => {
            // Slight xT bias: score = rand + 1.5 × zone_xt
            let scores: Vec<f32> = adj.iter().map(|pos| {
                let (zx, zy) = position_to_zone(pos, true);
                rng.gen::<f32>() + 1.5 * get_zone_xt(zx, zy)
            }).collect();
            let best = scores.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i).unwrap_or(0);
            adj[best].clone()
        }
        Difficulty::Hard | Difficulty::Insane => {
            // Strong xT bias: score = rand + 5.0 × zone_xt
            let scores: Vec<f32> = adj.iter().map(|pos| {
                let (zx, zy) = position_to_zone(pos, true);
                rng.gen::<f32>() + 5.0 * get_zone_xt(zx, zy)
            }).collect();
            let best = scores.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i).unwrap_or(0);
            adj[best].clone()
        }
    }
}

// ---------------------------------------------------------------------------
// Step 6 helpers
// ---------------------------------------------------------------------------

/// Build a lightweight GameState snapshot from the current match state.
fn build_game_state(
    match_state: &MatchState,
    team1: &Team,
    team2: &Team,
    last_event: Option<TurnEvent>,
) -> GameState {
    let ball_zone = position_index(match_state.prev_pos.as_deref().unwrap_or("g")) as u8;
    let mut players = Vec::with_capacity(team1.players.len() + team2.players.len());
    for (i, p) in team1.players.iter().enumerate() {
        players.push(PlayerState {
            id: i,
            name: p.name.clone(),
            zone: position_index(&p.position_key) as u8,
            role: format!("{:?}", p.role),
        });
    }
    for (i, p) in team2.players.iter().enumerate() {
        players.push(PlayerState {
            id: team1.players.len() + i,
            name: p.name.clone(),
            zone: position_index(&p.position_key) as u8,
            role: format!("{:?}", p.role),
        });
    }
    GameState {
        turn: match_state.minute,
        players,
        ball: BallState { zone: ball_zone, possessed_by: None },
        last_event,
    }
}

/// Derive a lightweight TurnEvent from the first significant GameEvent in a list.
fn events_to_turn_event(events: &[GameEvent]) -> Option<TurnEvent> {
    for e in events {
        match e {
            GameEvent::Goal { player, .. } =>
                return Some(TurnEvent::Shot { player_id: *player, success: true }),
            GameEvent::Save { .. } =>
                return Some(TurnEvent::Shot { player_id: 0, success: false }),
            GameEvent::Miss { player, .. } =>
                return Some(TurnEvent::Shot { player_id: *player, success: false }),
            GameEvent::PassSuccess { from, to, .. } =>
                return Some(TurnEvent::Move { player_id: *to, from: *from as u8, to: *to as u8 }),
            GameEvent::TackleFoul { attacker, .. } =>
                return Some(TurnEvent::Foul { player_id: *attacker }),
            GameEvent::Dribble { player, .. } =>
                return Some(TurnEvent::Dribble { player_id: *player, success: true }),
            GameEvent::DribbleFail { player } =>
                return Some(TurnEvent::Dribble { player_id: *player, success: false }),
            GameEvent::Interception { defender, .. } =>
                return Some(TurnEvent::Interception { defender_id: *defender }),
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// resolve_shot
// ---------------------------------------------------------------------------

pub fn resolve_shot(
    player_idx: usize,
    team: &mut Team,
    pos_key: &str,
    index: usize,
    minute: u32,
    rng: &mut impl Rng,
    xt_modifier: f32,
) -> GameEvent {
    let current = *team.xg_values.get(pos_key).unwrap_or(&base_xg(pos_key));
    let new_xg = def_xg(pos_key, current, index);
    team.xg_values.insert(pos_key.to_string(), new_xg);

    // Scale xt_modifier (±0.05 range) by 0.1 so xT adds at most ±0.005 to effective xG,
    // keeping the xT layer as an enrichment rather than a dominant factor.
    let effective_xg = new_xg + xt_modifier * 0.1;
    team.shots += 1;
    team.total_xg += effective_xg;

    let outcome = determine_outcome(effective_xg, rng);
    match outcome {
        0 => {
            team.shots_on_target += 1;
            GameEvent::Goal {
                player: player_idx,
                team_name: team.name.clone(),
                minute,
                xg: effective_xg,
                xt: xt_modifier,
            }
        }
        2 => {
            team.shots_on_target += 1;
            GameEvent::Save {
                goalkeeper: 0,
                xg: effective_xg,
            }
        }
        _ => GameEvent::Miss {
            player: player_idx,
            xg: effective_xg,
        },
    }
}

// ---------------------------------------------------------------------------
// step_turn
// ---------------------------------------------------------------------------

/// `player_movements`: list of (from_pos_key, to_pos_key) repositioning choices made via
/// the 'm' command.  Each entry gives a small xT boost if the target zone has higher xT.
pub fn step_turn(
    state: &mut MatchState,
    team1: &mut Team,
    team2: &mut Team,
    human_pos: Option<String>,
    human_guess: Option<String>,
    rng: &mut impl Rng,
    simple_mode: bool,
    turn_duration_secs: u32,
    difficulty: Difficulty,
    player_movements: &[(String, String)],
) -> (GameState, Vec<GameEvent>) {
    let mut events: Vec<GameEvent> = Vec::new();

    // Apply xT boosts from 'm' movements (role-constrained repositioning).
    for (from_pos, to_pos) in player_movements {
        let (fx, fy) = position_to_zone(from_pos, true);
        let (tx, ty) = position_to_zone(to_pos, true);
        let from_xt = get_zone_xt(fx, fy);
        let to_xt   = get_zone_xt(tx, ty);
        if to_xt > from_xt {
            let boost = (to_xt - from_xt) * 0.5; // partial boost for movement
            if state.team1_has_ball {
                team1.total_xt += boost;
            } else {
                team2.total_xt += boost;
            }
        }
    }

    // Track possession
    if state.team1_has_ball {
        state.poss1 += 1;
    } else {
        state.poss2 += 1;
    }

    // Clear reset flag at the start of each turn
    state.reset_needed = false;

    let current_pos = state.prev_pos.clone().unwrap_or_else(|| "g".to_string());

    let adj = adjacent_positions(&current_pos);
    let adj_owned: Vec<String> = adj.iter().map(|s| s.to_string()).collect();
    state.options = adj_owned.clone();

    // Determine role and movement range of the current ball carrier (for logging)
    let (carrier_name, carrier_role) = {
        let team = if state.team1_has_ball { &*team1 } else { &*team2 };
        let p = team.player_at_pos(&current_pos);
        (
            p.map(|pl| pl.name.clone()).unwrap_or_else(|| "Unknown".to_string()),
            p.map(|pl| format!("{:?}", pl.role)).unwrap_or_else(|| "Midfielder".to_string()),
        )
    };
    let _move_range = max_movement(&carrier_role);

    // --- Choose where ball moves ---
    let chosen_pos = if let Some(pos) = human_pos {
        if adj_owned.contains(&pos) { pos } else { adj_owned[rng.gen_range(0..adj_owned.len())].clone() }
    } else {
        ai_pick_move(&adj_owned, difficulty, rng)
    };

    // -----------------------------------------------------------------------
    // Dribble check: player chose to stay at the same position (same-pos trigger)
    // -----------------------------------------------------------------------
    if chosen_pos == current_pos {
        let player_xg = if state.team1_has_ball {
            *team1.xg_values.get(&chosen_pos).unwrap_or(&base_xg(&chosen_pos))
        } else {
            *team2.xg_values.get(&chosen_pos).unwrap_or(&base_xg(&chosen_pos))
        };
        let tactic_mult_pre = if state.team1_has_ball {
            tactic_xt_multiplier(&team1.tactic)
        } else {
            tactic_xt_multiplier(&team2.tactic)
        };
        let scale_pre = turn_duration_secs as f32 / 60.0;
        let dribble_chance = (player_xg * tactic_mult_pre).clamp(0.1, 0.95) * scale_pre;
        let dribble_roll: f32 = rng.gen();
        let current_pos_idx = position_index(&chosen_pos);
        if dribble_roll < dribble_chance {
            // Dribble success: +0.15 xT boost, bypass defender
            if state.team1_has_ball { team1.total_xt += 0.15; } else { team2.total_xt += 0.15; }
            events.push(GameEvent::Dribble { player: current_pos_idx, xt_gain: 0.15 });
            state.prev_pos = Some(chosen_pos.clone());
            let last_evt = events_to_turn_event(&events);
            return (build_game_state(state, team1, team2, last_evt), events);
        } else {
            // Dribble fail: possession loss + small foul chance
            let foul_roll: f32 = rng.gen();
            events.push(GameEvent::DribbleFail { player: current_pos_idx });
            if foul_roll < 0.05 * scale_pre {
                events.push(GameEvent::TackleFoul { defender: current_pos_idx, attacker: current_pos_idx });
            }
            events.push(GameEvent::PossessionChange {
                new_team: if state.team1_has_ball { team2.name.clone() } else { team1.name.clone() },
            });
            state.team1_has_ball = !state.team1_has_ball;
            state.prev_pos = Some("g".to_string());
            let last_evt = events_to_turn_event(&events);
            return (build_game_state(state, team1, team2, last_evt), events);
        }
    }

    // --- Determine if defence intercepts (guess-based) ---
    let intercept = if let Some(ref g) = human_guess {
        g == &chosen_pos
    } else {
        match difficulty {
            Difficulty::Insane => {
                // Two independent guesses; succeed if either matches
                let g1 = adj_owned[rng.gen_range(0..adj_owned.len())].clone();
                let g2 = adj_owned[rng.gen_range(0..adj_owned.len())].clone();
                g1 == chosen_pos || g2 == chosen_pos
            }
            _ => {
                let g = adj_owned[rng.gen_range(0..adj_owned.len())].clone();
                g == chosen_pos
            }
        }
    };

    if intercept {
        let new_team = if state.team1_has_ball { team2.name.clone() } else { team1.name.clone() };
        events.push(GameEvent::PossessionChange { new_team });
        state.team1_has_ball = !state.team1_has_ball;
        state.prev_pos = Some("g".to_string());
        return (build_game_state(state, team1, team2, None), events);
    }

    // --- xT calculation ---
    let (zx, zy) = position_to_zone(&chosen_pos, true);
    let xt_val = get_zone_xt(zx, zy);

    // Defenders drop deep when the ball is in the own half (cols 0-2)
    state.defenders_deep = zx <= 2;

    let tactic_mult = if state.team1_has_ball {
        tactic_xt_multiplier(&team1.tactic)
    } else {
        tactic_xt_multiplier(&team2.tactic)
    };

    let xt_mod = if simple_mode {
        0.0
    } else {
        xt_to_xg_modifier(xt_val * tactic_mult)
    };

    if state.team1_has_ball {
        team1.total_xt += xt_val;
    } else {
        team2.total_xt += xt_val;
    }

    // Tactical log: show which player moved and how much tactical xT boost applies
    println!("  [xT] {} chose zone {} (tactic boost {:.2})",
        carrier_name, chosen_pos, tactic_mult);

    // All probabilities scaled by turn_duration_secs/60 so expected events per 90 min remain consistent.
    let scale = turn_duration_secs as f32 / 60.0;

    // -----------------------------------------------------------------------
    // Path-based interception: check intermediate zones on the xT grid.
    // Low-xT (defensive) zones = higher interception chance.
    // Uses a small per-zone multiplier (0.02) to keep rates realistic.
    // -----------------------------------------------------------------------
    {
        let (start_zx, start_zy) = position_to_zone(&current_pos, state.team1_has_ball);
        let start_zone = (start_zx * 3 + start_zy) as u8;
        let end_zone   = (zx * 3 + zy) as u8;
        let path = get_path_zones(start_zone, end_zone);
        for &pz in &path {
            let pz_col = (pz / 3) as usize;
            let pz_row = (pz % 3) as usize;
            let zone_xt = get_zone_xt(pz_col, pz_row);
            // Inverted logic: defenders in low-xT zones are most effective
            let interception_chance = (1.0 - zone_xt).clamp(0.1, 0.9) * 0.02 * scale;
            let iroll: f32 = rng.gen();
            if iroll < interception_chance {
                let pos_idx_path = position_index(&chosen_pos);
                events.push(GameEvent::Interception {
                    defender: pos_idx_path,
                    attacker: pos_idx_path,
                    zone: pz,
                });
                events.push(GameEvent::PossessionChange {
                    new_team: if state.team1_has_ball { team2.name.clone() } else { team1.name.clone() },
                });
                state.team1_has_ball = !state.team1_has_ball;
                state.prev_pos = Some("g".to_string());
                let last_evt = events_to_turn_event(&events);
                return (build_game_state(state, team1, team2, last_evt), events);
            }
        }
    }

    let pos_idx = position_index(&chosen_pos);
    let is_attacking_pos = matches!(chosen_pos.as_str(), "9" | "0" | "8" | "7");

    if is_attacking_pos {
        let base_shot_prob: f32 = match chosen_pos.as_str() {
            "9" | "0" => 0.5,
            "8" | "7" => 0.3,
            _ => 0.2,
        };
        let shot_prob = base_shot_prob * scale;
        let roll: f32 = rng.gen();
        if roll < shot_prob {
            let evt = if state.team1_has_ball {
                resolve_shot(pos_idx, team1, &chosen_pos, pos_idx, state.minute, rng, xt_mod)
            } else {
                resolve_shot(pos_idx, team2, &chosen_pos, pos_idx, state.minute, rng, xt_mod)
            };

            let is_goal = matches!(evt, GameEvent::Goal { .. });
            if is_goal {
                if let GameEvent::Goal { player, ref team_name, minute, xg: _, .. } = evt {
                    let pname = if state.team1_has_ball {
                        team1.players.get(player).map(|p| p.name.clone())
                    } else {
                        team2.players.get(player).map(|p| p.name.clone())
                    }
                    .unwrap_or_else(|| format!("Player {}", player));

                    state.goal_scorers.push(GoalScorer {
                        name: pname,
                        team_name: team_name.clone(),
                        minute,
                    });
                    if state.team1_has_ball {
                        state.score1 += 1;
                    } else {
                        state.score2 += 1;
                    }
                    events.push(evt);
                    state.prev_pos = Some("g".to_string());
                    state.team1_has_ball = !state.team1_has_ball;
                    state.reset_needed = true; // signal caller to reset world positions
                    let last_evt = events_to_turn_event(&events);
                    return (build_game_state(state, team1, team2, last_evt), events);
                }
            }
            events.push(evt);
        } else {
            events.push(GameEvent::PassSuccess {
                from: position_index(&current_pos),
                to: pos_idx,
                xt_gain: xt_val,
            });
        }
    } else {
        let foul_prob = 0.03 * scale;
        let foul_roll: f32 = rng.gen();
        if foul_roll < foul_prob {
            events.push(GameEvent::TackleFoul {
                defender: pos_idx,
                attacker: pos_idx,
            });
            // Yellow card check
            let yellow_roll: f32 = rng.gen();
            if yellow_roll < 0.2 {
                let attacking_team = if state.team1_has_ball { &mut *team1 } else { &mut *team2 };
                if let Some(player) = attacking_team.players.get_mut(pos_idx) {
                    player.yellow_cards += 1;
                    if player.yellow_cards >= 2 {
                        player.red_card = true;
                        events.push(GameEvent::RedCard {
                            player: pos_idx,
                            team_name: attacking_team.name.clone(),
                            minute: state.minute,
                            reason: "Second Yellow".to_string(),
                        });
                    } else {
                        events.push(GameEvent::YellowCard {
                            player: pos_idx,
                            team_name: attacking_team.name.clone(),
                            minute: state.minute,
                        });
                    }
                }
            }
            state.team1_has_ball = !state.team1_has_ball;
            state.prev_pos = Some("g".to_string());
            // Early return to avoid overwriting prev_pos below
            let last_evt = events_to_turn_event(&events);
            return (build_game_state(state, team1, team2, last_evt), events);
        } else {
            events.push(GameEvent::PassSuccess {
                from: position_index(&current_pos),
                to: pos_idx,
                xt_gain: xt_val,
            });
        }
    }

    state.prev_pos = Some(chosen_pos);
    let last_evt = events_to_turn_event(&events);
    (build_game_state(state, team1, team2, last_evt), events)
}
