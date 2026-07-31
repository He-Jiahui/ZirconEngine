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

    let mut sdf_advances = Vec::with_capacity(sdf_glyph_count);
    let mut graphemes = text.graphemes(true);
    let mut layout_advances = layout_advances.iter().copied();
    loop {
        match (graphemes.next(), layout_advances.next()) {
            (Some(grapheme), Some(layout_advance)) => {
                let char_count = grapheme.chars().count();
                sdf_advances.extend(std::iter::repeat(0.0).take(char_count.saturating_sub(1)));
                sdf_advances.push(sanitized_advance(layout_advance));
            }
            (None, None) => break,
            _ => return None,
        }
    }

    if sdf_advances.len() != sdf_glyph_count {
        return None;
    }

    sanitized_nonzero_advances(sdf_advances)
}

fn sanitized_nonzero_advances(advances: impl IntoIterator<Item = f32>) -> Option<Vec<f32>> {
    let advances = advances.into_iter();
    let mut sanitized = Vec::with_capacity(advances.size_hint().0);
    let mut any_nonzero = false;
    for advance in advances {
        let advance = sanitized_advance(advance);
        any_nonzero |= advance > 0.0;
        sanitized.push(advance);
    }
    any_nonzero.then_some(sanitized)
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

    #[test]
    fn advance_mapping_streams_graphemes_and_nonzero_detection() {
        let source = include_str!("sdf_advances.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("SDF advance implementation");

        assert!(!implementation.contains("text.chars().count()"));
        assert!(!implementation.contains("text.graphemes(true).collect::<Vec<_>>()"));
        assert!(!implementation.contains(".any(|advance|"));
        assert!(implementation.contains("match (graphemes.next(), layout_advances.next())"));
        assert!(implementation.contains("any_nonzero |= advance > 0.0"));
    }
}
