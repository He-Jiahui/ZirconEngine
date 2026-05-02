use std::sync::OnceLock;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::PaneContentSize;
use crate::ui::slint_host as host_contract;
use crate::ui::template_runtime::EditorUiHostRuntime;
use slint::{Model, ModelRc};
use zircon_runtime_interface::ui::layout::UiSize;

use super::template_node_conversion::to_host_contract_template_node_owned;

mod pane_component_projection;
mod pane_menu_projection;
mod pane_option_projection;
mod pane_ui_asset_conversion;
mod pane_value_conversion;

use self::pane_component_projection::host_template_node;
pub(super) use self::pane_ui_asset_conversion::to_host_contract_ui_asset_pane;
use self::pane_value_conversion::{value_as_bool, value_as_string};

fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    T: Clone + 'static,
    U: Clone + 'static,
    F: FnMut(T) -> U,
{
    model_rc(
        (0..model.row_count())
            .filter_map(|row| model.row_data(row))
            .map(&mut map)
            .collect(),
    )
}

fn to_host_contract_scene_node(
    node: crate::ui::layouts::windows::workbench_host_window::SceneNodeData,
) -> host_contract::SceneNodeData {
    host_contract::SceneNodeData {
        id: node.id,
        name: node.name,
        depth: node.depth,
        selected: node.selected,
    }
}

fn to_host_contract_scene_nodes(
    nodes: &ModelRc<crate::ui::layouts::windows::workbench_host_window::SceneNodeData>,
) -> ModelRc<host_contract::SceneNodeData> {
    map_model_rc(nodes, to_host_contract_scene_node)
}

pub(super) fn to_host_contract_hierarchy_pane(
    data: crate::ui::layouts::windows::workbench_host_window::HierarchyPaneViewData,
) -> host_contract::HierarchyPaneData {
    host_contract::HierarchyPaneData {
        nodes: map_model_rc(&data.nodes, to_host_contract_template_node_owned),
        hierarchy_nodes: to_host_contract_scene_nodes(&data.hierarchy_nodes),
    }
}

pub(crate) fn to_host_contract_hierarchy_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> host_contract::HierarchyPaneData {
    hierarchy_template_projection(data, content_size)
        .unwrap_or_else(|| to_host_contract_hierarchy_pane(data.native_body.hierarchy.clone()))
}

pub(super) fn to_host_contract_inspector_pane(
    data: crate::ui::layouts::windows::workbench_host_window::InspectorPaneViewData,
) -> host_contract::InspectorPaneData {
    host_contract::InspectorPaneData {
        nodes: map_model_rc(&data.nodes, to_host_contract_template_node_owned),
        info: data.info,
        inspector_name: data.inspector_name,
        inspector_parent: data.inspector_parent,
        inspector_x: data.inspector_x,
        inspector_y: data.inspector_y,
        inspector_z: data.inspector_z,
        delete_enabled: data.delete_enabled,
    }
}

pub(crate) fn to_host_contract_inspector_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> host_contract::InspectorPaneData {
    inspector_template_projection(data, content_size)
        .unwrap_or_else(|| to_host_contract_inspector_pane(data.native_body.inspector.clone()))
}

pub(super) fn to_host_contract_console_pane(
    data: crate::ui::layouts::windows::workbench_host_window::ConsolePaneViewData,
) -> host_contract::ConsolePaneData {
    host_contract::ConsolePaneData {
        nodes: map_model_rc(&data.nodes, to_host_contract_console_legacy_node),
        status_text: data.status_text,
    }
}

pub(crate) fn to_host_contract_console_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> host_contract::ConsolePaneData {
    console_template_projection(data, content_size)
        .unwrap_or_else(|| to_host_contract_console_pane(data.native_body.console.clone()))
}

pub(super) fn to_host_contract_assets_activity_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AssetsActivityPaneViewData,
) -> host_contract::AssetsActivityPaneData {
    host_contract::AssetsActivityPaneData {
        nodes: map_model_rc(&data.nodes, to_host_contract_template_node_owned),
    }
}

pub(super) fn to_host_contract_animation_editor_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AnimationEditorPaneViewData,
) -> host_contract::AnimationEditorPaneData {
    host_contract::AnimationEditorPaneData {
        nodes: map_model_rc(&data.nodes, to_host_contract_template_node_owned),
        mode: data.mode,
        asset_path: data.asset_path,
        status: data.status,
        selection: data.selection,
        current_frame: data.current_frame,
        timeline_start_frame: data.timeline_start_frame,
        timeline_end_frame: data.timeline_end_frame,
        playback_label: data.playback_label,
        track_items: data.track_items,
        parameter_items: data.parameter_items,
        node_items: data.node_items,
        state_items: data.state_items,
        transition_items: data.transition_items,
    }
}

pub(crate) fn to_host_contract_animation_editor_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> host_contract::AnimationEditorPaneData {
    animation_template_projection(data, content_size).unwrap_or_else(|| {
        to_host_contract_animation_editor_pane(data.native_body.animation.clone())
    })
}

fn to_host_contract_shared_string_list(items: Vec<String>) -> ModelRc<slint::SharedString> {
    model_rc(items.into_iter().map(slint::SharedString::from).collect())
}

pub(super) fn to_host_contract_asset_browser_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AssetBrowserPaneViewData,
) -> host_contract::AssetBrowserPaneData {
    host_contract::AssetBrowserPaneData {
        nodes: map_model_rc(&data.nodes, to_host_contract_template_node_owned),
    }
}

fn console_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> Option<host_contract::ConsolePaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::ConsoleV1(_)
    ) {
        return None;
    }

    let runtime = builtin_host_runtime()?;
    let projection = runtime.project_pane_body(&presentation.body).ok()?;
    let mut surface = runtime
        .build_shared_surface(&presentation.body.document_id)
        .ok()?;
    surface
        .compute_layout(UiSize::new(
            content_size.width.max(0.0),
            content_size.height.max(0.0),
        ))
        .ok()?;
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .ok()?;
    let status_text = projection
        .root
        .attributes
        .get("payload_status_text")
        .and_then(value_as_string)
        .or_else(|| match &presentation.body.payload {
            crate::ui::layouts::windows::workbench_host_window::PanePayload::ConsoleV1(payload) => {
                Some(payload.status_text.clone())
            }
            _ => None,
        })
        .unwrap_or_default();

    Some(host_contract::ConsolePaneData {
        nodes: model_rc(
            host_model
                .nodes
                .into_iter()
                .filter_map(host_template_node)
                .collect(),
        ),
        status_text: status_text.into(),
    })
}

fn inspector_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> Option<host_contract::InspectorPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let crate::ui::layouts::windows::workbench_host_window::PanePayload::InspectorV1(payload) =
        &presentation.body.payload
    else {
        return None;
    };

    let runtime = builtin_host_runtime()?;
    let projection = runtime.project_pane_body(&presentation.body).ok()?;
    let mut surface = runtime
        .build_shared_surface(&presentation.body.document_id)
        .ok()?;
    surface
        .compute_layout(UiSize::new(
            content_size.width.max(0.0),
            content_size.height.max(0.0),
        ))
        .ok()?;
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .ok()?;

    Some(host_contract::InspectorPaneData {
        nodes: model_rc(
            host_model
                .nodes
                .into_iter()
                .filter_map(host_template_node)
                .collect(),
        ),
        info: data.info.clone(),
        inspector_name: projection
            .root
            .attributes
            .get("payload_name")
            .and_then(value_as_string)
            .unwrap_or_else(|| payload.name.clone())
            .into(),
        inspector_parent: projection
            .root
            .attributes
            .get("payload_parent")
            .and_then(value_as_string)
            .unwrap_or_else(|| payload.parent.clone())
            .into(),
        inspector_x: projection
            .root
            .attributes
            .get("payload_translation_x")
            .and_then(value_as_string)
            .unwrap_or_else(|| payload.translation[0].clone())
            .into(),
        inspector_y: projection
            .root
            .attributes
            .get("payload_translation_y")
            .and_then(value_as_string)
            .unwrap_or_else(|| payload.translation[1].clone())
            .into(),
        inspector_z: projection
            .root
            .attributes
            .get("payload_translation_z")
            .and_then(value_as_string)
            .unwrap_or_else(|| payload.translation[2].clone())
            .into(),
        delete_enabled: projection
            .root
            .attributes
            .get("payload_delete_enabled")
            .and_then(value_as_bool)
            .unwrap_or(payload.delete_enabled),
    })
}

fn hierarchy_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> Option<host_contract::HierarchyPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let crate::ui::layouts::windows::workbench_host_window::PanePayload::HierarchyV1(payload) =
        &presentation.body.payload
    else {
        return None;
    };

    let nodes = project_pane_template_nodes(&presentation.body, content_size)?;
    Some(host_contract::HierarchyPaneData {
        nodes: model_rc(nodes),
        hierarchy_nodes: model_rc(
            payload
                .nodes
                .iter()
                .map(|node| host_contract::SceneNodeData {
                    id: node.node_id.to_string().into(),
                    name: node.name.clone().into(),
                    depth: i32::try_from(node.depth).unwrap_or(i32::MAX),
                    selected: node.selected,
                })
                .collect(),
        ),
    })
}

fn animation_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> Option<host_contract::AnimationEditorPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let nodes = model_rc(project_pane_template_nodes(
        &presentation.body,
        content_size,
    )?);

    match &presentation.body.payload {
        crate::ui::layouts::windows::workbench_host_window::PanePayload::AnimationSequenceV1(
            payload,
        ) => Some(host_contract::AnimationEditorPaneData {
            nodes,
            mode: payload.mode.clone().into(),
            asset_path: payload.asset_path.clone().into(),
            status: payload.status.clone().into(),
            selection: payload.selection.clone().into(),
            current_frame: i32::try_from(payload.current_frame).unwrap_or(i32::MAX),
            timeline_start_frame: i32::try_from(payload.timeline_start_frame).unwrap_or(i32::MAX),
            timeline_end_frame: i32::try_from(payload.timeline_end_frame).unwrap_or(i32::MAX),
            playback_label: payload.playback_label.clone().into(),
            track_items: to_host_contract_shared_string_list(payload.track_items.clone()),
            parameter_items: ModelRc::default(),
            node_items: ModelRc::default(),
            state_items: ModelRc::default(),
            transition_items: ModelRc::default(),
        }),
        crate::ui::layouts::windows::workbench_host_window::PanePayload::AnimationGraphV1(
            payload,
        ) => Some(host_contract::AnimationEditorPaneData {
            nodes,
            mode: payload.mode.clone().into(),
            asset_path: payload.asset_path.clone().into(),
            status: payload.status.clone().into(),
            selection: payload.selection.clone().into(),
            current_frame: 0,
            timeline_start_frame: 0,
            timeline_end_frame: 0,
            playback_label: String::new().into(),
            track_items: ModelRc::default(),
            parameter_items: to_host_contract_shared_string_list(payload.parameter_items.clone()),
            node_items: to_host_contract_shared_string_list(payload.node_items.clone()),
            state_items: to_host_contract_shared_string_list(payload.state_items.clone()),
            transition_items: to_host_contract_shared_string_list(payload.transition_items.clone()),
        }),
        _ => None,
    }
}

fn project_pane_template_nodes(
    body: &crate::ui::layouts::windows::workbench_host_window::PaneBodyPresentation,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let runtime = builtin_host_runtime()?;
    let projection = runtime.project_pane_body(body).ok()?;
    let host_model = runtime.build_host_model(&projection).ok()?;

    Some(
        host_model
            .nodes
            .into_iter()
            .filter_map(|node| host_template_node_with_content_fallback(node, content_size))
            .collect(),
    )
}

fn host_template_node_with_content_fallback(
    node: crate::ui::template_runtime::SlintUiHostNodeProjection,
    content_size: PaneContentSize,
) -> Option<host_contract::TemplatePaneNodeData> {
    let control_id = node.control_id.clone();
    let mut node = host_template_node(node)?;
    if matches!(
        control_id.as_deref(),
        Some("HierarchyListPanel")
            | Some("HierarchyTreeSlotAnchor")
            | Some("AnimationEditorBodyPanel")
            | Some("AnimationSequenceContentPanel")
            | Some("AnimationTimelineSlotAnchor")
            | Some("AnimationGraphContentPanel")
            | Some("AnimationGraphCanvasSlotAnchor")
    ) && node.frame.width <= 0.0
        && node.frame.height <= 0.0
    {
        node.frame.width = content_size.width.max(0.0);
        node.frame.height = content_size.height.max(0.0);
    }
    Some(node)
}

fn to_host_contract_console_legacy_node(
    data: crate::ui::layouts::views::ViewTemplateNodeData,
) -> host_contract::TemplatePaneNodeData {
    let mut node = to_host_contract_template_node_owned(data);
    if node.control_id == "ConsoleTextPanel" {
        node.control_id = "ConsoleBodySection".into();
    }
    node
}

fn builtin_host_runtime() -> Option<&'static EditorUiHostRuntime> {
    static BUILTIN_HOST_RUNTIME: OnceLock<Option<EditorUiHostRuntime>> = OnceLock::new();
    BUILTIN_HOST_RUNTIME
        .get_or_init(|| {
            let mut runtime = EditorUiHostRuntime::default();
            runtime.load_builtin_host_templates().ok()?;
            Some(runtime)
        })
        .as_ref()
}

pub(super) fn to_host_contract_project_overview_pane(
    data: crate::ui::layouts::windows::workbench_host_window::ProjectOverviewPaneViewData,
) -> host_contract::ProjectOverviewPaneData {
    host_contract::ProjectOverviewPaneData {
        nodes: map_model_rc(&data.nodes, to_host_contract_template_node_owned),
    }
}

pub(crate) fn to_host_contract_component_showcase_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> host_contract::ProjectOverviewPaneData {
    builtin_host_runtime()
        .and_then(|runtime| component_showcase_template_projection(data, content_size, runtime))
        .unwrap_or_default()
}

pub(crate) fn to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::ProjectOverviewPaneData {
    component_showcase_template_projection(data, content_size, runtime).unwrap_or_default()
}

fn component_showcase_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> Option<host_contract::ProjectOverviewPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::UiComponentShowcaseV1(_)
    ) {
        return None;
    }

    let projection = runtime.project_pane_body(&presentation.body).ok()?;
    let mut surface = runtime
        .build_shared_surface(&presentation.body.document_id)
        .ok()?;
    surface
        .compute_layout(UiSize::new(
            content_size.width.max(0.0),
            content_size.height.max(0.0),
        ))
        .ok()?;
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .ok()?;

    Some(host_contract::ProjectOverviewPaneData {
        nodes: model_rc(
            host_model
                .nodes
                .into_iter()
                .filter_map(host_template_node)
                .collect(),
        ),
    })
}
