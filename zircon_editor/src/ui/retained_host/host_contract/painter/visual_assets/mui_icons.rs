use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use super::{
    parse_svg_tree_data, render_svg_tree_image, render_svg_tree_pixels, HostPaintImagePixels,
    RasterTargetSize,
};

pub(super) fn module_candidates(icon_name: &str, workspace_root: &Path) -> Vec<PathBuf> {
    let Some(module_name) = module_name(icon_name) else {
        return Vec::new();
    };
    vec![workspace_root
        .join("dev/material-ui/packages/mui-icons-material/lib")
        .join(module_name)
        .with_extension("js")]
}

pub(super) fn is_module_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("js"))
        && path
            .to_string_lossy()
            .replace('\\', "/")
            .contains("/mui-icons-material/lib/")
}

pub(super) fn render_module_pixels(
    path: &Path,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let svg = module_svg(path)?;
    let tree = parse_svg_tree_data(svg.as_bytes(), None).map(Arc::new)?;
    render_svg_tree_pixels(tree, target, tint)
}

pub(super) fn render_module_image(
    path: &Path,
) -> Option<crate::ui::retained_host::primitives::Image> {
    let svg = module_svg(path)?;
    let tree = parse_svg_tree_data(svg.as_bytes(), None).map(Arc::new)?;
    render_svg_tree_image(tree)
}

fn module_name(icon_name: &str) -> Option<String> {
    let mut name = icon_name.trim().replace('\\', "/");
    for prefix in [
        "mui:",
        "mui/",
        "mui-icons/",
        "@mui/icons-material/",
        "icons-material/",
    ] {
        if let Some(stripped) = name.strip_prefix(prefix) {
            name = stripped.to_string();
            break;
        }
    }
    let name = name
        .trim_start_matches('/')
        .trim_end_matches(".js")
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim();
    let name = if is_pascal_mui_module_name(name) {
        name.to_string()
    } else {
        pascal_case_ligature_name(name)?
    };
    let starts_like_mui_icon = name
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_uppercase());
    (starts_like_mui_icon
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
    .then_some(name)
}

fn is_pascal_mui_module_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|first| first.is_ascii_uppercase())
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn pascal_case_ligature_name(name: &str) -> Option<String> {
    if name.contains('-') {
        return None;
    }
    let mut out = String::new();
    for part in name.split(|ch: char| !ch.is_ascii_alphanumeric()) {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        let first = chars.next()?;
        out.push(first.to_ascii_uppercase());
        out.push_str(chars.as_str());
    }
    is_pascal_mui_module_name(&out).then_some(out)
}

fn module_svg(path: &Path) -> Option<String> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct MuiIconPathElement {
    d: String,
    opacity: Option<String>,
}

fn path_elements(source: &str) -> Vec<MuiIconPathElement> {
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

fn escape_xml_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_parser_preserves_paths_and_opacity() {
        let source = r#"
            jsx(_Fragment.Fragment, {
                children: [jsx("path", { d: "M1 1h2v2H1z", opacity: ".3" }, "0"),
                    jsx("path", { d: "M4 4h2v2H4z" }, "1")]
            })
        "#;

        let elements = path_elements(source);

        assert_eq!(
            elements,
            vec![
                MuiIconPathElement {
                    d: "M1 1h2v2H1z".to_string(),
                    opacity: Some(".3".to_string())
                },
                MuiIconPathElement {
                    d: "M4 4h2v2H4z".to_string(),
                    opacity: None
                }
            ]
        );
    }

    #[test]
    fn module_name_accepts_mui_icon_aliases() {
        for (source, expected) in [
            ("mui:Add", "Add"),
            ("mui/Add", "Add"),
            ("@mui/icons-material/Search.js", "Search"),
            ("icons-material/Menu", "Menu"),
            ("Delete", "Delete"),
            ("folder", "Folder"),
            ("add_circle", "AddCircle"),
        ] {
            assert_eq!(module_name(source), Some(expected.to_string()));
        }
        assert_eq!(module_name("folder-open-outline"), None);
    }
}
