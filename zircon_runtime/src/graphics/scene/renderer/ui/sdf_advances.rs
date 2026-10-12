use unicode_segmentation::UnicodeSegmentation;

/// 将布局的 grapheme 字距投影到逐字符 SDF 绘制槽，保持排版宽度与字形落点一致。
/// 计数或内容无法对齐时返回 None，调用方应退回自然字距而非猜测额外字符的位置。
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
    let mut any_nonzero = false;
    loop {
        match (graphemes.next(), layout_advances.next()) {
            (Some(grapheme), Some(layout_advance)) => {
                let char_count = grapheme.chars().count();
                sdf_advances.extend(std::iter::repeat(0.0).take(char_count.saturating_sub(1)));
                let layout_advance = sanitized_advance(layout_advance);
                any_nonzero |= layout_advance > 0.0;
                sdf_advances.push(layout_advance);
            }
            (None, None) => break,
            _ => return None,
        }
    }

    if sdf_advances.len() != sdf_glyph_count {
        return None;
    }

    any_nonzero.then_some(sdf_advances)
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
#[path = "tests/sdf_advances.rs"]
mod tests;
