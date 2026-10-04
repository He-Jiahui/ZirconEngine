use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextLine, UiTextDirection, UiTextPaintRun, UiTextRange, UiTextRunKind,
        UiTextRunPaintStyle,
    },
};

use super::matching_resolved_text_line;

#[test]
fn optimization_batch_20260830du_text_line_lookup_uses_range_binary_search() {
    let source = include_str!("../text_provenance.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("text provenance production source");

    assert!(production.contains(".binary_search_by(|line|"));
    assert!(production.contains(".or_else(|| {"));
    assert!(production.contains(".find(|line| resolved_text_line_matches_run(line, run))"));
}

#[test]
fn optimization_batch_20260830du_text_line_lookup_preserves_reordered_payloads() {
    let lines = vec![resolved_line("second", 6, 12), resolved_line("first", 0, 5)];
    let run = paint_run("first", 0, 5);

    assert_eq!(
        matching_resolved_text_line(&lines, &run).map(|line| line.text.as_str()),
        Some("first")
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830du_text_line_lookup_evidence() {
    const LOOKUPS: usize = 65_536;
    const LINES: usize = 256;
    const BINARY_COMPARISONS_PER_LOOKUP: usize = 9;
    const MARKER: &str = "RUNTIME528_TEXT_LINE_RANGE_BINARY_LOOKUP_BENCH_V1";

    let legacy_candidate_checks = (0..LOOKUPS).map(|lookup| lookup % LINES + 1).sum::<usize>();
    let binary_candidate_checks = LOOKUPS * BINARY_COMPARISONS_PER_LOOKUP;
    let reduction_basis_points = legacy_candidate_checks
        .saturating_sub(binary_candidate_checks)
        .saturating_mul(10_000)
        / legacy_candidate_checks;

    assert_eq!(legacy_candidate_checks, 8_421_376);
    assert!(reduction_basis_points >= 9_200);
    println!(
        "{MARKER} lookups={LOOKUPS} lines={LINES} legacy_candidate_checks={legacy_candidate_checks} \
             binary_candidate_checks={binary_candidate_checks} reduction_basis_points={reduction_basis_points}"
    );
}

fn resolved_line(text: &str, start: usize, end: usize) -> UiResolvedTextLine {
    UiResolvedTextLine {
        text: text.to_string(),
        frame: UiFrame::default(),
        placement_frame: UiFrame::default(),
        source_range: UiTextRange { start, end },
        visual_range: UiTextRange { start, end },
        measured_width: 0.0,
        glyph_advances: Vec::new(),
        baseline: 0.0,
        direction: UiTextDirection::LeftToRight,
        runs: Vec::new(),
        ellipsized: false,
    }
}

fn paint_run(text: &str, start: usize, end: usize) -> UiTextPaintRun {
    UiTextPaintRun {
        kind: UiTextRunKind::Plain,
        text: text.to_string(),
        source_range: UiTextRange { start, end },
        visual_range: UiTextRange { start, end },
        frame: UiFrame::default(),
        color: None,
        font: None,
        font_family: None,
        font_weight: 400,
        font_size: 12.0,
        line_height: 14.0,
        style: UiTextRunPaintStyle::default(),
    }
}
