use crate::ui::retained_host::measure_runtime_text_width;

const FILE_NAME_ELLIPSIS: &str = "...";
const MEASURE_EPSILON: f32 = 0.01;

#[derive(Clone, Copy)]
pub(super) struct RuntimeFileNameCompaction {
    pub(super) max_width: f32,
    pub(super) font_size: f32,
    pub(super) min_prefix_chars: usize,
    pub(super) min_tail_stem_chars: usize,
    pub(super) preferred_tail_stem_chars: usize,
}

pub(super) fn compact_file_like_display_name(
    display_name: &str,
    extension: &str,
    compaction: RuntimeFileNameCompaction,
) -> String {
    let name = display_name.trim();
    if !compaction.max_width.is_finite() || compaction.max_width <= 0.0 {
        return String::new();
    }
    if name.is_empty() || fits_runtime_width(name, compaction) {
        return name.to_string();
    }

    if let Some((stem, suffix)) = matching_file_parts(name, extension) {
        return compact_stem_with_suffix(stem, suffix, compaction)
            .unwrap_or_else(|| compact_whole_text(name, compaction));
    }

    compact_whole_text(name, compaction)
}

fn matching_file_parts<'a>(display_name: &'a str, extension: &str) -> Option<(&'a str, &'a str)> {
    let extension = extension.trim().trim_start_matches('.');
    if extension.is_empty() {
        return None;
    }

    let (stem, suffix) = display_name.rsplit_once('.')?;
    suffix
        .eq_ignore_ascii_case(extension)
        .then_some((stem, suffix))
}

fn compact_stem_with_suffix(
    stem: &str,
    suffix: &str,
    compaction: RuntimeFileNameCompaction,
) -> Option<String> {
    let stem_chars = chars(stem);
    if stem_chars.len() <= compaction.min_prefix_chars + compaction.min_tail_stem_chars {
        return None;
    }

    let max_tail = compaction
        .preferred_tail_stem_chars
        .max(compaction.min_tail_stem_chars)
        .min(
            stem_chars
                .len()
                .saturating_sub(compaction.min_prefix_chars + 1),
        );
    for tail_count in (compaction.min_tail_stem_chars..=max_tail).rev() {
        let max_prefix = stem_chars.len().saturating_sub(tail_count + 1);
        if max_prefix < compaction.min_prefix_chars {
            continue;
        }
        if let Some(candidate) = largest_fitting_candidate(
            compaction.min_prefix_chars,
            max_prefix,
            compaction,
            |prefix_count| candidate_with_suffix(&stem_chars, prefix_count, tail_count, suffix),
        ) {
            return Some(candidate);
        }
    }

    Some(candidate_with_suffix(
        &stem_chars,
        compaction.min_prefix_chars,
        compaction.min_tail_stem_chars,
        suffix,
    ))
}

fn compact_whole_text(text: &str, compaction: RuntimeFileNameCompaction) -> String {
    let text_chars = chars(text);
    if text_chars.len() <= compaction.min_prefix_chars + compaction.min_tail_stem_chars {
        return text.to_string();
    }

    let max_tail = compaction
        .preferred_tail_stem_chars
        .max(compaction.min_tail_stem_chars)
        .min(
            text_chars
                .len()
                .saturating_sub(compaction.min_prefix_chars + 1),
        );
    for tail_count in (compaction.min_tail_stem_chars..=max_tail).rev() {
        let max_prefix = text_chars.len().saturating_sub(tail_count + 1);
        if max_prefix < compaction.min_prefix_chars {
            continue;
        }
        if let Some(candidate) = largest_fitting_candidate(
            compaction.min_prefix_chars,
            max_prefix,
            compaction,
            |prefix_count| candidate_without_suffix(&text_chars, prefix_count, tail_count),
        ) {
            return candidate;
        }
    }

    candidate_without_suffix(
        &text_chars,
        compaction.min_prefix_chars,
        compaction.min_tail_stem_chars,
    )
}

fn largest_fitting_candidate(
    min_prefix: usize,
    max_prefix: usize,
    compaction: RuntimeFileNameCompaction,
    mut candidate_for_prefix: impl FnMut(usize) -> String,
) -> Option<String> {
    let mut low = min_prefix;
    let mut high = max_prefix;
    let mut best = None;
    while low <= high {
        let prefix_count = low + (high - low) / 2;
        let candidate = candidate_for_prefix(prefix_count);
        if fits_runtime_width(&candidate, compaction) {
            best = Some(candidate);
            low = prefix_count + 1;
        } else {
            if prefix_count == 0 {
                break;
            }
            high = prefix_count - 1;
        }
    }
    best
}

fn candidate_with_suffix(
    stem_chars: &[char],
    prefix_count: usize,
    tail_count: usize,
    suffix: &str,
) -> String {
    let prefix = &stem_chars[..prefix_count.min(stem_chars.len())];
    let tail = &stem_chars[stem_chars.len().saturating_sub(tail_count)..];
    let capacity =
        char_byte_len(prefix) + FILE_NAME_ELLIPSIS.len() + char_byte_len(tail) + 1 + suffix.len();
    let mut candidate = String::with_capacity(capacity);
    candidate.extend(prefix.iter().copied());
    candidate.push_str(FILE_NAME_ELLIPSIS);
    candidate.extend(tail.iter().copied());
    candidate.push('.');
    candidate.push_str(suffix);
    candidate
}

fn candidate_without_suffix(chars: &[char], prefix_count: usize, tail_count: usize) -> String {
    let prefix = &chars[..prefix_count.min(chars.len())];
    let tail = &chars[chars.len().saturating_sub(tail_count)..];
    let capacity = char_byte_len(prefix) + FILE_NAME_ELLIPSIS.len() + char_byte_len(tail);
    let mut candidate = String::with_capacity(capacity);
    candidate.extend(prefix.iter().copied());
    candidate.push_str(FILE_NAME_ELLIPSIS);
    candidate.extend(tail.iter().copied());
    candidate
}

fn fits_runtime_width(text: &str, compaction: RuntimeFileNameCompaction) -> bool {
    measure_runtime_text_width(text, compaction.font_size) <= compaction.max_width + MEASURE_EPSILON
}

fn chars(text: &str) -> Vec<char> {
    text.chars().collect()
}

fn char_byte_len(chars: &[char]) -> usize {
    chars.iter().map(|character| character.len_utf8()).sum()
}

#[cfg(test)]
mod tests {
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
        let source = include_str!("name_compaction.rs");
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
        let source = include_str!("name_compaction.rs");
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
}
