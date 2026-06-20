use std::{fs, path::Path};

use super::parser::path_elements;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn module_svg(
    path: &Path,
) -> Option<String> {
    let source = fs::read_to_string(path).ok()?;
    let paths = path_elements(&source);
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
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
