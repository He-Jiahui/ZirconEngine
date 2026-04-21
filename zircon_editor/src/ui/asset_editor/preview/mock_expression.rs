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
    let segments = parse_preview_mock_reference_segments(reference)?;
    let node_reference = segments.first()?.clone();
    let property = segments.get(1)?.clone();
    let nested_segments = segments.into_iter().skip(2).collect::<Vec<_>>();
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
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return None;
    }

    let chars = trimmed.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut segments = Vec::new();
    while index < chars.len() {
        skip_expression_whitespace(&chars, &mut index);
        if index >= chars.len() {
            break;
        }
        match chars[index] {
            '.' => {
                index += 1;
            }
            '[' => {
                let segment = parse_bracket_segment(&chars, &mut index)?;
                if segment.is_empty() {
                    return None;
                }
                segments.push(segment);
            }
            ']' => return None,
            _ => {
                let segment = parse_identifier_segment(&chars, &mut index);
                if segment.is_empty() {
                    return None;
                }
                segments.push(segment);
            }
        }
    }

    (segments.len() >= 2).then_some(segments)
}

fn parse_identifier_segment(chars: &[char], index: &mut usize) -> String {
    let start = *index;
    while *index < chars.len() && !matches!(chars[*index], '.' | '[' | ']') {
        *index += 1;
    }
    chars[start..*index]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

fn parse_bracket_segment(chars: &[char], index: &mut usize) -> Option<String> {
    *index += 1;
    skip_expression_whitespace(chars, index);
    if *index >= chars.len() {
        return None;
    }

    let segment = match chars[*index] {
        '"' | '\'' => parse_quoted_bracket_segment(chars, index)?,
        _ => {
            let start = *index;
            while *index < chars.len() && chars[*index] != ']' {
                *index += 1;
            }
            if *index >= chars.len() {
                return None;
            }
            chars[start..*index]
                .iter()
                .collect::<String>()
                .trim()
                .to_string()
        }
    };

    skip_expression_whitespace(chars, index);
    if *index >= chars.len() || chars[*index] != ']' {
        return None;
    }
    *index += 1;
    Some(segment)
}

fn parse_quoted_bracket_segment(chars: &[char], index: &mut usize) -> Option<String> {
    let quote = chars[*index];
    *index += 1;
    let start = *index;
    while *index < chars.len() && chars[*index] != quote {
        *index += 1;
    }
    if *index >= chars.len() {
        return None;
    }
    let segment = chars[start..*index].iter().collect::<String>();
    *index += 1;
    Some(segment)
}

fn skip_expression_whitespace(chars: &[char], index: &mut usize) {
    while *index < chars.len() && chars[*index].is_whitespace() {
        *index += 1;
    }
}

fn is_identifier_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
