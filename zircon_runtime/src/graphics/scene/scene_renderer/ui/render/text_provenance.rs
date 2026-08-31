use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiResolvedTextLine, UiTextAlign, UiTextPaintRun, UiTextRunKind, UiTextWrap,
    UiTextWritingMode,
};

pub(super) struct SourceIsomorphicTextPaintLine<'a> {
    pub(super) line: &'a UiResolvedTextLine,
    pub(super) text_align: UiTextAlign,
    pub(super) wrap: UiTextWrap,
    pub(super) writing_mode: UiTextWritingMode,
}

pub(super) fn is_source_isomorphic_resolved_text_line(
    command: &UiRenderCommand,
    line: &UiResolvedTextLine,
) -> bool {
    if crate::text::resolved_text_line_requires_visual_fallback(line)
        || line.source_range.start > line.source_range.end
    {
        return false;
    }
    command
        .text
        .as_deref()
        .and_then(|source| source.get(line.source_range.start..line.source_range.end))
        == Some(line.text.as_str())
}

pub(super) fn has_source_isomorphic_plain_text_provenance(
    command: &UiRenderCommand,
    line: &UiResolvedTextLine,
) -> bool {
    let has_plain_run_provenance = match line.runs.as_slice() {
        [] => true,
        [run] => {
            run.kind == UiTextRunKind::Plain
                && run.text == line.text
                && run.source_range == line.source_range
                && run.visual_range == line.visual_range
                && run.direction == line.direction
        }
        _ => false,
    };
    has_plain_run_provenance && is_source_isomorphic_resolved_text_line(command, line)
}

pub(super) fn source_isomorphic_text_paint_line<'a>(
    command: &'a UiRenderCommand,
    run: &UiTextPaintRun,
) -> Option<SourceIsomorphicTextPaintLine<'a>> {
    let layout = command.text_layout.as_ref()?;
    let line = matching_resolved_text_line(&layout.lines, run)?;
    let is_single_source_run = line.runs.len() == 1
        && line.runs.first().is_some_and(|line_run| {
            line_run.kind == run.kind
                && line_run.text == run.text
                && line_run.source_range == run.source_range
                && line_run.visual_range == run.visual_range
        });
    (is_single_source_run && is_source_isomorphic_resolved_text_line(command, line)).then_some(
        SourceIsomorphicTextPaintLine {
            line,
            text_align: layout.text_align,
            wrap: layout.wrap,
            writing_mode: layout.writing_mode,
        },
    )
}

fn matching_resolved_text_line<'a>(
    lines: &'a [UiResolvedTextLine],
    run: &UiTextPaintRun,
) -> Option<&'a UiResolvedTextLine> {
    let range = (run.source_range.start, run.source_range.end);
    lines
        .binary_search_by(|line| (line.source_range.start, line.source_range.end).cmp(&range))
        .ok()
        .and_then(|index| lines.get(index))
        .filter(|line| resolved_text_line_matches_run(line, run))
        .or_else(|| {
            lines
                .iter()
                .find(|line| resolved_text_line_matches_run(line, run))
        })
}

fn resolved_text_line_matches_run(line: &UiResolvedTextLine, run: &UiTextPaintRun) -> bool {
    line.source_range == run.source_range
        && line.visual_range == run.visual_range
        && line.text == run.text
}

#[cfg(test)]
mod optimization_tests {
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
        let source = include_str!("text_provenance.rs");
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
}
