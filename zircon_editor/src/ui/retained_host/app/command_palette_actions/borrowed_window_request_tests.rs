use std::hint::black_box;
use std::time::Instant;

use super::{parse_window_request, WindowRequestFocus};

const SAMPLE_PAIRS: usize = 31;
const PARSES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260828is_editor237_window_request_borrows_full_query_tail() {
    let request = String::from("8|20|last|render graph|transient resources");
    let (current, target, focus, query) = parse_window_request(request.as_str()).expect("request");

    assert_eq!(current, 8);
    assert_eq!(target, 20);
    assert!(matches!(focus, WindowRequestFocus::Last));
    assert_eq!(query, "render graph|transient resources");
    let request_range = request.as_ptr_range();
    assert!(request_range.contains(&query.as_ptr()));
}

#[test]
fn optimization_batch_20260828is_editor237_parser_has_no_query_copy() {
    let source = include_str!("../command_palette_actions.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let parser = implementation
        .split("fn parse_window_request")
        .nth(1)
        .and_then(|body| {
            body.split("fn workbench_command_palette_query_state")
                .next()
        })
        .expect("window request parser");

    assert!(parser.contains("WindowRequestFocus, &str"));
    assert!(parser.contains("let query = fields.next()?;"));
    assert!(!parser.contains("query.to_string()"));
    assert!(!parser.contains("fields.next()?.to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828is_editor237_borrowed_command_palette_window_query_bench() {
    let query = "render graph transient resource lifetime aliasing ".repeat(12);
    let request = format!("8|20|last|{query}");
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(request.as_str(), false));
            optimized_samples.push(measure(request.as_str(), true));
        } else {
            optimized_samples.push(measure(request.as_str(), true));
            legacy_samples.push(measure(request.as_str(), false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR237_BORROWED_COMMAND_PALETTE_WINDOW_QUERY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} request_bytes={} \
legacy_query_allocations_per_parse=1 optimized_query_allocations_per_parse=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        request.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_parse_window_request(value: &str) -> Option<(usize, usize, WindowRequestFocus, String)> {
    let mut fields = value.splitn(4, '|');
    let current_offset = fields.next()?.parse().ok()?;
    let target_offset = fields.next()?.parse().ok()?;
    let focus = match fields.next()? {
        "first" => WindowRequestFocus::First,
        "last" => WindowRequestFocus::Last,
        _ => return None,
    };
    let query = fields.next()?.to_string();
    Some((current_offset, target_offset, focus, query))
}

fn measure(request: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for iteration in 0..PARSES_PER_SAMPLE {
        let (current, target, focus_value, query_length) = if optimized {
            let (current, target, focus, query) =
                parse_window_request(black_box(request)).expect("optimized window request");
            (current, target, focus_value(focus), query.len())
        } else {
            let (current, target, focus, query) =
                legacy_parse_window_request(black_box(request)).expect("legacy window request");
            let values = (current, target, focus_value(focus), query.len());
            black_box(query);
            values
        };
        checksum ^= black_box(
            current
                .wrapping_add(target)
                .wrapping_add(focus_value)
                .wrapping_add(query_length)
                .wrapping_add(iteration),
        );
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn focus_value(focus: WindowRequestFocus) -> usize {
    match focus {
        WindowRequestFocus::First => 0,
        WindowRequestFocus::Last => 1,
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
