use super::*;

#[test]
fn text_rich_unterminated_marker_corpus_finishes_at_scale() {
    const MARKER_COUNT: usize = 100_000;

    for (marker, format) in [
        ('<', RichTextFormat::HtmlSubsetV1),
        ('[', RichTextFormat::BbCodeV1),
    ] {
        let markup = marker.to_string().repeat(MARKER_COUNT);
        let parsed = parse_with_parser(&RichTextParser::default(), &markup, format);

        assert_eq!(parsed.text.as_ref(), markup.as_str());
    }

    let markdown = "*".repeat(MARKER_COUNT);
    let parsed = parse_with_parser(
        &RichTextParser::default(),
        &markdown,
        RichTextFormat::MarkdownInlineV1,
    );
    assert!(parsed.text.len() <= markdown.len());
}

#[test]
fn text_rich_adjacent_malformed_marker_corpus_finishes_at_scale() {
    const MARKER_COUNT: usize = 100_000;

    for (marker, closer, format) in [
        ('<', '>', RichTextFormat::HtmlSubsetV1),
        ('[', ']', RichTextFormat::BbCodeV1),
    ] {
        let markup = format!("{}{}", marker.to_string().repeat(MARKER_COUNT), closer);
        let parsed = parse_with_parser(&RichTextParser::default(), &markup, format);

        assert_eq!(parsed.text.as_ref(), markup.as_str());
    }
}

#[test]
#[ignore = "release performance evidence"]
fn text_rich_unterminated_marker_release_benchmark_evidence() {
    use std::{hint::black_box, time::Instant};

    const MARKER_COUNT: usize = 20_000;
    const SAMPLE_PAIRS: usize = 21;

    for (marker, closer, format, format_name) in [
        (b'<', '>', RichTextFormat::HtmlSubsetV1, "html"),
        (b'[', ']', RichTextFormat::BbCodeV1, "bbcode"),
    ] {
        let markup = char::from(marker).to_string().repeat(MARKER_COUNT);
        let mut legacy_us = Vec::with_capacity(SAMPLE_PAIRS);
        let mut frontier_us = Vec::with_capacity(SAMPLE_PAIRS);
        let mut legacy_scan_visits = 0_u64;

        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                let started = Instant::now();
                legacy_scan_visits = legacy_unterminated_marker_scan(&markup, marker, closer);
                legacy_us.push(started.elapsed().as_micros());

                let parser = RichTextParser::default();
                let started = Instant::now();
                let parsed = parse_with_parser(&parser, black_box(&markup), format);
                frontier_us.push(started.elapsed().as_micros());
                assert_eq!(parsed.text.as_ref(), markup.as_str());
            } else {
                let parser = RichTextParser::default();
                let started = Instant::now();
                let parsed = parse_with_parser(&parser, black_box(&markup), format);
                frontier_us.push(started.elapsed().as_micros());
                assert_eq!(parsed.text.as_ref(), markup.as_str());

                let started = Instant::now();
                legacy_scan_visits = legacy_unterminated_marker_scan(&markup, marker, closer);
                legacy_us.push(started.elapsed().as_micros());
            }
        }

        let legacy_p95_us = nearest_rank_percentile(&legacy_us, 95);
        let frontier_p95_us = nearest_rank_percentile(&frontier_us, 95);

        println!(
            "RICH_UNTERMINATED_MARKER_BENCH_V1 format={format_name} marker_count={MARKER_COUNT} sample_pairs={SAMPLE_PAIRS} legacy_scan_visits={legacy_scan_visits} frontier_scan_visits={MARKER_COUNT} legacy_p95_us={legacy_p95_us} frontier_p95_us={frontier_p95_us} legacy_us={} frontier_us={}",
            join_samples(&legacy_us),
            join_samples(&frontier_us),
        );
        assert!(
            frontier_p95_us.saturating_mul(4) <= legacy_p95_us,
            "frontier P95 {frontier_p95_us}us must be at most 25% of legacy P95 {legacy_p95_us}us"
        );
    }
}

#[test]
fn text_rich_mismatched_close_corpus_hits_the_default_active_tag_depth_budget() {
    const DEPTH: usize = 10_000;

    for (open, mismatched_close, format) in [
        ("<b>", "</i>", RichTextFormat::HtmlSubsetV1),
        ("[b]", "[/i]", RichTextFormat::BbCodeV1),
    ] {
        let markup = format!("{}{}x", open.repeat(DEPTH), mismatched_close.repeat(DEPTH));
        let parser = RichTextParser::default();
        let max_depth = parser.budget().max_active_tag_depth;

        assert_eq!(
            parser.compile(&markup, format),
            Err(RichTextParseError::ActiveTagDepthBudgetExceeded {
                attempted_depth: max_depth + 1,
                max_depth,
            })
        );
    }
}

#[test]
fn text_rich_deep_duplicate_closes_keep_the_active_tag_index_consistent() {
    const DEPTH: usize = 40;

    for (open, close, format) in [
        ("<b>", "</b>", RichTextFormat::HtmlSubsetV1),
        ("[b]", "[/b]", RichTextFormat::BbCodeV1),
    ] {
        let markup = format!("{}{}x", open.repeat(DEPTH), close.repeat(DEPTH));
        let parsed = parse_with_parser(&RichTextParser::default(), &markup, format);

        assert_eq!(parsed.text.as_ref(), "x");
        assert_eq!(parsed.runs.len(), 1);
        assert_eq!(parsed.runs[0].style, StyleOverride::default());
    }
}

#[test]
#[ignore = "release performance evidence"]
fn text_rich_active_tag_index_release_benchmark_evidence() {
    use std::{hint::black_box, time::Instant};

    const DEPTH: usize = 5_000;
    const SAMPLE_PAIRS: usize = 21;

    for (open, mismatched_close, format, format_name) in [
        ("<b>", "</i>", RichTextFormat::HtmlSubsetV1, "html"),
        ("[b]", "[/i]", RichTextFormat::BbCodeV1, "bbcode"),
    ] {
        let markup = format!("{}{}x", open.repeat(DEPTH), mismatched_close.repeat(DEPTH));
        let mut legacy_us = Vec::with_capacity(SAMPLE_PAIRS);
        let mut indexed_us = Vec::with_capacity(SAMPLE_PAIRS);
        let mut legacy_tag_visits = 0_u64;

        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                let started = Instant::now();
                legacy_tag_visits = legacy_mismatched_close_scan(DEPTH, DEPTH);
                legacy_us.push(started.elapsed().as_micros());

                let parser = RichTextParser::with_budget(
                    RichParseBudget::default().with_max_active_tag_depth(DEPTH),
                );
                let started = Instant::now();
                let parsed = parse_with_parser(&parser, black_box(&markup), format);
                indexed_us.push(started.elapsed().as_micros());
                assert_eq!(parsed.text.as_ref(), "x");
                assert_eq!(parsed.runs[0].style.weight, Some(700));
            } else {
                let parser = RichTextParser::with_budget(
                    RichParseBudget::default().with_max_active_tag_depth(DEPTH),
                );
                let started = Instant::now();
                let parsed = parse_with_parser(&parser, black_box(&markup), format);
                indexed_us.push(started.elapsed().as_micros());
                assert_eq!(parsed.text.as_ref(), "x");
                assert_eq!(parsed.runs[0].style.weight, Some(700));

                let started = Instant::now();
                legacy_tag_visits = legacy_mismatched_close_scan(DEPTH, DEPTH);
                legacy_us.push(started.elapsed().as_micros());
            }
        }

        let legacy_p95_us = nearest_rank_percentile(&legacy_us, 95);
        let indexed_p95_us = nearest_rank_percentile(&indexed_us, 95);

        println!(
            "RICH_ACTIVE_TAG_BENCH_V1 format={format_name} depth={DEPTH} mismatched_closes={DEPTH} sample_pairs={SAMPLE_PAIRS} legacy_tag_visits={legacy_tag_visits} index_build_tag_visits={DEPTH} indexed_close_lookups={DEPTH} legacy_p95_us={legacy_p95_us} indexed_p95_us={indexed_p95_us} legacy_us={} indexed_us={}",
            join_samples(&legacy_us),
            join_samples(&indexed_us),
        );
        assert!(
            indexed_p95_us.saturating_mul(4) <= legacy_p95_us,
            "indexed P95 {indexed_p95_us}us must be at most 25% of legacy P95 {legacy_p95_us}us"
        );
    }
}

fn legacy_unterminated_marker_scan(input: &str, marker: u8, closer: char) -> u64 {
    use std::hint::black_box;

    let mut scan_visits = 0_u64;
    for (index, byte) in input.bytes().enumerate() {
        if byte != marker {
            continue;
        }
        let suffix = &input[index..];
        scan_visits = scan_visits.saturating_add(suffix.len() as u64);
        black_box(suffix.find(closer));
    }
    scan_visits
}

fn legacy_mismatched_close_scan(depth: usize, close_count: usize) -> u64 {
    use std::hint::black_box;

    let tags = vec!["b"; depth];
    for _ in 0..close_count {
        black_box(tags.iter().rposition(|name| *name == "i"));
    }
    (depth as u64).saturating_mul(close_count as u64)
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}
