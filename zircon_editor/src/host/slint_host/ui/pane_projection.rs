use super::model_rc::model_rc;
use super::viewport_chrome::{blank_viewport_chrome, scene_viewport_chrome};
use super::workbench_tabs::drawer_slot_key;
use super::*;

pub(super) fn side_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    slots: &[ActivityDrawerSlot],
    ui_asset_panes: &std::collections::BTreeMap<String, crate::UiAssetEditorPanePresentation>,
) -> PaneData {
    let stack = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .find(|stack| {
            stack.mode != crate::ActivityDrawerMode::Collapsed
                && stack.active_tab.is_some()
                && !stack.tabs.is_empty()
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| stack.active_tab.is_some() && !stack.tabs.is_empty())
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| !stack.tabs.is_empty())
        });

    let Some(stack) = stack else {
        return blank_pane();
    };
    let tab = stack
        .tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| stack.tabs.first());
    let Some(tab) = tab else {
        return blank_pane();
    };
    pane_from_tab(
        &tab.instance_id.0,
        drawer_slot_key(stack.slot),
        &tab.title,
        &tab.icon_key,
        tab.content_kind,
        tab.empty_state.as_ref(),
        find_tab_snapshot(chrome, &tab.instance_id.0),
        chrome,
        ui_asset_panes.get(&tab.instance_id.0),
    )
}

pub(super) fn document_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    ui_asset_panes: &std::collections::BTreeMap<String, crate::UiAssetEditorPanePresentation>,
) -> PaneData {
    let tab = model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first());
    let Some(tab) = tab else {
        return blank_pane();
    };
    pane_from_tab(
        &tab.instance_id.0,
        "",
        &tab.title,
        &tab.icon_key,
        tab.content_kind,
        tab.empty_state.as_ref(),
        find_tab_snapshot(chrome, &tab.instance_id.0),
        chrome,
        ui_asset_panes.get(&tab.instance_id.0),
    )
}

pub(super) fn pane_from_tab(
    instance_id: &str,
    slot: &str,
    title: &str,
    icon_key: &str,
    kind: ViewContentKind,
    empty_state: Option<&PaneEmptyStateModel>,
    snapshot: Option<&ViewTabSnapshot>,
    chrome: &EditorChromeSnapshot,
    ui_asset_pane: Option<&crate::UiAssetEditorPanePresentation>,
) -> PaneData {
    let (subtitle, info, show_toolbar) = pane_metadata(kind, snapshot, chrome);
    let viewport = match kind {
        ViewContentKind::Scene => scene_viewport_chrome(&chrome.scene_viewport_settings),
        _ => blank_viewport_chrome(),
    };
    let (
        empty_title,
        empty_body,
        primary_action_label,
        primary_action_id,
        secondary_action_label,
        secondary_action_id,
        secondary_hint,
    ) = empty_state
        .map(|state| {
            (
                state.title.clone(),
                state.body.clone(),
                state
                    .primary_action
                    .as_ref()
                    .map(|action| action.label.clone())
                    .unwrap_or_default(),
                state
                    .primary_action
                    .as_ref()
                    .map(action_id_from_model)
                    .unwrap_or_default(),
                state
                    .secondary_action
                    .as_ref()
                    .map(|action| action.label.clone())
                    .unwrap_or_default(),
                state
                    .secondary_action
                    .as_ref()
                    .map(action_id_from_model)
                    .unwrap_or_default(),
                state.secondary_hint.clone().unwrap_or_default(),
            )
        })
        .unwrap_or_else(|| {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            )
        });
    let ui_asset_pane = ui_asset_pane.cloned().unwrap_or_default();

    PaneData {
        id: instance_id.into(),
        slot: slot.into(),
        kind: pane_kind_key(kind).into(),
        title: title.into(),
        icon_key: icon_key.into(),
        subtitle: subtitle.into(),
        info: info.into(),
        show_empty: empty_state.is_some(),
        empty_title: SharedString::from(empty_title),
        empty_body: SharedString::from(empty_body),
        primary_action_label: SharedString::from(primary_action_label),
        primary_action_id: SharedString::from(primary_action_id),
        secondary_action_label: SharedString::from(secondary_action_label),
        secondary_action_id: SharedString::from(secondary_action_id),
        secondary_hint: SharedString::from(secondary_hint),
        show_toolbar,
        viewport,
        ui_asset_mode: ui_asset_pane.mode.into(),
        ui_asset_status: ui_asset_pane.last_error.into(),
        ui_asset_selection: ui_asset_pane.selection_summary.into(),
        ui_asset_source_text: ui_asset_pane.source_text.into(),
        ui_asset_preview_summary: ui_asset_pane.preview_summary.into(),
        ui_asset_can_save: ui_asset_pane.can_save,
        ui_asset_can_undo: ui_asset_pane.can_undo,
        ui_asset_can_redo: ui_asset_pane.can_redo,
        ui_asset_can_create_rule: ui_asset_pane.can_create_rule,
        ui_asset_can_extract_rule: ui_asset_pane.can_extract_rule,
        ui_asset_preview_available: ui_asset_pane.preview_available,
        ui_asset_style_state_hover: ui_asset_pane.style_state_hover,
        ui_asset_style_state_focus: ui_asset_pane.style_state_focus,
        ui_asset_style_state_pressed: ui_asset_pane.style_state_pressed,
        ui_asset_style_state_disabled: ui_asset_pane.style_state_disabled,
        ui_asset_style_state_selected: ui_asset_pane.style_state_selected,
        ui_asset_style_class_items: model_rc(
            ui_asset_pane
                .style_class_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_style_rule_items: model_rc(
            ui_asset_pane
                .style_rule_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_style_rule_selected_index: ui_asset_pane.style_rule_selected_index,
        ui_asset_style_selected_rule_selector: ui_asset_pane.style_selected_rule_selector.into(),
        ui_asset_style_can_edit_rule: ui_asset_pane.style_can_edit_rule,
        ui_asset_style_can_delete_rule: ui_asset_pane.style_can_delete_rule,
        ui_asset_style_matched_rule_items: model_rc(
            ui_asset_pane
                .style_matched_rule_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_style_matched_rule_selected_index: ui_asset_pane.style_matched_rule_selected_index,
        ui_asset_style_selected_matched_rule_origin: ui_asset_pane
            .style_selected_matched_rule_origin
            .into(),
        ui_asset_style_selected_matched_rule_selector: ui_asset_pane
            .style_selected_matched_rule_selector
            .into(),
        ui_asset_style_selected_matched_rule_specificity: ui_asset_pane
            .style_selected_matched_rule_specificity,
        ui_asset_style_selected_matched_rule_source_order: ui_asset_pane
            .style_selected_matched_rule_source_order,
        ui_asset_style_selected_matched_rule_declaration_items: model_rc(
            ui_asset_pane
                .style_selected_matched_rule_declaration_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_style_rule_declaration_items: model_rc(
            ui_asset_pane
                .style_rule_declaration_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_style_rule_declaration_selected_index: ui_asset_pane
            .style_rule_declaration_selected_index,
        ui_asset_style_selected_rule_declaration_path: ui_asset_pane
            .style_selected_rule_declaration_path
            .into(),
        ui_asset_style_selected_rule_declaration_value: ui_asset_pane
            .style_selected_rule_declaration_value
            .into(),
        ui_asset_style_can_edit_rule_declaration: ui_asset_pane.style_can_edit_rule_declaration,
        ui_asset_style_can_delete_rule_declaration: ui_asset_pane.style_can_delete_rule_declaration,
        ui_asset_style_token_items: model_rc(
            ui_asset_pane
                .style_token_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_style_token_selected_index: ui_asset_pane.style_token_selected_index,
        ui_asset_style_selected_token_name: ui_asset_pane.style_selected_token_name.into(),
        ui_asset_style_selected_token_value: ui_asset_pane.style_selected_token_value.into(),
        ui_asset_style_can_edit_token: ui_asset_pane.style_can_edit_token,
        ui_asset_style_can_delete_token: ui_asset_pane.style_can_delete_token,
        ui_asset_inspector_selected_node_id: ui_asset_pane.inspector_selected_node_id.into(),
        ui_asset_inspector_parent_node_id: ui_asset_pane.inspector_parent_node_id.into(),
        ui_asset_inspector_mount: ui_asset_pane.inspector_mount.into(),
        ui_asset_inspector_slot_padding: ui_asset_pane.inspector_slot_padding.into(),
        ui_asset_inspector_slot_width_preferred: ui_asset_pane
            .inspector_slot_width_preferred
            .into(),
        ui_asset_inspector_slot_height_preferred: ui_asset_pane
            .inspector_slot_height_preferred
            .into(),
        ui_asset_inspector_layout_width_preferred: ui_asset_pane
            .inspector_layout_width_preferred
            .into(),
        ui_asset_inspector_layout_height_preferred: ui_asset_pane
            .inspector_layout_height_preferred
            .into(),
        ui_asset_inspector_binding_items: model_rc(
            ui_asset_pane
                .inspector_binding_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_inspector_binding_selected_index: ui_asset_pane.inspector_binding_selected_index,
        ui_asset_inspector_binding_id: ui_asset_pane.inspector_binding_id.into(),
        ui_asset_inspector_binding_event: ui_asset_pane.inspector_binding_event.into(),
        ui_asset_inspector_binding_route: ui_asset_pane.inspector_binding_route.into(),
        ui_asset_inspector_can_edit_binding: ui_asset_pane.inspector_can_edit_binding,
        ui_asset_inspector_can_delete_binding: ui_asset_pane.inspector_can_delete_binding,
        ui_asset_inspector_widget_kind: ui_asset_pane.inspector_widget_kind.into(),
        ui_asset_inspector_widget_label: ui_asset_pane.inspector_widget_label.into(),
        ui_asset_inspector_control_id: ui_asset_pane.inspector_control_id.into(),
        ui_asset_inspector_text_prop: ui_asset_pane.inspector_text_prop.into(),
        ui_asset_inspector_can_edit_control_id: ui_asset_pane.inspector_can_edit_control_id,
        ui_asset_inspector_can_edit_text_prop: ui_asset_pane.inspector_can_edit_text_prop,
        ui_asset_palette_items: model_rc(
            ui_asset_pane
                .palette_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_hierarchy_items: model_rc(
            ui_asset_pane
                .hierarchy_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_inspector_items: model_rc(
            ui_asset_pane
                .inspector_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_stylesheet_items: model_rc(
            ui_asset_pane
                .stylesheet_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
        ui_asset_preview_items: model_rc(
            ui_asset_pane
                .preview_items
                .into_iter()
                .map(SharedString::from)
                .collect(),
        ),
    }
}

pub(super) fn find_tab_snapshot<'a>(
    chrome: &'a EditorChromeSnapshot,
    instance_id: &str,
) -> Option<&'a ViewTabSnapshot> {
    for drawer in chrome.workbench.drawers.values() {
        if let Some(tab) = drawer
            .tabs
            .iter()
            .find(|tab| tab.instance_id.0.as_str() == instance_id)
        {
            return Some(tab);
        }
    }

    for page in &chrome.workbench.main_pages {
        match page {
            MainPageSnapshot::Workbench { workspace, .. } => {
                if let Some(tab) = find_in_workspace(workspace, instance_id) {
                    return Some(tab);
                }
            }
            MainPageSnapshot::Exclusive { view, .. } if view.instance_id.0 == instance_id => {
                return Some(view);
            }
            MainPageSnapshot::Exclusive { .. } => {}
        }
    }

    for window in &chrome.workbench.floating_windows {
        if let Some(tab) = find_in_workspace(&window.workspace, instance_id) {
            return Some(tab);
        }
    }

    None
}

pub(super) fn blank_pane() -> PaneData {
    PaneData {
        id: SharedString::default(),
        slot: SharedString::default(),
        kind: "Placeholder".into(),
        title: SharedString::default(),
        icon_key: SharedString::default(),
        subtitle: SharedString::default(),
        info: SharedString::default(),
        show_empty: false,
        empty_title: SharedString::default(),
        empty_body: SharedString::default(),
        primary_action_label: SharedString::default(),
        primary_action_id: SharedString::default(),
        secondary_action_label: SharedString::default(),
        secondary_action_id: SharedString::default(),
        secondary_hint: SharedString::default(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        ui_asset_mode: SharedString::default(),
        ui_asset_status: SharedString::default(),
        ui_asset_selection: SharedString::default(),
        ui_asset_source_text: SharedString::default(),
        ui_asset_preview_summary: SharedString::default(),
        ui_asset_can_save: false,
        ui_asset_can_undo: false,
        ui_asset_can_redo: false,
        ui_asset_can_create_rule: false,
        ui_asset_can_extract_rule: false,
        ui_asset_preview_available: false,
        ui_asset_style_state_hover: false,
        ui_asset_style_state_focus: false,
        ui_asset_style_state_pressed: false,
        ui_asset_style_state_disabled: false,
        ui_asset_style_state_selected: false,
        ui_asset_style_class_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_style_rule_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_style_rule_selected_index: -1,
        ui_asset_style_selected_rule_selector: SharedString::default(),
        ui_asset_style_can_edit_rule: false,
        ui_asset_style_can_delete_rule: false,
        ui_asset_style_matched_rule_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_style_matched_rule_selected_index: -1,
        ui_asset_style_selected_matched_rule_origin: SharedString::default(),
        ui_asset_style_selected_matched_rule_selector: SharedString::default(),
        ui_asset_style_selected_matched_rule_specificity: -1,
        ui_asset_style_selected_matched_rule_source_order: -1,
        ui_asset_style_selected_matched_rule_declaration_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_style_rule_declaration_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_style_rule_declaration_selected_index: -1,
        ui_asset_style_selected_rule_declaration_path: SharedString::default(),
        ui_asset_style_selected_rule_declaration_value: SharedString::default(),
        ui_asset_style_can_edit_rule_declaration: false,
        ui_asset_style_can_delete_rule_declaration: false,
        ui_asset_style_token_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_style_token_selected_index: -1,
        ui_asset_style_selected_token_name: SharedString::default(),
        ui_asset_style_selected_token_value: SharedString::default(),
        ui_asset_style_can_edit_token: false,
        ui_asset_style_can_delete_token: false,
        ui_asset_inspector_selected_node_id: SharedString::default(),
        ui_asset_inspector_parent_node_id: SharedString::default(),
        ui_asset_inspector_mount: SharedString::default(),
        ui_asset_inspector_slot_padding: SharedString::default(),
        ui_asset_inspector_slot_width_preferred: SharedString::default(),
        ui_asset_inspector_slot_height_preferred: SharedString::default(),
        ui_asset_inspector_layout_width_preferred: SharedString::default(),
        ui_asset_inspector_layout_height_preferred: SharedString::default(),
        ui_asset_inspector_binding_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_inspector_binding_selected_index: -1,
        ui_asset_inspector_binding_id: SharedString::default(),
        ui_asset_inspector_binding_event: SharedString::default(),
        ui_asset_inspector_binding_route: SharedString::default(),
        ui_asset_inspector_can_edit_binding: false,
        ui_asset_inspector_can_delete_binding: false,
        ui_asset_inspector_widget_kind: SharedString::default(),
        ui_asset_inspector_widget_label: SharedString::default(),
        ui_asset_inspector_control_id: SharedString::default(),
        ui_asset_inspector_text_prop: SharedString::default(),
        ui_asset_inspector_can_edit_control_id: false,
        ui_asset_inspector_can_edit_text_prop: false,
        ui_asset_palette_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_hierarchy_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_inspector_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_stylesheet_items: model_rc(Vec::<SharedString>::new()),
        ui_asset_preview_items: model_rc(Vec::<SharedString>::new()),
    }
}

fn pane_metadata(
    kind: ViewContentKind,
    snapshot: Option<&ViewTabSnapshot>,
    chrome: &EditorChromeSnapshot,
) -> (String, String, bool) {
    match kind {
        ViewContentKind::Welcome => (
            chrome.welcome.subtitle.clone(),
            chrome.welcome.status_message.clone(),
            false,
        ),
        ViewContentKind::Project => (
            if chrome.project_overview.project_name.is_empty() {
                chrome.project_path.clone()
            } else {
                chrome.project_overview.project_name.clone()
            },
            format!(
                "{} folders • {} assets",
                chrome.project_overview.folder_count, chrome.project_overview.asset_count
            ),
            false,
        ),
        ViewContentKind::Assets => (
            chrome
                .asset_activity
                .selected_folder_id
                .clone()
                .unwrap_or_else(|| "res://".to_string()),
            format!(
                "{} folders • {} assets",
                chrome.asset_activity.visible_folders.len(),
                chrome.asset_activity.visible_assets.len()
            ),
            false,
        ),
        ViewContentKind::Hierarchy => (
            format!("{} nodes", chrome.scene_entries.len()),
            "Hierarchy selection drives Scene and Inspector".to_string(),
            false,
        ),
        ViewContentKind::Inspector => (
            "Selection Inspector".to_string(),
            chrome
                .inspector
                .as_ref()
                .map(|inspector| format!("Node {}", inspector.id))
                .unwrap_or_default(),
            false,
        ),
        ViewContentKind::Scene => (
            format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            String::new(),
            true,
        ),
        ViewContentKind::Game => (
            format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            String::new(),
            false,
        ),
        ViewContentKind::Console => ("Task Output".to_string(), chrome.status_line.clone(), false),
        ViewContentKind::PrefabEditor => (
            payload_path(snapshot).unwrap_or_else(|| "Prefab Workspace".to_string()),
            "Prefab editor host slot is ready. Asset-specific tooling is still placeholder.".into(),
            false,
        ),
        ViewContentKind::AssetBrowser => (
            chrome.asset_browser.project_name.clone(),
            format!(
                "{} folders • {} assets",
                chrome.asset_browser.visible_folders.len(),
                chrome.asset_browser.visible_assets.len()
            ),
            false,
        ),
        ViewContentKind::UiAssetEditor => (
            payload_path(snapshot).unwrap_or_else(|| "UI Asset Editor".to_string()),
            "Live UI asset preview and source editing session".to_string(),
            false,
        ),
        ViewContentKind::Placeholder => (
            "Missing View".to_string(),
            "This pane was restored from layout state but the descriptor is unavailable.".into(),
            false,
        ),
    }
}

fn payload_path(snapshot: Option<&ViewTabSnapshot>) -> Option<String> {
    snapshot
        .and_then(|view| {
            view.serializable_payload
                .get("path")
                .or_else(|| view.serializable_payload.get("asset_id"))
        })
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn action_id_from_model(action: &PaneActionModel) -> String {
    match action.binding.as_ref().map(EditorUiBinding::payload) {
        Some(EditorUiBindingPayload::MenuAction { action_id }) => action_id.clone(),
        _ => String::new(),
    }
}

fn pane_kind_key(kind: ViewContentKind) -> &'static str {
    match kind {
        ViewContentKind::Welcome => "Welcome",
        ViewContentKind::Project => "Project",
        ViewContentKind::Hierarchy => "Hierarchy",
        ViewContentKind::Inspector => "Inspector",
        ViewContentKind::Scene => "Scene",
        ViewContentKind::Game => "Game",
        ViewContentKind::Assets => "Assets",
        ViewContentKind::Console => "Console",
        ViewContentKind::PrefabEditor => "PrefabEditor",
        ViewContentKind::AssetBrowser => "AssetBrowser",
        ViewContentKind::UiAssetEditor => "UiAssetEditor",
        ViewContentKind::Placeholder => "Placeholder",
    }
}

fn find_in_workspace<'a>(
    workspace: &'a crate::DocumentWorkspaceSnapshot,
    instance_id: &str,
) -> Option<&'a ViewTabSnapshot> {
    match workspace {
        crate::DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            find_in_workspace(first, instance_id).or_else(|| find_in_workspace(second, instance_id))
        }
        crate::DocumentWorkspaceSnapshot::Tabs { tabs, .. } => tabs
            .iter()
            .find(|tab| tab.instance_id.0.as_str() == instance_id),
    }
}
