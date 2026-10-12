use std::hint::black_box;
use std::time::{Duration, Instant};

use super::menu_item_without_transient_flags;

const SAMPLE_PAIRS: usize = 101;
const ROWS_PER_SAMPLE: usize = 4_096;

#[test]
fn editor894_popup_transient_flags_single_buffer_preserves_exact_rows() {
    for raw in [
        "---",
        "",
        "Open",
        "  Open  | focused, hovered, pressed | Ctrl+O ",
        "Inspect|focused|Ctrl+I",
        "Save|,icon=folder,,Hovered, disabled,pressed,icon=check,|Ctrl+S",
        "文档🙂 | focused,插件=是, hovered | Shift+Alt+字",
        "Name| icon=grid,focused, disabled |shortcut|tail",
        " |  |  ",
    ] {
        assert_eq!(
            menu_item_without_transient_flags(raw),
            legacy_cleanup(raw),
            "{raw:?}"
        );
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn editor894_popup_transient_flags_single_buffer_release_percentiles() {
    let raw = format!(
        "文档 Export|{}|Ctrl+Shift+E",
        (0..32)
            .map(|index| if index % 4 == 0 {
                "Hovered".to_string()
            } else {
                format!("icon={index:02}")
            })
            .collect::<Vec<_>>()
            .join(",")
    );
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&raw, legacy_cleanup));
            optimized.push(measure(&raw, menu_item_without_transient_flags));
        } else {
            optimized.push(measure(&raw, menu_item_without_transient_flags));
            legacy.push(measure(&raw, legacy_cleanup));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "EDITOR894_POPUP_TRANSIENT_FLAGS_SINGLE_BUFFER_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn legacy_cleanup(raw: &str) -> String {
    if raw == "---" {
        return raw.to_string();
    }
    let mut parts = raw.splitn(3, '|');
    let label = parts.next().unwrap_or_default().trim();
    let flags = parts.next().unwrap_or_default();
    let shortcut = parts.next().unwrap_or_default().trim();
    let persistent_flags = flags
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .filter(|flag| !super::matches_transient_menu_item_flag(flag))
        .collect::<Vec<_>>();
    if persistent_flags.is_empty() && shortcut.is_empty() {
        label.to_string()
    } else if shortcut.is_empty() {
        format!("{label}|{}", persistent_flags.join(","))
    } else {
        format!("{label}|{}|{shortcut}", persistent_flags.join(","))
    }
}

fn measure(raw: &str, render: fn(&str) -> String) -> Duration {
    let started = Instant::now();
    let checksum = (0..ROWS_PER_SAMPLE)
        .map(|_| black_box(render(black_box(raw))).len())
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
