use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) fn chip_color_token(
    node: &TemplatePaneNodeData,
) -> &str {
    chip_color_token_for_variant(&node.component_variant)
}

fn chip_color_token_for_variant(component_variant: &str) -> &'static str {
    let mut best = u8::MAX;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        let candidate = if part.eq_ignore_ascii_case("primary")
            || part.eq_ignore_ascii_case("colorPrimary")
        {
            1
        } else if part.eq_ignore_ascii_case("secondary")
            || part.eq_ignore_ascii_case("colorSecondary")
        {
            2
        } else if part.eq_ignore_ascii_case("error") || part.eq_ignore_ascii_case("colorError") {
            3
        } else if part.eq_ignore_ascii_case("info") || part.eq_ignore_ascii_case("colorInfo") {
            4
        } else if part.eq_ignore_ascii_case("success") || part.eq_ignore_ascii_case("colorSuccess")
        {
            5
        } else if part.eq_ignore_ascii_case("warning") || part.eq_ignore_ascii_case("colorWarning")
        {
            6
        } else {
            continue;
        };
        best = best.min(candidate);
        if best == 1 {
            break;
        }
    }
    match best {
        1 => "primary",
        2 => "secondary",
        3 => "error",
        4 => "info",
        5 => "success",
        6 => "warning",
        _ => "default",
    }
}

#[cfg(test)]
#[path = "token/single_scan_color_tests.rs"]
mod single_scan_color_tests;
