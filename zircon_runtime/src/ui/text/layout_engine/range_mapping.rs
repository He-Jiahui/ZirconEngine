use zircon_runtime_interface::ui::surface::UiTextRange;

pub(super) fn source_subrange(
    source_range: UiTextRange,
    visual_len: usize,
    start: usize,
    end: usize,
) -> UiTextRange {
    if source_range.start == source_range.end {
        return source_range;
    }
    if source_range.end.saturating_sub(source_range.start) != visual_len {
        return source_range;
    }
    UiTextRange {
        start: source_range.start + start,
        end: source_range.start + end,
    }
}
