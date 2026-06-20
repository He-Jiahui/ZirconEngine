#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct MuiIconPathElement {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) d: String,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) opacity: Option<String>,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn path_elements(
    source: &str,
) -> Vec<MuiIconPathElement> {
    let mut elements = Vec::new();
    let mut cursor = 0;
    while let Some(relative_index) = source[cursor..].find("d: \"") {
        let value_start = cursor + relative_index + 4;
        let Some((d, value_end)) = parse_js_double_quoted_value(source, value_start) else {
            break;
        };
        let object_tail = source[value_end..]
            .find('}')
            .map(|end| &source[value_end..value_end + end])
            .unwrap_or("");
        elements.push(MuiIconPathElement {
            d,
            opacity: path_opacity(object_tail),
        });
        cursor = value_end;
    }
    elements
}

fn path_opacity(source: &str) -> Option<String> {
    let marker = "opacity: ";
    let start = source.find(marker)? + marker.len();
    let tail = source[start..].trim_start();
    if let Some(stripped) = tail.strip_prefix('"') {
        let (value, _) = parse_js_double_quoted_value(stripped, 0)?;
        return Some(value);
    }
    let value = tail
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    (!value.is_empty()).then_some(value)
}

fn parse_js_double_quoted_value(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    let mut out = String::new();
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Some((out, index + 1)),
            b'\\' => {
                index += 1;
                let escaped = *bytes.get(index)?;
                out.push(match escaped {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'/' => '/',
                    b'n' => '\n',
                    b'r' => '\r',
                    b't' => '\t',
                    other => other as char,
                });
            }
            byte => out.push(byte as char),
        }
        index += 1;
    }
    None
}
