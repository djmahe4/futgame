// === OpenFootManager-inspired ===
use std::collections::HashMap;
use rand::Rng;
use crate::player::{Player, new_player};
use crate::tactics::{Formation, Tactic, formation_positions};
use crate::xg::base_xg;

pub struct Team {
    pub name: String,
    pub players: Vec<Player>,
    pub formation: Formation,
    pub tactic: Tactic,
    pub xg_values: HashMap<String, f32>,
    pub total_xg: f32,
    pub total_xt: f32,
    pub shots: u32,
    pub shots_on_target: u32,
    pub possession_count: u32,
}

pub fn new_team(
    name: String,
    player_names: Vec<String>,
    formation: Formation,
    tactic: Tactic,
    rng: &mut impl Rng,
) -> Team {
    let positions = formation_positions(&formation);
    let mut players = Vec::new();
    let mut xg_values = HashMap::new();

    for (i, pos_key) in positions.iter().enumerate() {
        let pname = player_names.get(i).cloned().unwrap_or_else(|| format!("Player {}", i + 1));
        let player = new_player(pname, pos_key.to_string(), i, rng);
        xg_values.insert(pos_key.to_string(), base_xg(pos_key));
        players.push(player);
    }

    Team {
        name,
        players,
        formation,
        tactic,
        xg_values,
        total_xg: 0.0,
        total_xt: 0.0,
        shots: 0,
        shots_on_target: 0,
        possession_count: 0,
    }
}

impl Team {
    pub fn player_at_pos(&self, pos_key: &str) -> Option<&Player> {
        self.players.iter().find(|p| p.position_key == pos_key && !p.red_card)
    }

    pub fn player_at_pos_mut(&mut self, pos_key: &str) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.position_key == pos_key && !p.red_card)
    }

    pub fn reset_xg(&mut self) {
        for (key, val) in self.xg_values.iter_mut() {
            *val = base_xg(key);
        }
    }
}
