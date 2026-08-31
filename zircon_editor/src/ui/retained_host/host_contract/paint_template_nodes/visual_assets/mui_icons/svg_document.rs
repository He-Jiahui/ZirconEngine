use super::parser::path_elements;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn module_svg(
    source: &str,
) -> Option<String> {
    let paths = path_elements(source);
    if paths.is_empty() {
        return None;
    }

    let mut svg = String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">"#);
    for path in paths {
        svg.push_str(r#"<path d=""#);
        svg.push_str(&escape_xml_attribute(&path.d));
        svg.push('"');
        if let Some(opacity) = path.opacity {
            svg.push_str(r#" opacity=""#);
            svg.push_str(&escape_xml_attribute(&opacity));
            svg.push('"');
        }
        svg.push_str("/>");
    }
    svg.push_str("</svg>");
    Some(svg)
}

fn escape_xml_attribute(value: &str) -> String {
    let escaped_capacity = value.chars().fold(value.len(), |capacity, character| {
        capacity.saturating_add(match character {
            '&' => "&amp;".len() - 1,
            '"' => "&quot;".len() - 1,
            '<' => "&lt;".len() - 1,
            '>' => "&gt;".len() - 1,
            _ => 0,
        })
    });
    let mut escaped = String::with_capacity(escaped_capacity);
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
#[path = "svg_document/single_pass_escape_tests.rs"]
mod single_pass_escape_tests;
