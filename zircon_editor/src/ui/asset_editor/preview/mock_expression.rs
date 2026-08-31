use toml::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ParsedPreviewMockExpression {
    pub node_reference: String,
    pub property: String,
    pub nested_segments: Vec<String>,
}

pub(super) fn parse_preview_mock_expression(value: &Value) -> Option<ParsedPreviewMockExpression> {
    let expression = match value {
        Value::String(text) if text.trim_start().starts_with('=') => text.trim(),
        _ => return None,
    };
    let reference = expression.trim_start_matches('=').trim();
    parse_preview_mock_reference(reference)
}

pub(super) fn parse_preview_mock_reference(reference: &str) -> Option<ParsedPreviewMockExpression> {
    let mut segments = parse_preview_mock_reference_segments(reference)?.into_iter();
    let node_reference = segments.next()?;
    let property = segments.next()?;
    let nested_segments = segments.collect();
    Some(ParsedPreviewMockExpression {
        node_reference,
        property,
        nested_segments,
    })
}

pub(super) fn append_expression_path_segment(path: &mut String, segment: &str) {
    if segment.chars().all(|ch| ch.is_ascii_digit()) {
        path.push('[');
        path.push_str(segment);
        path.push(']');
    } else if is_identifier_segment(segment) {
        if !path.is_empty() {
            path.push('.');
        }
        path.push_str(segment);
    } else {
        path.push('[');
        path.push_str(&Value::String(segment.to_string()).to_string());
        path.push(']');
    }
}

fn parse_preview_mock_reference_segments(reference: &str) -> Option<Vec<String>> {
    let reference = reference.trim();
    if reference.is_empty() {
        return None;
    }

    let mut index = 0usize;
    let mut segments = Vec::new();
    while index < reference.len() {
        skip_expression_whitespace(reference, &mut index);
        if index >= reference.len() {
            break;
        }
        match reference.as_bytes()[index] {
            b'.' => {
                index += 1;
            }
            b'[' => {
                let segment = parse_bracket_segment(reference, &mut index)?;
                if segment.is_empty() {
                    return None;
                }
                segments.push(segment);
            }
            b']' => return None,
            _ => {
                let segment = parse_identifier_segment(reference, &mut index);
                if segment.is_empty() {
                    return None;
                }
                segments.push(segment);
            }
        }
    }

    (segments.len() >= 2).then_some(segments)
}

fn parse_identifier_segment(reference: &str, index: &mut usize) -> String {
    let start = *index;
    while *index < reference.len() && !matches!(reference.as_bytes()[*index], b'.' | b'[' | b']') {
        advance_expression_char(reference, index);
    }
    let segment = &reference[start..*index];
    segment.trim().to_string()
}

fn parse_bracket_segment(reference: &str, index: &mut usize) -> Option<String> {
    *index += 1;
    skip_expression_whitespace(reference, index);
    if *index >= reference.len() {
        return None;
    }

    let segment = match reference.as_bytes()[*index] {
        b'"' | b'\'' => parse_quoted_bracket_segment(reference, index)?,
        _ => {
            let start = *index;
            while *index < reference.len() && reference.as_bytes()[*index] != b']' {
                advance_expression_char(reference, index);
            }
            if *index >= reference.len() {
                return None;
            }
            let segment = &reference[start..*index];
            segment.trim().to_string()
        }
    };

    skip_expression_whitespace(reference, index);
    if *index >= reference.len() || reference.as_bytes()[*index] != b']' {
        return None;
    }
    *index += 1;
    Some(segment)
}

fn parse_quoted_bracket_segment(reference: &str, index: &mut usize) -> Option<String> {
    let quote = reference.as_bytes()[*index];
    *index += 1;
    let start = *index;
    while *index < reference.len() && reference.as_bytes()[*index] != quote {
        advance_expression_char(reference, index);
    }
    if *index >= reference.len() {
        return None;
    }
    let segment = reference[start..*index].to_string();
    *index += 1;
    Some(segment)
}

fn skip_expression_whitespace(reference: &str, index: &mut usize) {
    while *index < reference.len() {
        let current = reference[*index..]
            .chars()
            .next()
            .expect("index remains on a UTF-8 boundary");
        if !current.is_whitespace() {
            break;
        }
        *index += current.len_utf8();
    }
}

fn advance_expression_char(reference: &str, index: &mut usize) {
    let current = reference[*index..]
        .chars()
        .next()
        .expect("index remains on a UTF-8 boundary");
    *index += current.len_utf8();
}

fn is_identifier_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[cfg(test)]
#[path = "mock_expression/utf8_slice_parser_tests.rs"]
mod utf8_slice_parser_tests;
