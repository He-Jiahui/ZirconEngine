use std::collections::BTreeMap;

use crate::ui::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, RetainedUiHostComponentKind,
    RetainedUiHostNodeModel, RetainedUiHostProjection, RetainedUiHostValue,
};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use super::{
    export_wizard_panel_template_state, project_export_wizard_panel,
    ExportWizardPanelEntrySeverity, ExportWizardPanelSlotEntry, ExportWizardPanelSlotKind,
    ExportWizardPanelSlotState, ExportWizardPanelTemplateState, ExportWizardPanelViewModel,
    EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID,
};

const PANEL_ENTRY_NODE_PREFIX: &str = "desktop_export_panel_entry.";
const SLOT_ENTRY_PADDING: f32 = 8.0;
const SLOT_ENTRY_HEIGHT: f32 = 22.0;
const SLOT_ENTRY_GAP: f32 = 4.0;

pub fn export_wizard_panel_retained_projection(
    runtime: &EditorUiHostRuntime,
    view_model: &ExportWizardPanelViewModel,
    size: UiSize,
) -> Result<RetainedUiHostProjection, EditorUiHostRuntimeError> {
    let projection = project_export_wizard_panel(runtime)?;
    let mut surface = runtime.build_shared_surface(EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID)?;
    surface.compute_layout(size)?;
    let mut retained =
        runtime.build_retained_host_projection_with_surface(&projection, &surface)?;
    let state = export_wizard_panel_template_state(view_model);
    apply_export_wizard_panel_template_state(&mut retained, &state);
    Ok(retained)
}

pub fn apply_export_wizard_panel_template_state(
    projection: &mut RetainedUiHostProjection,
    state: &ExportWizardPanelTemplateState,
) {
    projection
        .nodes
        .retain(|node| !node.node_id.starts_with(PANEL_ENTRY_NODE_PREFIX));
    apply_control_bindings(projection, state);
    append_slot_entry_nodes(projection, &state.slots);
}

fn apply_control_bindings(
    projection: &mut RetainedUiHostProjection,
    state: &ExportWizardPanelTemplateState,
) {
    for control in &state.control_bindings {
        let Some(node) = projection_node_by_control_id_mut(projection, control.control_id) else {
            continue;
        };
        node.disabled = !control.enabled;
        node.properties.insert(
            "enabled".to_string(),
            RetainedUiHostValue::Bool(control.enabled),
        );
        node.properties.insert(
            "disabled".to_string(),
            RetainedUiHostValue::Bool(!control.enabled),
        );
        if control.enabled {
            if node.validation_level.as_deref() == Some("disabled") {
                node.validation_level = None;
            }
        } else {
            node.validation_level = Some("disabled".to_string());
        }
    }
}

fn append_slot_entry_nodes(
    projection: &mut RetainedUiHostProjection,
    slots: &[ExportWizardPanelSlotState],
) {
    let mut nodes = Vec::new();
    for slot in slots {
        let Some(anchor) = projection
            .nodes
            .iter()
            .find(|node| node.control_id.as_deref() == Some(slot.control_id))
        else {
            continue;
        };
        let parent_id = anchor.node_id.clone();
        let anchor_frame = anchor.frame;
        let z_index = anchor.z_index + 1;
        nodes.extend(slot.entries.iter().enumerate().map(|(index, entry)| {
            slot_entry_node(slot, entry, index, &parent_id, anchor_frame, z_index)
        }));
    }
    projection.nodes.extend(nodes);
}

fn slot_entry_node(
    slot: &ExportWizardPanelSlotState,
    entry: &ExportWizardPanelSlotEntry,
    index: usize,
    parent_id: &str,
    anchor_frame: UiFrame,
    z_index: i32,
) -> RetainedUiHostNodeModel {
    let text = slot_entry_text(entry);
    let mut properties = BTreeMap::new();
    insert_string(&mut properties, "text", text.clone());
    insert_string(&mut properties, "slot_kind", slot_kind_label(slot.kind));
    insert_string(&mut properties, "entry_key", entry.key.clone());
    insert_string(&mut properties, "label", entry.label.clone());
    insert_string(&mut properties, "detail", entry.detail.clone());
    insert_string(&mut properties, "severity", severity_label(entry.severity));
    if let Some(stage) = entry.stage {
        insert_string(&mut properties, "stage", stage.cli_id());
    }

    RetainedUiHostNodeModel {
        node_id: format!(
            "{PANEL_ENTRY_NODE_PREFIX}{}.{}",
            slot.control_id,
            stable_entry_key(&entry.key)
        ),
        parent_id: Some(parent_id.to_string()),
        kind: RetainedUiHostComponentKind::Label,
        component: "Label".to_string(),
        control_id: Some(format!("{}.{}", slot.control_id, entry.key)),
        frame: UiFrame::new(
            anchor_frame.x + SLOT_ENTRY_PADDING,
            anchor_frame.y
                + SLOT_ENTRY_PADDING
                + index as f32 * (SLOT_ENTRY_HEIGHT + SLOT_ENTRY_GAP),
            (anchor_frame.width - SLOT_ENTRY_PADDING * 2.0).max(0.0),
            SLOT_ENTRY_HEIGHT,
        ),
        clip_frame: None,
        z_index,
        text: Some(text),
        icon: None,
        component_role: Some("label".to_string()),
        value_text: Some(entry.detail.clone()),
        validation_level: Some(severity_label(entry.severity).to_ascii_lowercase()),
        validation_message: if entry.detail.is_empty() {
            None
        } else {
            Some(entry.detail.clone())
        },
        popup_open: false,
        has_popup_anchor: false,
        popup_anchor_x: 0.0,
        popup_anchor_y: 0.0,
        selection_state: None,
        options_text: None,
        options: Vec::new(),
        collection_items: Vec::new(),
        menu_items: Vec::new(),
        accepted_drag_payloads: Vec::new(),
        drop_source_summary: None,
        checked: false,
        expanded: false,
        focused: false,
        hovered: false,
        pressed: false,
        dragging: false,
        drop_hovered: false,
        active_drag_target: false,
        disabled: false,
        properties,
        style_tokens: BTreeMap::new(),
        routes: Vec::new(),
    }
}

fn projection_node_by_control_id_mut<'a>(
    projection: &'a mut RetainedUiHostProjection,
    control_id: &str,
) -> Option<&'a mut RetainedUiHostNodeModel> {
    projection
        .nodes
        .iter_mut()
        .find(|node| node.control_id.as_deref() == Some(control_id))
}

fn insert_string(
    properties: &mut BTreeMap<String, RetainedUiHostValue>,
    key: &str,
    value: impl Into<String>,
) {
    properties.insert(key.to_string(), RetainedUiHostValue::String(value.into()));
}

fn slot_entry_text(entry: &ExportWizardPanelSlotEntry) -> String {
    if entry.detail.is_empty() {
        entry.label.clone()
    } else {
        format!("{}: {}", entry.label, entry.detail)
    }
}

fn stable_entry_key(value: &str) -> String {
    let key = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if key.is_empty() {
        "entry".to_string()
    } else {
        key
    }
}

fn slot_kind_label(kind: ExportWizardPanelSlotKind) -> &'static str {
    match kind {
        ExportWizardPanelSlotKind::MissingInputs => "MissingInputs",
        ExportWizardPanelSlotKind::StageRows => "StageRows",
        ExportWizardPanelSlotKind::TerminalOutput => "TerminalOutput",
        ExportWizardPanelSlotKind::ArtifactPaths => "ArtifactPaths",
        ExportWizardPanelSlotKind::ReportBody => "ReportBody",
    }
}

fn severity_label(severity: ExportWizardPanelEntrySeverity) -> &'static str {
    match severity {
        ExportWizardPanelEntrySeverity::Neutral => "Neutral",
        ExportWizardPanelEntrySeverity::Info => "Info",
        ExportWizardPanelEntrySeverity::Success => "Success",
        ExportWizardPanelEntrySeverity::Warning => "Warning",
        ExportWizardPanelEntrySeverity::Danger => "Danger",
    }
}
