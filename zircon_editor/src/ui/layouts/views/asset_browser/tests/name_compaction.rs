use super::*;

#[test]
fn runtime_file_name_compaction_keeps_narrow_name_that_fits_width() {
    let name = "iiiiiiiiiiiiiiiiiiiiiiiiiiii.zui";
    let compaction = RuntimeFileNameCompaction {
        max_width: measure_runtime_text_width(name, 10.0) + 1.0,
        font_size: 10.0,
        min_prefix_chars: 4,
        min_tail_stem_chars: 3,
        preferred_tail_stem_chars: 6,
    };

    assert_eq!(
        compact_file_like_display_name(name, "zui", compaction),
        name
    );
}

#[test]
fn runtime_file_name_compaction_uses_glyph_width_not_character_count() {
    let narrow = "iiiiiiiiiiiiiiii.zui";
    let wide = "WWWWWWWWWWWWWWWW.zui";
    assert_eq!(narrow.chars().count(), wide.chars().count());

    let compaction = RuntimeFileNameCompaction {
        max_width: measure_runtime_text_width("WWWW...WWW.zui", 10.0) + 1.0,
        font_size: 10.0,
        min_prefix_chars: 4,
        min_tail_stem_chars: 3,
        preferred_tail_stem_chars: 6,
    };

    let narrow_label = compact_file_like_display_name(narrow, "zui", compaction);
    let wide_label = compact_file_like_display_name(wide, "zui", compaction);

    assert_eq!(narrow_label, narrow);
    assert_ne!(wide_label, wide);
    assert!(wide_label.ends_with(".zui"));
    assert!(
        measure_runtime_text_width(&wide_label, compaction.font_size)
            <= compaction.max_width + MEASURE_EPSILON,
        "wide label should fit measured width: {wide_label}"
    );
}

#[test]
fn runtime_file_name_compaction_uses_logarithmic_prefix_search() {
    let source = include_str!("../name_compaction.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(!implementation.contains("for prefix_count in"));
    assert!(implementation.contains("largest_fitting_candidate"));
}

#[test]
fn runtime_file_name_compaction_clears_text_without_a_drawable_width() {
    let mut compaction = RuntimeFileNameCompaction {
        max_width: 0.0,
        font_size: 10.0,
        min_prefix_chars: 4,
        min_tail_stem_chars: 3,
        preferred_tail_stem_chars: 6,
    };

    assert_eq!(
        compact_file_like_display_name("workbench_page_chrome.zui", "zui", compaction),
        ""
    );
    compaction.max_width = f32::NAN;
    assert_eq!(
        compact_file_like_display_name("workbench_page_chrome.zui", "zui", compaction),
        ""
    );
}

#[test]
fn single_allocation_name_candidates_preserve_unicode_content() {
    let chars = chars("资产BrowserWorkbenchPreviewAlphaBetaGamma");

    assert_eq!(
        candidate_with_suffix(&chars, 7, 5, "zasset"),
        retired_candidate_with_suffix(&chars, 7, 5, "zasset")
    );
    assert_eq!(
        candidate_without_suffix(&chars, 9, 6),
        retired_candidate_without_suffix(&chars, 9, 6)
    );
}

#[test]
fn single_allocation_name_candidates_build_directly_into_output() {
    let source = include_str!("../name_compaction.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");

    assert!(implementation.contains("String::with_capacity"));
    assert!(implementation.contains("extend(prefix.iter().copied())"));
    assert!(implementation.contains("extend(tail.iter().copied())"));
    assert!(!implementation.contains("fn collect_prefix"));
    assert!(!implementation.contains("fn collect_tail"));
    assert!(!implementation.contains("let fallback = candidate_"));
}

#[test]
#[ignore = "release performance benchmark"]
fn single_allocation_name_candidates_release_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 32;
    const NAME_COUNT: usize = 48;
    const CANDIDATES_PER_NAME: usize = 4;
    const RETIRED_BUFFERS_PER_CANDIDATE: usize = 3;
    const OPTIMIZED_BUFFERS_PER_CANDIDATE: usize = 1;

    let names = (0..NAME_COUNT)
        .map(|index| {
            format!("AssetBrowserName{index:02}WorkbenchPreviewAlphaBetaGamma")
                .chars()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut retired_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let benchmark = |builder: fn(&[char], usize, usize, &str) -> String| {
            let started = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                for name in &names {
                    for tail_count in 3..=6 {
                        std::hint::black_box(builder(name, 8, tail_count, "zasset"));
                    }
                }
            }
            started.elapsed().as_nanos()
        };

        if sample % 2 == 0 {
            retired_samples.push(benchmark(retired_candidate_with_suffix));
            optimized_samples.push(benchmark(candidate_with_suffix));
        } else {
            optimized_samples.push(benchmark(candidate_with_suffix));
            retired_samples.push(benchmark(retired_candidate_with_suffix));
        }
    }

    let retired_p95_ns = percentile_95(&mut retired_samples);
    let optimized_p95_ns = percentile_95(&mut optimized_samples);
    let reduction_bps = retired_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(10_000)
        / retired_p95_ns.max(1);
    println!(
        "EDITOR57_SINGLE_ALLOCATION_NAME_CANDIDATES_BENCH_V1 \
             retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             reduction_bps={reduction_bps} samples={SAMPLES} iterations={ITERATIONS} \
             names={NAME_COUNT} candidates_per_name={CANDIDATES_PER_NAME} \
             buffers_per_candidate={RETIRED_BUFFERS_PER_CANDIDATE}->{OPTIMIZED_BUFFERS_PER_CANDIDATE} \
             eager_fallback_candidates=1->0"
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(85),
        "optimized P95 must be at least 15% faster: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn retired_candidate_with_suffix(
    chars: &[char],
    prefix_count: usize,
    tail_count: usize,
    suffix: &str,
) -> String {
    let prefix = retired_collect_prefix(chars, prefix_count);
    let tail = retired_collect_tail(chars, tail_count);
    format!("{prefix}{FILE_NAME_ELLIPSIS}{tail}.{suffix}")
}

fn retired_candidate_without_suffix(
    chars: &[char],
    prefix_count: usize,
    tail_count: usize,
) -> String {
    let prefix = retired_collect_prefix(chars, prefix_count);
    let tail = retired_collect_tail(chars, tail_count);
    format!("{prefix}{FILE_NAME_ELLIPSIS}{tail}")
}

fn retired_collect_prefix(chars: &[char], count: usize) -> String {
    chars.iter().take(count).copied().collect()
}

fn retired_collect_tail(chars: &[char], count: usize) -> String {
    let start = chars.len().saturating_sub(count);
    chars[start..].iter().copied().collect()
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
