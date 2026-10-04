//! secondary 专用分派先检查 Inspector 属性行身份，再根据非空值和显示标签选资源或阴影外观。
//! 它只接管当前硬编码的视觉语义；未匹配行继续通用属性回退，标签改名会改变接管结果。

use super::super::super::data::TemplatePaneNodeData;
use super::constants::{
    COMPONENT_PROPERTY_SLOT_03, COMPONENT_PROPERTY_SLOT_04, COMPONENT_PROPERTY_VIRTUAL_PREFIX,
    INSPECTOR_LIGHTING_ROW, MATERIAL_PROPERTY_ROW, MESH_PROPERTY_ROW,
};
use super::kind::{InspectorResourceKind, InspectorRowKind};
use super::matching::matches_ignore_ascii_case;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn inspector_row_kind(
    node: &TemplatePaneNodeData,
) -> Option<InspectorRowKind> {
    if !is_inspector_property_row(node) {
        return None;
    }

    // TODO: [CR-EDITOR-PAINT-ROWS-0003] 插件属性通过 data_sync 投影可变的 display label，
    // 这里却用英文标签决定 Mesh、Material 和阴影绘制子类。需确认标签是否可本地化或重命名；
    // 若可变，应由稳定 field_id/value_kind 标记子类，否则合法属性会退回通用外观或被误识别。
    let label = node.text.trim();
    let value = node.value_text.trim();
    inspector_row_kind_from_text(label, value)
}

fn inspector_row_kind_from_text(label: &str, value: &str) -> Option<InspectorRowKind> {
    if value.is_empty() {
        return label
            .eq_ignore_ascii_case("Lighting")
            .then_some(InspectorRowKind::Disclosure);
    }
    if label.eq_ignore_ascii_case("Mesh") {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Mesh));
    }
    if matches_ignore_ascii_case(label, &["Material", "Materials"]) {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Material));
    }
    if label.eq_ignore_ascii_case("Cast Shadows") {
        return Some(InspectorRowKind::ShadowSelect);
    }
    if label.eq_ignore_ascii_case("Receive Shadows") {
        return Some(InspectorRowKind::ShadowCheck);
    }
    None
}

#[cfg(test)]
#[path = "classifier/tests/empty_value_short_circuit_tests.rs"]
mod empty_value_short_circuit_tests;

fn is_inspector_property_row(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        MESH_PROPERTY_ROW
            | MATERIAL_PROPERTY_ROW
            | COMPONENT_PROPERTY_SLOT_03
            | COMPONENT_PROPERTY_SLOT_04
            | INSPECTOR_LIGHTING_ROW
    ) || node
        .control_id
        .as_str()
        .starts_with(COMPONENT_PROPERTY_VIRTUAL_PREFIX)
}
