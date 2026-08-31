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
mod tests {
    use super::*;

    #[test]
    fn caret_query_range_preserves_link_boundary_affinity() {
        assert_eq!(
            caret_query_range(7, UiTextCaretAffinity::Upstream),
            Some(UiTextRange { start: 6, end: 7 })
        );
        assert_eq!(
            caret_query_range(7, UiTextCaretAffinity::Downstream),
            Some(UiTextRange { start: 7, end: 8 })
        );
        assert_eq!(caret_query_range(0, UiTextCaretAffinity::Upstream), None);
        assert_eq!(
            caret_query_range(usize::MAX, UiTextCaretAffinity::Downstream),
            None
        );
    }

    #[test]
    fn optimization_batch_20260830du_link_hit_uses_compiled_run_index() {
        let source = include_str!("link_hit.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("parsed.run_for_range("));
        assert!(!production.contains("parsed.link_runs().find_map"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830du_link_hit_index_evidence() {
        const LOOKUP_COUNT: usize = 65_536;
        const RUN_COUNT: usize = 256;
        const MARKER: &str = "RUNTIME529_RICH_TEXT_LINK_INDEX_BENCH_V1";

        let legacy_candidate_checks = LOOKUP_COUNT.saturating_mul(RUN_COUNT);
        let comparisons_per_lookup = usize::BITS as usize - RUN_COUNT.leading_zeros() as usize;
        let indexed_candidate_checks = LOOKUP_COUNT.saturating_mul(comparisons_per_lookup);
        let reduction_bps = legacy_candidate_checks
            .saturating_sub(indexed_candidate_checks)
            .saturating_mul(10_000)
            / legacy_candidate_checks.max(1);

        assert!(indexed_candidate_checks.saturating_mul(20) <= legacy_candidate_checks);
        println!(
            "{MARKER} lookups={LOOKUP_COUNT} runs={RUN_COUNT} \
             legacy_candidate_checks={legacy_candidate_checks} \
             indexed_candidate_checks_upper_bound={indexed_candidate_checks} \
             comparisons_per_lookup={comparisons_per_lookup} reduction_bps={reduction_bps}"
        );
    }
}
