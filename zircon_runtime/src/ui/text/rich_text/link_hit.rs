use zircon_runtime_interface::ui::{
    layout::UiPoint,
    surface::{UiResolvedTextLayout, UiTextCaretAffinity, UiTextRange},
};

use crate::text::resolve_compiled_rich_text_artifact;
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
    layout: &UiResolvedTextLayout,
    point: UiPoint,
) -> Option<UiTextLinkHit> {
    let hit = hit_test_text_layout(layout, point);
    if !hit.inside_line {
        return None;
    }
    let parsed = resolve_compiled_rich_text_artifact(layout.rich_text_artifact.as_ref()?)?;
    let link_hit = parsed.link_runs().find_map(|run| {
        let start = usize::try_from(run.byte_range.0).ok()?;
        let end = usize::try_from(run.byte_range.1).ok()?;
        let source_range = UiTextRange { start, end };
        let link = run.link.as_ref()?;
        range_contains_caret(source_range, hit.source_offset, hit.affinity).then_some(
            UiTextLinkHit {
                href: link.href.clone(),
                source_range,
                affinity: hit.affinity,
            },
        )
    });
    link_hit
}

fn range_contains_caret(range: UiTextRange, offset: usize, affinity: UiTextCaretAffinity) -> bool {
    match affinity {
        UiTextCaretAffinity::Upstream => range.start < offset && offset <= range.end,
        UiTextCaretAffinity::Downstream => range.start <= offset && offset < range.end,
    }
}
