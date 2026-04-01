// === xT Layer (New) ===

// Standard 12x8 xT grid (columns=x attacking direction, rows=y width)
// Values based on published xT literature (Karun Singh)
const XT_GRID: [[f32; 8]; 12] = [
    // Own half (columns 0-5), attacking half (6-11)
    [0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001], // col 0 own goal
    [0.002, 0.002, 0.002, 0.002, 0.002, 0.002, 0.002, 0.002], // col 1
    [0.004, 0.004, 0.004, 0.004, 0.004, 0.004, 0.004, 0.004], // col 2
    [0.007, 0.007, 0.008, 0.008, 0.008, 0.008, 0.007, 0.007], // col 3
    [0.010, 0.011, 0.012, 0.013, 0.013, 0.012, 0.011, 0.010], // col 4
    [0.016, 0.018, 0.020, 0.022, 0.022, 0.020, 0.018, 0.016], // col 5 midfield
    [0.030, 0.035, 0.040, 0.045, 0.045, 0.040, 0.035, 0.030], // col 6 attacking mid
    [0.060, 0.070, 0.080, 0.090, 0.090, 0.080, 0.070, 0.060], // col 7 final third
    [0.100, 0.130, 0.150, 0.170, 0.170, 0.150, 0.130, 0.100], // col 8
    [0.150, 0.200, 0.240, 0.270, 0.270, 0.240, 0.200, 0.150], // col 9 penalty area
    [0.250, 0.300, 0.340, 0.380, 0.380, 0.340, 0.300, 0.250], // col 10
    [0.300, 0.350, 0.390, 0.420, 0.420, 0.390, 0.350, 0.300], // col 11 six-yard box
];

pub fn get_zone_xt(x: usize, y: usize) -> f32 {
    let cx = x.min(11);
    let cy = y.min(7);
    XT_GRID[cx][cy]
}

pub fn xt_to_xg_modifier(xt: f32) -> f32 {
    // Sigmoid-like mapping: 0.0..1.0 xt -> max ±0.05 modifier
    let clamped = xt.clamp(0.0, 1.0);
    let sig = 1.0 / (1.0 + (-12.0 * (clamped - 0.2)).exp());
    (sig - 0.5) * 0.10
}

pub fn position_to_zone(pos_key: &str, attacking: bool) -> (usize, usize) {
    // Map game position keys to approximate xT grid coordinates
    // Grid: x=0 own goal, x=11 opp goal; y=0..7 width
    if attacking {
        match pos_key {
            "g" => (11, 3), // attacking towards opp goal area
            "0" => (10, 6),
            "9" => (10, 1),
            "8" => (9, 5),
            "7" => (9, 2),
            "6" => (8, 5),
            "5" => (8, 2),
            "4" => (7, 5),
            "3" => (7, 2),
            "2" => (6, 5),
            "1" => (6, 2),
            _ => (6, 3),
        }
    } else {
        match pos_key {
            "g" => (0, 3),
            "1" => (1, 2),
            "2" => (1, 5),
            "3" => (2, 2),
            "4" => (2, 5),
            "5" => (3, 2),
            "6" => (3, 5),
            "7" => (4, 2),
            "8" => (4, 5),
            "9" => (5, 2),
            "0" => (5, 5),
            _ => (0, 3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xt_grid_values() {
        assert!(get_zone_xt(0, 0) < get_zone_xt(11, 3));
        assert!(get_zone_xt(11, 3) > 0.3);
    }

    #[test]
    fn test_xt_modifier_range() {
        let low = xt_to_xg_modifier(0.0);
        let high = xt_to_xg_modifier(1.0);
        // low xT -> negative modifier (around -0.05), high xT -> positive modifier (around +0.05)
        assert!(low >= -0.05 - 1e-5, "low modifier should be >= -0.05");
        assert!(low <= 0.05 + 1e-5, "low modifier should be <= 0.05");
        assert!(high >= -0.05 - 1e-5, "high modifier should be >= -0.05");
        assert!(high <= 0.05 + 1e-5, "high modifier should be <= 0.05");
        assert!(high > low, "higher xT should produce higher modifier");
    }
}
