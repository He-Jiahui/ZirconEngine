use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{
    blank_viewport_chrome, scene_viewport_chrome, SceneViewportChromeData,
};
use crate::ui::widgets::common::drawer_slot_key;
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

pub(super) fn side_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    slots: &[ActivityDrawerSlot],
    ui_asset_panes: &std::collections::BTreeMap<
        String,
        crate::ui::asset_editor::UiAssetEditorPanePresentation,
    >,
    animation_panes: &std::collections::BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
    runtime_diagnostics: Option<&RuntimeDiagnosticsSnapshot>,
    module_plugins: &ModulePluginsPaneViewData,
) -> PaneData {
    let stack = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .find(|stack| {
            stack.mode != crate::ui::workbench::layout::ActivityDrawerMode::Collapsed
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
        animation_panes.get(&tab.instance_id.0),
        runtime_diagnostics,
        module_plugins,
    )
}

pub(crate) fn document_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    ui_asset_panes: &std::collections::BTreeMap<
        String,
        crate::ui::asset_editor::UiAssetEditorPanePresentation,
    >,
    animation_panes: &std::collections::BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
    runtime_diagnostics: Option<&RuntimeDiagnosticsSnapshot>,
    module_plugins: &ModulePluginsPaneViewData,
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
        animation_panes.get(&tab.instance_id.0),
        runtime_diagnostics,
        module_plugins,
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
    ui_asset_pane: Option<&crate::ui::asset_editor::UiAssetEditorPanePresentation>,
    animation_pane: Option<&crate::ui::animation_editor::AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&RuntimeDiagnosticsSnapshot>,
    module_plugins: &ModulePluginsPaneViewData,
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
    let animation_pane = animation_pane.cloned().unwrap_or_default();
    let pane_presentation = build_compat_pane_presentation(
        title,
        icon_key,
        &subtitle,
        &info,
        empty_state,
        show_toolbar,
        &viewport,
        snapshot,
        chrome,
        Some(&animation_pane),
        runtime_diagnostics,
    );

    PaneData {
        id: instance_id.into(),
        slot: slot.into(),
        kind: pane_kind_key(kind).into(),
        title: title.into(),
        icon_key: icon_key.into(),
        subtitle: subtitle.into(),
        info: info.clone().into(),
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
        body_compat: PaneBodyCompatData {
            hierarchy: hierarchy_pane_data(chrome),
            inspector: inspector_pane_data(chrome, &info),
            console: console_pane_data(chrome),
            assets_activity: AssetsActivityPaneViewData::default(),
            asset_browser: AssetBrowserPaneViewData::default(),
            project_overview: ProjectOverviewPaneViewData::default(),
            module_plugins: module_plugins.clone(),
            ui_asset: ui_asset_pane,
            animation: animation_pane_data(animation_pane),
        },
        pane_presentation,
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

fn shared_string_list(items: Vec<String>) -> slint::ModelRc<slint::SharedString> {
    model_rc(items.into_iter().map(SharedString::from).collect())
}

fn animation_pane_data(
    animation_pane: crate::ui::animation_editor::AnimationEditorPanePresentation,
) -> AnimationEditorPaneViewData {
    AnimationEditorPaneViewData {
        nodes: Default::default(),
        mode: animation_pane.mode.into(),
        asset_path: animation_pane.asset_path.into(),
        status: animation_pane.status.into(),
        selection: animation_pane.selection_summary.into(),
        current_frame: animation_pane.current_frame as i32,
        timeline_start_frame: animation_pane.timeline_start_frame as i32,
        timeline_end_frame: animation_pane.timeline_end_frame as i32,
        playback_label: animation_pane.playback_label.into(),
        track_items: shared_string_list(animation_pane.track_items),
        parameter_items: shared_string_list(animation_pane.parameter_items),
        node_items: shared_string_list(animation_pane.node_items),
        state_items: shared_string_list(animation_pane.state_items),
        transition_items: shared_string_list(animation_pane.transition_items),
    }
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
        body_compat: PaneBodyCompatData {
            hierarchy: HierarchyPaneViewData::default(),
            inspector: InspectorPaneViewData::default(),
            console: ConsolePaneViewData::default(),
            assets_activity: AssetsActivityPaneViewData::default(),
            asset_browser: AssetBrowserPaneViewData::default(),
            project_overview: ProjectOverviewPaneViewData::default(),
            module_plugins: ModulePluginsPaneViewData::default(),
            ui_asset: crate::ui::asset_editor::UiAssetEditorPanePresentation::default(),
            animation: animation_pane_data(
                crate::ui::animation_editor::AnimationEditorPanePresentation::default(),
            ),
        },
        pane_presentation: None,
    }
}

fn build_compat_pane_presentation(
    title: &str,
    icon_key: &str,
    subtitle: &str,
    info: &str,
    empty_state: Option<&PaneEmptyStateModel>,
    show_toolbar: bool,
    viewport: &SceneViewportChromeData,
    snapshot: Option<&ViewTabSnapshot>,
    chrome: &EditorChromeSnapshot,
    animation_pane: Option<&crate::ui::animation_editor::AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&RuntimeDiagnosticsSnapshot>,
) -> Option<PanePresentation> {
    let pane_template = snapshot.and_then(|snapshot| snapshot.pane_template.as_ref())?;
    let mut context = PanePayloadBuildContext::new(chrome);
    if let Some(animation_pane) = animation_pane {
        context = context.with_animation_pane(animation_pane);
    }
    if let Some(runtime_diagnostics) = runtime_diagnostics {
        context = context.with_runtime_diagnostics(runtime_diagnostics);
    }

    Some(PanePresentation::new(
        PaneShellPresentation::new(
            title,
            icon_key,
            subtitle,
            info,
            empty_state.map(empty_state_presentation),
            show_toolbar,
            viewport.clone(),
        ),
        build_pane_body_presentation(&pane_template.body, &context),
    ))
}

fn empty_state_presentation(empty_state: &PaneEmptyStateModel) -> PaneEmptyStatePresentation {
    PaneEmptyStatePresentation {
        title: empty_state.title.clone(),
        body: empty_state.body.clone(),
        primary_action: empty_state.primary_action.as_ref().map(action_presentation),
        secondary_action: empty_state
            .secondary_action
            .as_ref()
            .map(action_presentation),
        secondary_hint: empty_state.secondary_hint.clone().unwrap_or_default(),
    }
}

fn action_presentation(action: &PaneActionModel) -> PaneActionPresentation {
    PaneActionPresentation {
        label: action.label.clone(),
        action_id: action_id_from_model(action),
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
        ViewContentKind::RuntimeDiagnostics => (
            "Runtime Services".to_string(),
            "Render, physics, and animation diagnostics".to_string(),
            false,
        ),
        ViewContentKind::ModulePlugins => (
            "Project Plugins".to_string(),
            "Builtin and native plugin packages".to_string(),
            false,
        ),
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
        ViewContentKind::UiComponentShowcase => (
            "UI Component Showcase".to_string(),
            "Runtime component semantics and editor bindings".to_string(),
            false,
        ),
        ViewContentKind::AnimationSequenceEditor => (
            payload_path(snapshot).unwrap_or_else(|| "Animation Sequence".to_string()),
            "Sequence timeline and property-track authoring".to_string(),
            false,
        ),
        ViewContentKind::AnimationGraphEditor => (
            payload_path(snapshot).unwrap_or_else(|| "Animation Graph".to_string()),
            "Graph and state-machine authoring surface".to_string(),
            false,
        ),
        ViewContentKind::Placeholder => (
            "Missing View".to_string(),
            "This pane was restored from layout state but the descriptor is unavailable.".into(),
            false,
        ),
    }
}

fn hierarchy_pane_data(chrome: &EditorChromeSnapshot) -> HierarchyPaneViewData {
    HierarchyPaneViewData {
        nodes: Default::default(),
        hierarchy_nodes: model_rc(
            chrome
                .scene_entries
                .iter()
                .map(|entry| SceneNodeData {
                    id: entry.id.to_string().into(),
                    name: entry.name.clone().into(),
                    depth: entry.depth as i32,
                    selected: entry.selected,
                })
                .collect(),
        ),
    }
}

fn inspector_pane_data(chrome: &EditorChromeSnapshot, info: &str) -> InspectorPaneViewData {
    InspectorPaneViewData {
        nodes: Default::default(),
        info: SharedString::from(info),
        inspector_name: chrome
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.clone())
            .unwrap_or_default()
            .into(),
        inspector_parent: chrome
            .inspector
            .as_ref()
            .map(|inspector| inspector.parent.clone())
            .unwrap_or_default()
            .into(),
        inspector_x: chrome
            .inspector
            .as_ref()
            .map(|inspector| inspector.translation[0].clone())
            .unwrap_or_default()
            .into(),
        inspector_y: chrome
            .inspector
            .as_ref()
            .map(|inspector| inspector.translation[1].clone())
            .unwrap_or_default()
            .into(),
        inspector_z: chrome
            .inspector
            .as_ref()
            .map(|inspector| inspector.translation[2].clone())
            .unwrap_or_default()
            .into(),
        delete_enabled: chrome.inspector.is_some(),
    }
}

fn console_pane_data(chrome: &EditorChromeSnapshot) -> ConsolePaneViewData {
    ConsolePaneViewData {
        nodes: Default::default(),
        status_text: chrome.status_line.clone().into(),
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
        ViewContentKind::UiComponentShowcase => "UiComponentShowcase",
        ViewContentKind::AnimationSequenceEditor => "AnimationSequenceEditor",
        ViewContentKind::AnimationGraphEditor => "AnimationGraphEditor",
        ViewContentKind::RuntimeDiagnostics => "RuntimeDiagnostics",
        ViewContentKind::ModulePlugins => "ModulePlugins",
        ViewContentKind::Placeholder => "Placeholder",
    }
}

fn find_in_workspace<'a>(
    workspace: &'a crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot,
    instance_id: &str,
) -> Option<&'a ViewTabSnapshot> {
    match workspace {
        crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot::Split {
            first, second, ..
        } => {
            find_in_workspace(first, instance_id).or_else(|| find_in_workspace(second, instance_id))
        }
        crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot::Tabs { tabs, .. } => tabs
            .iter()
            .find(|tab| tab.instance_id.0.as_str() == instance_id),
    }
}
