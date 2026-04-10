// === ENHANCED: Floating-Point Position System (105x68m) + 'm' Per-Guess Movements + 'p' Pause + Dribble/Interception + Insights Viz ===
// === OpenFootManager-inspired ===

/// Real-world coordinates on a standard 105×68 m pitch.
/// x: 0.0 (own goal line) → 105.0 (opponent goal line)
/// y: 0.0 (left touchline) → 68.0 (right touchline)
#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    /// Euclidean distance to another position (metres).
    pub fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Map a string position key to real-world pitch coordinates (centre point of the role's area).
pub fn pos_to_world(pos_key: &str) -> Position {
    match pos_key {
        "g" => Position { x: 5.0,  y: 34.0 }, // Goalkeeper
        "1" => Position { x: 20.0, y: 13.0 }, // Left Back
        "2" => Position { x: 20.0, y: 55.0 }, // Right Back
        "3" => Position { x: 32.0, y: 20.0 }, // CDM-L / CB-L
        "4" => Position { x: 32.0, y: 48.0 }, // CDM-R / CB-R
        "5" => Position { x: 50.0, y: 22.0 }, // CM-L
        "6" => Position { x: 50.0, y: 46.0 }, // CM-R
        "7" => Position { x: 65.0, y: 15.0 }, // AMF-L / LW
        "8" => Position { x: 65.0, y: 53.0 }, // AMF-R / RW
        "9" => Position { x: 82.0, y: 25.0 }, // Striker-L
        "0" => Position { x: 82.0, y: 43.0 }, // Striker-R
        _   => Position { x: 52.5, y: 34.0 }, // Centre circle fallback
    }
}

/// Compute intermediate zones on the 6×3 xT grid along the path from `start_zone` to
/// `end_zone`.  Zone index = `col * 3 + row` (range 0–17).  Uses Bresenham-style integer walk.
/// Returns only the intermediate zones (start and end zones are excluded).
pub fn get_path_zones(start_zone: u8, end_zone: u8) -> Vec<u8> {
    if start_zone == end_zone {
        return Vec::new();
    }
    let x0 = (start_zone / 3) as i32; // column
    let y0 = (start_zone % 3) as i32; // row
    let x1 = (end_zone / 3) as i32;
    let y1 = (end_zone % 3) as i32;

    let mut path = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1i32 } else { -1 };
    let sy = if y0 < y1 { 1i32 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        let zone = (x * 3 + y).clamp(0, 17) as u8;
        if zone != start_zone && zone != end_zone {
            path.push(zone);
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 > -dy { err -= dy; x += sx; }
        if e2 < dx  { err += dx; y += sy; }
    }
    path
}

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
