pub(crate) fn estimate_dock_tab_width(label: &str) -> f32 {
    (estimate_text_width(label, 6.0) + 30.0).max(68.0)
}

pub(crate) fn estimate_document_tab_width(label: &str, closeable: bool) -> f32 {
    let min_width = if closeable { 114.0 } else { 92.0 };
    let chrome_width = if closeable { 54.0 } else { 42.0 };
    (estimate_text_width(label, 6.5) + chrome_width).max(min_width)
}

fn estimate_text_width(label: &str, ascii_char_width: f32) -> f32 {
    label
        .chars()
        .map(|ch| {
            if ch.is_ascii_uppercase() {
                ascii_char_width + 1.0
            } else if ch.is_ascii_whitespace() {
                ascii_char_width * 0.5
            } else if ch.is_ascii_punctuation() {
                ascii_char_width * 0.75
            } else if ch.is_ascii() {
                ascii_char_width
            } else {
                ascii_char_width * 1.8
            }
        })
        .sum::<f32>()
        + 2.0
}
