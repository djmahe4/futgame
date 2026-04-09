// === UPDATED: Step 5 - AI Difficulty + Role Movement Constraints ===

/// AI difficulty level — controls how smart the computer is when attacking and defending.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Insane,
}

impl Difficulty {
    /// Parse a difficulty from a string (case-insensitive). Defaults to `Easy`.
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "medium" => Difficulty::Medium,
            "hard"   => Difficulty::Hard,
            "insane" => Difficulty::Insane,
            _        => Difficulty::Easy,
        }
    }
}

/// Match configuration — controls timing granularity and optional feature flags.
///
/// # Turn Duration
///
/// By default each interactive turn represents **60 seconds** (= 1 match minute),
/// so a full 90-minute match takes exactly 90 turns.
///
/// Set `turn_duration_secs` to any positive value to change the granularity:
///
/// | `turn_duration_secs` | Turns per minute | Total turns (90 min) |
/// |----------------------|-----------------|----------------------|
/// | 60 (default)         | 1               | 90                   |
/// | 30                   | 2               | 180                  |
/// | 45                   | ~1.33           | 120                  |
/// | 20                   | 3               | 270                  |
/// | 10                   | 6               | 540                  |
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Seconds each turn represents on the match clock.
    /// Must be ≥ 1; values that don't evenly divide 60 are rounded down where needed.
    pub turn_duration_secs: u32,
    /// AI difficulty level. Affects how the computer chooses moves and defends.
    pub difficulty: Difficulty,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            turn_duration_secs: 60,
            difficulty: Difficulty::Easy,
        }
    }
}

impl GameConfig {
    /// Create a config with a custom turn duration (clamped to at least 1 second).
    pub fn with_turn_duration(secs: u32) -> Self {
        GameConfig {
            turn_duration_secs: secs.max(1),
            difficulty: Difficulty::Easy,
        }
    }

    /// Total number of interactive turns required to play a full 90-minute match.
    ///
    /// ```
    /// use futgame::config::GameConfig;
    /// assert_eq!(GameConfig::default().total_turns(), 90);
    /// assert_eq!(GameConfig::with_turn_duration(30).total_turns(), 180);
    /// assert_eq!(GameConfig::with_turn_duration(45).total_turns(), 120);
    /// ```
    pub fn total_turns(&self) -> u32 {
        // 90 minutes × 60 seconds = 5400 seconds total
        (90 * 60 + self.turn_duration_secs - 1) / self.turn_duration_secs
    }

    /// Convert a **0-indexed** turn number to the current match minute (1–90).
    ///
    /// The minute is the ceiling of elapsed game time so that the very first
    /// turn always shows minute 1.
    ///
    /// ```
    /// use futgame::config::GameConfig;
    /// let cfg = GameConfig::with_turn_duration(30); // 2 turns per minute
    /// assert_eq!(cfg.turn_to_minute(0), 1);  // turn 0 → first half of minute 1
    /// assert_eq!(cfg.turn_to_minute(1), 1);  // turn 1 → second half of minute 1
    /// assert_eq!(cfg.turn_to_minute(2), 2);  // turn 2 → minute 2 starts
    /// let cfg60 = GameConfig::default();     // 1 turn per minute
    /// assert_eq!(cfg60.turn_to_minute(0), 1);
    /// assert_eq!(cfg60.turn_to_minute(44), 45);
    /// assert_eq!(cfg60.turn_to_minute(89), 90);
    /// ```
    pub fn turn_to_minute(&self, turn: u32) -> u32 {
        // elapsed seconds after this turn completes (1-indexed)
        let elapsed_secs = (turn + 1) * self.turn_duration_secs;
        // +59 achieves ceiling division: (elapsed_secs + 59) / 60 == ceil(elapsed_secs / 60)
        let minute = (elapsed_secs + 59) / 60;
        minute.clamp(1, 90)
    }

    /// The 0-indexed turn number **after which half-time is shown** (the last turn
    /// of the first half, i.e. the turn that completes exactly 45 minutes = 2700 s).
    ///
    /// Formula: `(2700 / turn_duration_secs).saturating_sub(1)`
    ///
    /// | `turn_duration_secs` | `halftime_turn()` | minute at that turn |
    /// |----------------------|-------------------|---------------------|
    /// | 60                   | 44                | 45                  |
    /// | 30                   | 89                | 45                  |
    /// | 45                   | 59                | 45                  |
    ///
    /// ```
    /// use futgame::config::GameConfig;
    /// assert_eq!(GameConfig::default().halftime_turn(), 44);              // 2700/60 - 1
    /// assert_eq!(GameConfig::with_turn_duration(30).halftime_turn(), 89); // 2700/30 - 1
    /// assert_eq!(GameConfig::with_turn_duration(45).halftime_turn(), 59); // 2700/45 - 1
    /// ```
    pub fn halftime_turn(&self) -> u32 {
        // 2700 s = exactly 45 minutes; dividing by secs gives the number of complete
        // turns in 45 min.  Subtract 1 to get the 0-indexed index of that last turn.
        (2700 / self.turn_duration_secs).saturating_sub(1)
    }

    /// Human-readable description of the turn granularity.
    pub fn describe(&self) -> String {
        if self.turn_duration_secs == 60 {
            "1 turn = 1 minute (default)".to_string()
        } else if 60 % self.turn_duration_secs == 0 {
            let per_min = 60 / self.turn_duration_secs;
            format!(
                "1 turn = {}s ({} turns per minute, {} turns total)",
                self.turn_duration_secs,
                per_min,
                self.total_turns()
            )
        } else {
            format!(
                "1 turn = {}s ({} turns total)",
                self.turn_duration_secs,
                self.total_turns()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_60_secs() {
        let cfg = GameConfig::default();
        assert_eq!(cfg.turn_duration_secs, 60);
    }

    #[test]
    fn total_turns_default() {
        assert_eq!(GameConfig::default().total_turns(), 90);
    }

    #[test]
    fn total_turns_30s() {
        assert_eq!(GameConfig::with_turn_duration(30).total_turns(), 180);
    }

    #[test]
    fn total_turns_45s() {
        assert_eq!(GameConfig::with_turn_duration(45).total_turns(), 120);
    }

    #[test]
    fn total_turns_20s() {
        assert_eq!(GameConfig::with_turn_duration(20).total_turns(), 270);
    }

    #[test]
    fn minute_mapping_default() {
        let cfg = GameConfig::default();
        assert_eq!(cfg.turn_to_minute(0), 1);
        assert_eq!(cfg.turn_to_minute(44), 45);
        assert_eq!(cfg.turn_to_minute(89), 90);
    }

    #[test]
    fn minute_mapping_30s() {
        let cfg = GameConfig::with_turn_duration(30);
        // turns 0 and 1 both fall in minute 1
        assert_eq!(cfg.turn_to_minute(0), 1);
        assert_eq!(cfg.turn_to_minute(1), 1);
        // turn 2 = 90 seconds elapsed → minute 2
        assert_eq!(cfg.turn_to_minute(2), 2);
        // last two turns → minute 90
        assert_eq!(cfg.turn_to_minute(178), 90);
        assert_eq!(cfg.turn_to_minute(179), 90);
    }

    #[test]
    fn halftime_turn_default() {
        // With 60s turns: halftime at turn 44 (minute 45)
        let cfg = GameConfig::default();
        assert_eq!(cfg.halftime_turn(), 44);
        assert_eq!(cfg.turn_to_minute(cfg.halftime_turn()), 45);
    }

    #[test]
    fn halftime_turn_30s() {
        let cfg = GameConfig::with_turn_duration(30);
        // formula: 2700 / 30 - 1 = 89
        assert_eq!(cfg.halftime_turn(), 89);
        // turn 89 completes exactly 2700 s = minute 45
        assert_eq!(cfg.turn_to_minute(89), 45);
    }

    #[test]
    fn last_turn_is_minute_90() {
        for dur in [10, 20, 30, 45, 60] {
            let cfg = GameConfig::with_turn_duration(dur);
            let last = cfg.total_turns() - 1;
            assert_eq!(cfg.turn_to_minute(last), 90, "dur={}", dur);
        }
    }

    #[test]
    fn clamp_minimum_1_second() {
        let cfg = GameConfig::with_turn_duration(0);
        assert_eq!(cfg.turn_duration_secs, 1);
    }
}
