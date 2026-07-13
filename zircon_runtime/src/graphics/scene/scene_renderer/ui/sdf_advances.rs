use unicode_segmentation::UnicodeSegmentation;

pub(super) fn resolved_layout_advances_for_sdf_glyphs(
    text: &str,
    layout_advances: &[f32],
    sdf_glyph_count: usize,
) -> Option<Vec<f32>> {
    if layout_advances.is_empty() {
        return None;
    }

    if layout_advances.len() == sdf_glyph_count {
        return sanitized_nonzero_advances(layout_advances.iter().copied());
    }

    let sdf_char_count = text.chars().count();
    if sdf_char_count != sdf_glyph_count {
        return None;
    }

    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    if graphemes.len() != layout_advances.len() {
        return None;
    }

    let mut sdf_advances = Vec::with_capacity(sdf_glyph_count);
    for (grapheme, layout_advance) in graphemes.into_iter().zip(layout_advances) {
        let char_count = grapheme.chars().count();
        if char_count == 0 {
            continue;
        }
        sdf_advances.extend(std::iter::repeat(0.0).take(char_count.saturating_sub(1)));
        sdf_advances.push(sanitized_advance(*layout_advance));
    }

    if sdf_advances.len() != sdf_glyph_count {
        return None;
    }

    sanitized_nonzero_advances(sdf_advances)
}

fn sanitized_nonzero_advances(advances: impl IntoIterator<Item = f32>) -> Option<Vec<f32>> {
    let advances = advances
        .into_iter()
        .map(sanitized_advance)
        .collect::<Vec<_>>();
    advances
        .iter()
        .any(|advance| *advance > 0.0)
        .then_some(advances)
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_grapheme_advances_to_sdf_character_advances() {
        let advances = resolved_layout_advances_for_sdf_glyphs("e\u{301}A", &[19.0, 11.0], 3)
            .expect("grapheme advances should map to SDF char advances");

        assert_eq!(advances, vec![0.0, 19.0, 11.0]);
    }

    #[test]
    fn keeps_prior_character_advances_when_counts_match() {
        let advances = resolved_layout_advances_for_sdf_glyphs("ABC", &[5.0, 7.0, 9.0], 3)
            .expect("character advances should stay usable");

        assert_eq!(advances, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn rejects_empty_or_all_zero_advances() {
        assert!(resolved_layout_advances_for_sdf_glyphs("ABC", &[], 3).is_none());
        assert!(resolved_layout_advances_for_sdf_glyphs("ABC", &[0.0, 0.0, 0.0], 3).is_none());
    }
}
