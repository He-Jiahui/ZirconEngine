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
    let line = layout.lines.iter().find(|line| {
        line.source_range == run.source_range
            && line.visual_range == run.visual_range
            && line.text == run.text
    })?;
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
