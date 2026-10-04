use zircon_runtime::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use zircon_runtime::ui::tree::UiRuntimeTreeScrollExt;
use zircon_runtime_interface::ui::{component::UiValue, layout::UiContainerKind};

const REVIEW_STATES: &[&str] = &[
    "default",
    "hover",
    "pressed",
    "focused",
    "disabled",
    "selected",
    "open",
    "closed",
    "dragging",
    "drop-allowed",
    "drop-blocked",
    "empty",
    "running",
    "complete",
    "blocked",
    "success",
    "failure",
    "pending",
    "approved",
    "denied",
    "normal",
    "warning",
    "exceeded",
    "error",
    "long-en",
    "long-zh",
    "scroll-before",
    "scroll-after",
];

pub(super) fn supported(state: &str) -> bool {
    REVIEW_STATES.contains(&state)
}

// Match the shared Penpot review host: default preserves authored state;
// every explicit state starts from a deterministic transient-state baseline.
pub(super) fn apply(surface: &mut UiSurface, state: &str) -> Result<(), String> {
    if state == "default" || state == "scroll-before" || state == "scroll-after" {
        return Ok(());
    }
    let node_ids = surface.tree.nodes.keys().copied().collect::<Vec<_>>();
    if node_ids.is_empty() {
        return Err("state review requires a mounted component consumer".into());
    }

    for node_id in node_ids.iter().copied() {
        reset_transient_state(surface, node_id, state)?;
    }

    match state {
        "hover" | "pressed" | "focused" | "disabled" | "selected" => {
            let property = match state {
                "hover" => "hovered",
                "pressed" => "pressed",
                "focused" => "focused",
                "disabled" => "disabled",
                "selected" => "selected",
                _ => unreachable!(),
            };
            for node_id in node_ids {
                set_property(surface, node_id, property, UiValue::Bool(true), state)?;
                if state == "focused" {
                    set_property(
                        surface,
                        node_id,
                        "focus_visible",
                        UiValue::Bool(true),
                        state,
                    )?;
                }
            }
        }
        _ => apply_native_state(surface, &node_ids, state)?,
    }
    Ok(())
}

fn reset_transient_state(
    surface: &mut UiSurface,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    state: &str,
) -> Result<(), String> {
    for property in [
        "enabled",
        "hover",
        "hovered",
        "pressed",
        "enter_pressed",
        "active",
        "focused",
        "focus",
        "focus_visible",
        "focusVisible",
        "focus-visible",
        "selected",
        "disabled",
        "checked",
        "loading",
        "open",
        "popup_open",
        "dragging",
        "drop_hovered",
        "active_drag_target",
        "drop_allowed",
    ] {
        let value = UiValue::Bool(property == "enabled");
        set_property(surface, node_id, property, value, state)?;
    }
    set_property(
        surface,
        node_id,
        "button_interaction_state",
        UiValue::String("normal".into()),
        state,
    )
}

fn set_property(
    surface: &mut UiSurface,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    property: &str,
    value: UiValue,
    state: &str,
) -> Result<(), String> {
    let report = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            property,
            value.clone(),
        ))
        .map_err(|error| error.to_string())?;
    if report.status == UiPropertyMutationStatus::Rejected {
        return Err(format!(
            "native {state} state rejected {property}: {:?}",
            report.message
        ));
    }
    let node = surface
        .tree
        .node_mut(node_id)
        .ok_or("native review node disappeared while applying state")?;
    let metadata = node
        .template_metadata
        .as_mut()
        .ok_or("native review node has no template metadata")?;
    metadata.attributes.insert(property.into(), value.to_toml());
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeComponent {
    Dialog,
    ConfirmDialog,
    CommandPalette,
    NotificationCenter,
    DragOverlay,
    WorkbenchToast,
    AgentPlan,
    ToolCalls,
    AgentApproval,
    AiUsage,
    DataGrid,
    TreeView,
    AgentChat,
    ChatComposer,
}

fn native_component(
    surface: &UiSurface,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
) -> Option<NativeComponent> {
    let node = surface.tree.node(node_id)?;
    let metadata = node.template_metadata.as_ref()?;
    for value in [
        Some(metadata.component.as_str()),
        metadata
            .attributes
            .get("component_role")
            .and_then(toml::Value::as_str),
    ]
    .into_iter()
    .flatten()
    {
        let normalized = value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>();
        let component = match normalized.as_str() {
            "dialog" => NativeComponent::Dialog,
            "confirmdialog" => NativeComponent::ConfirmDialog,
            "commandpalette" | "commandpalettepopup" => NativeComponent::CommandPalette,
            "notificationcenter" => NativeComponent::NotificationCenter,
            "dragoverlay" => NativeComponent::DragOverlay,
            "workbenchtoast" | "toast" => NativeComponent::WorkbenchToast,
            "agentplan" | "muixagentplan" => NativeComponent::AgentPlan,
            "toolcalls" | "muixtoolcalls" => NativeComponent::ToolCalls,
            "agentapproval" | "muixagentapproval" => NativeComponent::AgentApproval,
            "aiusage" | "muixaiusage" => NativeComponent::AiUsage,
            "datagrid" | "muixdatagrid" => NativeComponent::DataGrid,
            "treeview" | "materialtreeview" | "muixtreeview" => NativeComponent::TreeView,
            "agentchat" | "muixagentchat" => NativeComponent::AgentChat,
            "chatcomposer" | "muixchatcomposer" => NativeComponent::ChatComposer,
            _ => continue,
        };
        return Some(component);
    }
    None
}

fn native_state_supported(component: NativeComponent, state: &str) -> bool {
    match component {
        NativeComponent::Dialog | NativeComponent::ConfirmDialog => {
            matches!(state, "open" | "closed" | "focused")
        }
        NativeComponent::CommandPalette => matches!(state, "open" | "focused" | "empty"),
        NativeComponent::NotificationCenter => matches!(state, "open" | "selected" | "empty"),
        NativeComponent::DragOverlay => {
            matches!(state, "dragging" | "drop-allowed" | "drop-blocked")
        }
        NativeComponent::WorkbenchToast => state == "open",
        NativeComponent::AgentPlan => matches!(state, "running" | "complete" | "blocked"),
        NativeComponent::ToolCalls => matches!(state, "running" | "success" | "failure"),
        NativeComponent::AgentApproval => matches!(state, "pending" | "approved" | "denied"),
        NativeComponent::AiUsage => matches!(state, "normal" | "warning" | "exceeded"),
        NativeComponent::DataGrid => matches!(state, "selected" | "empty"),
        NativeComponent::TreeView => matches!(state, "open" | "selected" | "empty"),
        NativeComponent::AgentChat => matches!(
            state,
            "focused" | "disabled" | "empty" | "running" | "error" | "long-en" | "long-zh"
        ),
        NativeComponent::ChatComposer => matches!(
            state,
            "focused" | "disabled" | "empty" | "running" | "error" | "long-en" | "long-zh"
        ),
    }
}

fn apply_native_state(
    surface: &mut UiSurface,
    node_ids: &[zircon_runtime_interface::ui::event_ui::UiNodeId],
    state: &str,
) -> Result<(), String> {
    let mut matched = 0;
    for node_id in node_ids.iter().copied() {
        let Some(component) = native_component(surface, node_id) else {
            continue;
        };
        if !native_state_supported(component, state) {
            continue;
        }
        matched += 1;
        match state {
            "open" => {
                if component == NativeComponent::TreeView {
                    set_property(surface, node_id, "expanded", UiValue::Bool(true), state)?;
                } else {
                    set_property(surface, node_id, "open", UiValue::Bool(true), state)?;
                    set_property(surface, node_id, "popup_open", UiValue::Bool(true), state)?;
                }
            }
            "closed" => {
                set_property(surface, node_id, "open", UiValue::Bool(false), state)?;
                set_property(surface, node_id, "popup_open", UiValue::Bool(false), state)?;
            }
            "focused" => {
                set_property(surface, node_id, "focused", UiValue::Bool(true), state)?;
                set_property(
                    surface,
                    node_id,
                    "focus_visible",
                    UiValue::Bool(true),
                    state,
                )?;
            }
            "selected" => {
                if component == NativeComponent::NotificationCenter {
                    if let Some(id) = first_notification_id(surface, node_id) {
                        set_property(
                            surface,
                            node_id,
                            "selected_notification_id",
                            UiValue::String(id),
                            state,
                        )?;
                    }
                }
                set_property(surface, node_id, "selected", UiValue::Bool(true), state)?;
            }
            "dragging" => {
                set_property(surface, node_id, "dragging", UiValue::Bool(true), state)?;
            }
            "drop-allowed" | "drop-blocked" => {
                for property in ["dragging", "drop_hovered", "active_drag_target"] {
                    set_property(surface, node_id, property, UiValue::Bool(true), state)?;
                }
                set_property(
                    surface,
                    node_id,
                    "drop_allowed",
                    UiValue::Bool(state == "drop-allowed"),
                    state,
                )?;
            }
            "empty" => match component {
                NativeComponent::CommandPalette => {
                    for property in ["commands", "filtered_commands", "recent_commands"] {
                        set_property(surface, node_id, property, empty_array(), state)?;
                    }
                    set_property(
                        surface,
                        node_id,
                        "selected_command_id",
                        UiValue::String(String::new()),
                        state,
                    )?;
                }
                NativeComponent::NotificationCenter => {
                    set_property(surface, node_id, "notifications", empty_array(), state)?;
                    set_property(surface, node_id, "unread_count", UiValue::Int(0), state)?;
                    set_property(
                        surface,
                        node_id,
                        "selected_notification_id",
                        UiValue::String(String::new()),
                        state,
                    )?;
                }
                NativeComponent::DataGrid | NativeComponent::TreeView => {
                    set_property(surface, node_id, "collection_items", empty_array(), state)?;
                }
                NativeComponent::AgentChat => {
                    set_property(surface, node_id, "messages", empty_array(), state)?;
                    set_property(
                        surface,
                        node_id,
                        "composer_text",
                        UiValue::String(String::new()),
                        state,
                    )?;
                    set_property(surface, node_id, "streaming", UiValue::Bool(false), state)?;
                    set_property(surface, node_id, "error", UiValue::Bool(false), state)?;
                }
                NativeComponent::ChatComposer => {
                    set_property(
                        surface,
                        node_id,
                        "composer_text",
                        UiValue::String(String::new()),
                        state,
                    )?;
                    set_property(surface, node_id, "streaming", UiValue::Bool(false), state)?;
                }
                _ => {}
            },
            "running" => {
                if component == NativeComponent::AgentPlan {
                    set_property(
                        surface,
                        node_id,
                        "component_variant",
                        UiValue::String("running".into()),
                        state,
                    )?;
                } else if matches!(
                    component,
                    NativeComponent::AgentChat | NativeComponent::ChatComposer
                ) {
                    set_property(surface, node_id, "streaming", UiValue::Bool(true), state)?;
                    if component == NativeComponent::AgentChat {
                        set_property(surface, node_id, "error", UiValue::Bool(false), state)?;
                    }
                }
            }
            "error" => {
                if matches!(
                    component,
                    NativeComponent::AgentChat | NativeComponent::ChatComposer
                ) {
                    set_property(surface, node_id, "streaming", UiValue::Bool(false), state)?;
                    set_property(surface, node_id, "error", UiValue::Bool(true), state)?;
                }
            }
            "long-en" => match component {
                NativeComponent::AgentChat => {
                    set_property(
                        surface,
                        node_id,
                        "messages",
                        string_array_value(vec![
                            "user|Please keep the full context visible while checking the responsive shell at the narrow breakpoint.".into(),
                            "agent|The retained host keeps the conversation and cited tool result aligned without hiding overflow.".into(),
                        ]),
                        state,
                    )?;
                    set_property(surface, node_id, "streaming", UiValue::Bool(false), state)?;
                }
                NativeComponent::ChatComposer => {
                    set_property(
                        surface,
                        node_id,
                        "composer_text",
                        UiValue::String(
                            "Continue the parity review across the full shell, compact rail, and mobile action bar.".into(),
                        ),
                        state,
                    )?;
                }
                _ => {}
            },
            "long-zh" => match component {
                NativeComponent::AgentChat => {
                    set_property(
                        surface,
                        node_id,
                        "messages",
                        string_array_value(vec![
                            "user|请在窄屏断点检查响应式外壳，并保持完整上下文可见。".into(),
                            "agent|保留的宿主会让对话、工具结果和引用文件保持对齐，不隐藏溢出内容。".into(),
                        ]),
                        state,
                    )?;
                    set_property(surface, node_id, "streaming", UiValue::Bool(false), state)?;
                }
                NativeComponent::ChatComposer => {
                    set_property(
                        surface,
                        node_id,
                        "composer_text",
                        UiValue::String(
                            "请继续检查完整外壳、紧凑图标栏和移动端底部操作栏的一致性。".into(),
                        ),
                        state,
                    )?;
                }
                _ => {}
            },
            "complete" => {
                if component == NativeComponent::AgentPlan {
                    set_property(
                        surface,
                        node_id,
                        "component_variant",
                        UiValue::String("complete".into()),
                        state,
                    )?;
                    set_property(surface, node_id, "value", UiValue::Float(1.0), state)?;
                    let values = string_array(surface, node_id, "collection_items")
                        .into_iter()
                        .map(|value| replace_status(&value, "done"))
                        .collect();
                    set_property(
                        surface,
                        node_id,
                        "collection_items",
                        string_array_value(values),
                        state,
                    )?;
                }
            }
            "blocked" => {
                if component == NativeComponent::AgentPlan {
                    set_property(
                        surface,
                        node_id,
                        "component_variant",
                        UiValue::String("blocked".into()),
                        state,
                    )?;
                    set_property(
                        surface,
                        node_id,
                        "validation_level",
                        UiValue::String("error".into()),
                        state,
                    )?;
                }
            }
            "success" | "failure" => {
                if component == NativeComponent::ToolCalls {
                    set_property(
                        surface,
                        node_id,
                        "component_variant",
                        UiValue::String(state.into()),
                        state,
                    )?;
                    let values = string_array(surface, node_id, "collection_items")
                        .into_iter()
                        .map(|value| replace_status(&value, state))
                        .collect();
                    set_property(
                        surface,
                        node_id,
                        "collection_items",
                        string_array_value(values),
                        state,
                    )?;
                }
            }
            "pending" | "approved" | "denied" => {
                if component == NativeComponent::AgentApproval {
                    set_property(
                        surface,
                        node_id,
                        "approval_state",
                        UiValue::String(state.into()),
                        state,
                    )?;
                    set_property(
                        surface,
                        node_id,
                        "component_variant",
                        UiValue::String(state.into()),
                        state,
                    )?;
                    if state == "approved" {
                        set_property(surface, node_id, "destructive", UiValue::Bool(false), state)?;
                    }
                }
            }
            "normal" | "warning" | "exceeded" => {
                if component == NativeComponent::AiUsage {
                    set_property(
                        surface,
                        node_id,
                        "component_variant",
                        UiValue::String(state.into()),
                        state,
                    )?;
                    set_property(
                        surface,
                        node_id,
                        "validation_level",
                        UiValue::String(if state == "exceeded" { "error" } else { state }.into()),
                        state,
                    )?;
                    set_property(
                        surface,
                        node_id,
                        "value",
                        UiValue::Float(match state {
                            "normal" => 0.39,
                            "warning" => 0.75,
                            _ => 1.0,
                        }),
                        state,
                    )?;
                }
            }
            _ => {}
        }
    }
    if matched == 0 {
        return Err(format!(
            "native state {state} requires a matching painter component"
        ));
    }
    Ok(())
}

fn first_notification_id(
    surface: &UiSurface,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
) -> Option<String> {
    string_array(surface, node_id, "notifications")
        .first()
        .and_then(|value| value.split('|').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn string_array(
    surface: &UiSurface,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    property: &str,
) -> Vec<String> {
    surface
        .tree
        .node(node_id)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(property))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}

fn empty_array() -> UiValue {
    UiValue::Array(Vec::new())
}

fn string_array_value(values: Vec<String>) -> UiValue {
    UiValue::Array(values.into_iter().map(UiValue::String).collect())
}

fn replace_status(value: &str, status: &str) -> String {
    value
        .find('|')
        .map(|separator| format!("{status}{}", &value[separator..]))
        .unwrap_or_else(|| value.to_string())
}

/// Set a real retained-tree scroll offset after the first layout pass.  The
/// second layout pass performed by the caller then produces the same clipped
/// render extract that an input-driven scroll would produce.
pub(super) fn apply_scroll_position(surface: &mut UiSurface, state: &str) -> Result<bool, String> {
    let end = match state {
        "scroll-before" => false,
        "scroll-after" => true,
        _ => return Ok(false),
    };
    let node_ids = surface
        .tree
        .nodes
        .iter()
        .filter_map(|(node_id, node)| {
            matches!(node.container, UiContainerKind::ScrollableBox(_)).then_some(*node_id)
        })
        .collect::<Vec<_>>();
    if node_ids.is_empty() {
        return Err(format!("{state} review requires an authored ScrollableBox"));
    }
    let mut changed = false;
    for node_id in node_ids {
        let scroll = surface
            .tree
            .node(node_id)
            .and_then(|node| node.scroll_state)
            .unwrap_or_default();
        let offset = if end {
            (scroll.content_extent - scroll.viewport_extent).max(0.0)
        } else {
            0.0
        };
        changed |= surface
            .tree
            .set_scroll_offset(node_id, offset)
            .map_err(|error| error.to_string())?;
    }
    Ok(changed)
}
