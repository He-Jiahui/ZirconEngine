use std::sync::OnceLock;

use crate::ui::asset_editor;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::layouts::windows::workbench_host_window::PaneContentSize;
use crate::ui::slint_host as slint_ui;
use crate::ui::template_runtime::EditorUiHostRuntime;
use slint::{Model, ModelRc};
use toml::Value;
use zircon_runtime::ui::component::{UiComponentDescriptorRegistry, UiValue};
use zircon_runtime::ui::layout::UiSize;

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

fn to_slint_scene_node(
    node: crate::ui::layouts::windows::workbench_host_window::SceneNodeData,
) -> slint_ui::SceneNodeData {
    slint_ui::SceneNodeData {
        id: node.id,
        name: node.name,
        depth: node.depth,
        selected: node.selected,
    }
}

fn to_slint_scene_nodes(
    nodes: &ModelRc<crate::ui::layouts::windows::workbench_host_window::SceneNodeData>,
) -> ModelRc<slint_ui::SceneNodeData> {
    map_model_rc(nodes, to_slint_scene_node)
}

pub(super) fn to_slint_hierarchy_pane(
    data: crate::ui::layouts::windows::workbench_host_window::HierarchyPaneViewData,
) -> slint_ui::HierarchyPaneData {
    slint_ui::HierarchyPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
        hierarchy_nodes: to_slint_scene_nodes(&data.hierarchy_nodes),
    }
}

pub(crate) fn to_slint_hierarchy_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> slint_ui::HierarchyPaneData {
    hierarchy_template_projection(data, content_size)
        .unwrap_or_else(|| to_slint_hierarchy_pane(data.body_compat.hierarchy.clone()))
}

pub(super) fn to_slint_inspector_pane(
    data: crate::ui::layouts::windows::workbench_host_window::InspectorPaneViewData,
) -> slint_ui::InspectorPaneData {
    slint_ui::InspectorPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
        info: data.info,
        inspector_name: data.inspector_name,
        inspector_parent: data.inspector_parent,
        inspector_x: data.inspector_x,
        inspector_y: data.inspector_y,
        inspector_z: data.inspector_z,
        delete_enabled: data.delete_enabled,
    }
}

pub(crate) fn to_slint_inspector_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> slint_ui::InspectorPaneData {
    inspector_template_projection(data, content_size)
        .unwrap_or_else(|| to_slint_inspector_pane(data.body_compat.inspector.clone()))
}

pub(super) fn to_slint_console_pane(
    data: crate::ui::layouts::windows::workbench_host_window::ConsolePaneViewData,
) -> slint_ui::ConsolePaneData {
    slint_ui::ConsolePaneData {
        nodes: map_model_rc(&data.nodes, to_slint_console_legacy_node),
        status_text: data.status_text,
    }
}

pub(crate) fn to_slint_console_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> slint_ui::ConsolePaneData {
    console_template_projection(data, content_size)
        .unwrap_or_else(|| to_slint_console_pane(data.body_compat.console.clone()))
}

pub(super) fn to_slint_assets_activity_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AssetsActivityPaneViewData,
) -> slint_ui::AssetsActivityPaneData {
    slint_ui::AssetsActivityPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
    }
}

pub(super) fn to_slint_animation_editor_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AnimationEditorPaneViewData,
) -> slint_ui::AnimationEditorPaneData {
    slint_ui::AnimationEditorPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
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

pub(crate) fn to_slint_animation_editor_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> slint_ui::AnimationEditorPaneData {
    animation_template_projection(data, content_size)
        .unwrap_or_else(|| to_slint_animation_editor_pane(data.body_compat.animation.clone()))
}

fn to_slint_shared_string_list(items: Vec<String>) -> ModelRc<slint::SharedString> {
    model_rc(items.into_iter().map(slint::SharedString::from).collect())
}

pub(super) fn to_slint_asset_browser_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AssetBrowserPaneViewData,
) -> slint_ui::AssetBrowserPaneData {
    slint_ui::AssetBrowserPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
    }
}

fn to_slint_template_frame(frame: ViewTemplateFrameData) -> slint_ui::TemplateNodeFrameData {
    slint_ui::TemplateNodeFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn to_slint_template_node(data: ViewTemplateNodeData) -> slint_ui::TemplatePaneNodeData {
    slint_ui::TemplatePaneNodeData {
        node_id: data.node_id,
        control_id: data.control_id,
        role: data.role,
        text: data.text,
        component_role: "".into(),
        value_text: "".into(),
        value_number: 0.0,
        value_percent: 0.0,
        value_color: slint::Color::from_argb_u8(0, 0, 0, 0),
        media_source: "".into(),
        icon_name: "".into(),
        vector_components: ModelRc::default(),
        validation_level: "".into(),
        validation_message: "".into(),
        popup_open: false,
        selection_state: "".into(),
        options_text: "".into(),
        options: ModelRc::default(),
        collection_items: ModelRc::default(),
        menu_items: ModelRc::default(),
        actions: ModelRc::default(),
        accepted_drag_payloads: "".into(),
        checked: false,
        expanded: false,
        focused: false,
        hovered: false,
        pressed: false,
        dragging: false,
        drop_hovered: false,
        disabled: false,
        dispatch_kind: data.dispatch_kind,
        action_id: data.action_id,
        begin_drag_action_id: "".into(),
        drag_action_id: "".into(),
        end_drag_action_id: "".into(),
        edit_action_id: "".into(),
        surface_variant: data.surface_variant,
        text_tone: data.text_tone,
        button_variant: data.button_variant,
        font_size: data.font_size,
        font_weight: data.font_weight,
        text_align: data.text_align,
        overflow: data.overflow,
        corner_radius: data.corner_radius,
        border_width: data.border_width,
        frame: to_slint_template_frame(data.frame),
    }
}

fn console_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> Option<slint_ui::ConsolePaneData> {
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

    Some(slint_ui::ConsolePaneData {
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
) -> Option<slint_ui::InspectorPaneData> {
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

    Some(slint_ui::InspectorPaneData {
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
) -> Option<slint_ui::HierarchyPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let crate::ui::layouts::windows::workbench_host_window::PanePayload::HierarchyV1(payload) =
        &presentation.body.payload
    else {
        return None;
    };

    let nodes = project_pane_template_nodes(&presentation.body, content_size)?;
    Some(slint_ui::HierarchyPaneData {
        nodes: model_rc(nodes),
        hierarchy_nodes: model_rc(
            payload
                .nodes
                .iter()
                .map(|node| slint_ui::SceneNodeData {
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
) -> Option<slint_ui::AnimationEditorPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let nodes = model_rc(project_pane_template_nodes(
        &presentation.body,
        content_size,
    )?);

    match &presentation.body.payload {
        crate::ui::layouts::windows::workbench_host_window::PanePayload::AnimationSequenceV1(
            payload,
        ) => Some(slint_ui::AnimationEditorPaneData {
            nodes,
            mode: payload.mode.clone().into(),
            asset_path: payload.asset_path.clone().into(),
            status: payload.status.clone().into(),
            selection: payload.selection.clone().into(),
            current_frame: i32::try_from(payload.current_frame).unwrap_or(i32::MAX),
            timeline_start_frame: i32::try_from(payload.timeline_start_frame).unwrap_or(i32::MAX),
            timeline_end_frame: i32::try_from(payload.timeline_end_frame).unwrap_or(i32::MAX),
            playback_label: payload.playback_label.clone().into(),
            track_items: to_slint_shared_string_list(payload.track_items.clone()),
            parameter_items: ModelRc::default(),
            node_items: ModelRc::default(),
            state_items: ModelRc::default(),
            transition_items: ModelRc::default(),
        }),
        crate::ui::layouts::windows::workbench_host_window::PanePayload::AnimationGraphV1(
            payload,
        ) => Some(slint_ui::AnimationEditorPaneData {
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
            parameter_items: to_slint_shared_string_list(payload.parameter_items.clone()),
            node_items: to_slint_shared_string_list(payload.node_items.clone()),
            state_items: to_slint_shared_string_list(payload.state_items.clone()),
            transition_items: to_slint_shared_string_list(payload.transition_items.clone()),
        }),
        _ => None,
    }
}

fn project_pane_template_nodes(
    body: &crate::ui::layouts::windows::workbench_host_window::PaneBodyPresentation,
    content_size: PaneContentSize,
) -> Option<Vec<slint_ui::TemplatePaneNodeData>> {
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
) -> Option<slint_ui::TemplatePaneNodeData> {
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

fn to_slint_console_legacy_node(
    data: crate::ui::layouts::views::ViewTemplateNodeData,
) -> slint_ui::TemplatePaneNodeData {
    let mut node = to_slint_template_node(data);
    if node.control_id == "ConsoleTextPanel" {
        node.control_id = "ConsoleBodySection".into();
    }
    node
}

fn host_template_node(
    node: crate::ui::template_runtime::SlintUiHostNodeProjection,
) -> Option<slint_ui::TemplatePaneNodeData> {
    let control_id = node.control_id?;
    let component = node.component.clone();
    let component_descriptor = runtime_component_registry().descriptor(&component);
    let disabled = node
        .attributes
        .get("disabled")
        .and_then(value_as_bool)
        .unwrap_or(false)
        || node.attributes.get("enabled").and_then(value_as_bool) == Some(false);
    let component_role = component_descriptor
        .map(|descriptor| descriptor.role.clone())
        .unwrap_or_default();
    let value_text = node
        .attributes
        .get("value_text")
        .and_then(value_as_string)
        .or_else(|| {
            node.attributes
                .get("value")
                .or_else(|| node.attributes.get("items"))
                .or_else(|| node.attributes.get("entries"))
                .map(UiValue::from_toml)
                .map(|value| value.display_text())
        })
        .unwrap_or_default();
    let value_number = node
        .attributes
        .get("value")
        .and_then(value_as_f64)
        .unwrap_or(0.0);
    let value_percent = normalized_value_percent(
        value_number,
        node.attributes.get("min").and_then(value_as_f64),
        node.attributes.get("max").and_then(value_as_f64),
    );
    let value_color = node
        .attributes
        .get("value")
        .and_then(value_as_color)
        .unwrap_or_else(|| slint::Color::from_argb_u8(0, 0, 0, 0));
    let media_source = node
        .attributes
        .get("image")
        .or_else(|| node.attributes.get("source"))
        .or_else(|| node.attributes.get("media"))
        .or_else(|| {
            if matches!(component_role.as_str(), "image" | "svg-icon") {
                node.attributes.get("value")
            } else {
                None
            }
        })
        .and_then(value_as_string)
        .unwrap_or_default();
    let icon_name = node
        .attributes
        .get("icon")
        .or_else(|| {
            if component_role.as_str() == "icon" {
                node.attributes.get("value")
            } else {
                None
            }
        })
        .and_then(value_as_string)
        .unwrap_or_default();
    let vector_components = node
        .attributes
        .get("value")
        .and_then(value_as_float_array)
        .unwrap_or_default();
    let validation_level = node
        .attributes
        .get("validation_level")
        .and_then(value_as_string)
        .or_else(|| {
            component_descriptor.map(|_| if disabled { "disabled" } else { "normal" }.to_string())
        })
        .unwrap_or_default();
    let selection_state = node
        .attributes
        .get("selection_state")
        .and_then(value_as_string)
        .or_else(|| {
            node.attributes
                .get("multiple")
                .and_then(value_as_bool)
                .map(|multiple| if multiple { "multi" } else { "single" }.to_string())
        })
        .unwrap_or_default();
    let options = node
        .attributes
        .get("options")
        .and_then(value_as_options)
        .unwrap_or_default();
    let options_text = options.join(", ");
    let collection_items = node
        .attributes
        .get("collection_items")
        .and_then(value_as_options)
        .unwrap_or_default();
    let menu_items = node
        .attributes
        .get("menu_items")
        .and_then(value_as_options)
        .unwrap_or_default();
    let popup_open = node
        .attributes
        .get("popup_open")
        .and_then(value_as_bool)
        .unwrap_or(false);
    let accepted_drag_payloads = component_descriptor
        .map(|descriptor| {
            descriptor
                .drop_policy
                .accepts
                .iter()
                .map(|kind| kind.as_str())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    let action_id = component_descriptor
        .and_then(|_| preferred_showcase_action_id(&control_id, popup_open, &node.bindings))
        .unwrap_or_default();
    let drag_action_id = component_descriptor
        .and_then(|_| preferred_showcase_drag_action_id(&control_id, &node.bindings))
        .unwrap_or_default();
    let begin_drag_action_id = component_descriptor
        .and_then(|_| {
            preferred_showcase_pointer_drag_action_id(&control_id, "DragBegin", &node.bindings)
        })
        .unwrap_or_default();
    let end_drag_action_id = component_descriptor
        .and_then(|_| {
            preferred_showcase_pointer_drag_action_id(&control_id, "DragEnd", &node.bindings)
        })
        .unwrap_or_default();
    let edit_action_id = component_descriptor
        .and_then(|_| preferred_showcase_edit_action_id(&control_id, &node.bindings))
        .unwrap_or_default();
    let actions = if component_descriptor.is_some() {
        preferred_showcase_action_buttons(&control_id, &node.bindings)
    } else {
        Vec::new()
    };
    let dispatch_kind = if !disabled && !action_id.is_empty() {
        "showcase"
    } else {
        ""
    };
    Some(slint_ui::TemplatePaneNodeData {
        node_id: node.node_id.into(),
        control_id: control_id.into(),
        role: component.into(),
        text: node
            .attributes
            .get("text")
            .or_else(|| node.attributes.get("label"))
            .and_then(value_as_string)
            .unwrap_or_default()
            .into(),
        component_role: component_role.into(),
        value_text: value_text.into(),
        value_number: value_number as f32,
        value_percent,
        value_color,
        media_source: media_source.into(),
        icon_name: icon_name.into(),
        vector_components: model_rc(vector_components),
        validation_level: validation_level.into(),
        validation_message: node
            .attributes
            .get("validation_message")
            .and_then(value_as_string)
            .unwrap_or_default()
            .into(),
        popup_open,
        selection_state: selection_state.into(),
        options_text: options_text.into(),
        options: to_slint_shared_string_list(options),
        collection_items: to_slint_shared_string_list(collection_items),
        menu_items: to_slint_shared_string_list(menu_items),
        actions: model_rc(actions),
        accepted_drag_payloads: accepted_drag_payloads.into(),
        checked: node
            .attributes
            .get("checked")
            .or_else(|| node.attributes.get("value"))
            .and_then(value_as_bool)
            .unwrap_or(false),
        expanded: node
            .attributes
            .get("expanded")
            .and_then(value_as_bool)
            .unwrap_or(false),
        focused: node
            .attributes
            .get("focused")
            .and_then(value_as_bool)
            .unwrap_or(false),
        hovered: node
            .attributes
            .get("hovered")
            .and_then(value_as_bool)
            .unwrap_or(false),
        pressed: node
            .attributes
            .get("pressed")
            .and_then(value_as_bool)
            .unwrap_or(false),
        dragging: node
            .attributes
            .get("dragging")
            .and_then(value_as_bool)
            .unwrap_or(false),
        drop_hovered: node
            .attributes
            .get("drop_hovered")
            .and_then(value_as_bool)
            .unwrap_or(false),
        disabled,
        dispatch_kind: dispatch_kind.into(),
        action_id: action_id.into(),
        begin_drag_action_id: begin_drag_action_id.into(),
        drag_action_id: drag_action_id.into(),
        end_drag_action_id: end_drag_action_id.into(),
        edit_action_id: edit_action_id.into(),
        surface_variant: "".into(),
        text_tone: "".into(),
        button_variant: "".into(),
        font_size: 0.0,
        font_weight: 0,
        text_align: "left".into(),
        overflow: "".into(),
        corner_radius: 0.0,
        border_width: 0.0,
        frame: slint_ui::TemplateNodeFrameData {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
    })
}

fn preferred_showcase_action_id(
    control_id: &str,
    popup_open: bool,
    bindings: &[crate::ui::template_runtime::SlintUiHostBindingProjection],
) -> Option<String> {
    let preferred = match control_id {
        "NumberFieldDemo" => Some("NumberFieldDragUpdate"),
        "RangeFieldDemo" => Some("RangeFieldChanged"),
        "DropdownDemo" => Some(if popup_open {
            "DropdownChanged"
        } else {
            "DropdownOpenPopup"
        }),
        "ComboBoxDemo" => Some(if popup_open {
            "ComboBoxChanged"
        } else {
            "ComboBoxOpenPopup"
        }),
        "EnumFieldDemo" => Some(if popup_open {
            "EnumFieldChanged"
        } else {
            "EnumFieldOpenPopup"
        }),
        "FlagsFieldDemo" => Some(if popup_open {
            "FlagsFieldChanged"
        } else {
            "FlagsFieldOpenPopup"
        }),
        "SearchSelectDemo" => Some(if popup_open {
            "SearchSelectChanged"
        } else {
            "SearchSelectOpenPopup"
        }),
        "AssetFieldDemo" => Some("AssetFieldDropped"),
        "InstanceFieldDemo" => Some("InstanceFieldDropped"),
        "ObjectFieldDemo" => Some("ObjectFieldDropped"),
        "GroupDemo" => Some("GroupToggled"),
        "FoldoutDemo" => Some("FoldoutToggled"),
        "ArrayFieldDemo" => Some("ArrayFieldAddElement"),
        "MapFieldDemo" => Some("MapFieldAddEntry"),
        "TreeRowDemo" => Some("TreeRowToggled"),
        "ContextActionMenuDemo" => Some(if popup_open {
            "ContextActionMenuChanged"
        } else {
            "ContextActionMenuOpenPopup"
        }),
        _ => None,
    };
    preferred
        .and_then(|suffix| {
            bindings.iter().find(|binding| {
                binding.binding_id.starts_with("UiComponentShowcase/")
                    && binding.binding_id.ends_with(suffix)
            })
        })
        .or_else(|| {
            bindings
                .iter()
                .find(|binding| binding.binding_id.starts_with("UiComponentShowcase/"))
        })
        .map(|binding| binding.binding_id.clone())
}

fn preferred_showcase_drag_action_id(
    control_id: &str,
    bindings: &[crate::ui::template_runtime::SlintUiHostBindingProjection],
) -> Option<String> {
    let suffix = match control_id {
        "NumberFieldDemo" => Some("NumberFieldDragUpdate"),
        "RangeFieldDemo" => Some("RangeFieldDragUpdate"),
        _ => None,
    }?;
    bindings
        .iter()
        .find(|binding| {
            binding.binding_id.starts_with("UiComponentShowcase/")
                && binding.binding_id.ends_with(suffix)
        })
        .map(|binding| binding.binding_id.clone())
}

fn preferred_showcase_pointer_drag_action_id(
    control_id: &str,
    event_suffix: &str,
    bindings: &[crate::ui::template_runtime::SlintUiHostBindingProjection],
) -> Option<String> {
    let suffix = match (control_id, event_suffix) {
        ("NumberFieldDemo", "DragBegin") => Some("NumberFieldDragBegin"),
        ("NumberFieldDemo", "DragEnd") => Some("NumberFieldDragEnd"),
        _ => None,
    }?;
    bindings
        .iter()
        .find(|binding| {
            binding.binding_id.starts_with("UiComponentShowcase/")
                && binding.binding_id.ends_with(suffix)
        })
        .map(|binding| binding.binding_id.clone())
}

fn preferred_showcase_edit_action_id(
    control_id: &str,
    bindings: &[crate::ui::template_runtime::SlintUiHostBindingProjection],
) -> Option<String> {
    let suffix = match control_id {
        "InputFieldDemo" => Some("InputFieldChanged"),
        "TextFieldDemo" => Some("TextFieldChanged"),
        "NumberFieldDemo" => Some("NumberFieldChanged"),
        "RangeFieldDemo" => Some("RangeFieldChanged"),
        _ => None,
    }?;
    bindings
        .iter()
        .find(|binding| {
            binding.binding_id.starts_with("UiComponentShowcase/")
                && binding.binding_id.ends_with(suffix)
        })
        .map(|binding| binding.binding_id.clone())
}

fn preferred_showcase_action_buttons(
    control_id: &str,
    bindings: &[crate::ui::template_runtime::SlintUiHostBindingProjection],
) -> Vec<slint_ui::TemplatePaneActionData> {
    let actions: &[(&str, &str)] = match control_id {
        "AssetFieldDemo" => &[
            ("Find", "AssetFieldLocate"),
            ("Open", "AssetFieldOpen"),
            ("Clear", "AssetFieldClear"),
        ],
        "ArrayFieldDemo" => &[
            ("Add", "ArrayFieldAddElement"),
            ("Set", "ArrayFieldSetElement"),
            ("Remove", "ArrayFieldRemoveElement"),
            ("Move", "ArrayFieldMoveElement"),
        ],
        "MapFieldDemo" => &[
            ("Add", "MapFieldAddEntry"),
            ("Set", "MapFieldSetEntry"),
            ("Remove", "MapFieldRemoveEntry"),
        ],
        _ => &[],
    };
    actions
        .iter()
        .filter_map(|(label, suffix)| {
            bindings
                .iter()
                .find(|binding| {
                    binding.binding_id.starts_with("UiComponentShowcase/")
                        && binding.binding_id.ends_with(suffix)
                })
                .map(|binding| slint_ui::TemplatePaneActionData {
                    label: (*label).into(),
                    action_id: binding.binding_id.clone().into(),
                })
        })
        .collect()
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

fn runtime_component_registry() -> &'static UiComponentDescriptorRegistry {
    static UI_COMPONENT_REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();
    UI_COMPONENT_REGISTRY.get_or_init(UiComponentDescriptorRegistry::editor_showcase)
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn value_as_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Boolean(value) => Some(*value),
        _ => None,
    }
}

fn value_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Float(value) => Some(*value),
        Value::Integer(value) => Some(*value as f64),
        _ => None,
    }
}

fn value_as_float_array(value: &Value) -> Option<Vec<f32>> {
    let Value::Array(values) = value else {
        return None;
    };
    let components = values
        .iter()
        .filter_map(value_as_f64)
        .map(|value| value as f32)
        .collect::<Vec<_>>();
    if components.is_empty() {
        None
    } else {
        Some(components)
    }
}

fn normalized_value_percent(value: f64, min: Option<f64>, max: Option<f64>) -> f32 {
    match (min, max) {
        (Some(min), Some(max)) if max > min => ((value - min) / (max - min)).clamp(0.0, 1.0) as f32,
        _ => value.clamp(0.0, 1.0) as f32,
    }
}

fn value_as_color(value: &Value) -> Option<slint::Color> {
    parse_hex_color(value_as_string(value)?.as_str())
}

fn parse_hex_color(value: &str) -> Option<slint::Color> {
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        6 => Some(slint::Color::from_rgb_u8(
            parse_hex_pair(&hex[0..2])?,
            parse_hex_pair(&hex[2..4])?,
            parse_hex_pair(&hex[4..6])?,
        )),
        8 => Some(slint::Color::from_argb_u8(
            parse_hex_pair(&hex[6..8])?,
            parse_hex_pair(&hex[0..2])?,
            parse_hex_pair(&hex[2..4])?,
            parse_hex_pair(&hex[4..6])?,
        )),
        _ => None,
    }
}

fn parse_hex_pair(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}

fn value_as_options(value: &Value) -> Option<Vec<String>> {
    let Value::Array(values) = value else {
        return None;
    };
    let options = values
        .iter()
        .filter_map(value_as_string)
        .collect::<Vec<_>>();
    if options.is_empty() {
        None
    } else {
        Some(options)
    }
}

fn to_slint_template_nodes(
    items: Vec<ViewTemplateNodeData>,
) -> ModelRc<slint_ui::TemplatePaneNodeData> {
    model_rc(items.into_iter().map(to_slint_template_node).collect())
}

pub(super) fn to_slint_project_overview_pane(
    data: crate::ui::layouts::windows::workbench_host_window::ProjectOverviewPaneViewData,
) -> slint_ui::ProjectOverviewPaneData {
    slint_ui::ProjectOverviewPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
    }
}

pub(crate) fn to_slint_component_showcase_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> slint_ui::ProjectOverviewPaneData {
    builtin_host_runtime()
        .and_then(|runtime| component_showcase_template_projection(data, content_size, runtime))
        .unwrap_or_default()
}

pub(crate) fn to_slint_component_showcase_pane_from_host_pane_with_runtime(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> slint_ui::ProjectOverviewPaneData {
    component_showcase_template_projection(data, content_size, runtime).unwrap_or_default()
}

fn component_showcase_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> Option<slint_ui::ProjectOverviewPaneData> {
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

    Some(slint_ui::ProjectOverviewPaneData {
        nodes: model_rc(
            host_model
                .nodes
                .into_iter()
                .filter_map(host_template_node)
                .collect(),
        ),
    })
}

fn to_slint_ui_asset_string_selection(
    items: Vec<String>,
    selected_index: i32,
) -> slint_ui::UiAssetStringSelectionData {
    slint_ui::UiAssetStringSelectionData {
        items: to_slint_shared_string_list(items),
        selected_index,
    }
}

fn to_slint_ui_asset_canvas_nodes(
    items: Vec<asset_editor::UiAssetEditorPreviewCanvasNode>,
) -> ModelRc<slint_ui::UiAssetCanvasNodeData> {
    model_rc(
        items
            .into_iter()
            .map(|item| slint_ui::UiAssetCanvasNodeData {
                node_id: item.node_id.into(),
                label: item.label.into(),
                kind: item.kind.into(),
                x: item.x,
                y: item.y,
                width: item.width,
                height: item.height,
                depth: item.depth,
                z_index: item.z_index,
                selected: item.selected,
            })
            .collect(),
    )
}

fn to_slint_ui_asset_canvas_slot_targets(
    items: Vec<asset_editor::UiAssetEditorPreviewCanvasSlotTarget>,
) -> ModelRc<slint_ui::UiAssetCanvasSlotTargetData> {
    model_rc(
        items
            .into_iter()
            .map(|item| slint_ui::UiAssetCanvasSlotTargetData {
                label: item.label.into(),
                detail: item.detail.into(),
                x: item.x,
                y: item.y,
                width: item.width,
                height: item.height,
                selected: item.selected,
            })
            .collect(),
    )
}

pub(super) fn to_slint_ui_asset_pane(
    data: asset_editor::UiAssetEditorPanePresentation,
) -> slint_ui::UiAssetEditorPaneData {
    slint_ui::UiAssetEditorPaneData {
        nodes: to_slint_template_nodes(data.nodes),
        center_column_node: to_slint_template_node(data.center_column_node),
        designer_panel_node: to_slint_template_node(data.designer_panel_node),
        designer_canvas_panel_node: to_slint_template_node(data.designer_canvas_panel_node),
        inspector_panel_node: to_slint_template_node(data.inspector_panel_node),
        stylesheet_panel_node: to_slint_template_node(data.stylesheet_panel_node),
        header: slint_ui::UiAssetPaneHeaderData {
            asset_id: data.asset_id.into(),
            mode: data.mode.into(),
            status: data.last_error.into(),
            selection: data.selection_summary.into(),
        },
        actions: slint_ui::UiAssetActionStateData {
            can_save: data.can_save,
            can_undo: data.can_undo,
            can_redo: data.can_redo,
            can_insert_child: data.can_insert_child,
            can_insert_after: data.can_insert_after,
            can_move_up: data.can_move_up,
            can_move_down: data.can_move_down,
            can_reparent_into_previous: data.can_reparent_into_previous,
            can_reparent_into_next: data.can_reparent_into_next,
            can_reparent_outdent: data.can_reparent_outdent,
            can_open_reference: data.can_open_reference,
            can_convert_to_reference: data.can_convert_to_reference,
            can_extract_component: data.can_extract_component,
            can_promote_to_external_widget: data.can_promote_to_external_widget,
            can_wrap_in_vertical_box: data.can_wrap_in_vertical_box,
            can_unwrap: data.can_unwrap,
            can_create_rule: data.can_create_rule,
            can_extract_rule: data.can_extract_rule,
        },
        collections: slint_ui::UiAssetCollectionPanelData {
            palette: to_slint_ui_asset_string_selection(
                data.palette_items,
                data.palette_selected_index,
            ),
            hierarchy: to_slint_ui_asset_string_selection(
                data.hierarchy_items,
                data.hierarchy_selected_index,
            ),
            preview: to_slint_ui_asset_string_selection(
                data.preview_items,
                data.preview_selected_index,
            ),
        },
        source: slint_ui::UiAssetSourcePanelData {
            text: data.source_text.into(),
            detail: slint_ui::UiAssetSourceDetailData {
                block_label: data.source_selected_block_label.into(),
                selected_line: data.source_selected_line,
                cursor_byte_offset: data.source_cursor_byte_offset,
                selected_excerpt: data.source_selected_excerpt.into(),
                roundtrip_status: data.source_roundtrip_status.into(),
                outline: to_slint_ui_asset_string_selection(
                    data.source_outline_items,
                    data.source_outline_selected_index,
                ),
            },
        },
        preview: slint_ui::UiAssetPreviewPanelData {
            preset: data.preview_preset.into(),
            summary: data.preview_summary.into(),
            available: data.preview_available,
            canvas: slint_ui::UiAssetPreviewCanvasData {
                width: data.preview_surface_width,
                height: data.preview_surface_height,
                items: to_slint_ui_asset_canvas_nodes(data.preview_canvas_items),
            },
            mock: slint_ui::UiAssetPreviewMockData {
                subject_collection: to_slint_ui_asset_string_selection(
                    data.preview_mock_subject_items,
                    data.preview_mock_subject_selected_index,
                ),
                subject_node_id: data.preview_mock_subject_node_id.into(),
                collection: to_slint_ui_asset_string_selection(
                    data.preview_mock_items,
                    data.preview_mock_selected_index,
                ),
                property: data.preview_mock_property.into(),
                kind: data.preview_mock_kind.into(),
                value: data.preview_mock_value.into(),
                expression_result: data.preview_mock_expression_result.into(),
                nested_collection: to_slint_ui_asset_string_selection(
                    data.preview_mock_nested_items,
                    data.preview_mock_nested_selected_index,
                ),
                nested_key: data.preview_mock_nested_key.into(),
                nested_kind: data.preview_mock_nested_kind.into(),
                nested_value: data.preview_mock_nested_value.into(),
                suggestion_collection: to_slint_ui_asset_string_selection(
                    data.preview_mock_suggestion_items,
                    -1,
                ),
                schema_items: to_slint_shared_string_list(data.preview_mock_schema_items),
                state_graph_items: to_slint_shared_string_list(data.preview_state_graph_items),
                can_edit: data.preview_mock_can_edit,
                can_clear: data.preview_mock_can_clear,
                nested_can_edit: data.preview_mock_nested_can_edit,
                nested_can_add: data.preview_mock_nested_can_add,
                nested_can_delete: data.preview_mock_nested_can_delete,
            },
        },
        palette_drag: slint_ui::UiAssetPaletteDragData {
            target_preview_index: data.palette_drag_target_preview_index,
            target_action: data.palette_drag_target_action.into(),
            target_label: data.palette_drag_target_label.into(),
            slot_target_items: to_slint_ui_asset_canvas_slot_targets(
                data.palette_drag_slot_target_items,
            ),
            candidate_items: to_slint_shared_string_list(data.palette_drag_candidate_items),
            candidate_selected_index: data.palette_drag_candidate_selected_index,
            target_chooser_active: data.palette_target_chooser_active,
        },
        style: slint_ui::UiAssetStylePanelData {
            states: slint_ui::UiAssetStyleStateData {
                hover: data.style_state_hover,
                focus: data.style_state_focus,
                pressed: data.style_state_pressed,
                disabled: data.style_state_disabled,
                selected: data.style_state_selected,
            },
            class_items: to_slint_shared_string_list(data.style_class_items),
            theme_source: slint_ui::UiAssetThemeSourceData {
                collection: to_slint_ui_asset_string_selection(
                    data.theme_source_items,
                    data.theme_source_selected_index,
                ),
                selected_source_reference: data.theme_selected_source_reference.into(),
                selected_source_kind: data.theme_selected_source_kind.into(),
                selected_source_token_count: data.theme_selected_source_token_count,
                selected_source_rule_count: data.theme_selected_source_rule_count,
                selected_source_available: data.theme_selected_source_available,
                can_promote_local: data.theme_can_promote_local,
                selected_source_token_items: to_slint_shared_string_list(
                    data.theme_selected_source_token_items,
                ),
                selected_source_rule_items: to_slint_shared_string_list(
                    data.theme_selected_source_rule_items,
                ),
                cascade_layer_items: to_slint_shared_string_list(data.theme_cascade_layer_items),
                cascade_token_items: to_slint_shared_string_list(data.theme_cascade_token_items),
                cascade_rule_items: to_slint_shared_string_list(data.theme_cascade_rule_items),
                compare_items: to_slint_shared_string_list(data.theme_compare_items),
                merge_preview_items: to_slint_shared_string_list(data.theme_merge_preview_items),
                rule_helper_items: to_slint_shared_string_list(data.theme_rule_helper_items),
                refactor_items: to_slint_shared_string_list(data.theme_refactor_items),
                promote_asset_id: data.theme_promote_asset_id.into(),
                promote_document_id: data.theme_promote_document_id.into(),
                promote_display_name: data.theme_promote_display_name.into(),
                can_edit_promote_draft: data.theme_can_edit_promote_draft,
                can_prune_duplicate_local_overrides: data.theme_can_prune_duplicate_local_overrides,
            },
            rule: slint_ui::UiAssetStyleRuleData {
                items: to_slint_shared_string_list(data.style_rule_items),
                selected_index: data.style_rule_selected_index,
                selected_selector: data.style_selected_rule_selector.into(),
                can_edit: data.style_can_edit_rule,
                can_delete: data.style_can_delete_rule,
            },
            matched_rule: slint_ui::UiAssetMatchedStyleRuleData {
                collection: to_slint_ui_asset_string_selection(
                    data.style_matched_rule_items,
                    data.style_matched_rule_selected_index,
                ),
                selected_origin: data.style_selected_matched_rule_origin.into(),
                selected_selector: data.style_selected_matched_rule_selector.into(),
                selected_specificity: data.style_selected_matched_rule_specificity,
                selected_source_order: data.style_selected_matched_rule_source_order,
                selected_declaration_items: to_slint_shared_string_list(
                    data.style_selected_matched_rule_declaration_items,
                ),
            },
            rule_declaration: slint_ui::UiAssetStyleRuleDeclarationData {
                items: to_slint_shared_string_list(data.style_rule_declaration_items),
                selected_index: data.style_rule_declaration_selected_index,
                selected_path: data.style_selected_rule_declaration_path.into(),
                selected_value: data.style_selected_rule_declaration_value.into(),
                can_edit: data.style_can_edit_rule_declaration,
                can_delete: data.style_can_delete_rule_declaration,
            },
            token: slint_ui::UiAssetStyleTokenData {
                items: to_slint_shared_string_list(data.style_token_items),
                selected_index: data.style_token_selected_index,
                selected_name: data.style_selected_token_name.into(),
                selected_value: data.style_selected_token_value.into(),
                can_edit: data.style_can_edit_token,
                can_delete: data.style_can_delete_token,
            },
            can_create_rule: data.can_create_rule,
            can_extract_rule: data.can_extract_rule,
            stylesheet_items: to_slint_shared_string_list(data.stylesheet_items),
        },
        inspector: slint_ui::UiAssetInspectorPanelData {
            widget: slint_ui::UiAssetInspectorWidgetData {
                selected_node_id: data.inspector_selected_node_id.into(),
                parent_node_id: data.inspector_parent_node_id.into(),
                mount: data.inspector_mount.into(),
                widget_kind: data.inspector_widget_kind.into(),
                widget_label: data.inspector_widget_label.into(),
                control_id: data.inspector_control_id.into(),
                text_prop: data.inspector_text_prop.into(),
                can_edit_control_id: data.inspector_can_edit_control_id,
                can_edit_text_prop: data.inspector_can_edit_text_prop,
                promote_asset_id: data.inspector_promote_asset_id.into(),
                promote_component_name: data.inspector_promote_component_name.into(),
                promote_document_id: data.inspector_promote_document_id.into(),
                can_edit_promote_draft: data.inspector_can_edit_promote_draft,
                items: to_slint_shared_string_list(data.inspector_items),
            },
            slot: slint_ui::UiAssetInspectorSlotData {
                padding: data.inspector_slot_padding.into(),
                width_preferred: data.inspector_slot_width_preferred.into(),
                height_preferred: data.inspector_slot_height_preferred.into(),
                semantic: slint_ui::UiAssetInspectorSemanticData {
                    title: data.inspector_slot_semantic_title.into(),
                    collection: to_slint_ui_asset_string_selection(
                        data.inspector_slot_semantic_items,
                        data.inspector_slot_semantic_selected_index,
                    ),
                    path: data.inspector_slot_semantic_path.into(),
                    value: data.inspector_slot_semantic_value.into(),
                },
                kind: data.inspector_slot_kind.into(),
                linear_main_weight: data.inspector_slot_linear_main_weight.into(),
                linear_main_stretch: data.inspector_slot_linear_main_stretch.into(),
                linear_cross_weight: data.inspector_slot_linear_cross_weight.into(),
                linear_cross_stretch: data.inspector_slot_linear_cross_stretch.into(),
                overlay_anchor_x: data.inspector_slot_overlay_anchor_x.into(),
                overlay_anchor_y: data.inspector_slot_overlay_anchor_y.into(),
                overlay_pivot_x: data.inspector_slot_overlay_pivot_x.into(),
                overlay_pivot_y: data.inspector_slot_overlay_pivot_y.into(),
                overlay_position_x: data.inspector_slot_overlay_position_x.into(),
                overlay_position_y: data.inspector_slot_overlay_position_y.into(),
                overlay_z_index: data.inspector_slot_overlay_z_index.into(),
                grid_row: data.inspector_slot_grid_row.into(),
                grid_column: data.inspector_slot_grid_column.into(),
                grid_row_span: data.inspector_slot_grid_row_span.into(),
                grid_column_span: data.inspector_slot_grid_column_span.into(),
                flow_break_before: data.inspector_slot_flow_break_before.into(),
                flow_alignment: data.inspector_slot_flow_alignment.into(),
            },
            layout: slint_ui::UiAssetInspectorLayoutData {
                width_preferred: data.inspector_layout_width_preferred.into(),
                height_preferred: data.inspector_layout_height_preferred.into(),
                semantic: slint_ui::UiAssetInspectorSemanticData {
                    title: data.inspector_layout_semantic_title.into(),
                    collection: to_slint_ui_asset_string_selection(
                        data.inspector_layout_semantic_items,
                        data.inspector_layout_semantic_selected_index,
                    ),
                    path: data.inspector_layout_semantic_path.into(),
                    value: data.inspector_layout_semantic_value.into(),
                },
                kind: data.inspector_layout_kind.into(),
                box_gap: data.inspector_layout_box_gap.into(),
                scroll_axis: data.inspector_layout_scroll_axis.into(),
                scroll_gap: data.inspector_layout_scroll_gap.into(),
                scrollbar_visibility: data.inspector_layout_scrollbar_visibility.into(),
                virtualization_item_extent: data.inspector_layout_virtualization_item_extent.into(),
                virtualization_overscan: data.inspector_layout_virtualization_overscan.into(),
                clip: data.inspector_layout_clip.into(),
            },
            binding: slint_ui::UiAssetInspectorBindingData {
                collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_items,
                    data.inspector_binding_selected_index,
                ),
                binding_id: data.inspector_binding_id.into(),
                binding_event: data.inspector_binding_event.into(),
                event_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_event_items,
                    data.inspector_binding_event_selected_index,
                ),
                binding_route: data.inspector_binding_route.into(),
                binding_route_target: data.inspector_binding_route_target.into(),
                binding_action_target: data.inspector_binding_action_target.into(),
                route_suggestion_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_route_suggestion_items,
                    -1,
                ),
                action_suggestion_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_action_suggestion_items,
                    -1,
                ),
                action_kind_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_action_kind_items,
                    data.inspector_binding_action_kind_selected_index,
                ),
                payload_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_payload_items,
                    data.inspector_binding_payload_selected_index,
                ),
                payload_suggestion_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_payload_suggestion_items,
                    -1,
                ),
                payload_key: data.inspector_binding_payload_key.into(),
                payload_value: data.inspector_binding_payload_value.into(),
                schema_items: to_slint_shared_string_list(data.inspector_binding_schema_items),
                can_edit: data.inspector_can_edit_binding,
                can_delete: data.inspector_can_delete_binding,
            },
        },
    }
}
