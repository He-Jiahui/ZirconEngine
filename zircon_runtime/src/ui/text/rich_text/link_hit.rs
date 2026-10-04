use std::sync::Arc;

use zircon_runtime_interface::ui::{
    layout::UiPoint,
    surface::{UiResolvedTextLayout, UiTextCaretAffinity, UiTextRange},
    text::UiRichLinkTarget,
};

use crate::text::resolve_compiled_rich_text_artifact;
use crate::ui::text::hit_test_text_layout;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextLinkHit {
    pub(crate) target: UiRichLinkTarget,
    pub(crate) tooltip: Option<Arc<str>>,
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
    let query_range = caret_query_range(hit.source_offset, hit.affinity)?;
    let run = parsed.run_for_range(query_range.start, query_range.end)?;
    let link = run.link.as_ref()?;
    Some(UiTextLinkHit {
        target: link.target.clone(),
        tooltip: link.tooltip.clone(),
        source_range: UiTextRange {
            start: usize::try_from(run.byte_range.0).ok()?,
            end: usize::try_from(run.byte_range.1).ok()?,
        },
        affinity: hit.affinity,
    })
}

fn caret_query_range(offset: usize, affinity: UiTextCaretAffinity) -> Option<UiTextRange> {
    match affinity {
        UiTextCaretAffinity::Upstream => Some(UiTextRange {
            start: offset.checked_sub(1)?,
            end: offset,
        }),
        UiTextCaretAffinity::Downstream => Some(UiTextRange {
            start: offset,
            end: offset.checked_add(1)?,
        }),
    }
}

#[cfg(test)]
#[path = "tests/link_hit.rs"]
mod tests;
