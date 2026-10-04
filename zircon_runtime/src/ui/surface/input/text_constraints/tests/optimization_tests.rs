use std::hint::black_box;
use std::time::Instant;

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    dispatch::{UiImePreeditClause, UiImePreeditClauseKind, UiTextByteRange},
    surface::UiTextRange,
};

use super::{
    retained_grapheme_count_from_source, SanitizedTextInputReplacement, TextInputBoundaryMap,
    TextInputConstraints, TextInputFilter, TextInputRetainedGraphemeCount,
    UiTextInputConstraintReceipt, TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES,
};

const PROFILE_MARKER: &str = "RUNTIME82_UNCONSTRAINED_REPLACEMENT_BYTE_COPY_BENCH_V1";
const PROFILE_GRAPHEMES: usize = 1_000_000;
const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;
const MAX_NEW_P95_PERCENT: u128 = 80;

#[test]
fn runtime82_unconstrained_replacement_preserves_all_utf8_bytes_and_empty_receipts() {
    let constraints = TextInputConstraints::default();
    let range = UiTextRange { start: 1, end: 4 };
    for replacement in [
        "",
        "ASCII insertion 123",
        "\u{754c}\u{e9}a\u{0301}\u{1f469}\u{200d}\u{1f4bb}",
        "a\r\nb\rc\nd\u{000b}e\u{000c}f\u{0085}g\u{2028}h\u{2029}i",
        "\0",
    ] {
        let scanned = constraints.sanitize_replacement("Ae\u{0301}Z", range, replacement);
        assert_identity(&scanned, replacement);
        for retained in [0, 2, usize::MAX] {
            let indexed = constraints.sanitize_replacement_with_retained_grapheme_count(
                "Ae\u{0301}Z",
                range,
                replacement,
                TextInputRetainedGraphemeCount::DocumentIndex(retained),
            );
            assert_identity(&indexed, replacement);
        }
    }
}

#[test]
fn runtime82_unconstrained_replacement_guard_keeps_max_grapheme_limits() {
    let constraints = TextInputConstraints {
        max_graphemes: Some(3),
        ..TextInputConstraints::default()
    };
    for retained in [
        TextInputRetainedGraphemeCount::SourceScan,
        TextInputRetainedGraphemeCount::DocumentIndex(2),
    ] {
        let sanitized = constraints.sanitize_replacement_with_retained_grapheme_count(
            "Ae\u{0301}Z",
            UiTextRange { start: 1, end: 4 },
            "\u{754c}\u{1f469}\u{200d}\u{1f4bb}",
            retained,
        );
        assert_eq!(sanitized.text, "\u{754c}");
        assert_eq!(
            sanitized.receipt,
            UiTextInputConstraintReceipt {
                max_graphemes_truncated: true,
                ..UiTextInputConstraintReceipt::default()
            }
        );
    }
}

#[test]
fn runtime82_unconstrained_replacement_guard_keeps_character_filter_receipts() {
    let constraints = TextInputConstraints {
        filter: TextInputFilter::Digits,
        ..TextInputConstraints::default()
    };
    let sanitized =
        constraints.sanitize_replacement("", UiTextRange { start: 0, end: 0 }, "a1\u{754c}2\r\n3");
    assert_eq!(sanitized.text, "123");
    assert_eq!(
        sanitized.receipt,
        UiTextInputConstraintReceipt {
            removed_filter_scalar_count: 4,
            ..UiTextInputConstraintReceipt::default()
        }
    );
}

#[test]
fn runtime82_unconstrained_replacement_guard_keeps_single_line_separator_receipts() {
    let constraints = TextInputConstraints {
        multiline: false,
        ..TextInputConstraints::default()
    };
    let sanitized =
        constraints.sanitize_replacement("", UiTextRange { start: 0, end: 0 }, "a\r\nb\u{2028}c\n");
    assert_eq!(sanitized.text, "abc");
    assert_eq!(
        sanitized.receipt,
        UiTextInputConstraintReceipt {
            removed_hard_line_count: 3,
            ..UiTextInputConstraintReceipt::default()
        }
    );
}

#[test]
fn runtime82_unconstrained_replacement_guard_keeps_preedit_boundary_mapping() {
    let replacement = "\u{e9}a\u{0301}\u{1f469}\u{200d}\u{1f4bb}\r\nx";
    let cursor = Some(UiTextByteRange::new(5, 16));
    let clauses = [
        UiImePreeditClause::new(UiTextByteRange::new(0, 2), UiImePreeditClauseKind::Input),
        UiImePreeditClause::new(
            UiTextByteRange::new(2, 5),
            UiImePreeditClauseKind::Converted,
        ),
        UiImePreeditClause::new(
            UiTextByteRange::new(5, 16),
            UiImePreeditClauseKind::TargetConverted,
        ),
        UiImePreeditClause::new(UiTextByteRange::new(16, 18), UiImePreeditClauseKind::Input),
    ];
    let sanitized = TextInputConstraints::default().sanitize_preedit_replacement(
        "",
        UiTextRange { start: 0, end: 0 },
        replacement,
        cursor,
        &clauses,
    );
    assert_eq!(sanitized.text, replacement);
    assert_eq!(sanitized.cursor_range, cursor);
    assert_eq!(sanitized.preedit_clauses, clauses);
    assert!(sanitized.receipt.is_empty());

    let empty = TextInputConstraints::default().sanitize_preedit_replacement(
        "",
        UiTextRange { start: 0, end: 0 },
        "",
        Some(UiTextByteRange::new(0, 0)),
        &[],
    );
    assert!(empty.text.is_empty());
    assert_eq!(empty.cursor_range, Some(UiTextByteRange::new(0, 0)));
    assert!(empty.preedit_clauses.is_empty());
    assert!(empty.receipt.is_empty());
}

#[test]
#[ignore = "managed Windows Release comparison of million-grapheme replacement sanitization"]
fn runtime82_unconstrained_replacement_byte_copy_release_profile() {
    assert!(!cfg!(debug_assertions), "run this profile with --release");
    eprintln!(
        "{PROFILE_MARKER} os={} arch={} crate={} processor={:?} profile=release warmup_pairs={WARMUP_PAIRS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_old_first_even",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
        std::env::var("PROCESSOR_IDENTIFIER").ok()
    );
    for (shape, replacement) in [
        ("ascii", "a".repeat(PROFILE_GRAPHEMES)),
        ("unicode_cjk", "\u{754c}".repeat(PROFILE_GRAPHEMES)),
    ] {
        assert_eq!(replacement.graphemes(true).count(), PROFILE_GRAPHEMES);
        for pair in 0..WARMUP_PAIRS {
            measure_pair(pair, &replacement);
        }
        let mut old_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut new_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let (old, new) = measure_pair(pair, &replacement);
            old_ns.push(old);
            new_ns.push(new);
        }
        let old_p50_ns = percentile(&old_ns, 50);
        let old_p95_ns = percentile(&old_ns, 95);
        let old_p99_ns = percentile(&old_ns, 99);
        let new_p50_ns = percentile(&new_ns, 50);
        let new_p95_ns = percentile(&new_ns, 95);
        let new_p99_ns = percentile(&new_ns, 99);
        eprintln!(
            "{PROFILE_MARKER} shape={shape} graphemes={PROFILE_GRAPHEMES} bytes={} warmup_pairs={WARMUP_PAIRS} sample_pairs={SAMPLE_PAIRS} old_p50_ns={old_p50_ns} old_p95_ns={old_p95_ns} old_p99_ns={old_p99_ns} new_p50_ns={new_p50_ns} new_p95_ns={new_p95_ns} new_p99_ns={new_p99_ns} old_ns={old_ns:?} new_ns={new_ns:?}",
            replacement.len()
        );
        assert!(old_p95_ns > 0, "positive baseline timing for {shape}");
        assert!(
            new_p95_ns.saturating_mul(100)
                <= old_p95_ns.saturating_mul(MAX_NEW_P95_PERCENT),
            "{shape}: replacement P95 {new_p95_ns} ns exceeds {MAX_NEW_P95_PERCENT}% of baseline {old_p95_ns} ns"
        );
    }
}

fn measure_pair(pair: usize, replacement: &str) -> (u128, u128) {
    if pair % 2 == 0 {
        (
            measure(
                TextInputConstraints::legacy_sanitize_replacement,
                replacement,
            ),
            measure(TextInputConstraints::sanitize_replacement, replacement),
        )
    } else {
        let new = measure(TextInputConstraints::sanitize_replacement, replacement);
        let old = measure(
            TextInputConstraints::legacy_sanitize_replacement,
            replacement,
        );
        (old, new)
    }
}

fn measure(
    sanitize: fn(TextInputConstraints, &str, UiTextRange, &str) -> SanitizedTextInputReplacement,
    replacement: &str,
) -> u128 {
    let sanitize = black_box(sanitize);
    let constraints = black_box(TextInputConstraints::default());
    let current_text = black_box("existing text");
    let range = black_box(UiTextRange { start: 8, end: 8 });
    let replacement = black_box(replacement);
    let started = Instant::now();
    let sanitized = sanitize(constraints, current_text, range, replacement);
    let elapsed = started.elapsed().as_nanos();
    assert_identity(&sanitized, replacement);
    black_box(sanitized);
    elapsed
}

fn assert_identity(sanitized: &SanitizedTextInputReplacement, replacement: &str) {
    assert_eq!(sanitized.text.as_bytes(), replacement.as_bytes());
    assert_eq!(sanitized.receipt, UiTextInputConstraintReceipt::default());
    assert!(sanitized.receipt_if_changed().is_none());
}

fn percentile(samples: &[u128], percent: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percent).div_ceil(100).saturating_sub(1)]
}

// Frozen full production scan from source SHA256
// 769de443484c2063543c7519dab8973491471dba90d64c50148e07373b25e8ba.
// Keep the original constraint and boundary-map branches in the timed baseline.
impl TextInputConstraints {
    fn legacy_sanitize_replacement(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
    ) -> SanitizedTextInputReplacement {
        self.legacy_sanitize_replacement_with_boundary_map(
            current_text,
            replaced_range,
            replacement,
            TextInputRetainedGraphemeCount::SourceScan,
            None,
        )
    }

    fn legacy_sanitize_replacement_with_boundary_map(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        retained_graphemes: TextInputRetainedGraphemeCount,
        mut boundary_map: Option<&mut TextInputBoundaryMap>,
    ) -> SanitizedTextInputReplacement {
        let mut receipt = UiTextInputConstraintReceipt::default();
        let mut filtered = String::with_capacity(replacement.len());
        let mut characters = replacement.char_indices().peekable();
        if let Some(boundary_map) = boundary_map.as_deref_mut() {
            boundary_map.record(0, 0);
        }
        while let Some((offset, character)) = characters.next() {
            let character_end = offset + character.len_utf8();
            if !self.multiline && crate::text::is_hard_line_separator(character) {
                receipt.removed_hard_line_count = receipt.removed_hard_line_count.saturating_add(1);
                if let Some(boundary_map) = boundary_map.as_deref_mut() {
                    boundary_map.record(character_end, filtered.len());
                }
                if character == '\r'
                    && characters
                        .peek()
                        .is_some_and(|(_, next_character)| *next_character == '\n')
                {
                    let (line_feed_offset, line_feed) = characters.next().unwrap();
                    if let Some(boundary_map) = boundary_map.as_deref_mut() {
                        boundary_map
                            .record(line_feed_offset + line_feed.len_utf8(), filtered.len());
                    }
                }
                continue;
            }
            if !self.filter.accepts(character) {
                receipt.removed_filter_scalar_count =
                    receipt.removed_filter_scalar_count.saturating_add(1);
                if let Some(boundary_map) = boundary_map.as_deref_mut() {
                    boundary_map.record(character_end, filtered.len());
                }
                continue;
            }
            filtered.push(character);
            if let Some(boundary_map) = boundary_map.as_deref_mut() {
                boundary_map.record(character_end, filtered.len());
            }
        }

        if let Some(max_graphemes) = self.max_graphemes {
            let retained = match retained_graphemes {
                TextInputRetainedGraphemeCount::DocumentIndex(retained) => {
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[0],
                        1
                    );
                    retained
                }
                TextInputRetainedGraphemeCount::SourceScan => {
                    let (retained, scanned_bytes) =
                        retained_grapheme_count_from_source(current_text, replaced_range);
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[1],
                        1
                    );
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[2],
                        scanned_bytes
                    );
                    retained
                }
            };
            let available = max_graphemes.saturating_sub(retained);
            if let Some((truncate_at, _)) = filtered.grapheme_indices(true).nth(available) {
                filtered.truncate(truncate_at);
                receipt.max_graphemes_truncated = true;
            }
        }
        if let Some(boundary_map) = boundary_map {
            boundary_map.clamp_output(filtered.len());
        }
        SanitizedTextInputReplacement {
            text: filtered,
            receipt,
        }
    }
}
