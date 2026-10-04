use super::*;
use std::hint::black_box;
use std::time::{Duration, Instant};

#[test]
fn runtime_name_lines_use_width_before_character_limit() {
    let wide_name = "WWWWWWWWWWWWWWWWWW";
    let split = RuntimeNameLineSplit {
        max_width: 96.0,
        primary_font_size: 10.0,
        continuation_font_size: 9.0,
    };
    assert!(wide_name.chars().count() <= NAME_SINGLE_LINE_LIMIT);
    assert!(
        measure_runtime_text_width(wide_name, split.primary_font_size) > split.max_width,
        "test fixture must exceed the thumbnail name slot"
    );

    let (name, continuation) = split_display_name_lines(wide_name, split);

    assert!(!continuation.is_empty());
    assert!(
        measure_runtime_text_width(name.as_str(), split.primary_font_size)
            <= split.max_width + MEASURE_EPSILON,
        "primary thumbnail title line should fit measured width: {name}"
    );
    assert!(
        measure_runtime_text_width(continuation.as_str(), split.continuation_font_size)
            <= split.max_width + MEASURE_EPSILON,
        "continuation thumbnail title line should fit measured width: {continuation}"
    );
}

#[test]
fn runtime_name_lines_keep_camel_case_boundaries_when_width_allows() {
    let split = RuntimeNameLineSplit {
        max_width: 96.0,
        primary_font_size: 10.0,
        continuation_font_size: 9.0,
    };

    assert_eq!(
        split_display_name_lines("NavigationSettingsRuntimeProfile", split),
        (
            "NavigationSettings".to_string(),
            "RuntimeProfile".to_string()
        )
    );
}

#[test]
fn allocation_free_name_split_preserves_legacy_splits() {
    let split = RuntimeNameLineSplit {
        max_width: 96.0,
        primary_font_size: 10.0,
        continuation_font_size: 9.0,
    };
    let names = [
        "NavigationSettingsRuntimeProfile",
        "materials/environment/weathered_marble_surface",
        "CharacterLOD2AnimationRuntimeSettings",
        "\u{573a}\u{666f}\u{8d44}\u{6e90}\u{7ba1}\u{7406}\u{5668}_NavigationRuntimeProfile_\u{9ad8}\u{7cbe}\u{5ea6}\u{8d34}\u{56fe}",
        "WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW",
    ];

    for name in names {
        assert_eq!(
            split_display_name_lines(name, split),
            legacy_split_display_name_lines(name, split),
            "optimized scan must preserve the legacy split for {name}"
        );
    }
}

#[test]
fn allocation_free_name_split_avoids_buffers_sorts_and_temporary_line_strings() {
    let source = include_str!("../name_lines.rs");
    let implementation = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(!implementation.contains("let mut candidates = Vec::new()"));
    assert!(!implementation.contains("candidates.sort_unstable()"));
    assert!(implementation.contains("name.char_indices().enumerate()"));
    assert!(implementation.contains("trim_name_line(first)"));
}

#[test]
#[ignore = "release-only asset name split performance gate"]
fn allocation_free_name_split_release_benchmark() {
    const SAMPLE_COUNT: usize = 11;
    const ITERATIONS_PER_SAMPLE: usize = 32;
    const NAME_COUNT: usize = 64;
    const MAX_OPTIMIZED_TO_LEGACY_PERCENT: u128 = 90;

    let split = RuntimeNameLineSplit {
        max_width: 112.0,
        primary_font_size: 10.0,
        continuation_font_size: 9.0,
    };
    let names = (0..NAME_COUNT)
        .map(|index| {
            format!("environment/material_{index:04}_WeatheredMarbleSurfaceRuntimeProfileLOD2")
        })
        .collect::<Vec<_>>();

    for name in &names {
        black_box(split_display_name_lines(black_box(name), split));
        black_box(legacy_split_display_name_lines(black_box(name), split));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure_name_split_batch(
                &names,
                split,
                ITERATIONS_PER_SAMPLE,
                legacy_split_display_name_lines,
            ));
            optimized_samples.push(measure_name_split_batch(
                &names,
                split,
                ITERATIONS_PER_SAMPLE,
                split_display_name_lines,
            ));
        } else {
            optimized_samples.push(measure_name_split_batch(
                &names,
                split,
                ITERATIONS_PER_SAMPLE,
                split_display_name_lines,
            ));
            legacy_samples.push(measure_name_split_batch(
                &names,
                split,
                ITERATIONS_PER_SAMPLE,
                legacy_split_display_name_lines,
            ));
        }
    }

    let legacy_p95_ns = duration_p95_ns(legacy_samples);
    let optimized_p95_ns = duration_p95_ns(optimized_samples);
    let reduction_basis_points = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(10_000)
        / legacy_p95_ns.max(1);
    let candidate_count = names[0]
        .chars()
        .count()
        .saturating_sub(NAME_MIN_LINE_CHARS * 2)
        + 1;

    println!(
        "EDITOR57_ALLOCATION_FREE_NAME_SPLIT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} reduction_basis_points={reduction_basis_points} samples={SAMPLE_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE} names={NAME_COUNT} candidates_per_name={candidate_count} candidate_buffers_per_name=1->0 score_string_allocations_per_name={}->0",
        candidate_count * 2
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(MAX_OPTIMIZED_TO_LEGACY_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {MAX_OPTIMIZED_TO_LEGACY_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn measure_name_split_batch(
    names: &[String],
    split: RuntimeNameLineSplit,
    iterations: usize,
    split_name: fn(&str, RuntimeNameLineSplit) -> (String, String),
) -> Duration {
    let started = Instant::now();
    for _ in 0..iterations {
        for name in names {
            black_box(split_name(black_box(name), split));
        }
    }
    started.elapsed()
}

fn duration_p95_ns(mut samples: Vec<Duration>) -> u128 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index].as_nanos()
}

fn legacy_split_display_name_lines(
    display_name: &str,
    split: RuntimeNameLineSplit,
) -> (String, String) {
    let name = display_name.trim();
    let char_count = name.chars().count();
    if char_count <= NAME_SINGLE_LINE_LIMIT
        && runtime_name_line_fits(name, split.primary_font_size, split.max_width)
    {
        return (name.to_string(), String::new());
    }

    let target = name_target(char_count);
    let mut candidates = Vec::new();
    for (index, ch) in name.chars().enumerate() {
        if is_name_separator(ch) && is_valid_name_break(index, char_count) {
            candidates.push(index);
        }
    }
    let mut previous: Option<char> = None;
    for (index, ch) in name.chars().enumerate() {
        if let Some(previous) = previous {
            let is_boundary = ch.is_ascii_uppercase()
                && (previous.is_ascii_lowercase() || previous.is_ascii_digit());
            if is_boundary && is_valid_name_break(index, char_count) {
                candidates.push(index);
            }
        }
        previous = Some(ch);
    }
    if char_count >= NAME_MIN_LINE_CHARS * 2 {
        candidates.extend(NAME_MIN_LINE_CHARS..=char_count - NAME_MIN_LINE_CHARS);
    }
    candidates.push(target);
    candidates.sort_unstable();
    candidates.dedup();
    let split_char = candidates
        .into_iter()
        .filter(|candidate| is_valid_name_break(*candidate, char_count))
        .min_by_key(|candidate| legacy_name_split_score(name, *candidate, target, split))
        .unwrap_or(target);
    let split_byte = byte_index_at_char(name, split_char);
    let (first, second) = name.split_at(split_byte);
    let first = name_line_text(first);
    let second = name_line_text(second);
    if first.is_empty() || second.is_empty() {
        let fallback_byte = byte_index_at_char(name, name_target(char_count));
        let (fallback_first, fallback_second) = name.split_at(fallback_byte);
        return (
            fallback_first.trim().to_string(),
            fallback_second.trim().to_string(),
        );
    }
    (first, second)
}

fn legacy_name_split_score(
    name: &str,
    split_char: usize,
    target: usize,
    split: RuntimeNameLineSplit,
) -> (u32, u8, u32, usize, bool) {
    let split_byte = byte_index_at_char(name, split_char);
    let (first, second) = name.split_at(split_byte);
    let first = name_line_text(first);
    let second = name_line_text(second);
    let first_width = measure_runtime_text_width(&first, split.primary_font_size);
    let second_width = measure_runtime_text_width(&second, split.continuation_font_size);
    let overflow =
        (first_width - split.max_width).max(0.0) + (second_width - split.max_width).max(0.0);
    let balance = (first_width - second_width).abs();
    (
        width_score(overflow),
        legacy_break_rank(name, split_char),
        width_score(balance),
        split_char.abs_diff(target),
        split_char > target,
    )
}

fn legacy_break_rank(name: &str, split_char: usize) -> u8 {
    let mut previous: Option<char> = None;
    for (index, ch) in name.chars().enumerate() {
        if index == split_char {
            if is_name_separator(ch) {
                return 0;
            }
            if let Some(previous) = previous {
                if ch.is_ascii_uppercase()
                    && (previous.is_ascii_lowercase() || previous.is_ascii_digit())
                {
                    return 0;
                }
            }
            return 1;
        }
        previous = Some(ch);
    }
    1
}
