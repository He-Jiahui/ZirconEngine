use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::ui::host::{
    export_wizard_panel_retained_projection, export_wizard_pipeline_plan,
    register_export_wizard_panel_template, ExportWizardPanelViewModel, ExportWizardPipelineOptions,
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_START_BUTTON,
};
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostComponentKind, RetainedUiHostNodeModel,
    RetainedUiHostProjection, RetainedUiHostValue,
};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

pub(super) const EXPORT_WIZARD_PANEL_DISPATCH_KIND: &str = "export_wizard_panel";

const EXPORT_WIZARD_PANEL_JOB_ID: &str = "workbench.build_export_desktop";
const EXPORT_WIZARD_DEFAULT_PROJECT: &str = "zircon-project.toml";
const EXPORT_WIZARD_DEFAULT_OUT: &str = "zircon-export";
const EXPORT_WIZARD_DEFAULT_ASSET_MANIFEST: &str = "zircon-export/assets/assets.json";
const EXPORT_WIZARD_DEFAULT_HOST_EXECUTABLE: &str = "zircon-export/host/zircon_game.exe";

pub(super) fn build_export_pane_supports_wizard_projection(data: &PaneData) -> bool {
    data.pane_presentation
        .as_ref()
        .map(|presentation| matches!(&presentation.body.payload, PanePayload::BuildExportV1(_)))
        .unwrap_or(false)
}

pub(super) fn build_export_wizard_panel_nodes(
    data: &BuildExportPaneViewData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let runtime = export_wizard_panel_runtime()?;
    let view_model = build_export_wizard_panel_view_model(data);
    let projection = export_wizard_panel_retained_projection(
        runtime,
        &view_model,
        UiSize::new(content_size.width.max(0.0), content_size.height.max(0.0)),
    )
    .ok()?;
    let profile_name = build_export_wizard_panel_profile_name_for_view_model(data, &view_model);
    let nodes = retained_projection_template_nodes(&projection, &profile_name);
    (!nodes.is_empty()).then_some(nodes)
}

fn export_wizard_panel_runtime() -> Option<&'static EditorUiHostRuntime> {
    static EXPORT_WIZARD_PANEL_RUNTIME: OnceLock<Option<EditorUiHostRuntime>> = OnceLock::new();
    EXPORT_WIZARD_PANEL_RUNTIME
        .get_or_init(|| {
            let mut runtime = EditorUiHostRuntime::default();
            register_export_wizard_panel_template(
                &mut runtime,
                desktop_export_panel_template_path()?,
            )
            .ok()?;
            Some(runtime)
        })
        .as_ref()
}

fn desktop_export_panel_template_path() -> Option<PathBuf> {
    Some(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()?
            .join("zircon_plugins")
            .join("editor_build_export_desktop")
            .join("editor")
            .join("panel.v2.ui.toml"),
    )
}

fn build_export_wizard_panel_view_model(
    data: &BuildExportPaneViewData,
) -> ExportWizardPanelViewModel {
    if let Some(view_model) = data.wizard_view_model.clone() {
        return view_model;
    }

    let first_target = build_export_wizard_panel_first_target(data);
    let profile = build_export_wizard_panel_profile_name(data);
    let mut options = ExportWizardPipelineOptions::new(
        profile,
        EXPORT_WIZARD_DEFAULT_PROJECT,
        EXPORT_WIZARD_DEFAULT_OUT,
    );
    options.offline = true;
    options.dry_run = true;

    if let Some(target) = first_target.as_ref() {
        if !target.platform.is_empty() {
            options.target_platform = Some(target.platform.to_string());
        }
        if !target.fatal {
            options.source_asset_manifest = Some(EXPORT_WIZARD_DEFAULT_ASSET_MANIFEST.to_string());
            options.host_executable = Some(EXPORT_WIZARD_DEFAULT_HOST_EXECUTABLE.to_string());
        }
    }

    let plan = export_wizard_pipeline_plan(options);
    ExportWizardPanelViewModel::from_plan(EXPORT_WIZARD_PANEL_JOB_ID, &plan)
}

fn build_export_wizard_panel_first_target(
    data: &BuildExportPaneViewData,
) -> Option<crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData> {
    (0..data.targets.row_count())
        .filter_map(|row| data.targets.row_data(row))
        .next()
}

fn build_export_wizard_panel_profile_name(data: &BuildExportPaneViewData) -> String {
    build_export_wizard_panel_first_target(data)
        .as_ref()
        .map(|target| target.profile_name.to_string())
        .filter(|profile| !profile.is_empty())
        .unwrap_or_else(|| "desktop_windows".to_string())
}

fn build_export_wizard_panel_profile_name_for_view_model(
    data: &BuildExportPaneViewData,
    view_model: &ExportWizardPanelViewModel,
) -> String {
    let profile = view_model.snapshot().profile.clone();
    if profile.is_empty() {
        build_export_wizard_panel_profile_name(data)
    } else {
        profile
    }
}

fn retained_projection_template_nodes(
    projection: &RetainedUiHostProjection,
    profile_name: &str,
) -> Vec<host_contract::TemplatePaneNodeData> {
    projection
        .nodes
        .iter()
        .filter_map(|node| retained_template_node(node, profile_name))
        .collect()
}

fn retained_template_node(
    node: &RetainedUiHostNodeModel,
    profile_name: &str,
) -> Option<host_contract::TemplatePaneNodeData> {
    let control_id = node.control_id.clone()?;
    let text = node
        .text
        .clone()
        .or_else(|| retained_string_property(&node.properties, "text"))
        .or_else(|| retained_string_property(&node.properties, "label"))
        .unwrap_or_default();
    let primary_route = node.routes.first();
    let retained_action_id = primary_route
        .map(|route| route.action_id.clone())
        .unwrap_or_default();
    let binding_id = primary_route
        .map(|route| route.binding_id.clone())
        .unwrap_or_default();
    let action_id = build_export_wizard_panel_action_id(&control_id, profile_name)
        .unwrap_or(retained_action_id);
    let dispatch_kind = if !node.disabled && !action_id.is_empty() {
        EXPORT_WIZARD_PANEL_DISPATCH_KIND
    } else {
        ""
    };
    let actions = primary_route
        .map(|route| {
            vec![host_contract::TemplatePaneActionData {
                label: SharedString::from(text.clone()),
                action_id: SharedString::from(
                    build_export_wizard_panel_action_id(&control_id, profile_name)
                        .unwrap_or_else(|| route.action_id.clone()),
                ),
            }]
        })
        .unwrap_or_default();

    Some(host_contract::TemplatePaneNodeData {
        node_id: node.node_id.clone().into(),
        control_id: control_id.into(),
        role: retained_role(node).into(),
        text: text.into(),
        component_role: retained_component_role(node).into(),
        value_text: node.value_text.clone().unwrap_or_default().into(),
        validation_level: node.validation_level.clone().unwrap_or_default().into(),
        validation_message: node.validation_message.clone().unwrap_or_default().into(),
        options_text: node.options_text.clone().unwrap_or_default().into(),
        options: retained_shared_string_list(node.options.clone()),
        collection_items: retained_shared_string_list(node.collection_items.clone()),
        menu_items: retained_shared_string_list(node.menu_items.clone()),
        actions: model_rc(actions),
        accepted_drag_payloads: node.accepted_drag_payloads.join(",").into(),
        drop_source_summary: node.drop_source_summary.clone().unwrap_or_default().into(),
        checked: node.checked,
        expanded: node.expanded,
        focused: node.focused,
        hovered: node.hovered,
        pressed: node.pressed,
        dragging: node.dragging,
        drop_hovered: node.drop_hovered,
        active_drag_target: node.active_drag_target,
        disabled: node.disabled,
        dispatch_kind: dispatch_kind.into(),
        action_id: action_id.into(),
        binding_id: binding_id.into(),
        surface_variant: retained_surface_variant(node).into(),
        text_tone: retained_text_tone(node).into(),
        button_variant: retained_string_property(&node.properties, "button_variant")
            .unwrap_or_default()
            .into(),
        z_index: node.z_index,
        has_clip_frame: node.clip_frame.is_some(),
        clip_frame: node.clip_frame.map(template_node_frame).unwrap_or_default(),
        frame: template_node_frame(node.frame),
        ..host_contract::TemplatePaneNodeData::default()
    })
}

fn build_export_wizard_panel_action_id(control_id: &str, profile_name: &str) -> Option<String> {
    let action = match control_id {
        DESKTOP_EXPORT_GENERATE_PLAN_BUTTON => "plan",
        DESKTOP_EXPORT_START_BUTTON => "execute",
        DESKTOP_EXPORT_CANCEL_BUTTON => "cancel",
        _ => return None,
    };
    Some(format!("workbench.build_export.{action}.{profile_name}"))
}

fn retained_role(node: &RetainedUiHostNodeModel) -> String {
    match node.kind {
        RetainedUiHostComponentKind::IconButton => "Button".to_string(),
        RetainedUiHostComponentKind::Unknown => node.component.clone(),
        _ => node.kind.as_str().to_string(),
    }
}

fn retained_component_role(node: &RetainedUiHostNodeModel) -> String {
    node.component_role
        .clone()
        .filter(|role| !role.is_empty())
        .unwrap_or_else(|| match node.kind {
            RetainedUiHostComponentKind::Root => "root".to_string(),
            RetainedUiHostComponentKind::IconButton => "button".to_string(),
            RetainedUiHostComponentKind::Label => "label".to_string(),
            RetainedUiHostComponentKind::HorizontalBox
            | RetainedUiHostComponentKind::VerticalBox => "layout".to_string(),
            _ => String::new(),
        })
}

fn retained_surface_variant(node: &RetainedUiHostNodeModel) -> String {
    retained_string_property(&node.properties, "surface_variant").unwrap_or_else(|| {
        if matches!(
            node.kind,
            RetainedUiHostComponentKind::PaneSurface
                | RetainedUiHostComponentKind::HorizontalBox
                | RetainedUiHostComponentKind::VerticalBox
        ) {
            "panel".to_string()
        } else {
            String::new()
        }
    })
}

fn retained_text_tone(node: &RetainedUiHostNodeModel) -> String {
    let severity = retained_string_property(&node.properties, "severity")
        .or_else(|| node.validation_level.clone())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match severity.as_str() {
        "danger" | "error" | "fatal" => "danger".to_string(),
        "warning" => "warning".to_string(),
        "success" => "success".to_string(),
        "info" => "info".to_string(),
        "disabled" => "muted".to_string(),
        _ => retained_string_property(&node.properties, "text_tone").unwrap_or_default(),
    }
}

fn retained_string_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<String> {
    match properties.get(key) {
        Some(RetainedUiHostValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn retained_shared_string_list(items: Vec<String>) -> ModelRc<SharedString> {
    model_rc(items.into_iter().map(SharedString::from).collect())
}

fn template_node_frame(frame: UiFrame) -> host_contract::TemplateNodeFrameData {
    host_contract::TemplateNodeFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
