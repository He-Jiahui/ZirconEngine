use crate::core::framework::render::RichTextFormat;
use zircon_runtime_interface::ui::{
    layout::UiPoint,
    surface::{UiResolvedTextLayout, UiTextCaretAffinity, UiTextRange},
};

use super::parse_source_text;
use crate::ui::text::hit_test_text_layout;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextLinkHit {
    pub(crate) href: String,
    pub(crate) source_range: UiTextRange,
    pub(crate) affinity: UiTextCaretAffinity,
}

/// Resolves a surface-space point through the shared caret geometry and then
/// applies affinity at run boundaries so the trailing half of a link's final
/// grapheme still belongs to that link.
pub(crate) fn link_at_layout_point(
    markup: &str,
    format: RichTextFormat,
    layout: &UiResolvedTextLayout,
    point: UiPoint,
) -> Option<UiTextLinkHit> {
    let hit = hit_test_text_layout(layout, point);
    if !hit.inside_line {
        return None;
    }
    let parsed = parse_source_text(markup, format);
    parsed.runs.into_iter().find_map(|run| {
        let link = run.link?;
        range_contains_caret(run.source_range, hit.source_offset, hit.affinity).then_some(
            UiTextLinkHit {
                href: link.href,
                source_range: run.source_range,
                affinity: hit.affinity,
            },
        )
    })
}

fn range_contains_caret(range: UiTextRange, offset: usize, affinity: UiTextCaretAffinity) -> bool {
    match affinity {
        UiTextCaretAffinity::Upstream => range.start < offset && offset <= range.end,
        UiTextCaretAffinity::Downstream => range.start <= offset && offset < range.end,
    }
}
