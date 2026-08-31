use crate::core::framework::text::TextGlyph;
use crate::text::TextRange;

use super::LogicalVirtualLineSequence;

pub(super) fn project_logical_glyphs(
    sequence: &LogicalVirtualLineSequence,
    mut glyphs: Vec<TextGlyph>,
    visual_advances: &[f32],
) -> Option<Vec<TextGlyph>> {
    if !sequence.artifact_projection_allowed()
        || sequence.clusters.is_empty()
        || visual_advances.len() != sequence.clusters.len()
    {
        return None;
    }

    let mut projected = Vec::with_capacity(glyphs.len());
    let mut cluster_start = 0_usize;
    let mut cluster_end = 0_usize;
    let mut previous_logical_range: Option<std::ops::Range<usize>> = None;
    for (logical_glyph_index, mut glyph) in glyphs.drain(..).enumerate() {
        let logical_range = glyph.source_range.clone();
        if logical_range.start >= logical_range.end
            || logical_range.end > sequence.text.len()
            || previous_logical_range.as_ref().is_some_and(|previous| {
                logical_range.start < previous.start
                    || (logical_range.start == previous.start && logical_range.end != previous.end)
            })
        {
            return None;
        }
        while sequence
            .clusters
            .get(cluster_start)
            .is_some_and(|cluster| cluster.logical_range.end <= logical_range.start)
        {
            cluster_start = cluster_start.saturating_add(1);
        }
        cluster_end = cluster_end.max(cluster_start);
        while sequence
            .clusters
            .get(cluster_end)
            .is_some_and(|cluster| cluster.logical_range.start < logical_range.end)
        {
            cluster_end = cluster_end.saturating_add(1);
        }
        if cluster_start == cluster_end
            || sequence
                .clusters
                .get(cluster_end.saturating_sub(1))
                .is_none_or(|cluster| cluster.logical_range.end < logical_range.end)
        {
            return None;
        }

        let glyph_clusters = &sequence.clusters[cluster_start..cluster_end];
        let first_cluster = glyph_clusters.first()?;
        let external_glyph = glyph_clusters.iter().all(|cluster| cluster.external);
        if glyph_clusters
            .iter()
            .any(|cluster| cluster.external != external_glyph)
        {
            return None;
        }
        if external_glyph {
            previous_logical_range = Some(logical_range);
            continue;
        }
        let virtual_glyph = glyph_clusters
            .iter()
            .all(|cluster| cluster.source_range.start == cluster.source_range.end);
        if (!virtual_glyph
            && glyph_clusters
                .iter()
                .any(|cluster| cluster.source_range.start == cluster.source_range.end))
            || (virtual_glyph
                && glyph_clusters
                    .iter()
                    .any(|cluster| cluster.source_range != first_cluster.source_range))
            || glyph_clusters
                .iter()
                .any(|cluster| cluster.bidi_level % 2 != first_cluster.bidi_level % 2)
        {
            return None;
        }

        let source_range =
            glyph_clusters
                .iter()
                .copied()
                .fold(first_cluster.source_range, |range, cluster| TextRange {
                    start: range.start.min(cluster.source_range.start),
                    end: range.end.max(cluster.source_range.end),
                });
        glyph.source_range = source_range.start..source_range.end;
        glyph.bidi_level = first_cluster.bidi_level;
        glyph.flags.right_to_left = first_cluster.bidi_level % 2 == 1;
        glyph.flags.virtual_glyph = virtual_glyph;
        glyph.advance = 0.0;
        projected.push(ProjectedLogicalGlyph {
            glyph,
            logical_glyph_index,
            visual_index: glyph_clusters
                .iter()
                .map(|cluster| cluster.visual_index)
                .min()
                .unwrap_or(usize::MAX),
            cluster_range: cluster_start..cluster_end,
        });
        previous_logical_range = Some(logical_range);
    }

    projected.sort_by(|left, right| {
        left.visual_index
            .cmp(&right.visual_index)
            .then_with(|| left.logical_glyph_index.cmp(&right.logical_glyph_index))
    });
    let mut first_glyph_for_visual_cluster = vec![None; sequence.clusters.len()];
    for (glyph_index, glyph) in projected.iter().enumerate() {
        for cluster in &sequence.clusters[glyph.cluster_range.clone()] {
            first_glyph_for_visual_cluster[cluster.visual_index].get_or_insert(glyph_index);
        }
    }
    for (visual_index, advance) in visual_advances.iter().copied().enumerate() {
        let glyph_index = first_glyph_for_visual_cluster
            .get(visual_index)
            .copied()
            .flatten();
        if glyph_index.is_none()
            && sequence
                .visual_to_logical
                .get(visual_index)
                .and_then(|logical_index| sequence.clusters.get(*logical_index))
                .is_some_and(|cluster| cluster.external)
        {
            continue;
        }
        let glyph_index = glyph_index?;
        if advance.is_finite() {
            projected[glyph_index].glyph.advance += advance.max(0.0);
        }
    }
    Some(projected.into_iter().map(|glyph| glyph.glyph).collect())
}

struct ProjectedLogicalGlyph {
    glyph: TextGlyph,
    logical_glyph_index: usize,
    visual_index: usize,
    cluster_range: std::ops::Range<usize>,
}
