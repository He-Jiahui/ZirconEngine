pub(super) fn title_case_identifier(value: &str) -> String {
    let mut segments = value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty());
    let Some(first_segment) = segments.next() else {
        return value.to_string();
    };

    // A delimiter between words is never longer than the source it replaces.
    let mut title = String::with_capacity(value.len());
    append_title_word(&mut title, first_segment);
    for segment in segments {
        title.push(' ');
        append_title_word(&mut title, segment);
    }
    title
}

fn append_title_word(title: &mut String, segment: &str) {
    let mut chars = segment.chars();
    if let Some(first) = chars.next() {
        title.push(first.to_ascii_uppercase());
        for ch in chars {
            title.push(ch.to_ascii_lowercase());
        }
    }
}

#[cfg(test)]
#[path = "title_case/tests/cases.rs"]
mod tests;
