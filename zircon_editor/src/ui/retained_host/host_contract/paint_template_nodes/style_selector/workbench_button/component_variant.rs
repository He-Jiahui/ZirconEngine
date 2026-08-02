use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_compact_icon_text_workbench_button(
    node: &TemplatePaneNodeData,
) -> bool {
    node.component_variant
        .split_ascii_whitespace()
        .any(|token| token.eq_ignore_ascii_case("compact_icon_text"))
}
