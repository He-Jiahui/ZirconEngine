pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn module_name(
    icon_name: &str,
) -> Option<String> {
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
    let mut out = String::with_capacity(name.len());
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

#[cfg(test)]
#[path = "names/capacity_tests.rs"]
mod capacity_tests;
