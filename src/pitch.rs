// === OpenFootManager-inspired ===

pub struct Zone {
    pub x: usize,
    pub y: usize,
    pub name: String,
}

pub fn get_pitch_zones() -> Vec<Zone> {
    vec![
        Zone { x: 0, y: 3, name: "Own Goal Area".to_string() },
        Zone { x: 2, y: 3, name: "Own Half".to_string() },
        Zone { x: 5, y: 3, name: "Midfield".to_string() },
        Zone { x: 7, y: 3, name: "Final Third".to_string() },
        Zone { x: 9, y: 3, name: "Penalty Box".to_string() },
        Zone { x: 11, y: 3, name: "Six-Yard Box".to_string() },
    ]
}

pub fn zone_name_for_pos(pos_key: &str) -> &'static str {
    match pos_key {
        "g" => "Goalkeeper Area",
        "1" | "2" => "Defence",
        "3" | "4" => "Defensive Midfield",
        "5" | "6" => "Midfield",
        "7" | "8" => "Attacking Midfield",
        "9" | "0" => "Attack",
        _ => "Unknown",
    }
}
