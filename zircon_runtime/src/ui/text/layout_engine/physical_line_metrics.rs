use std::{collections::HashMap, sync::Arc};

use crate::text::layout::{
    CanonicalPhysicalLineFragment, TextLineMetrics,
    shape_horizontal_physical_line_fragment_with_provider,
};
use crate::text::shaping::{BidiLineOrder, TextLayoutOutcome, TextShapingOutcome};
use crate::text::{EphemeralCacheHash, SharedTextLayoutSession, TextRange, text_style};
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection};

use super::candidate_line::CandidateLine;

/// One shaped final-line fragment and the input identity required to reuse it after overflow
/// materialization changes only a subset of candidate lines.
#[derive(Clone)]
struct PhysicalLineFragmentEntry {
    lookup: FragmentLookup,
    fragment: Arc<CanonicalPhysicalLineFragment>,
}

impl PhysicalLineFragmentEntry {
    fn matches(&self, input: &FragmentInput<'_>) -> bool {
        self.lookup == input.lookup
            && self.fragment.shaped().source_range == input.source_range
            && self.fragment.shaped().source_text.as_ref() == input.text
    }
}

/// Hash indexing narrows retained-fragment lookup; `PhysicalLineFragmentEntry::matches` still
/// compares source text and the full range before reusing a fragment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct FragmentLookup {
    source_start: usize,
    source_end: usize,
    text_hash: EphemeralCacheHash,
    text_len: usize,
}

struct FragmentInput<'a> {
    lookup: FragmentLookup,
    source_range: TextRange,
    text: &'a str,
}

/// Immutable final-line fragments for one non-virtualized UI layout pass.
///
/// The initial collection is built after wrapping. If ellipsis or tatweel changes a candidate,
/// `refresh_with_provider` retains matching source-congruent fragments and leaves the virtual
/// candidate on its existing explicit fallback path.
pub(super) struct PhysicalLineFragments {
    fragments: Vec<Option<PhysicalLineFragmentEntry>>,
}

impl PhysicalLineFragments {
    pub(super) fn shape_with_provider(
        source_text: &str,
        source_origin: usize,
        lines: &[CandidateLine],
        style: &UiResolvedStyle,
        direction: UiTextDirection,
        provider: &mut SharedTextLayoutSession,
    ) -> TextLayoutOutcome<Self> {
        crate::profile_scope!("runtime", "text.layout", "shape_physical_line_fragments");
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let cache_report_before =
            super::layout_profile_metrics_enabled().then(|| provider.cache_report());
        let mut fragments = Self {
            fragments: Vec::new(),
        };
        match fragments.refresh_with_provider(
            source_text,
            source_origin,
            lines,
            style,
            direction,
            provider,
        ) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        if let Some(cache_report_before) = cache_report_before {
            let cache_report_after = provider.cache_report();
            crate::profile_counter!(
                "runtime",
                "physical_line_fragment_initial_shape_request_count",
                fragments.fragments.iter().flatten().count()
            );
            crate::profile_counter!(
                "runtime",
                "physical_line_fragment_shaped_cache_hit_count",
                cache_report_after
                    .hit_count
                    .saturating_sub(cache_report_before.hit_count)
            );
            crate::profile_counter!(
                "runtime",
                "physical_line_fragment_shaped_cache_miss_count",
                cache_report_after
                    .miss_count
                    .saturating_sub(cache_report_before.miss_count)
            );
        }
        TextShapingOutcome::Ready(fragments)
    }

    pub(super) fn refresh_with_provider(
        &mut self,
        source_text: &str,
        source_origin: usize,
        lines: &[CandidateLine],
        style: &UiResolvedStyle,
        direction: UiTextDirection,
        provider: &mut SharedTextLayoutSession,
    ) -> TextLayoutOutcome<()> {
        let mut retained: HashMap<FragmentLookup, PhysicalLineFragmentEntry> =
            HashMap::with_capacity(self.fragments.len());
        for entry in self.fragments.iter().flatten() {
            retained
                .entry(entry.lookup)
                .or_insert_with(|| entry.clone());
        }

        let mut refreshed = Vec::with_capacity(lines.len());
        for line in lines {
            let Some(input) = fragment_input(line, source_text, source_origin) else {
                refreshed.push(None);
                continue;
            };
            if let Some(entry) = retained
                .get(&input.lookup)
                .filter(|entry| entry.matches(&input))
            {
                refreshed.push(Some(entry.clone()));
                continue;
            }

            let fragment = match shape_horizontal_physical_line_fragment_with_provider(
                input.text,
                &text_style(style),
                direction.into(),
                input.source_range,
                provider,
            ) {
                TextShapingOutcome::Ready(fragment) => Arc::new(fragment),
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            refreshed.push(Some(PhysicalLineFragmentEntry {
                lookup: input.lookup,
                fragment,
            }));
        }
        self.fragments = refreshed;
        TextShapingOutcome::Ready(())
    }

    pub(super) fn metrics(&self, fallback: TextLineMetrics) -> Vec<TextLineMetrics> {
        self.fragments
            .iter()
            .map(|entry| {
                entry
                    .as_ref()
                    .map(|entry| entry.fragment.metrics())
                    .unwrap_or(fallback)
            })
            .collect()
    }

    pub(super) fn fragment_at(&self, index: usize) -> Option<&CanonicalPhysicalLineFragment> {
        self.fragments
            .get(index)
            .and_then(Option::as_ref)
            .map(|entry| entry.fragment.as_ref())
    }

    /// Tab placement depends on the physical pen position, while a shaped fragment stores raw
    /// advances. Keep using the line-width owner for those advances, but retain the fragment for
    /// selected-face metrics and source-identity artifact projection.
    pub(super) fn grapheme_advances_for_layout(
        &self,
        index: usize,
        line: &CandidateLine,
    ) -> Option<Vec<f32>> {
        raw_fragment_advances_are_layout_safe(line)
            .then(|| {
                self.fragment_at(index)
                    .map(|fragment| fragment.grapheme_advances().to_vec())
            })
            .flatten()
    }

    pub(super) fn fragment_handle_at(
        &self,
        index: usize,
    ) -> Option<Arc<CanonicalPhysicalLineFragment>> {
        self.fragments
            .get(index)
            .and_then(Option::as_ref)
            .map(|entry| Arc::clone(&entry.fragment))
    }

    pub(super) fn visual_order_for_layout(&self, index: usize) -> Option<&BidiLineOrder> {
        self.fragment_at(index)
            .and_then(CanonicalPhysicalLineFragment::visual_order)
    }
}

fn fragment_input<'a>(
    line: &'a CandidateLine,
    source_text: &str,
    source_origin: usize,
) -> Option<FragmentInput<'a>> {
    let source_range = source_congruent_range(line, source_text, source_origin)?;
    Some(FragmentInput {
        lookup: FragmentLookup {
            source_start: source_range.start,
            source_end: source_range.end,
            text_hash: hash_text(&line.text),
            text_len: line.text.len(),
        },
        source_range,
        text: &line.text,
    })
}

fn raw_fragment_advances_are_layout_safe(line: &CandidateLine) -> bool {
    !line.text.contains('\t')
}

fn source_congruent_range(
    line: &CandidateLine,
    source_text: &str,
    source_origin: usize,
) -> Option<TextRange> {
    (!line.ellipsized)
        .then_some(())
        .and_then(|_| {
            (line
                .source_range
                .end
                .saturating_sub(line.source_range.start)
                == line.text.len())
            .then_some(())
        })
        .and_then(|_| {
            (source_text.get(line.source_range.start..line.source_range.end)
                == Some(line.text.as_str()))
            .then_some(())
            .and_then(|_| {
                let start = source_origin.checked_add(line.source_range.start)?;
                let end = source_origin.checked_add(line.source_range.end)?;
                Some(TextRange { start, end })
            })
        })
}

fn hash_text(text: &str) -> EphemeralCacheHash {
    EphemeralCacheHash::from_hashable(text)
}

pub(super) fn visible_line_capacity(metrics: &[TextLineMetrics], frame_height: f32) -> usize {
    let frame_height = frame_height.max(0.0);
    let mut occupied_height = 0.0_f32;
    let mut count = 0_usize;
    for metrics in metrics {
        let line_height = metrics.line_height.max(0.0);
        if count > 0 && occupied_height + line_height > frame_height {
            break;
        }
        occupied_height += line_height;
        count = count.saturating_add(1);
    }
    count.max(1)
}

pub(super) fn maximum_line_height(metrics: &[TextLineMetrics], fallback_line_height: f32) -> f32 {
    metrics
        .iter()
        .map(|metrics| metrics.line_height)
        .fold(fallback_line_height, f32::max)
}

pub(super) fn total_line_height(metrics: &[TextLineMetrics]) -> f32 {
    metrics.iter().map(|metrics| metrics.line_height).sum()
}

#[cfg(test)]
mod tests {
    use crate::text::TextRange;
    use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

    use super::super::candidate_line::append_segment;
    use super::{
        CandidateLine, fragment_input, raw_fragment_advances_are_layout_safe,
        source_congruent_range,
    };

    #[test]
    fn source_congruent_candidate_preserves_the_absolute_shaper_range() {
        let mut line = CandidateLine::empty();
        append_segment(
            &mut line,
            UiTextRunKind::Plain,
            "world",
            UiTextRange { start: 0, end: 5 },
        );

        assert_eq!(
            source_congruent_range(&line, "world", 11),
            Some(TextRange { start: 11, end: 16 })
        );
    }

    #[test]
    fn tab_containing_source_congruent_candidate_retains_a_metric_fragment_input() {
        let mut line = CandidateLine::empty();
        append_segment(
            &mut line,
            UiTextRunKind::Plain,
            "alpha\tbeta",
            UiTextRange { start: 0, end: 10 },
        );

        let input = fragment_input(&line, "alpha\tbeta", 0)
            .expect("tab placement does not invalidate source-congruent font metrics");

        assert_eq!(input.source_range, TextRange { start: 0, end: 10 });
        assert_eq!(input.text, "alpha\tbeta");
        assert!(
            !raw_fragment_advances_are_layout_safe(&line),
            "tab stop placement must keep owning the final x advances"
        );
    }

    #[test]
    fn source_congruent_range_rejects_an_unrepresentable_absolute_offset() {
        let mut line = CandidateLine::empty();
        append_segment(
            &mut line,
            UiTextRunKind::Plain,
            "word",
            UiTextRange { start: 0, end: 4 },
        );

        assert_eq!(source_congruent_range(&line, "word", usize::MAX), None);
    }
}
