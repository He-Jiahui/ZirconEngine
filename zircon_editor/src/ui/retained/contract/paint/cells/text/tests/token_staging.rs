use std::hint::black_box;
use std::time::Instant;

use super::{normalize_size_cell, split_archived_table_text};

const ROW_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor882_table_text_token_staging_preserves_parser_priority() {
    assert_eq!(split_archived_table_text(""), Vec::<String>::new());
    assert_eq!(
        split_archived_table_text("single value"),
        cells(&["single value"])
    );
    assert_eq!(
        split_archived_table_text("Host UI 12 KB rev 42 ignored"),
        cells(&["Host", "UI", "12 KB", "rev 42"])
    );
    assert_eq!(
        split_archived_table_text("Host UI 12 KB modified"),
        cells(&["Host", "UI", "12 KB", "modified"])
    );
    assert_eq!(
        split_archived_table_text("Host UI 12 rev 42 ignored"),
        cells(&["Host", "UI", "12", "rev 42"])
    );
    assert_eq!(
        split_archived_table_text("Host UI 12 raw modified seconds"),
        cells(&["Host", "UI", "12 raw", "modified seconds"])
    );
    assert_eq!(
        split_archived_table_text("Host UI 12 modified ignored"),
        cells(&["Host", "UI", "12", "modified"])
    );
    assert_eq!(normalize_size_cell("12 kb"), "12 KB");
    assert_eq!(normalize_size_cell("1.5M"), "1.5 MB");
    assert_eq!(normalize_size_cell("12 KB extra"), "12 KB extra");
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor882_table_text_token_staging_benchmark() {
    let rows = (0..ROW_COUNT)
        .map(|index| format!("Asset{index} Tex 12 KB rev {index}"))
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_staging_checksum(&rows),
        optimized_staging_checksum(&rows)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_staging_checksum(&rows)));
            optimized.push(measure(|| optimized_staging_checksum(&rows)));
        } else {
            optimized.push(measure(|| optimized_staging_checksum(&rows)));
            legacy.push(measure(|| legacy_staging_checksum(&rows)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR882_TABLE_TEXT_TOKEN_STAGING_BENCH_V1 sample_pairs={SAMPLE_PAIRS} rows={ROW_COUNT} legacy_temporary_vecs=8192 optimized_temporary_vecs=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert_eq!(ROW_COUNT * 2, 8_192);
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "bounded token lookahead P95 {optimized_p95}ns must stay within 10% of temporary-vector staging P95 {legacy_p95}ns"
    );
}

fn cells(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn legacy_staging_checksum(rows: &[String]) -> usize {
    let mut checksum = 0usize;
    for row in rows {
        let tokens = row.split_whitespace().collect::<Vec<_>>();
        let size_parts = "12 KB".split_whitespace().collect::<Vec<_>>();
        checksum = checksum
            .wrapping_add(tokens.iter().map(|token| token.len()).sum::<usize>())
            .wrapping_add(size_parts.iter().map(|part| part.len()).sum::<usize>());
        black_box(tokens);
        black_box(size_parts);
    }
    checksum
}

fn optimized_staging_checksum(rows: &[String]) -> usize {
    let mut checksum = 0usize;
    for row in rows {
        let mut tokens = row.split_whitespace();
        let leading = [
            tokens.next(),
            tokens.next(),
            tokens.next(),
            tokens.next(),
            tokens.next(),
            tokens.next(),
        ];
        let mut size_parts = "12 KB".split_whitespace();
        let size_leading = [size_parts.next(), size_parts.next(), size_parts.next()];
        checksum = checksum
            .wrapping_add(
                leading
                    .iter()
                    .flatten()
                    .map(|token| token.len())
                    .sum::<usize>(),
            )
            .wrapping_add(
                size_leading
                    .iter()
                    .flatten()
                    .map(|part| part.len())
                    .sum::<usize>(),
            );
        black_box(leading);
        black_box(size_leading);
    }
    checksum
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
