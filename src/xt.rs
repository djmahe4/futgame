// === xT Layer (New) ===

// 6-column × 3-row xT grid  (18 zones total, 9 per half)
//
// Layout:
//   Columns (x): 0 = own goal area … 5 = penalty box  (attacking direction →)
//   Rows    (y): 0 = left channel, 1 = central channel, 2 = right channel
//
//   Own half  = cols 0-2  (9 zones)
//   Att. half = cols 3-5  (9 zones)
//
// Values based on published xT literature (Karun Singh / StatsBomb open data).
// The central channel (y=1) carries a small premium over the flanks.
const XT_GRID: [[f32; 3]; 6] = [
    //                [left,  center, right]
    [0.001, 0.002, 0.001], // col 0 – own goal area
    [0.008, 0.012, 0.008], // col 1 – own half
    [0.025, 0.035, 0.025], // col 2 – own midfield   (boundary of own half)
    [0.060, 0.080, 0.060], // col 3 – attacking midfield (boundary of att. half)
    [0.150, 0.200, 0.150], // col 4 – final third
    [0.320, 0.420, 0.320], // col 5 – penalty box / six-yard area
];

pub fn get_zone_xt(x: usize, y: usize) -> f32 {
    // Clamp to valid grid bounds (6 cols, 3 rows)
    let cx = x.min(5);
    let cy = y.min(2);
    XT_GRID[cx][cy]
}

pub fn xt_to_xg_modifier(xt: f32) -> f32 {
    // Sigmoid maps clamped xT (0..1) to approximately ±0.05:
    //   sig output range ≈ 0.0 to 1.0 → (sig-0.5)*0.10 ≈ -0.05 to +0.05
    let clamped = xt.clamp(0.0, 1.0);
    let sig = 1.0 / (1.0 + (-12.0 * (clamped - 0.2)).exp());
    (sig - 0.5) * 0.10
}

pub fn position_to_zone(pos_key: &str, attacking: bool) -> (usize, usize) {
    // Map game position keys to 6×3 xT grid coordinates.
    //
    // x (column, 0-5): attacking direction
    //   0 = own goal area | 1 = own half | 2 = own midfield
    //   3 = att. midfield | 4 = final third | 5 = penalty box
    //
    // y (row, 0-2): lateral channel
    //   0 = left | 1 = central | 2 = right
    //
    // When `attacking` is true the team is moving toward the opponent's goal,
    // so positions are mapped to the attacking half (cols 3-5).
    // When `attacking` is false (defending / own half) positions map to cols 0-2.
    if attacking {
        match pos_key {
            "g"  => (5, 1), // own GK surging forward → deep in opp penalty box
            "9"  => (5, 0), // striker left
            "0"  => (5, 2), // striker right
            "7"  => (4, 0), // att. mid left
            "8"  => (4, 2), // att. mid right
            "5"  => (3, 0), // cm left pushing up
            "6"  => (3, 2), // cm right pushing up
            "3"  => (3, 1), // def. mid joining attack  (central)
            "4"  => (3, 1), // def. mid joining attack  (central)
            "1"  => (2, 0), // left back overlapping
            "2"  => (2, 2), // right back overlapping
            _    => (3, 1), // fallback: attacking mid central
        }
    } else {
        match pos_key {
            "g"  => (0, 1), // goalkeeper — own goal area, central
            "1"  => (1, 0), // left back
            "2"  => (1, 2), // right back
            "3"  => (2, 0), // left def. mid
            "4"  => (2, 2), // right def. mid
            "5"  => (2, 1), // cm — own midfield central
            "6"  => (2, 1), // cm — own midfield central
            "7"  => (3, 0), // att. mid left (deep own-half view)
            "8"  => (3, 2), // att. mid right (deep own-half view)
            "9"  => (3, 0), // striker left (defensive position)
            "0"  => (3, 2), // striker right (defensive position)
            _    => (0, 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xt_grid_dimensions() {
        // 6 columns, 3 rows — 18 zones in total
        assert_eq!(XT_GRID.len(), 6);
        assert_eq!(XT_GRID[0].len(), 3);
    }

    #[test]
    fn test_xt_grid_values() {
        // own goal area < penalty box
        assert!(get_zone_xt(0, 1) < get_zone_xt(5, 1));
        // penalty box central value is 0.42
        assert!((get_zone_xt(5, 1) - 0.420).abs() < 1e-5);
    }

    #[test]
    fn test_xt_grid_own_half_vs_att_half() {
        // every own-half column must be lower than the lowest att-half column
        for row in 0..3 {
            assert!(get_zone_xt(2, row) < get_zone_xt(3, row));
        }
    }

    #[test]
    fn test_xt_central_premium() {
        // center channel (y=1) should have higher xT than flanks (y=0 or y=2) in each col
        for col in 0..6 {
            assert!(get_zone_xt(col, 1) >= get_zone_xt(col, 0));
            assert!(get_zone_xt(col, 1) >= get_zone_xt(col, 2));
        }
    }

    #[test]
    fn test_xt_clamp_bounds() {
        // out-of-bounds coords are clamped to last valid cell
        assert_eq!(get_zone_xt(99, 99), get_zone_xt(5, 2));
    }

    #[test]
    fn test_xt_modifier_range() {
        let low  = xt_to_xg_modifier(0.0);
        let high = xt_to_xg_modifier(1.0);
        assert!(low  >= -0.05 - 1e-5);
        assert!(low  <=  0.05 + 1e-5);
        assert!(high >= -0.05 - 1e-5);
        assert!(high <=  0.05 + 1e-5);
        assert!(high > low, "higher xT should produce higher modifier");
    }

    #[test]
    fn test_position_to_zone_attacking_striker() {
        // strikers in attacking mode should land in the penalty box (col 5)
        let (x9, _) = position_to_zone("9", true);
        let (x0, _) = position_to_zone("0", true);
        assert_eq!(x9, 5);
        assert_eq!(x0, 5);
    }

    #[test]
    fn test_position_to_zone_gk_defending() {
        // GK in defending mode should be in own goal area (col 0)
        let (x, y) = position_to_zone("g", false);
        assert_eq!(x, 0);
        assert_eq!(y, 1); // central
    }

    #[test]
    fn test_all_zones_in_bounds() {
        // every position key in both modes must map within the 6×3 grid
        let keys = ["g","1","2","3","4","5","6","7","8","9","0"];
        for key in keys {
            for &att in &[true, false] {
                let (x, y) = position_to_zone(key, att);
                assert!(x < 6, "col out of bounds for pos={} att={}", key, att);
                assert!(y < 3, "row out of bounds for pos={} att={}", key, att);
            }
        }
    }
}
