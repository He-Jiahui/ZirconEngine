use crate::text::{FontFamilyName, FontQuery, FontStretch, FontStyle, FontWeight, TextStyle};

/// Builds the single font-selection query used by shaping and metric certification.
pub(crate) fn font_query_for_text_style(style: &TextStyle) -> FontQuery {
    let families = style
        .font_family
        .as_deref()
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(|family| vec![FontFamilyName::from(family)])
        .unwrap_or_default();
    FontQuery {
        families,
        weight: FontWeight::clamped(TextStyle::normalized_font_weight(style.font_weight)),
        style: if style.italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        },
        stretch: FontStretch::NORMAL,
    }
}

#[cfg(test)]
#[path = "tests/query.rs"]
mod tests;
