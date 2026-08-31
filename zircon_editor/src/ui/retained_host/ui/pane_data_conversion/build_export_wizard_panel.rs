use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::ui::host::{
    export_wizard_panel_retained_projection, register_export_wizard_panel_template,
    ExportWizardPanelViewModel, ExportWizardPipelinePlan, DESKTOP_EXPORT_CANCEL_BUTTON,
    DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_START_BUTTON,
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
const EXPORT_WIZARD_DEFAULT_OUT: &str = "zircon-export";

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
    let nodes = retained_projection_template_nodes(projection, &profile_name);
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
            .join("panel.zui"),
    )
}

fn build_export_wizard_panel_view_model(
    data: &BuildExportPaneViewData,
) -> Cow<'_, ExportWizardPanelViewModel> {
    if let Some(view_model) = data.wizard_view_model.as_ref() {
        return Cow::Borrowed(view_model);
    }

    let profile = build_export_wizard_panel_profile_name(data);
    let plan = ExportWizardPipelinePlan::unavailable(
        profile.clone(),
        EXPORT_WIZARD_DEFAULT_OUT,
        format!("No loaded export preset is available for `{profile}`"),
    );
    Cow::Owned(ExportWizardPanelViewModel::from_plan(
        EXPORT_WIZARD_PANEL_JOB_ID,
        &plan,
    ))
}

fn build_export_wizard_panel_first_target(
    data: &BuildExportPaneViewData,
) -> Option<&crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData> {
    data.targets.iter().next()
}

fn build_export_wizard_panel_profile_name(data: &BuildExportPaneViewData) -> String {
    build_export_wizard_panel_first_target(data)
        .map(|target| target.preset_name.to_string())
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
    projection: RetainedUiHostProjection,
    profile_name: &str,
) -> Vec<host_contract::TemplatePaneNodeData> {
    projection
        .nodes
        .into_iter()
        .filter_map(|node| retained_template_node(node, profile_name))
        .collect()
}

fn retained_template_node(
    mut node: RetainedUiHostNodeModel,
    profile_name: &str,
) -> Option<host_contract::TemplatePaneNodeData> {
    let control_id = node.control_id.take()?;
    let text = node
        .text
        .take()
        .or_else(|| retained_string_property(&node.properties, "text"))
        .or_else(|| retained_string_property(&node.properties, "label"))
        .unwrap_or_default();
    let role = retained_role(&node);
    let component_role = retained_component_role(&node);
    let surface_variant = retained_surface_variant(&node);
    let text_tone = retained_text_tone(&node);
    let button_variant =
        retained_string_property(&node.properties, "button_variant").unwrap_or_default();
    let primary_route = node.routes.into_iter().next();
    let has_primary_route = primary_route.is_some();
    let (retained_action_id, binding_id) = primary_route
        .map(|route| (route.action_id, route.binding_id))
        .unwrap_or_default();
    let action_id = build_export_wizard_panel_action_id(&control_id, profile_name)
        .unwrap_or(retained_action_id);
    let dispatch_kind = if !node.disabled && !action_id.is_empty() {
        EXPORT_WIZARD_PANEL_DISPATCH_KIND
    } else {
        ""
    };
    let actions = has_primary_route
        .then(|| {
            vec![host_contract::TemplatePaneActionData {
                label: SharedString::from(text.clone()),
                action_id: SharedString::from(action_id.clone()),
            }]
        })
        .unwrap_or_default();

    Some(host_contract::TemplatePaneNodeData {
        node_id: node.node_id.into(),
        control_id: control_id.into(),
        role: role.into(),
        text: text.into(),
        component_role: component_role.into(),
        value_text: node.value_text.unwrap_or_default().into(),
        validation_level: node.validation_level.unwrap_or_default().into(),
        validation_message: node.validation_message.unwrap_or_default().into(),
        options_text: node.options_text.unwrap_or_default().into(),
        options: retained_shared_string_list(node.options),
        collection_items: retained_shared_string_list(node.collection_items),
        menu_items: retained_shared_string_list(node.menu_items),
        actions: model_rc(actions),
        accepted_drag_payloads: node.accepted_drag_payloads.join(",").into(),
        drop_source_summary: node.drop_source_summary.unwrap_or_default().into(),
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
        surface_variant: surface_variant.into(),
        text_tone: text_tone.into(),
        button_variant: button_variant.into(),
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
    let severity = retained_string_property_ref(&node.properties, "severity")
        .or(node.validation_level.as_deref())
        .unwrap_or_default();
    if matches_ignore_ascii_case(severity, &["danger", "error", "fatal"]) {
        "danger".to_string()
    } else if severity.eq_ignore_ascii_case("warning") {
        "warning".to_string()
    } else if severity.eq_ignore_ascii_case("success") {
        "success".to_string()
    } else if severity.eq_ignore_ascii_case("info") {
        "info".to_string()
    } else if severity.eq_ignore_ascii_case("disabled") {
        "muted".to_string()
    } else {
        retained_string_property(&node.properties, "text_tone").unwrap_or_default()
    }
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn retained_string_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<String> {
    retained_string_property_ref(properties, key).map(str::to_owned)
}

fn retained_string_property_ref<'a>(
    properties: &'a BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<&'a str> {
    match properties.get(key) {
        Some(RetainedUiHostValue::String(value)) => Some(value),
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

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn published_wizard_view_model_is_borrowed_without_a_payload_clone() {
        let plan = ExportWizardPipelinePlan::unavailable(
            "desktop_windows",
            EXPORT_WIZARD_DEFAULT_OUT,
            "fixture unavailable",
        );
        let data = BuildExportPaneViewData {
            wizard_view_model: Some(ExportWizardPanelViewModel::from_plan(
                EXPORT_WIZARD_PANEL_JOB_ID,
                &plan,
            )),
            ..BuildExportPaneViewData::default()
        };

        let selected = build_export_wizard_panel_view_model(&data);

        assert!(matches!(&selected, Cow::Borrowed(_)));
        assert!(std::ptr::eq(
            selected.as_ref(),
            data.wizard_view_model
                .as_ref()
                .expect("published wizard view model")
        ));
    }

    #[test]
    fn missing_wizard_view_model_constructs_an_owned_fallback() {
        let selected = build_export_wizard_panel_view_model(&BuildExportPaneViewData::default());

        assert!(matches!(selected, Cow::Owned(_)));
    }
}
