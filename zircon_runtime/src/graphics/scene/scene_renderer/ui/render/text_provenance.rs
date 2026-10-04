use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiResolvedTextLine, UiTextAlign, UiTextPaintRun, UiTextRunKind, UiTextWrap,
    UiTextWritingMode,
};

/// 只在源文本、布局行和绘制 run 一一对应时允许复用行位置；富文本或视觉替换不可偷走该快捷路径。
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

/// 为普通文本复用已解析行提供保守准入；混合 run、重排来源或视觉回退会拒绝此优化。
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
#[path = "tests/text_provenance_optimization_tests.rs"]
mod optimization_tests;
