use unicode_segmentation::UnicodeSegmentation;

use super::super::render::ScreenSpaceUiTextBatch;
use super::super::sdf_advances::resolved_layout_advances_for_sdf_glyphs;

pub(in crate::graphics::scene::scene_renderer::ui) fn resolved_horizontal_shaped_glyph_advances(
    text: &ScreenSpaceUiTextBatch,
) -> Vec<f32> {
    let mut natural_advances = None;
    if visual_range_topology_requires_projection(text) {
        let advances = natural_shaped_glyph_advances(text);
        // Rebased visual glyph ranges retain cluster and ligature topology even when counts cancel out.
        if let Some(resolved_advances) = resolved_layout_advances_for_shaped_glyphs(text, &advances)
        {
            return resolved_advances;
        }
        natural_advances = Some(advances);
    }
    if let Some(advances) = resolved_layout_advances_for_sdf_glyphs(
        text.text.as_str(),
        text.glyph_advances.as_slice(),
        text.shaped_glyphs.len(),
    ) {
        return advances;
    }

    let natural_advances = natural_advances.unwrap_or_else(|| natural_shaped_glyph_advances(text));
    resolved_layout_advances_for_shaped_glyphs(text, &natural_advances).unwrap_or(natural_advances)
}

pub(super) fn visual_range_topology_requires_projection(text: &ScreenSpaceUiTextBatch) -> bool {
    let Some(visual_origin) = text.source_range.map(|range| range.start) else {
        return false;
    };
    text.shaped_glyphs.iter().any(|glyph| {
        let Some(start) = glyph.source_range.start.checked_sub(visual_origin) else {
            return false;
        };
        let Some(end) = glyph.source_range.end.checked_sub(visual_origin) else {
            return false;
        };
        start < end && end - start != glyph.source_scalar.len_utf8()
    })
}

fn natural_shaped_glyph_advances(text: &ScreenSpaceUiTextBatch) -> Vec<f32> {
    text.shaped_glyphs
        .iter()
        .map(|glyph| sanitized_advance(glyph.advance))
        .collect()
}

fn resolved_layout_advances_for_shaped_glyphs(
    text: &ScreenSpaceUiTextBatch,
    natural_advances: &[f32],
) -> Option<Vec<f32>> {
    // Visual-fallback batches retain a source line range for layout provenance, but re-shape
    // `text.text`. Their shaped glyph ranges are visual byte offsets rebased by the line start,
    // so they can legitimately extend beyond the source range after virtual insertion.
    let visual_origin = text.source_range?.start;
    if text.glyph_advances.is_empty() || natural_advances.is_empty() {
        return None;
    }

    let mut glyphs = text
        .shaped_glyphs
        .iter()
        .enumerate()
        .map(|(index, glyph)| {
            let start = glyph.source_range.start.checked_sub(visual_origin)?;
            let end = glyph.source_range.end.checked_sub(visual_origin)?;
            Some((index, start, end, sanitized_advance(glyph.advance)))
        })
        .collect::<Option<Vec<_>>>()?;
    if glyphs.iter().any(|(_, start, end, _)| {
        *start >= *end
            || *end > text.text.len()
            || !text.text.is_char_boundary(*start)
            || !text.text.is_char_boundary(*end)
    }) {
        return None;
    }
    glyphs.sort_unstable_by_key(|(_, start, end, _)| (*start, *end));

    let mut resolved_advances = vec![0.0; glyphs.len()];
    let mut matched_glyphs = vec![false; glyphs.len()];
    let mut layout_advances = text.glyph_advances.iter().copied();
    let mut first_glyph = 0;
    for (start, grapheme) in text.text.grapheme_indices(true) {
        let advance = sanitized_advance(layout_advances.next()?);
        let end = start + grapheme.len();
        while glyphs
            .get(first_glyph)
            .is_some_and(|(_, _, glyph_end, _)| *glyph_end <= start)
        {
            first_glyph += 1;
        }
        let last = glyphs.partition_point(|(_, glyph_start, _, _)| *glyph_start < end);
        let candidates = &glyphs[first_glyph..last];
        let mut fallback_index = None;
        let mut natural_total = 0.0;
        let mut nonzero_count = 0;
        for (index, glyph_start, glyph_end, natural_advance) in candidates {
            if *glyph_start >= end || *glyph_end <= start {
                continue;
            }
            fallback_index.get_or_insert(*index);
            natural_total += *natural_advance;
            if *natural_advance > 0.0 {
                nonzero_count += 1;
            }
        }
        let Some(fallback_index) = fallback_index else {
            return None;
        };
        if !natural_total.is_finite() || nonzero_count == 0 {
            for (index, glyph_start, glyph_end, _) in candidates {
                if *glyph_start < end && *glyph_end > start {
                    matched_glyphs[*index] = true;
                }
            }
            resolved_advances[fallback_index] += advance;
            matched_glyphs[fallback_index] = true;
            continue;
        }

        let mut assigned = 0.0;
        let mut remaining_nonzero = nonzero_count;
        for (index, glyph_start, glyph_end, natural_advance) in candidates {
            if *glyph_start >= end || *glyph_end <= start {
                continue;
            }
            matched_glyphs[*index] = true;
            if *natural_advance <= 0.0 {
                continue;
            }
            remaining_nonzero -= 1;
            let resolved_advance = if remaining_nonzero == 0 {
                advance - assigned
            } else {
                advance * *natural_advance / natural_total
            };
            resolved_advances[*index] += resolved_advance;
            assigned += resolved_advance;
        }
    }
    if layout_advances.next().is_some()
        || natural_advances
            .iter()
            .enumerate()
            .any(|(index, advance)| *advance > 0.0 && !matched_glyphs[index])
        || !resolved_advances.iter().any(|advance| *advance > 0.0)
    {
        return None;
    }

    Some(resolved_advances)
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}
