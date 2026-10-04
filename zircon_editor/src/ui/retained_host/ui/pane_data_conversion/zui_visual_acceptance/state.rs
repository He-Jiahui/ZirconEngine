use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_runtime::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use zircon_runtime::ui::tree::UiRuntimeTreeScrollExt;
use zircon_runtime_interface::ui::{
    component::UiValue, event_ui::UiNodeId, layout::UiContainerKind, widget::UiPopupAnchor,
};

use super::contract::ReviewCase;

const REVIEW_STATES: &[&str] = &[
    "default",
    "hover",
    "pressed",
    "focused",
    "disabled",
    "selected",
    "open",
    "closed",
    "focus-return",
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
    "scroll-before",
    "scroll-after",
];

pub(super) fn supported(state: &str) -> bool {
    REVIEW_STATES.contains(&state)
}

// Explicit review states replace transient inputs on the cloned retained host;
// default and scroll cases preserve authored values until the layout pass.
pub(super) fn apply(surface: &mut UiSurface, state: &str) -> Result<(), String> {
    apply_with_target(surface, state, None)
}

pub(super) fn apply_case(surface: &mut UiSurface, case: &ReviewCase) -> Result<(), String> {
    let selector = workbench_state_selector(&case.data)?;
    let target = selector
        .as_ref()
        .map(|selector| find_workbench_node(surface, selector))
        .transpose()?;
    let snapshot_state = case.data.get("workbenchPresentation").is_some()
        && matches!(case.state.as_str(), "empty" | "selected");
    if !snapshot_state {
        apply_with_target(surface, &case.state, target)?;
    }
    if let Some(selector) = selector {
        apply_text_overrides(surface, &selector.text_overrides)?;
    }
    Ok(())
}

pub(super) struct WorkbenchStateSelector {
    pub source_path: String,
    pub control_id: String,
    pub source_node_id: Option<String>,
    pub instance_path: Option<String>,
    pub scroll_target: Option<WorkbenchTargetSelector>,
    pub text_overrides: Vec<WorkbenchTextOverride>,
}

#[derive(Clone, Debug)]
pub(super) struct WorkbenchTargetSelector {
    pub source_path: String,
    pub control_id: String,
    pub source_node_id: Option<String>,
    pub instance_path: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct WorkbenchTextOverride {
    pub source_path: String,
    pub source_node_id: String,
    pub control_id: String,
    pub instance_path: Option<String>,
    pub value: String,
}

pub(super) fn workbench_state_selector(
    data: &Value,
) -> Result<Option<WorkbenchStateSelector>, String> {
    let Some(value) = data.get("workbenchState") else {
        return Ok(None);
    };
    let object = value
        .as_object()
        .ok_or("Workbench state selector must be an object")?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "sourcePath"
                | "controlId"
                | "sourceNodeId"
                | "instancePath"
                | "scrollTarget"
                | "textOverrides"
        )
    }) {
        return Err("Workbench state selector contains an unsupported field".into());
    }
    let source_path = object
        .get("sourcePath")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or("Workbench state sourcePath must be a nonempty string")?;
    let control_id = object
        .get("controlId")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or("Workbench state controlId must be a nonempty string")?;
    let source_node_id = match object.get("sourceNodeId") {
        None => None,
        Some(value) => Some(
            value
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .ok_or("Workbench state sourceNodeId must be a nonempty string")?
                .to_owned(),
        ),
    };
    let instance_path =
        parse_instance_path(object.get("instancePath"), "Workbench state instancePath")?;
    let scroll_target = object
        .get("scrollTarget")
        .map(|value| parse_target_selector(value, "Workbench scrollTarget"))
        .transpose()?;
    let text_overrides = parse_text_overrides(object.get("textOverrides"))?;
    Ok(Some(WorkbenchStateSelector {
        source_path: source_path.to_owned(),
        control_id: control_id.to_owned(),
        source_node_id,
        instance_path,
        scroll_target,
        text_overrides,
    }))
}

fn parse_target_selector(value: &Value, field: &str) -> Result<WorkbenchTargetSelector, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{field} must be an object"))?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "sourcePath" | "controlId" | "sourceNodeId" | "instancePath"
        )
    }) {
        return Err(format!("{field} contains an unsupported selector property"));
    }
    let string_field = |key: &str| {
        object
            .get(key)
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned)
            .ok_or_else(|| format!("{field} requires a nonempty {key}"))
    };
    let source_path = string_field("sourcePath")?;
    let control_id = string_field("controlId")?;
    let source_node_id = object
        .get("sourceNodeId")
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("{field} sourceNodeId must be a nonempty string"))
        })
        .transpose()?;
    let instance_path =
        parse_instance_path(object.get("instancePath"), &format!("{field} instancePath"))?;
    Ok(WorkbenchTargetSelector {
        source_path,
        control_id,
        source_node_id,
        instance_path,
    })
}

fn parse_instance_path(value: Option<&Value>, field: &str) -> Result<Option<String>, String> {
    let Some(value) = value else { return Ok(None) };
    let text = value
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{field} must be a nonempty canonical JSON string"))?;
    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct InstancePathStep {
        source_path: String,
        source_node_id: String,
    }
    let steps: Vec<InstancePathStep> = serde_json::from_str(text)
        .map_err(|_| format!("{field} must be canonical JSON invocation steps"))?;
    if steps
        .iter()
        .any(|step| step.source_path.trim().is_empty() || step.source_node_id.trim().is_empty())
    {
        return Err(format!("{field} step identities must be nonempty"));
    }
    let canonical = serde_json::to_string(&steps).map_err(|error| error.to_string())?;
    if canonical != text {
        return Err(format!("{field} must use canonical compact camelCase JSON"));
    }
    Ok(Some(canonical))
}

fn parse_text_overrides(value: Option<&Value>) -> Result<Vec<WorkbenchTextOverride>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let entries = value
        .as_array()
        .filter(|entries| !entries.is_empty())
        .ok_or("Workbench state textOverrides must be a nonempty array")?;
    let mut seen = HashSet::new();
    let mut overrides = Vec::with_capacity(entries.len());
    for (index, value) in entries.iter().enumerate() {
        let object = value
            .as_object()
            .ok_or_else(|| format!("Workbench text override {index} must be an object"))?;
        if object.keys().any(|key| {
            !matches!(
                key.as_str(),
                "sourcePath" | "sourceNodeId" | "controlId" | "instancePath" | "property" | "value"
            )
        }) {
            return Err(format!(
                "Workbench text override {index} contains unsupported fields"
            ));
        }
        let string_field = |name: &str| {
            object
                .get(name)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("Workbench text override {index} {name} must be nonempty"))
        };
        let source_path = string_field("sourcePath")?;
        let source_node_id = string_field("sourceNodeId")?;
        let control_id = string_field("controlId")?;
        if object.get("property").and_then(Value::as_str) != Some("text") {
            return Err(format!(
                "Workbench text override {index} property must be text"
            ));
        }
        let value_text = string_field("value")?;
        if value_text.as_bytes().len() > 4096 {
            return Err(format!(
                "Workbench text override {index} exceeds 4096 UTF-8 bytes"
            ));
        }
        if value_text.chars().any(|character| character.is_control()) {
            return Err(format!(
                "Workbench text override {index} contains a control character"
            ));
        }
        let instance_path = parse_instance_path(
            object.get("instancePath"),
            &format!("Workbench text override {index} instancePath"),
        )?;
        let key = format!(
            "{source_path}\0{source_node_id}\0{control_id}\0{}",
            instance_path.as_deref().unwrap_or("")
        );
        if !seen.insert(key) {
            return Err(format!("Duplicate workbench text override {index}"));
        }
        overrides.push(WorkbenchTextOverride {
            source_path,
            source_node_id,
            control_id,
            instance_path,
            value: value_text,
        });
    }
    Ok(overrides)
}

fn find_workbench_node(
    surface: &UiSurface,
    selector: &WorkbenchStateSelector,
) -> Result<UiNodeId, String> {
    find_workbench_target(
        surface,
        &WorkbenchTargetSelector {
            source_path: selector.source_path.clone(),
            control_id: selector.control_id.clone(),
            source_node_id: selector.source_node_id.clone(),
            instance_path: selector.instance_path.clone(),
        },
    )
}

fn find_workbench_target(
    surface: &UiSurface,
    selector: &WorkbenchTargetSelector,
) -> Result<UiNodeId, String> {
    let matches = surface
        .tree
        .nodes
        .iter()
        .filter_map(|(node_id, node)| {
            let metadata = node.template_metadata.as_ref()?;
            (metadata.control_id.as_deref() == Some(selector.control_id.as_str())
                && metadata.source_path.as_deref() == Some(selector.source_path.as_str())
                && (selector.source_node_id.is_none()
                    || metadata.source_node_id.as_deref() == selector.source_node_id.as_deref())
                && (selector.instance_path.is_none()
                    || canonical_metadata_instance_path(metadata).as_deref()
                        == selector.instance_path.as_deref()))
            .then_some(*node_id)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [node_id] => Ok(*node_id),
        [] => Err(format!(
            "Workbench state control {} was not mounted",
            selector.control_id
        )),
        _ => Err(format!(
            "Workbench state control {} matched {} mounted nodes",
            selector.control_id,
            matches.len()
        )),
    }
}

fn canonical_metadata_instance_path(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> Option<String> {
    metadata
        .instance_path
        .as_ref()
        .and_then(|path| serde_json::to_string(path).ok())
}

fn apply_text_overrides(
    surface: &mut UiSurface,
    overrides: &[WorkbenchTextOverride],
) -> Result<(), String> {
    for override_value in overrides {
        let matches = surface
            .tree
            .nodes
            .iter()
            .filter_map(|(node_id, node)| {
                let metadata = node.template_metadata.as_ref()?;
                (metadata.control_id.as_deref() == Some(override_value.control_id.as_str())
                    && metadata.source_path.as_deref() == Some(override_value.source_path.as_str())
                    && metadata.source_node_id.as_deref()
                        == Some(override_value.source_node_id.as_str())
                    && (override_value.instance_path.is_none()
                        || canonical_metadata_instance_path(metadata).as_deref()
                            == override_value.instance_path.as_deref())
                    && metadata
                        .attributes
                        .get("text")
                        .and_then(toml::Value::as_str)
                        .is_some())
                .then_some(*node_id)
            })
            .collect::<Vec<_>>();
        let node_id = match matches.as_slice() {
            [node_id] => *node_id,
            [] => {
                return Err(format!(
                    "Workbench text override {}#{} was not mounted",
                    override_value.source_path, override_value.source_node_id
                ));
            }
            _ => {
                return Err(format!(
                    "Workbench text override {}#{} matched {} mounted nodes",
                    override_value.source_path,
                    override_value.source_node_id,
                    matches.len()
                ));
            }
        };
        set_property(
            surface,
            node_id,
            "text",
            UiValue::String(override_value.value.clone()),
            "text override",
        )?;
    }
    Ok(())
}

fn apply_with_target(
    surface: &mut UiSurface,
    state: &str,
    target: Option<UiNodeId>,
) -> Result<(), String> {
    if state == "default" || state == "scroll-before" || state == "scroll-after" {
        return Ok(());
    }
    let node_ids = match target {
        Some(node_id) if surface.tree.node(node_id).is_some() => vec![node_id],
        Some(_) => return Err("Workbench state target disappeared before projection".into()),
        None => surface.tree.nodes.keys().copied().collect::<Vec<_>>(),
    };
    if node_ids.is_empty() {
        return Err("Editor state review requires a mounted consumer".into());
    }
    let mut tab_groups = if matches!(
        state,
        "hover" | "pressed" | "focused" | "disabled" | "selected"
    ) {
        single_select_tab_groups(surface)?
    } else {
        Vec::new()
    };
    if let Some(target) = target {
        tab_groups.retain(|group| group.root == target || group.tabs.contains(&target));
    }
    let mut tab_group_nodes = HashSet::new();
    for group in &tab_groups {
        let mut pending = vec![group.root];
        while let Some(node_id) = pending.pop() {
            if !tab_group_nodes.insert(node_id) {
                continue;
            }
            pending.extend(
                surface
                    .tree
                    .node(node_id)
                    .ok_or("Editor tab group descendant disappeared")?
                    .children
                    .iter()
                    .copied(),
            );
        }
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
            for group in &tab_groups {
                let target_index = target
                    .and_then(|selected| group.tabs.iter().position(|tab_id| *tab_id == selected))
                    .unwrap_or_else(|| (group.selected_index + 1) % group.tabs.len());
                let selected = if state == "selected" {
                    target_index
                } else {
                    group.selected_index
                };
                for (index, tab_id) in group.tabs.iter().copied().enumerate() {
                    set_property(
                        surface,
                        tab_id,
                        "selected",
                        UiValue::Bool(index == selected),
                        state,
                    )?;
                }
                if state == "selected" {
                    let text = surface
                        .tree
                        .node(group.tabs[target_index])
                        .and_then(|node| node.template_metadata.as_ref())
                        .and_then(|metadata| metadata.attributes.get("text"))
                        .and_then(toml::Value::as_str)
                        .filter(|text| !text.is_empty())
                        .ok_or("Single-select Tabs review needs a named target tab")?
                        .to_owned();
                    set_property(surface, group.root, "value", UiValue::String(text), state)?;
                    set_property(
                        surface,
                        group.root,
                        "selected_index",
                        UiValue::Float(target_index as f64),
                        state,
                    )?;
                } else {
                    set_property(
                        surface,
                        group.tabs[target_index],
                        property,
                        UiValue::Bool(true),
                        state,
                    )?;
                    if state == "focused" {
                        set_property(
                            surface,
                            group.tabs[target_index],
                            "focus_visible",
                            UiValue::Bool(true),
                            state,
                        )?;
                    }
                }
            }
            for node_id in node_ids {
                if tab_group_nodes.contains(&node_id) {
                    continue;
                }
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

struct SingleSelectTabGroup {
    root: UiNodeId,
    tabs: Vec<UiNodeId>,
    selected_index: usize,
}

fn single_select_tab_groups(surface: &UiSurface) -> Result<Vec<SingleSelectTabGroup>, String> {
    let mut groups = Vec::new();
    for (root, node) in &surface.tree.nodes {
        let Some(metadata) = node.template_metadata.as_ref() else {
            continue;
        };
        if metadata.component != "Tabs"
            || metadata
                .attributes
                .get("selection_state")
                .and_then(toml::Value::as_str)
                != Some("single")
        {
            continue;
        }
        let mut tabs = Vec::new();
        let mut pending = node.children.iter().rev().copied().collect::<Vec<_>>();
        while let Some(child_id) = pending.pop() {
            let child = surface
                .tree
                .node(child_id)
                .ok_or("Editor tab group child disappeared")?;
            if child
                .template_metadata
                .as_ref()
                .map(|item| item.component.as_str())
                == Some("Tab")
            {
                tabs.push(child_id);
            } else {
                pending.extend(child.children.iter().rev().copied());
            }
        }
        if tabs.len() < 2 {
            continue;
        }
        let selected_index = metadata
            .attributes
            .get("selected_index")
            .and_then(|value| {
                value
                    .as_float()
                    .or_else(|| value.as_integer().map(|value| value as f64))
            })
            .ok_or("Single-select Tabs review requires an authored selected_index")?;
        if !selected_index.is_finite()
            || selected_index < 0.0
            || selected_index.fract() != 0.0
            || selected_index >= tabs.len() as f64
        {
            return Err("Single-select Tabs review has an invalid selected_index".into());
        }
        groups.push(SingleSelectTabGroup {
            root: *root,
            tabs,
            selected_index: selected_index as usize,
        });
    }
    Ok(groups)
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
            "Editor {state} state rejected {property}: {:?}",
            report.message
        ));
    }
    let node = surface
        .tree
        .node_mut(node_id)
        .ok_or("Editor review node disappeared while applying state")?;
    let metadata = node
        .template_metadata
        .as_mut()
        .ok_or("Editor review node has no template metadata")?;
    metadata.attributes.insert(property.into(), value.to_toml());
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeComponent {
    Dialog,
    ConfirmDialog,
    CommandPalette,
    NotificationCenter,
    Popup,
    Popover,
    Modal,
    ContextMenu,
    ContextActionMenu,
    Dropdown,
    DropdownPopup,
    Select,
    DragOverlay,
    WorkbenchToast,
    AgentPlan,
    ToolCalls,
    AgentApproval,
    AiUsage,
    DataGrid,
    TreeView,
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
            "popup" => NativeComponent::Popup,
            "popover" => NativeComponent::Popover,
            "modal" => NativeComponent::Modal,
            "contextmenu" => NativeComponent::ContextMenu,
            "contextactionmenu" => NativeComponent::ContextActionMenu,
            "dropdown" => NativeComponent::Dropdown,
            "dropdownpopup" => NativeComponent::DropdownPopup,
            "select" => NativeComponent::Select,
            "dragoverlay" => NativeComponent::DragOverlay,
            "workbenchtoast" | "toast" => NativeComponent::WorkbenchToast,
            "agentplan" | "muixagentplan" => NativeComponent::AgentPlan,
            "toolcalls" | "muixtoolcalls" => NativeComponent::ToolCalls,
            "agentapproval" | "muixagentapproval" => NativeComponent::AgentApproval,
            "aiusage" | "muixaiusage" => NativeComponent::AiUsage,
            "datagrid" | "muixdatagrid" => NativeComponent::DataGrid,
            "treeview" | "materialtreeview" | "muixtreeview" => NativeComponent::TreeView,
            _ => continue,
        };
        return Some(component);
    }
    None
}

fn native_state_supported(component: NativeComponent, state: &str) -> bool {
    match component {
        NativeComponent::Dialog | NativeComponent::ConfirmDialog => {
            matches!(state, "open" | "closed" | "focused" | "focus-return")
        }
        NativeComponent::CommandPalette => {
            matches!(state, "open" | "focused" | "empty" | "focus-return")
        }
        NativeComponent::NotificationCenter => {
            matches!(state, "open" | "selected" | "empty" | "focus-return")
        }
        NativeComponent::Popup
        | NativeComponent::Popover
        | NativeComponent::Modal
        | NativeComponent::ContextMenu
        | NativeComponent::ContextActionMenu
        | NativeComponent::Dropdown
        | NativeComponent::DropdownPopup
        | NativeComponent::Select => matches!(state, "open" | "closed" | "focus-return"),
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
                } else if is_authored_popup_component(component) {
                    set_authored_popup_flags(surface, node_id, true, state)?;
                } else {
                    set_property(surface, node_id, "open", UiValue::Bool(true), state)?;
                    set_property(surface, node_id, "popup_open", UiValue::Bool(true), state)?;
                }
            }
            "closed" => {
                if is_authored_popup_component(component) {
                    set_authored_popup_flags(surface, node_id, false, state)?;
                } else {
                    set_property(surface, node_id, "open", UiValue::Bool(false), state)?;
                    set_property(surface, node_id, "popup_open", UiValue::Bool(false), state)?;
                }
            }
            "focus-return" => {
                if is_authored_popup_component(component) {
                    set_authored_popup_flags(surface, node_id, false, state)?;
                } else {
                    set_property(surface, node_id, "open", UiValue::Bool(false), state)?;
                    set_property(surface, node_id, "popup_open", UiValue::Bool(false), state)?;
                }
                apply_focus_to_popup_anchor(surface, node_id, state)?;
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
                        set_property(
                            surface,
                            node_id,
                            property,
                            UiValue::Array(Vec::new()),
                            state,
                        )?;
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
                    set_property(
                        surface,
                        node_id,
                        "notifications",
                        UiValue::Array(Vec::new()),
                        state,
                    )?;
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
                    set_property(
                        surface,
                        node_id,
                        "collection_items",
                        UiValue::Array(Vec::new()),
                        state,
                    )?;
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
                }
            }
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
            "Editor state {state} requires a matching painter component"
        ));
    }
    Ok(())
}

fn apply_focus_to_popup_anchor(
    surface: &mut UiSurface,
    popup_node_id: UiNodeId,
    state: &str,
) -> Result<(), String> {
    let (control_id, source_path, instance_path) = {
        let metadata = surface
            .tree
            .node(popup_node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .ok_or("Editor focus-return popup has no retained widget metadata")?;
        let control_id = match &metadata.widget.popup_anchor {
            UiPopupAnchor::Control { control_id } => control_id.clone(),
            UiPopupAnchor::Pointer { owner_property } => metadata
                .attributes
                .get(owner_property)
                .and_then(toml::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned)
                .ok_or("Editor focus-return pointer popup has no resolved owner property")?,
            UiPopupAnchor::None | UiPopupAnchor::Surface => {
                return Err("Editor focus-return popup has no authored invoking control".into());
            }
        };
        let source_path = metadata
            .source_path
            .as_deref()
            .ok_or("Editor focus-return popup has no authored source path")?
            .to_owned();
        let instance_path = metadata
            .instance_path
            .clone()
            .ok_or("Editor focus-return popup has no authored instance path")?;
        (control_id, source_path, instance_path)
    };
    let anchors = surface
        .tree
        .nodes
        .iter()
        .filter_map(|(node_id, node)| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| {
                    metadata.control_id.as_deref() == Some(control_id.as_str())
                        && metadata.source_path.as_deref() == Some(source_path.as_str())
                        && metadata.instance_path.as_ref() == Some(&instance_path)
                })
                .map(|_| *node_id)
        })
        .collect::<Vec<_>>();
    let anchor_node_id = match anchors.as_slice() {
        [node_id] => *node_id,
        [] => {
            return Err(format!(
                "Editor focus-return invoking control {control_id} is not mounted"
            ));
        }
        _ => {
            return Err(format!(
                "Editor focus-return invoking control {control_id} matched {} mounted nodes",
                anchors.len()
            ));
        }
    };
    set_property(
        surface,
        anchor_node_id,
        "focused",
        UiValue::Bool(true),
        state,
    )?;
    set_property(
        surface,
        anchor_node_id,
        "focus_visible",
        UiValue::Bool(true),
        state,
    )
}

fn is_authored_popup_component(component: NativeComponent) -> bool {
    matches!(
        component,
        NativeComponent::Popup
            | NativeComponent::Popover
            | NativeComponent::Modal
            | NativeComponent::ContextMenu
            | NativeComponent::ContextActionMenu
            | NativeComponent::Dropdown
            | NativeComponent::DropdownPopup
            | NativeComponent::Select
    )
}

fn set_authored_popup_flags(
    surface: &mut UiSurface,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    open: bool,
    state: &str,
) -> Result<(), String> {
    let properties = surface
        .tree
        .node(node_id)
        .and_then(|node| node.template_metadata.as_ref())
        .map(|metadata| {
            ["open", "popup_open"]
                .into_iter()
                .filter(|property| metadata.attributes.contains_key(*property))
                .collect::<Vec<_>>()
        })
        .ok_or("Editor popup node has no template metadata")?;
    if properties.is_empty() {
        return Err(format!(
            "Editor {state} popup has no authored open property"
        ));
    }
    for property in properties {
        set_property(surface, node_id, property, UiValue::Bool(open), state)?;
    }
    Ok(())
}

/// Apply the start/end offset of every authored ScrollableBox after the first
/// layout pass. Capture performs one more layout pass before painting.
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

pub(super) fn apply_scroll_position_for_case(
    surface: &mut UiSurface,
    case: &ReviewCase,
) -> Result<bool, String> {
    let state = case.scroll_review_state();
    let end = match state {
        "scroll-before" => false,
        "scroll-after" => true,
        _ => return Ok(false),
    };
    let Some(selector) = workbench_state_selector(&case.data)? else {
        return apply_scroll_position(surface, state);
    };
    let node_id = if let Some(scroll_target) = selector.scroll_target.as_ref() {
        find_workbench_target(surface, scroll_target)?
    } else {
        find_workbench_node(surface, &selector)?
    };
    if !matches!(
        surface.tree.node(node_id).map(|node| &node.container),
        Some(UiContainerKind::ScrollableBox(_))
    ) {
        return Err(format!(
            "Workbench scroll target {} is not an authored ScrollableBox",
            selector.control_id
        ));
    }
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
    surface
        .tree
        .set_scroll_offset(node_id, offset)
        .map_err(|error| error.to_string())
}
