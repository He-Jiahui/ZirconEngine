use super::*;

fn counter(name: &str, value: f64) -> ProfileCounterSnapshot {
    ProfileCounterSnapshot {
        stream: "editor".to_string(),
        name: name.to_string(),
        value,
        timestamp_us: 0,
        frame_index: None,
    }
}
