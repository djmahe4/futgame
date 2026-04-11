// === OpenFootManager-inspired ===

#[derive(Debug, Clone)]
pub enum Formation {
    F442,
    F433,
    F4231,
    F352,
    F532,
}

impl std::fmt::Display for Formation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formation::F442 => write!(f, "4-4-2"),
            Formation::F433 => write!(f, "4-3-3"),
            Formation::F4231 => write!(f, "4-2-3-1"),
            Formation::F352 => write!(f, "3-5-2"),
            Formation::F532 => write!(f, "5-3-2"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tactic {
    Attacking,
    Defensive,
    Possession,
    Counter,
    Pressing,
}

impl std::fmt::Display for Tactic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tactic::Attacking => write!(f, "Attacking"),
            Tactic::Defensive => write!(f, "Defensive"),
            Tactic::Possession => write!(f, "Possession"),
            Tactic::Counter => write!(f, "Counter"),
            Tactic::Pressing => write!(f, "Pressing"),
        }
    }
}

pub struct TransitionWeights {
    pub pass: f32,
    pub dribble: f32,
    pub shot: f32,
    pub cross: f32,
    pub foul_risk: f32,
}

pub fn formation_positions(f: &Formation) -> Vec<&'static str> {
    match f {
        Formation::F442 =>  vec!["g","1","2","3","4","5","6","7","8","9","0"],
        Formation::F433 =>  vec!["g","1","2","3","4","5","6","7","8","9","0"],
        Formation::F4231 => vec!["g","1","2","3","4","5","6","7","8","9","0"],
        Formation::F352 =>  vec!["g","1","2","3","4","5","6","7","8","9","0"],
        Formation::F532 =>  vec!["g","1","2","3","4","5","6","7","8","9","0"],
    }
}

pub fn tactic_xt_multiplier(t: &Tactic) -> f32 {
    match t {
        Tactic::Attacking => 1.3,
        Tactic::Possession => 1.1,
        Tactic::Counter => 1.0,
        Tactic::Pressing => 1.15,
        Tactic::Defensive => 0.8,
    }
}

pub fn tactic_transition_weights(t: &Tactic) -> TransitionWeights {
    match t {
        Tactic::Attacking => TransitionWeights {
            pass: 0.35, dribble: 0.20, shot: 0.25, cross: 0.15, foul_risk: 0.05,
        },
        Tactic::Defensive => TransitionWeights {
            pass: 0.55, dribble: 0.10, shot: 0.10, cross: 0.10, foul_risk: 0.15,
        },
        Tactic::Possession => TransitionWeights {
            pass: 0.60, dribble: 0.15, shot: 0.10, cross: 0.05, foul_risk: 0.10,
        },
        Tactic::Counter => TransitionWeights {
            pass: 0.40, dribble: 0.25, shot: 0.20, cross: 0.10, foul_risk: 0.05,
        },
        Tactic::Pressing => TransitionWeights {
            pass: 0.45, dribble: 0.20, shot: 0.15, cross: 0.10, foul_risk: 0.10,
        },
    }
}
