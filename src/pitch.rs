// === OpenFootManager-inspired ===

pub struct Zone {
    pub x: usize, // column 0-5 (attacking direction)
    pub y: usize, // row    0-2 (0=left, 1=central, 2=right)
    pub name: String,
}

/// Return all 18 pitch zones (6 columns × 3 rows, 9 per half).
///
/// Columns: 0 = own goal area, 1 = own half, 2 = own midfield,
///          3 = att. midfield, 4 = final third, 5 = penalty box
/// Rows:    0 = left channel, 1 = central channel, 2 = right channel
pub fn get_pitch_zones() -> Vec<Zone> {
    let col_names = [
        "Own Goal Area",
        "Own Half",
        "Own Midfield",
        "Attacking Midfield",
        "Final Third",
        "Penalty Box",
    ];
    let row_names = ["Left", "Central", "Right"];

    let mut zones = Vec::with_capacity(18);
    for (x, col) in col_names.iter().enumerate() {
        for (y, row) in row_names.iter().enumerate() {
            zones.push(Zone {
                x,
                y,
                name: format!("{} ({})", col, row),
            });
        }
    }
    zones
}

pub fn zone_name_for_pos(pos_key: &str) -> &'static str {
    match pos_key {
        "g"      => "Goalkeeper Area",
        "1" | "2" => "Defence",
        "3" | "4" => "Defensive Midfield",
        "5" | "6" => "Midfield",
        "7" | "8" => "Attacking Midfield",
        "9" | "0" => "Attack",
        _         => "Unknown",
    }
}
