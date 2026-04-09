// === UPDATED: Step 1 - rename step_minute→step_turn, probability scaling ===
// === OpenFootManager-inspired ===
// === Original xG Core ===
// === xT Layer (New) ===

use rand::Rng;

use crate::events::{GameEvent, GoalScorer};
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

pub fn step_turn(
    state: &mut MatchState,
    team1: &mut Team,
    team2: &mut Team,
    human_pos: Option<String>,
    human_guess: Option<String>,
    rng: &mut impl Rng,
    simple_mode: bool,
    turn_duration_secs: u32,
) -> Vec<GameEvent> {
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

    // Choose where ball moves
    let chosen_pos = if let Some(pos) = human_pos {
        if adj_owned.contains(&pos) { pos } else { adj_owned[rng.gen_range(0..adj_owned.len())].clone() }
    } else {
        adj_owned[rng.gen_range(0..adj_owned.len())].clone()
    };

    // Defending team guesses
    let computer_guess = if let Some(g) = human_guess {
        g
    } else {
        adj_owned[rng.gen_range(0..adj_owned.len())].clone()
    };

    // If guess matches: turnover
    if computer_guess == chosen_pos {
        let new_team = if state.team1_has_ball {
            team2.name.clone()
        } else {
            team1.name.clone()
        };
        events.push(GameEvent::PossessionChange { new_team });
        state.team1_has_ball = !state.team1_has_ball;
        state.prev_pos = Some("g".to_string());
        return events;
    }

    // xT calculation - compute before borrowing team mutably
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
            // Resolve shot for the attacking team
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
                    return events;
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
        } else {
            events.push(GameEvent::PassSuccess {
                from: position_index(&current_pos),
                to: pos_idx,
                xt_gain: xt_val,
            });
        }
    }

    state.prev_pos = Some(chosen_pos);
    events
}
