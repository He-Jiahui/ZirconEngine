use zircon_runtime_interface::{ProfileSnapshot, ProfileSpanSnapshot};

use super::{analyze_hotspots, percentile};

#[test]
fn hotspots_sort_by_total_then_p95() {
    let mut snapshot = ProfileSnapshot {
        session_id: "test".to_string(),
        frame_budget_ms: 16.67,
        ..ProfileSnapshot::default()
    };
    snapshot.spans = vec![
        span("runtime", "render", "submit", 0, 10_000),
        span("runtime", "render", "submit", 1, 20_000),
        span("editor", "ui", "tick", 1, 5_000),
    ];

    let report = analyze_hotspots(&snapshot);

    assert_eq!(report.hotspots[0].stream, "runtime");
    assert_eq!(report.hotspots[0].total_us, 30_000);
    assert_eq!(report.hotspots[0].frame_count, 2);
    assert_eq!(report.hotspots[0].p95_us, 20_000);
}

#[test]
fn percentile_selection_matches_the_sorted_order_statistic() {
    for mut values in [
        Vec::new(),
        vec![9],
        vec![9, 1],
        vec![8, 3, 5, 1, 9, 2, 7, 4, 6],
        (0..101).rev().collect::<Vec<_>>(),
    ] {
        let mut sorted = values.clone();
        sorted.sort_unstable();
        let expected = if sorted.is_empty() {
            0
        } else {
            let index = ((sorted.len() - 1) * 95).div_ceil(100);
            sorted[index]
        };

        assert_eq!(percentile(&mut values, 95), expected);
    }
}

fn span(
    stream: &str,
    category: &str,
    name: &str,
    frame_index: u64,
    duration_us: u64,
) -> ProfileSpanSnapshot {
    ProfileSpanSnapshot {
        id: frame_index + 1,
        parent_id: None,
        frame_index: Some(frame_index),
        stream: stream.to_string(),
        category: category.to_string(),
        name: name.to_string(),
        path: format!("{stream}/{category}:{name}"),
        start_us: 0,
        duration_us,
        depth: 0,
    }
}
