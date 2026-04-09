// === UPDATED: Step 5 - AI Difficulty + Role Movement Constraints ===
// === UPDATED: Step 6 - Lightweight GameState + Tactical Rendering ===
// === OpenFootManager-inspired ===
// === Original xG Core ===
// === xT Layer (New) ===

use rand::Rng;

use crate::config::Difficulty;
use crate::events::{GameEvent, GoalScorer};
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
        }
    }
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
) -> (GameState, Vec<GameEvent>) {
    let mut events: Vec<GameEvent> = Vec::new();

    // Track possession
    if state.team1_has_ball {
        state.poss1 += 1;
    } else {
        state.poss2 += 1;
    }

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

    // --- Determine if defence intercepts ---
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
        carrier_name, chosen_pos, tactic_mult.clamp(0.5_f32, 1.5_f32));

    // All probabilities scaled by turn_duration_secs/60 so expected events per 90 min remain consistent.
    let scale = turn_duration_secs as f32 / 60.0;

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
