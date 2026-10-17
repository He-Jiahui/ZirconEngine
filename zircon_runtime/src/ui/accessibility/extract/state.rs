//! 从作者属性、运行时组件状态和控件契约投影快照状态，不执行输入动作或回写这些来源。
//! 各状态的回退顺序独立；安全文本输入在这里隐去值与选区，名称/描述由文本解析模块负责。

use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    accessibility::{UiA11yCheckedState, UiA11yRole, UiA11yTextSelection},
    component::{UiComponentState, UiValue},
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::UiWidgetBehavior,
};

use crate::ui::{
    surface::{editable_text_input_is_secure, ui_surface_effective_disabled, UiSurface},
    text::clamp_grapheme_boundary,
};

use super::widget_behavior;

/// 按行为读取展开状态：Disclosure/Popup 使用各自的 open 属性及运行时回退，其他行为读取作者展开属性。
/// 缺少元数据时返回无状态；两个展开类行为总能得到各自默认值。
pub(super) fn expanded_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> Option<bool> {
    let metadata = metadata?;
    match widget_behavior(metadata) {
        UiWidgetBehavior::Disclosure => {
            let property = metadata
                .widget
                .open_property
                .as_deref()
                .unwrap_or("expanded");
            Some(open_state_for(
                surface,
                node,
                metadata,
                property,
                "expanded",
                &["expanded"],
                default_expanded_state(metadata),
            ))
        }
        UiWidgetBehavior::Popup => {
            let property = metadata
                .widget
                .open_property
                .as_deref()
                .unwrap_or("popup_open");
            Some(open_state_for(
                surface,
                node,
                metadata,
                property,
                "popup_open",
                &["popup_open", "open"],
                false,
            ))
        }
        _ => bool_attribute_value(&metadata.attributes, "expanded"),
    }
}

fn open_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: &UiTemplateNodeMetadata,
    property: &str,
    canonical_property: &str,
    fallback_properties: &[&str],
    default_value: bool,
) -> bool {
    let component_state = surface.component_states.get(node.node_id);
    bool_attribute_value(&metadata.attributes, property)
        .or_else(|| component_state.and_then(|state| bool_component_state_value(state, property)))
        .or_else(|| {
            fallback_properties
                .iter()
                .copied()
                .filter(|fallback_property| *fallback_property != property)
                .find_map(|fallback_property| {
                    bool_attribute_value(&metadata.attributes, fallback_property)
                })
        })
        .or_else(|| {
            component_state.and_then(|state| {
                fallback_properties
                    .iter()
                    .copied()
                    .filter(|fallback_property| *fallback_property != property)
                    .find_map(|fallback_property| {
                        bool_component_state_value(state, fallback_property)
                    })
            })
        })
        .or_else(|| {
            component_state.and_then(|state| open_component_state_flag(state, canonical_property))
        })
        .unwrap_or(default_value)
}

fn open_component_state_flag(state: &UiComponentState, canonical_property: &str) -> Option<bool> {
    match canonical_property {
        "expanded" => state.flags.expanded.then_some(true),
        "popup_open" => state.flags.popup_open.then_some(true),
        _ => None,
    }
}

fn default_expanded_state(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "Group" | "InspectorSection" | "TreeView"
    )
}

/// 使用 Surface 的统一禁用判定，快照提取不维护另一份禁用状态。
pub(super) fn disabled_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    ui_surface_effective_disabled(surface, node.node_id, node, metadata)
}

/// 作者选中属性优先于组件值/标志；所有来源缺省时返回未选中。
pub(super) fn selected_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata
        .and_then(|metadata| bool_attribute_value(&metadata.attributes, "selected"))
        .or_else(|| {
            surface
                .component_states
                .get(node.node_id)
                .and_then(|state| {
                    bool_component_state_value(state, "selected")
                        .or_else(|| state.flags.selected.then_some(true))
                })
        })
        .unwrap_or(false)
}

/// 合并组件、作者属性及节点按下标志；存在肯定证据才返回按下状态，否则不设置该可选状态。
pub(super) fn pressed_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> Option<bool> {
    let pressed = surface
        .component_states
        .get(node.node_id)
        .is_some_and(|state| {
            bool_component_state_value(state, "pressed")
                .or_else(|| bool_component_state_value(state, "active"))
                .or_else(|| state.flags.pressed.then_some(true))
                .unwrap_or(false)
        })
        || metadata.is_some_and(|metadata| {
            bool_attribute_value(&metadata.attributes, "pressed") == Some(true)
                || bool_attribute_value(&metadata.attributes, "active") == Some(true)
        })
        || node.state_flags.pressed;
    pressed.then_some(true)
}

/// 依次采用作者 checked 属性、组件状态、控件契约与节点标志。
/// Checkbox/Radio 缺省为未勾选，其他角色没有来源时不设置该可选状态。
pub(super) fn checked_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
    role: UiA11yRole,
) -> Option<UiA11yCheckedState> {
    let checked = metadata
        .and_then(checked_attribute_value_for)
        .or_else(|| checked_component_state_value_for(surface, node, metadata))
        .or_else(|| metadata.and_then(|metadata| metadata.widget.checked))
        .or_else(|| {
            if node.state_flags.checked {
                Some(true)
            } else if matches!(role, UiA11yRole::Checkbox | UiA11yRole::Radio) {
                Some(false)
            } else {
                None
            }
        })?;
    Some(if checked {
        UiA11yCheckedState::True
    } else {
        UiA11yCheckedState::False
    })
}

fn checked_attribute_value_for(metadata: &UiTemplateNodeMetadata) -> Option<bool> {
    let property = metadata
        .widget
        .checked_property
        .as_deref()
        .unwrap_or("checked");
    bool_attribute_value(&metadata.attributes, property)
}

fn checked_component_state_value_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> Option<bool> {
    let state = surface.component_states.get(node.node_id)?;
    let property = metadata
        .and_then(|metadata| metadata.widget.checked_property.as_deref())
        .unwrap_or("checked");
    if property == "checked" {
        return bool_component_state_value(state, property)
            .or_else(|| state.flags.checked.then_some(true));
    }
    bool_component_state_value(state, property)
}

/// 按属性、组件状态、控件初值的顺序取得快照值，返回独立拥有的显示文本。
///
/// # 参数与返回
/// `role` 决定文本输入的回退与安全检查；缺少元数据或有效来源时返回无值。
/// 安全文本输入直接返回无值，此约束只作用于状态值，文本命名路径另行处理。
pub(super) fn value_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
    role: UiA11yRole,
) -> Option<String> {
    let metadata = metadata?;
    if role == UiA11yRole::TextInput && editable_text_input_is_secure(surface, node.node_id) {
        return None;
    }
    value_attribute_text(metadata, role)
        .or_else(|| component_state_value_text(surface, node.node_id, metadata, role))
        .or_else(|| {
            metadata
                .widget
                .value
                .as_ref()
                .map(|value| value.display_text())
        })
}

fn value_attribute_text(metadata: &UiTemplateNodeMetadata, role: UiA11yRole) -> Option<String> {
    if let Some(property) = metadata.widget.value_property.as_deref() {
        return metadata
            .attributes
            .get(property)
            .and_then(attribute_display_text);
    }
    metadata
        .attributes
        .get("value")
        .and_then(attribute_display_text)
        .or_else(|| {
            matches!(role, UiA11yRole::TextInput)
                .then(|| {
                    metadata
                        .attributes
                        .get("text")
                        .and_then(attribute_display_text)
                })
                .flatten()
        })
}

/// 为非安全文本输入投影选区；其他角色、安全输入或缺少元数据时不提供选区。
///
/// # 参数与返回
/// `value_text` 是本次快照值，缺省按空串处理。作者偏移优先于组件状态，未给出时使用末尾光标。
/// 偏移以 UTF-8 字节计，并收敛到该值的字素边界；返回数据不借用输入字符串。
pub(super) fn text_selection_state_for(
    surface: &UiSurface,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
    role: UiA11yRole,
    value_text: Option<&str>,
) -> Option<UiA11yTextSelection> {
    if role != UiA11yRole::TextInput {
        return None;
    }
    if editable_text_input_is_secure(surface, node.node_id) {
        return None;
    }
    let metadata = metadata?;
    let value_text = value_text.unwrap_or_default();
    let component_state = surface.component_states.get(node.node_id);
    let caret = usize_attribute_or_component_state_value(metadata, component_state, "caret_offset")
        .unwrap_or_else(|| value_text.len());
    let anchor =
        usize_attribute_or_component_state_value(metadata, component_state, "selection_anchor")
            .unwrap_or(caret);
    let focus =
        usize_attribute_or_component_state_value(metadata, component_state, "selection_focus")
            .unwrap_or(caret);

    Some(UiA11yTextSelection {
        caret: clamp_grapheme_boundary(value_text, caret),
        anchor: clamp_grapheme_boundary(value_text, anchor),
        focus: clamp_grapheme_boundary(value_text, focus),
    })
}

fn usize_attribute_or_component_state_value(
    metadata: &UiTemplateNodeMetadata,
    component_state: Option<&UiComponentState>,
    property: &str,
) -> Option<usize> {
    usize_attribute_value(&metadata.attributes, property)
        .or_else(|| component_state.and_then(|state| usize_component_state_value(state, property)))
}

fn usize_attribute_value(
    attributes: &BTreeMap<String, toml::Value>,
    property: &str,
) -> Option<usize> {
    attributes
        .get(property)
        .and_then(toml::Value::as_integer)
        .and_then(|value| usize::try_from(value).ok())
}

fn usize_component_state_value(state: &UiComponentState, property: &str) -> Option<usize> {
    match state.value(property) {
        Some(UiValue::Int(value)) => usize::try_from(*value).ok(),
        Some(UiValue::Float(value))
            if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 =>
        {
            Some(*value as usize)
        }
        Some(UiValue::String(value)) => value.parse::<usize>().ok(),
        _ => None,
    }
}

fn bool_component_state_value(state: &UiComponentState, property: &str) -> Option<bool> {
    match state.value(property) {
        Some(UiValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn component_state_value_text(
    surface: &UiSurface,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    role: UiA11yRole,
) -> Option<String> {
    let state = surface.component_states.get(node_id)?;
    if let Some(property) = metadata.widget.value_property.as_deref() {
        return state.value(property).map(|value| value.display_text());
    }
    state
        .value("value")
        .map(|value| value.display_text())
        .or_else(|| {
            matches!(role, UiA11yRole::TextInput)
                .then(|| state.value("text").map(|value| value.display_text()))
                .flatten()
        })
}

fn bool_attribute_value(
    attributes: &BTreeMap<String, toml::Value>,
    property: &str,
) -> Option<bool> {
    attributes.get(property).and_then(toml::Value::as_bool)
}

fn attribute_display_text(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(value) => Some(value.clone()),
        toml::Value::Integer(value) => Some(value.to_string()),
        toml::Value::Float(value) if value.is_finite() => Some(value.to_string()),
        toml::Value::Boolean(value) => Some(value.to_string()),
        toml::Value::Datetime(value) => Some(value.to_string()),
        _ => None,
    }
}
