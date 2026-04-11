// === Original xG Core ===
use rand::Rng;

pub fn base_xg(pos_key: &str) -> f32 {
    match pos_key {
        "g" => 0.01,
        "1" | "2" | "3" | "4" => 0.05,
        "5" | "6" | "7" | "8" => 0.15,
        "9" | "0" => 0.25,
        _ => 0.01,
    }
}

pub fn def_xg(_pos_key: &str, current_xg: f32, index: usize) -> f32 {
    let add = if index == 0 {
        0.01 * 0.8
    } else if index < 5 {
        0.05 * 0.8
    } else if index < 9 {
        0.15 * 0.8
    } else {
        0.25 * 0.8
    };
    current_xg + add
}

pub fn determine_outcome(xg: f32, rng: &mut impl Rng) -> u8 {
    let xg_int = (xg * 100.0) as u32;
    // goal_w + miss_w + save_w == 1.0; saves occur when roll >= goal_w + miss_w
    let (goal_w, miss_w): (f32, f32) = if xg_int < 15 {
        (0.05, 0.10)
    } else if xg_int < 25 {
        (0.10, 0.05)
    } else if xg_int < 50 {
        (0.40, 0.20)
    } else if xg_int < 75 {
        (0.60, 0.20)
    } else {
        (0.75, 0.20)
    };
    let roll: f32 = rng.gen();
    if roll < goal_w {
        0
    } else if roll < goal_w + miss_w {
        3
    } else {
        2 // save: remainder probability
    }
}

pub fn adjacent_positions(pos: &str) -> Vec<&'static str> {
    match pos {
        "g" => vec!["9", "0", "g", "1", "2"],
        "1" => vec!["0", "g", "1", "2", "3"],
        "2" => vec!["g", "1", "2", "3", "4"],
        "3" => vec!["1", "2", "3", "4", "5"],
        "4" => vec!["2", "3", "4", "5", "6"],
        "5" => vec!["3", "4", "5", "6", "7"],
        "6" => vec!["4", "5", "6", "7", "8"],
        "7" => vec!["5", "6", "7", "8", "9"],
        "8" => vec!["6", "7", "8", "9", "0"],
        "9" => vec!["7", "8", "9", "0", "g"],
        "0" => vec!["8", "9", "0", "g", "1"],
        _ => vec!["g"],
    }
}

pub fn all_positions() -> Vec<&'static str> {
    vec!["g", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]
}

pub fn position_index(pos: &str) -> usize {
    match pos {
        "g" => 0,
        "1" => 1,
        "2" => 2,
        "3" => 3,
        "4" => 4,
        "5" => 5,
        "6" => 6,
        "7" => 7,
        "8" => 8,
        "9" => 9,
        "0" => 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn test_base_xg_goalkeeper() {
        assert!((base_xg("g") - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_base_xg_defender() {
        assert!((base_xg("1") - 0.05).abs() < 1e-6);
        assert!((base_xg("4") - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_base_xg_midfielder() {
        assert!((base_xg("5") - 0.15).abs() < 1e-6);
        assert!((base_xg("8") - 0.15).abs() < 1e-6);
    }

    #[test]
    fn test_base_xg_attacker() {
        assert!((base_xg("9") - 0.25).abs() < 1e-6);
        assert!((base_xg("0") - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_def_xg_goalkeeper() {
        let result = def_xg("g", 0.01, 0);
        assert!((result - 0.01 - 0.008).abs() < 1e-5);
    }

    #[test]
    fn test_def_xg_attacker() {
        let result = def_xg("9", 0.25, 9);
        assert!((result - 0.25 - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_determine_outcome_very_low_xg() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut goals = 0u32;
        let mut saves = 0u32;
        let mut misses = 0u32;
        for _ in 0..10000 {
            match determine_outcome(0.05, &mut rng) {
                0 => goals += 1,
                2 => saves += 1,
                3 => misses += 1,
                _ => {}
            }
        }
        // at xg=0.05 -> xg_int=5 -> < 15 -> goal_w=0.05, miss_w=0.10, save=(remainder 0.85)
        assert!(goals < misses, "goals should be less than misses at low xG");
        assert!(saves > goals, "saves should dominate at low xG");
    }

    #[test]
    fn test_determine_outcome_high_xg() {
        let mut rng = SmallRng::seed_from_u64(99);
        let mut goals = 0u32;
        let mut saves = 0u32;
        for _ in 0..10000 {
            match determine_outcome(0.80, &mut rng) {
                0 => goals += 1,
                2 => saves += 1,
                _ => {}
            }
        }
        assert!(goals > saves, "goals should dominate at high xG");
    }

    #[test]
    fn test_adjacent_positions_goalkeeper() {
        let adj = adjacent_positions("g");
        assert!(adj.contains(&"9"));
        assert!(adj.contains(&"0"));
        assert!(adj.contains(&"1"));
        assert!(adj.contains(&"2"));
    }
}
