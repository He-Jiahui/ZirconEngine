use unicode_segmentation::UnicodeSegmentation;

pub(super) fn resolved_layout_advances_for_sdf_glyphs(
    text: &str,
    layout_advances: &[f32],
    sdf_glyph_count: usize,
) -> Option<Vec<f32>> {
    if layout_advances.is_empty() {
        return None;
    }

    if layout_advances.len() == sdf_glyph_count {
        return sanitized_nonzero_advances(layout_advances.iter().copied());
    }

    let mut sdf_advances = Vec::with_capacity(sdf_glyph_count);
    let mut graphemes = text.graphemes(true);
    let mut layout_advances = layout_advances.iter().copied();
    let mut any_nonzero = false;
    loop {
        match (graphemes.next(), layout_advances.next()) {
            (Some(grapheme), Some(layout_advance)) => {
                let char_count = grapheme.chars().count();
                sdf_advances.extend(std::iter::repeat(0.0).take(char_count.saturating_sub(1)));
                let layout_advance = sanitized_advance(layout_advance);
                any_nonzero |= layout_advance > 0.0;
                sdf_advances.push(layout_advance);
            }
            (None, None) => break,
            _ => return None,
        }
    }

    if sdf_advances.len() != sdf_glyph_count {
        return None;
    }

    any_nonzero.then_some(sdf_advances)
}

fn sanitized_nonzero_advances(advances: impl IntoIterator<Item = f32>) -> Option<Vec<f32>> {
    let advances = advances.into_iter();
    let mut sanitized = Vec::with_capacity(advances.size_hint().0);
    let mut any_nonzero = false;
    for advance in advances {
        let advance = sanitized_advance(advance);
        any_nonzero |= advance > 0.0;
        sanitized.push(advance);
    }
    any_nonzero.then_some(sanitized)
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    fn legacy_resolved_layout_advances_for_sdf_glyphs(
        text: &str,
        layout_advances: &[f32],
        sdf_glyph_count: usize,
    ) -> Option<Vec<f32>> {
        if layout_advances.is_empty() {
            return None;
        }

        if layout_advances.len() == sdf_glyph_count {
            return sanitized_nonzero_advances(layout_advances.iter().copied());
        }

        let mut sdf_advances = Vec::with_capacity(sdf_glyph_count);
        let mut graphemes = text.graphemes(true);
        let mut layout_advances = layout_advances.iter().copied();
        loop {
            match (graphemes.next(), layout_advances.next()) {
                (Some(grapheme), Some(layout_advance)) => {
                    let char_count = grapheme.chars().count();
                    sdf_advances.extend(std::iter::repeat(0.0).take(char_count.saturating_sub(1)));
                    sdf_advances.push(sanitized_advance(layout_advance));
                }
                (None, None) => break,
                _ => return None,
            }
        }

        if sdf_advances.len() != sdf_glyph_count {
            return None;
        }

        sanitized_nonzero_advances(sdf_advances)
    }

    #[test]
    fn maps_grapheme_advances_to_sdf_character_advances() {
        let advances = resolved_layout_advances_for_sdf_glyphs("e\u{301}A", &[19.0, 11.0], 3)
            .expect("grapheme advances should map to SDF char advances");

        assert_eq!(advances, vec![0.0, 19.0, 11.0]);
    }

    #[test]
    fn keeps_prior_character_advances_when_counts_match() {
        let advances = resolved_layout_advances_for_sdf_glyphs("ABC", &[5.0, 7.0, 9.0], 3)
            .expect("character advances should stay usable");

        assert_eq!(advances, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn rejects_empty_or_all_zero_advances() {
        assert!(resolved_layout_advances_for_sdf_glyphs("ABC", &[], 3).is_none());
        assert!(resolved_layout_advances_for_sdf_glyphs("ABC", &[0.0, 0.0, 0.0], 3).is_none());
    }

    #[test]
    fn advance_mapping_streams_graphemes_and_nonzero_detection() {
        let source = include_str!("sdf_advances.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("SDF advance implementation");

        assert!(!implementation.contains("text.chars().count()"));
        assert!(!implementation.contains("text.graphemes(true).collect::<Vec<_>>()"));
        assert!(!implementation.contains(".any(|advance|"));
        assert!(implementation.contains("match (graphemes.next(), layout_advances.next())"));
        assert!(implementation.contains("any_nonzero |= advance > 0.0"));
    }

    #[test]
    fn optimization_batch_en_mismatch_path_reuses_sdf_advance_allocation() {
        let source = include_str!("sdf_advances.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("SDF advance implementation");
        let mismatch_path = production
            .split("let mut sdf_advances = Vec::with_capacity(sdf_glyph_count);")
            .nth(1)
            .expect("mismatched SDF advance path");

        assert!(!mismatch_path.contains("sanitized_nonzero_advances(sdf_advances)"));
        assert!(mismatch_path.contains("any_nonzero.then_some(sdf_advances)"));

        let text = "e\u{301}A\u{308}";
        let advances = [19.0, f32::NAN];
        assert_eq!(
            resolved_layout_advances_for_sdf_glyphs(text, &advances, 4),
            legacy_resolved_layout_advances_for_sdf_glyphs(text, &advances, 4)
        );
    }

    #[test]
    #[ignore = "release-only reused SDF advance allocation benchmark"]
    fn optimization_batch_en_reused_sdf_advance_allocation_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const RESOLUTIONS_PER_SAMPLE: usize = 128;
        const GRAPHEME_COUNT: usize = 4_096;

        fn measure_legacy(text: &str, advances: &[f32], glyph_count: usize) -> u128 {
            let started = Instant::now();
            for _ in 0..RESOLUTIONS_PER_SAMPLE {
                black_box(legacy_resolved_layout_advances_for_sdf_glyphs(
                    black_box(text),
                    black_box(advances),
                    glyph_count,
                ));
            }
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(text: &str, advances: &[f32], glyph_count: usize) -> u128 {
            let started = Instant::now();
            for _ in 0..RESOLUTIONS_PER_SAMPLE {
                black_box(resolved_layout_advances_for_sdf_glyphs(
                    black_box(text),
                    black_box(advances),
                    glyph_count,
                ));
            }
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let text = "e\u{301}".repeat(GRAPHEME_COUNT);
        let advances = (0..GRAPHEME_COUNT)
            .map(|index| 1.0 + (index % 31) as f32)
            .collect::<Vec<_>>();
        let glyph_count = GRAPHEME_COUNT * 2;
        assert_eq!(
            legacy_resolved_layout_advances_for_sdf_glyphs(&text, &advances, glyph_count),
            resolved_layout_advances_for_sdf_glyphs(&text, &advances, glyph_count)
        );

        for _ in 0..4 {
            black_box(measure_legacy(&text, &advances, glyph_count));
            black_box(measure_optimized(&text, &advances, glyph_count));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&text, &advances, glyph_count));
                optimized_samples.push(measure_optimized(&text, &advances, glyph_count));
            } else {
                optimized_samples.push(measure_optimized(&text, &advances, glyph_count));
                legacy_samples.push(measure_legacy(&text, &advances, glyph_count));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME448_REUSED_SDF_ADVANCE_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             resolutions_per_sample={RESOLUTIONS_PER_SAMPLE} grapheme_count={GRAPHEME_COUNT} \
             glyph_count={glyph_count} pair_order=alternating_legacy_even \
             legacy_vector_allocations_per_resolution=2 optimized_vector_allocations_per_resolution=1 \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "reused SDF advance allocation must reduce P95 by at least 15%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
