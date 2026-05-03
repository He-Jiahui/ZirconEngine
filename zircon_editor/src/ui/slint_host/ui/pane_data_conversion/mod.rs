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

const MODULE_PLUGIN_ROW_HEIGHT: f32 = 112.0;
const MODULE_PLUGIN_ROW_GAP: f32 = 8.0;
const MODULE_PLUGIN_ROW_PADDING: f32 = 8.0;
const MODULE_PLUGIN_BUTTON_HEIGHT: f32 = 24.0;
const MODULE_PLUGIN_BUTTON_GAP: f32 = 6.0;
const MODULE_PLUGIN_MIN_BUTTON_WIDTH: f32 = 56.0;
const MODULE_PLUGIN_MAX_BUTTON_WIDTH: f32 = 92.0;
const BUILD_EXPORT_ROW_HEIGHT: f32 = 118.0;
const BUILD_EXPORT_ROW_GAP: f32 = 8.0;
const BUILD_EXPORT_ROW_PADDING: f32 = 8.0;
const BUILD_EXPORT_BUTTON_HEIGHT: f32 = 24.0;
const BUILD_EXPORT_BUTTON_WIDTH: f32 = 84.0;
const INSPECTOR_FIELD_ROW_HEIGHT: f32 = 28.0;
const INSPECTOR_FIELD_ROW_GAP: f32 = 6.0;
const INSPECTOR_FIELD_PADDING: f32 = 8.0;
const INSPECTOR_VECTOR_FIELD_GAP: f32 = 6.0;
const INSPECTOR_ACTION_BUTTON_WIDTH: f32 = 84.0;
const INSPECTOR_ACTION_BUTTON_HEIGHT: f32 = 24.0;

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
    content_size: PaneContentSize,
) -> host_contract::InspectorPaneData {
    let fields = InspectorVisualFields::from_view_data(&data);
    let mut nodes = (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .map(to_host_contract_template_node_owned)
        .collect::<Vec<_>>();
    let inspector_nodes = inspector_field_nodes(&fields, &nodes, content_size);
    nodes.extend(inspector_nodes);

    host_contract::InspectorPaneData {
        nodes: model_rc(nodes),
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
    inspector_template_projection(data, content_size).unwrap_or_else(|| {
        to_host_contract_inspector_pane(data.native_body.inspector.clone(), content_size)
    })
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

pub(crate) fn to_host_contract_module_plugins_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> host_contract::ModulePluginsPaneData {
    let native = &data.native_body.module_plugins;
    let mut nodes = module_plugins_template_projection(data, content_size).unwrap_or_default();
    nodes.extend(module_plugin_row_nodes(native, &nodes, content_size));

    host_contract::ModulePluginsPaneData {
        nodes: model_rc(nodes),
        plugins: map_model_rc(&native.plugins, to_host_contract_module_plugin_status),
        diagnostics: native.diagnostics.clone(),
    }
}

pub(crate) fn to_host_contract_build_export_pane_from_host_pane(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> host_contract::BuildExportPaneData {
    let native = &data.native_body.build_export;
    let mut nodes = build_export_template_projection(data, content_size).unwrap_or_default();
    nodes.extend(build_export_target_row_nodes(native, &nodes, content_size));

    host_contract::BuildExportPaneData {
        nodes: model_rc(nodes),
        targets: map_model_rc(&native.targets, to_host_contract_build_export_target),
        diagnostics: native.diagnostics.clone(),
    }
}

fn to_host_contract_build_export_target(
    data: crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData,
) -> host_contract::BuildExportTargetData {
    host_contract::BuildExportTargetData {
        profile_name: data.profile_name,
        platform: data.platform,
        target_mode: data.target_mode,
        strategies: data.strategies,
        status: data.status,
        enabled_plugins: data.enabled_plugins,
        linked_runtime_crates: data.linked_runtime_crates,
        native_dynamic_packages: data.native_dynamic_packages,
        generated_files: data.generated_files,
        diagnostics: data.diagnostics,
        fatal: data.fatal,
    }
}

fn to_host_contract_module_plugin_status(
    data: crate::ui::layouts::windows::workbench_host_window::ModulePluginStatusViewData,
) -> host_contract::ModulePluginStatusData {
    host_contract::ModulePluginStatusData {
        plugin_id: data.plugin_id,
        display_name: data.display_name,
        package_source: data.package_source,
        load_state: data.load_state,
        enabled: data.enabled,
        required: data.required,
        target_modes: data.target_modes,
        packaging: data.packaging,
        runtime_crate: data.runtime_crate,
        editor_crate: data.editor_crate,
        runtime_capabilities: data.runtime_capabilities,
        editor_capabilities: data.editor_capabilities,
        optional_features: data.optional_features,
        feature_action_label: data.feature_action_label,
        feature_action_id: data.feature_action_id,
        diagnostics: data.diagnostics,
        primary_action_label: data.primary_action_label,
        primary_action_id: data.primary_action_id,
        packaging_action_label: data.packaging_action_label,
        packaging_action_id: data.packaging_action_id,
        target_modes_action_label: data.target_modes_action_label,
        target_modes_action_id: data.target_modes_action_id,
        unload_action_label: data.unload_action_label,
        unload_action_id: data.unload_action_id,
        hot_reload_action_label: data.hot_reload_action_label,
        hot_reload_action_id: data.hot_reload_action_id,
    }
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

    let inspector_name = projection
        .root
        .attributes
        .get("payload_name")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.name.clone());
    let inspector_parent = projection
        .root
        .attributes
        .get("payload_parent")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.parent.clone());
    let inspector_x = projection
        .root
        .attributes
        .get("payload_translation_x")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.translation[0].clone());
    let inspector_y = projection
        .root
        .attributes
        .get("payload_translation_y")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.translation[1].clone());
    let inspector_z = projection
        .root
        .attributes
        .get("payload_translation_z")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.translation[2].clone());
    let delete_enabled = projection
        .root
        .attributes
        .get("payload_delete_enabled")
        .and_then(value_as_bool)
        .unwrap_or(payload.delete_enabled);
    let fields = InspectorVisualFields {
        info: data.info.to_string(),
        name: inspector_name.clone(),
        parent: inspector_parent.clone(),
        x: inspector_x.clone(),
        y: inspector_y.clone(),
        z: inspector_z.clone(),
        delete_enabled,
        plugin_components: payload
            .plugin_components
            .iter()
            .map(|component| {
                crate::ui::layouts::windows::workbench_host_window::InspectorPluginComponentViewData {
                    component_id: component.component_id.clone(),
                    display_name: component.display_name.clone(),
                    plugin_id: component.plugin_id.clone(),
                    drawer_available: component.drawer_available,
                    diagnostic: component.diagnostic.clone(),
                    properties: component
                        .properties
                        .iter()
                        .map(|property| {
                            crate::ui::layouts::windows::workbench_host_window::InspectorPluginComponentPropertyViewData {
                                field_id: property.field_id.clone(),
                                name: property.name.clone(),
                                label: property.label.clone(),
                                value: property.value.clone(),
                                value_kind: property.value_kind.clone(),
                                editable: property.editable,
                            }
                        })
                        .collect(),
                }
            })
            .collect(),
    };
    let mut nodes = host_model
        .nodes
        .into_iter()
        .filter_map(host_template_node)
        .collect::<Vec<_>>();
    let inspector_nodes = inspector_field_nodes(&fields, &nodes, content_size);
    nodes.extend(inspector_nodes);

    Some(host_contract::InspectorPaneData {
        nodes: model_rc(nodes),
        info: data.info.clone(),
        inspector_name: inspector_name.into(),
        inspector_parent: inspector_parent.into(),
        inspector_x: inspector_x.into(),
        inspector_y: inspector_y.into(),
        inspector_z: inspector_z.into(),
        delete_enabled,
    })
}

struct InspectorVisualFields {
    info: String,
    name: String,
    parent: String,
    x: String,
    y: String,
    z: String,
    delete_enabled: bool,
    plugin_components:
        Vec<crate::ui::layouts::windows::workbench_host_window::InspectorPluginComponentViewData>,
}

impl InspectorVisualFields {
    fn from_view_data(
        data: &crate::ui::layouts::windows::workbench_host_window::InspectorPaneViewData,
    ) -> Self {
        Self {
            info: data.info.to_string(),
            name: data.inspector_name.to_string(),
            parent: data.inspector_parent.to_string(),
            x: data.inspector_x.to_string(),
            y: data.inspector_y.to_string(),
            z: data.inspector_z.to_string(),
            delete_enabled: data.delete_enabled,
            plugin_components: data.plugin_components.clone(),
        }
    }

    fn has_selection(&self) -> bool {
        self.delete_enabled || !self.name.trim().is_empty()
    }
}

fn inspector_field_nodes(
    fields: &InspectorVisualFields,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let body_frame = inspector_body_frame(template_nodes, content_size);
    let width = if body_frame.width > 0.0 {
        body_frame.width
    } else {
        content_size.width.max(0.0)
    };
    let field_width = (width - INSPECTOR_FIELD_PADDING * 2.0).max(0.0);
    let start_x = body_frame.x + INSPECTOR_FIELD_PADDING;
    let start_y = body_frame.y + INSPECTOR_FIELD_PADDING;
    let field_disabled = !fields.has_selection();
    let mut nodes = Vec::new();

    let panel_height = inspector_field_panel_height(fields);
    let mut panel = inspector_node(
        "inspector_field_panel",
        "InspectorEditableFieldsPanel",
        "Panel",
        "Inspector",
        host_contract::TemplateNodeFrameData {
            x: body_frame.x,
            y: body_frame.y,
            width,
            height: panel_height,
        },
    );
    panel.surface_variant = "inspector-fields".into();
    nodes.push(panel);

    nodes.push(inspector_text_field_node(
        "name",
        "NameField",
        "Name",
        &fields.name,
        "InspectorView/NameField",
        start_x,
        start_y,
        field_width,
        field_disabled,
    ));
    nodes.push(inspector_text_field_node(
        "parent",
        "ParentField",
        "Parent",
        &fields.parent,
        "InspectorView/ParentField",
        start_x,
        start_y + INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP,
        field_width,
        field_disabled,
    ));

    let transform_label_y = start_y + (INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP) * 2.0;
    let mut transform_label = inspector_node(
        "inspector_transform_label",
        "InspectorTransformLabel",
        "Label",
        "Transform",
        host_contract::TemplateNodeFrameData {
            x: start_x,
            y: transform_label_y,
            width: field_width,
            height: 18.0,
        },
    );
    transform_label.text_tone = "muted".into();
    nodes.push(transform_label);

    let vector_y = transform_label_y + 20.0;
    let vector_width = ((field_width - INSPECTOR_VECTOR_FIELD_GAP * 2.0) / 3.0).max(0.0);
    nodes.push(inspector_number_field_node(
        "position_x",
        "PositionXField",
        "X",
        &fields.x,
        "InspectorView/PositionXField",
        start_x,
        vector_y,
        vector_width,
        field_disabled,
    ));
    nodes.push(inspector_number_field_node(
        "position_y",
        "PositionYField",
        "Y",
        &fields.y,
        "InspectorView/PositionYField",
        start_x + vector_width + INSPECTOR_VECTOR_FIELD_GAP,
        vector_y,
        vector_width,
        field_disabled,
    ));
    nodes.push(inspector_number_field_node(
        "position_z",
        "PositionZField",
        "Z",
        &fields.z,
        "InspectorView/PositionZField",
        start_x + (vector_width + INSPECTOR_VECTOR_FIELD_GAP) * 2.0,
        vector_y,
        vector_width,
        field_disabled,
    ));

    let mut next_y = vector_y + INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP;
    let plugin_nodes =
        inspector_plugin_component_nodes(fields, start_x, next_y, field_width, field_disabled);
    if !plugin_nodes.is_empty() {
        next_y += inspector_plugin_component_height(fields);
        nodes.extend(plugin_nodes);
    }

    if let Some(message) = inspector_plugin_component_fallback_message(&fields.info) {
        let mut diagnostic = inspector_node(
            "inspector_plugin_component_fallback",
            "InspectorPluginComponentFallback",
            "Diagnostic",
            "Plugin component protected",
            host_contract::TemplateNodeFrameData {
                x: start_x,
                y: next_y,
                width: field_width,
                height: INSPECTOR_FIELD_ROW_HEIGHT,
            },
        );
        diagnostic.value_text = fields.info.clone().into();
        diagnostic.validation_level = "warning".into();
        diagnostic.validation_message = message.into();
        diagnostic.text_tone = "warning".into();
        diagnostic.disabled = true;
        nodes.push(diagnostic);
        next_y += INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP;
    } else if field_disabled {
        let mut empty = inspector_node(
            "inspector_empty_selection_hint",
            "InspectorEmptySelectionHint",
            "Label",
            "No scene entity selected",
            host_contract::TemplateNodeFrameData {
                x: start_x,
                y: next_y,
                width: field_width,
                height: 20.0,
            },
        );
        empty.text_tone = "muted".into();
        nodes.push(empty);
        next_y += 20.0 + INSPECTOR_FIELD_ROW_GAP;
    }

    nodes.push(inspector_action_button_node(
        "apply",
        "ApplyBatchButton",
        "Apply",
        "InspectorView/ApplyBatchButton",
        start_x,
        next_y,
        field_disabled,
    ));
    nodes.push(inspector_action_button_node(
        "delete",
        "DeleteSelected",
        "Delete",
        "InspectorView/DeleteSelected",
        start_x + INSPECTOR_ACTION_BUTTON_WIDTH + INSPECTOR_FIELD_ROW_GAP,
        next_y,
        !fields.delete_enabled,
    ));

    nodes
}

fn inspector_field_panel_height(fields: &InspectorVisualFields) -> f32 {
    let base_rows = 4.0;
    let diagnostic_rows = if inspector_plugin_component_fallback_message(&fields.info).is_some()
        || !fields.has_selection()
    {
        1.0
    } else {
        0.0
    };
    INSPECTOR_FIELD_PADDING * 2.0
        + base_rows * INSPECTOR_FIELD_ROW_HEIGHT
        + (base_rows + diagnostic_rows) * INSPECTOR_FIELD_ROW_GAP
        + diagnostic_rows * INSPECTOR_FIELD_ROW_HEIGHT
        + inspector_plugin_component_height(fields)
        + INSPECTOR_ACTION_BUTTON_HEIGHT
}

fn inspector_plugin_component_nodes(
    fields: &InspectorVisualFields,
    x: f32,
    mut y: f32,
    width: f32,
    field_disabled: bool,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let mut nodes = Vec::new();
    for component in &fields.plugin_components {
        let component_key = inspector_component_key(&component.component_id);
        let mut header = inspector_node(
            format!("inspector_plugin_component_header_{component_key}"),
            format!("PluginComponentHeader:{}", component.component_id),
            "Label",
            component.display_name.clone(),
            host_contract::TemplateNodeFrameData {
                x,
                y,
                width,
                height: 20.0,
            },
        );
        header.text_tone = if component.drawer_available {
            "strong".into()
        } else {
            "warning".into()
        };
        nodes.push(header);
        y += 20.0 + INSPECTOR_FIELD_ROW_GAP;

        if let Some(diagnostic) = &component.diagnostic {
            let mut diagnostic_node = inspector_node(
                format!("inspector_plugin_component_diagnostic_{component_key}"),
                format!("PluginComponentDiagnostic:{}", component.component_id),
                "Diagnostic",
                "Plugin component protected",
                host_contract::TemplateNodeFrameData {
                    x,
                    y,
                    width,
                    height: INSPECTOR_FIELD_ROW_HEIGHT,
                },
            );
            diagnostic_node.value_text = diagnostic.clone().into();
            diagnostic_node.validation_level = "warning".into();
            diagnostic_node.validation_message = diagnostic.clone().into();
            diagnostic_node.text_tone = "warning".into();
            diagnostic_node.disabled = true;
            nodes.push(diagnostic_node);
            y += INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP;
        }

        for property in &component.properties {
            let control_id = inspector_dynamic_component_control_id(&property.field_id);
            let disabled = field_disabled || !component.drawer_available || !property.editable;
            let mut node = if inspector_numeric_kind(&property.value_kind) {
                inspector_number_field_node(
                    &inspector_component_key(&property.field_id),
                    &control_id,
                    property.label.as_str(),
                    &property.value,
                    &format!("InspectorView/{control_id}"),
                    x,
                    y,
                    width,
                    disabled,
                )
            } else {
                inspector_text_field_node(
                    &inspector_component_key(&property.field_id),
                    &control_id,
                    property.label.as_str(),
                    &property.value,
                    &format!("InspectorView/{control_id}"),
                    x,
                    y,
                    width,
                    disabled,
                )
            };
            if !component.drawer_available {
                node.validation_level = "warning".into();
                node.validation_message = component
                    .diagnostic
                    .clone()
                    .unwrap_or_else(|| "Plugin component drawer unavailable".to_string())
                    .into();
            }
            nodes.push(node);
            y += INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP;
        }
    }
    nodes
}

fn inspector_plugin_component_height(fields: &InspectorVisualFields) -> f32 {
    fields
        .plugin_components
        .iter()
        .map(|component| {
            let diagnostic_rows = if component.diagnostic.is_some() {
                1.0
            } else {
                0.0
            };
            20.0 + INSPECTOR_FIELD_ROW_GAP
                + diagnostic_rows * (INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP)
                + component.properties.len() as f32
                    * (INSPECTOR_FIELD_ROW_HEIGHT + INSPECTOR_FIELD_ROW_GAP)
        })
        .sum()
}

fn inspector_dynamic_component_control_id(field_id: &str) -> String {
    format!("DynamicComponentField:{field_id}")
}

fn inspector_component_key(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn inspector_numeric_kind(value_kind: &str) -> bool {
    matches!(
        value_kind.to_ascii_lowercase().as_str(),
        "number"
            | "float"
            | "scalar"
            | "real"
            | "double"
            | "integer"
            | "int"
            | "signed"
            | "unsigned"
            | "u32"
            | "u64"
            | "i32"
            | "i64"
    )
}

fn inspector_body_frame(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> host_contract::TemplateNodeFrameData {
    template_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "InspectorBodySection")
        .map(|node| node.frame.clone())
        .filter(|frame| frame.width > 0.0 || frame.height > 0.0)
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: content_size.width.max(0.0),
            height: content_size.height.max(0.0),
        })
}

fn inspector_text_field_node(
    suffix: &str,
    control_id: &str,
    label: &str,
    value: &str,
    edit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    disabled: bool,
) -> host_contract::TemplatePaneNodeData {
    let mut node = inspector_node(
        format!("inspector_field_{suffix}"),
        control_id,
        "InputField",
        label,
        host_contract::TemplateNodeFrameData {
            x,
            y,
            width,
            height: INSPECTOR_FIELD_ROW_HEIGHT,
        },
    );
    node.component_role = "input-field".into();
    node.value_text = value.to_string().into();
    node.edit_action_id = edit_action_id.to_string().into();
    node.commit_action_id = "InspectorView/ApplyBatchButton".into();
    node.disabled = disabled;
    node.surface_variant = "inspector-field".into();
    node.corner_radius = 4.0;
    node.border_width = 1.0;
    node
}

fn inspector_number_field_node(
    suffix: &str,
    control_id: &str,
    label: &str,
    value: &str,
    edit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    disabled: bool,
) -> host_contract::TemplatePaneNodeData {
    let mut node = inspector_text_field_node(
        suffix,
        control_id,
        label,
        value,
        edit_action_id,
        x,
        y,
        width,
        disabled,
    );
    node.role = "NumberField".into();
    node.component_role = "number-field".into();
    node.value_number = value.parse::<f32>().unwrap_or(0.0);
    node
}

fn inspector_action_button_node(
    suffix: &str,
    control_id: &str,
    label: &str,
    action_id: &str,
    x: f32,
    y: f32,
    disabled: bool,
) -> host_contract::TemplatePaneNodeData {
    let mut node = inspector_node(
        format!("inspector_action_{suffix}"),
        control_id,
        "Button",
        label,
        host_contract::TemplateNodeFrameData {
            x,
            y,
            width: INSPECTOR_ACTION_BUTTON_WIDTH,
            height: INSPECTOR_ACTION_BUTTON_HEIGHT,
        },
    );
    let dispatch_kind = if disabled { "" } else { "inspector" };
    node.dispatch_kind = dispatch_kind.into();
    node.action_id = action_id.to_string().into();
    node.button_variant = "secondary".into();
    node.disabled = disabled;
    node
}

fn inspector_plugin_component_fallback_message(info: &str) -> Option<String> {
    let lower = info.to_ascii_lowercase();
    let mentions_plugin_component =
        lower.contains("plugin") || lower.contains("component drawer") || lower.contains("drawer");
    let mentions_unavailable = lower.contains("unloaded")
        || lower.contains("missing")
        || lower.contains("unavailable")
        || lower.contains("disabled");
    (mentions_plugin_component && mentions_unavailable).then(|| {
        "Plugin component drawer is unavailable; serialized component data stays protected until the plugin reloads."
            .to_string()
    })
}

fn inspector_node(
    node_id: impl Into<String>,
    control_id: impl Into<String>,
    role: impl Into<String>,
    text: impl Into<String>,
    frame: host_contract::TemplateNodeFrameData,
) -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        node_id: node_id.into().into(),
        control_id: control_id.into().into(),
        role: role.into().into(),
        text: text.into().into(),
        frame,
        ..host_contract::TemplatePaneNodeData::default()
    }
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

fn module_plugins_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::ModulePluginsV1(_)
    ) {
        return None;
    }

    project_pane_template_nodes(&presentation.body, content_size)
}

fn build_export_template_projection(
    data: &crate::ui::layouts::windows::workbench_host_window::PaneData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        crate::ui::layouts::windows::workbench_host_window::PanePayload::BuildExportV1(_)
    ) {
        return None;
    }

    project_pane_template_nodes(&presentation.body, content_size)
}

fn build_export_target_row_nodes(
    data: &crate::ui::layouts::windows::workbench_host_window::BuildExportPaneViewData,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let list_frame = template_nodes
        .iter()
        .find(|node| {
            matches!(
                node.control_id.as_str(),
                "BuildExportTargetsSlotAnchor" | "BuildExportTargetsPanel"
            )
        })
        .map(|node| node.frame.clone())
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: content_size.width.max(0.0),
            height: content_size.height.max(0.0),
        });
    let list_width = list_frame.width.max(content_size.width).max(0.0);
    let mut nodes = Vec::new();

    for row in 0..data.targets.row_count() {
        let Some(target) = data.targets.row_data(row) else {
            continue;
        };
        let platform_id = target.platform.to_string().to_ascii_lowercase();
        let row_y = list_frame.y + row as f32 * (BUILD_EXPORT_ROW_HEIGHT + BUILD_EXPORT_ROW_GAP);
        let export_busy = matches!(
            target.status.as_str(),
            "Queued" | "Running" | "Cancel requested"
        );
        let action_label = if export_busy { "Cancel" } else { "Export" };
        let action_id = if export_busy {
            format!("BuildExport.Cancel.{}", target.profile_name)
        } else {
            format!("BuildExport.Execute.{}", target.profile_name)
        };
        let mut row_node = module_plugin_node(
            format!("build_export_row_{platform_id}"),
            format!("BuildExportRow.{platform_id}"),
            "Panel",
            target.profile_name.to_string(),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x,
                y: row_y,
                width: list_width,
                height: BUILD_EXPORT_ROW_HEIGHT,
            },
        );
        row_node.surface_variant = if target.fatal {
            "diagnostic-error".into()
        } else {
            "build-export-row".into()
        };
        row_node.corner_radius = 6.0;
        row_node.border_width = 1.0;
        row_node.actions = model_rc(vec![host_contract::TemplatePaneActionData {
            label: action_label.into(),
            action_id: action_id.clone().into(),
        }]);
        nodes.push(row_node);

        nodes.push(module_plugin_node(
            format!("build_export_title_{platform_id}"),
            format!("BuildExportTitle.{platform_id}"),
            "Label",
            format!("{} | {}", target.platform, target.status),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
                y: row_y + 8.0,
                width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
                height: 20.0,
            },
        ));

        let mut strategy = module_plugin_node(
            format!("build_export_strategy_{platform_id}"),
            format!("BuildExportStrategy.{platform_id}"),
            "Label",
            format!("{} | {}", target.target_mode, target.strategies),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
                y: row_y + 30.0,
                width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
                height: 18.0,
            },
        );
        strategy.text_tone = "muted".into();
        nodes.push(strategy);

        let mut counts = module_plugin_node(
            format!("build_export_counts_{platform_id}"),
            format!("BuildExportCounts.{platform_id}"),
            "Label",
            format!(
                "plugins {} | linked {} | native {} | files {}",
                target.enabled_plugins,
                target.linked_runtime_crates,
                target.native_dynamic_packages,
                target.generated_files
            ),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
                y: row_y + 48.0,
                width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
                height: 18.0,
            },
        );
        counts.text_tone = "muted".into();
        nodes.push(counts);

        if !target.diagnostics.is_empty() {
            let mut diagnostics = module_plugin_node(
                format!("build_export_diagnostics_{platform_id}"),
                format!("BuildExportDiagnostics.{platform_id}"),
                "Label",
                target.diagnostics.to_string(),
                host_contract::TemplateNodeFrameData {
                    x: list_frame.x + BUILD_EXPORT_ROW_PADDING,
                    y: row_y + 66.0,
                    width: (list_width - BUILD_EXPORT_ROW_PADDING * 2.0).max(0.0),
                    height: 18.0,
                },
            );
            diagnostics.text_tone = if target.fatal { "danger" } else { "muted" }.into();
            nodes.push(diagnostics);
        }

        nodes.push(build_export_action_button_node(
            &platform_id,
            action_label,
            &action_id,
            row_y,
            list_frame.x,
            list_width,
            target.fatal,
        ));
    }

    nodes
}

fn build_export_action_button_node(
    platform_id: &str,
    label: &str,
    action_id: &str,
    row_y: f32,
    row_x: f32,
    row_width: f32,
    disabled: bool,
) -> host_contract::TemplatePaneNodeData {
    let mut node = module_plugin_node(
        format!("build_export_action_{platform_id}"),
        "BuildExportAction",
        "Button",
        label,
        host_contract::TemplateNodeFrameData {
            x: row_x + row_width - BUILD_EXPORT_ROW_PADDING - BUILD_EXPORT_BUTTON_WIDTH,
            y: row_y + BUILD_EXPORT_ROW_HEIGHT
                - BUILD_EXPORT_ROW_PADDING
                - BUILD_EXPORT_BUTTON_HEIGHT,
            width: BUILD_EXPORT_BUTTON_WIDTH,
            height: BUILD_EXPORT_BUTTON_HEIGHT,
        },
    );
    node.dispatch_kind = "build_export".into();
    node.action_id = action_id.into();
    node.button_variant = "primary".into();
    node.disabled = disabled;
    node
}

fn module_plugin_row_nodes(
    data: &crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let list_frame = template_nodes
        .iter()
        .find(|node| {
            matches!(
                node.control_id.as_str(),
                "ModulePluginListSlotAnchor" | "ModulePluginListPanel"
            )
        })
        .map(|node| node.frame.clone())
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: content_size.width.max(0.0),
            height: content_size.height.max(0.0),
        });
    let list_width = list_frame.width.max(content_size.width).max(0.0);
    let mut nodes = Vec::new();

    for row in 0..data.plugins.row_count() {
        let Some(plugin) = data.plugins.row_data(row) else {
            continue;
        };
        let plugin_id = plugin.plugin_id.to_string();
        let row_y = list_frame.y + row as f32 * (MODULE_PLUGIN_ROW_HEIGHT + MODULE_PLUGIN_ROW_GAP);
        let row_frame = host_contract::TemplateNodeFrameData {
            x: list_frame.x,
            y: row_y,
            width: list_width,
            height: MODULE_PLUGIN_ROW_HEIGHT,
        };
        let actions = module_plugin_row_actions(&plugin);
        let mut row_node = module_plugin_node(
            format!("module_plugin_row_{plugin_id}"),
            format!("ModulePluginRow.{plugin_id}"),
            "Panel",
            plugin.display_name.to_string(),
            row_frame.clone(),
        );
        row_node.surface_variant = "module-plugin-row".into();
        row_node.corner_radius = 6.0;
        row_node.border_width = 1.0;
        row_node.actions = model_rc(
            actions
                .iter()
                .map(|action| host_contract::TemplatePaneActionData {
                    label: action.label.clone().into(),
                    action_id: action.action_id.clone().into(),
                })
                .collect(),
        );
        nodes.push(row_node);

        nodes.push(module_plugin_node(
            format!("module_plugin_title_{plugin_id}"),
            format!("ModulePluginTitle.{plugin_id}"),
            "Label",
            plugin.display_name.to_string(),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + MODULE_PLUGIN_ROW_PADDING,
                y: row_y + 8.0,
                width: (list_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0),
                height: 20.0,
            },
        ));

        let mut meta = module_plugin_node(
            format!("module_plugin_meta_{plugin_id}"),
            format!("ModulePluginMeta.{plugin_id}"),
            "Label",
            format!(
                "{} | {} | {} | {}",
                plugin.package_source, plugin.load_state, plugin.packaging, plugin.target_modes
            ),
            host_contract::TemplateNodeFrameData {
                x: list_frame.x + MODULE_PLUGIN_ROW_PADDING,
                y: row_y + 30.0,
                width: (list_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0),
                height: 18.0,
            },
        );
        meta.text_tone = "muted".into();
        nodes.push(meta);

        let mut detail_y = row_y + 48.0;
        if !plugin.optional_features.is_empty() {
            let mut features = module_plugin_node(
                format!("module_plugin_features_{plugin_id}"),
                format!("ModulePluginFeatures.{plugin_id}"),
                "Label",
                plugin.optional_features.to_string(),
                host_contract::TemplateNodeFrameData {
                    x: list_frame.x + MODULE_PLUGIN_ROW_PADDING,
                    y: detail_y,
                    width: (list_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0),
                    height: 16.0,
                },
            );
            features.text_tone = "muted".into();
            nodes.push(features);
            detail_y += 18.0;
        }

        if !plugin.diagnostics.is_empty() {
            let mut diagnostics = module_plugin_node(
                format!("module_plugin_diagnostics_{plugin_id}"),
                format!("ModulePluginDiagnostics.{plugin_id}"),
                "Label",
                plugin.diagnostics.to_string(),
                host_contract::TemplateNodeFrameData {
                    x: list_frame.x + MODULE_PLUGIN_ROW_PADDING,
                    y: detail_y,
                    width: (list_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0),
                    height: 16.0,
                },
            );
            diagnostics.text_tone = "warning".into();
            nodes.push(diagnostics);
        }

        nodes.extend(module_plugin_action_button_nodes(
            &plugin_id,
            row_y,
            list_frame.x,
            list_width,
            &actions,
        ));
    }

    nodes
}

struct ModulePluginRowAction {
    label: String,
    action_id: String,
}

fn module_plugin_row_actions(
    plugin: &crate::ui::layouts::windows::workbench_host_window::ModulePluginStatusViewData,
) -> Vec<ModulePluginRowAction> {
    [
        (
            plugin.primary_action_label.to_string(),
            plugin.primary_action_id.to_string(),
        ),
        (
            plugin.feature_action_label.to_string(),
            plugin.feature_action_id.to_string(),
        ),
        (
            plugin.packaging_action_label.to_string(),
            plugin.packaging_action_id.to_string(),
        ),
        (
            plugin.target_modes_action_label.to_string(),
            plugin.target_modes_action_id.to_string(),
        ),
        (
            plugin.unload_action_label.to_string(),
            plugin.unload_action_id.to_string(),
        ),
        (
            plugin.hot_reload_action_label.to_string(),
            plugin.hot_reload_action_id.to_string(),
        ),
    ]
    .into_iter()
    .filter(|(label, _)| !label.is_empty())
    .map(|(label, action_id)| ModulePluginRowAction { label, action_id })
    .collect()
}

fn module_plugin_action_button_nodes(
    plugin_id: &str,
    row_y: f32,
    row_x: f32,
    row_width: f32,
    actions: &[ModulePluginRowAction],
) -> Vec<host_contract::TemplatePaneNodeData> {
    if actions.is_empty() {
        return Vec::new();
    }

    let available_width = (row_width - MODULE_PLUGIN_ROW_PADDING * 2.0).max(0.0);
    let gap_total = MODULE_PLUGIN_BUTTON_GAP * actions.len().saturating_sub(1) as f32;
    let button_width = ((available_width - gap_total) / actions.len() as f32).clamp(
        MODULE_PLUGIN_MIN_BUTTON_WIDTH,
        MODULE_PLUGIN_MAX_BUTTON_WIDTH,
    );
    let start_x = row_x + MODULE_PLUGIN_ROW_PADDING;
    let button_y =
        row_y + MODULE_PLUGIN_ROW_HEIGHT - MODULE_PLUGIN_ROW_PADDING - MODULE_PLUGIN_BUTTON_HEIGHT;

    actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let mut node = module_plugin_node(
                format!("module_plugin_action_{plugin_id}_{index}"),
                "ModulePluginAction",
                "Button",
                compact_module_plugin_action_label(&action.label),
                host_contract::TemplateNodeFrameData {
                    x: start_x + index as f32 * (button_width + MODULE_PLUGIN_BUTTON_GAP),
                    y: button_y,
                    width: button_width,
                    height: MODULE_PLUGIN_BUTTON_HEIGHT,
                },
            );
            node.dispatch_kind = "module_plugin".into();
            node.action_id = action.action_id.clone().into();
            node.button_variant = "secondary".into();
            node.disabled = action.action_id.is_empty();
            node
        })
        .collect()
}

fn compact_module_plugin_action_label(label: &str) -> String {
    if label == "Cycle targets" {
        return "Targets".to_string();
    }
    if label.starts_with("Cycle ") {
        return "Package".to_string();
    }
    match label {
        "Hot Reload" => "Reload".to_string(),
        "Enable Deps" => "Deps".to_string(),
        "Enable Feature" => "Feature".to_string(),
        "Disable Feature" => "Feature Off".to_string(),
        other => other.to_string(),
    }
}

fn module_plugin_node(
    node_id: impl Into<String>,
    control_id: impl Into<String>,
    role: impl Into<String>,
    text: impl Into<String>,
    frame: host_contract::TemplateNodeFrameData,
) -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        node_id: node_id.into().into(),
        control_id: control_id.into().into(),
        role: role.into().into(),
        text: text.into().into(),
        frame,
        ..host_contract::TemplatePaneNodeData::default()
    }
}

#[cfg(test)]
mod inspector_pane_tests {
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::layouts::views::blank_viewport_chrome;
    use crate::ui::layouts::windows::workbench_host_window::{
        InspectorPaneViewData, InspectorPluginComponentPropertyViewData,
        InspectorPluginComponentViewData, PaneData, PaneNativeBodyData,
    };
    use slint::{Model, ModelRc};

    #[test]
    fn inspector_pane_projects_editable_field_nodes_and_actions() {
        let pane = inspector_pane_fixture("scene entity selected");
        let data = to_host_contract_inspector_pane_from_host_pane(
            &pane,
            PaneContentSize::new(360.0, 240.0),
        );

        let name = find_node(&data.nodes, "NameField");
        assert_eq!(name.role.as_str(), "InputField");
        assert_eq!(name.value_text.as_str(), "Camera");
        assert_eq!(name.edit_action_id.as_str(), "InspectorView/NameField");
        assert_eq!(
            name.commit_action_id.as_str(),
            "InspectorView/ApplyBatchButton"
        );
        assert!(!name.disabled);

        let position_x = find_node(&data.nodes, "PositionXField");
        assert_eq!(position_x.role.as_str(), "NumberField");
        assert_eq!(position_x.value_text.as_str(), "1.25");
        assert_eq!(
            position_x.edit_action_id.as_str(),
            "InspectorView/PositionXField"
        );

        let apply = find_node(&data.nodes, "ApplyBatchButton");
        assert_eq!(apply.role.as_str(), "Button");
        assert_eq!(apply.action_id.as_str(), "InspectorView/ApplyBatchButton");
        assert!(!apply.disabled);

        let delete = find_node(&data.nodes, "DeleteSelected");
        assert_eq!(delete.action_id.as_str(), "InspectorView/DeleteSelected");
        assert!(!delete.disabled);
    }

    #[test]
    fn inspector_pane_marks_plugin_component_drawer_fallback() {
        let pane = inspector_pane_fixture(
            "plugin component drawer unavailable: particles plugin unloaded",
        );
        let data = to_host_contract_inspector_pane_from_host_pane(
            &pane,
            PaneContentSize::new(360.0, 240.0),
        );

        let fallback = find_node(&data.nodes, "InspectorPluginComponentFallback");
        assert_eq!(fallback.role.as_str(), "Diagnostic");
        assert_eq!(fallback.validation_level.as_str(), "warning");
        assert!(fallback
            .validation_message
            .as_str()
            .contains("serialized component data stays protected"));
        assert!(fallback.disabled);
    }

    #[test]
    fn inspector_pane_projects_plugin_component_drawer_fields_and_unload_degradation() {
        let mut pane = inspector_pane_fixture("scene entity selected");
        pane.native_body.inspector.plugin_components = vec![
            InspectorPluginComponentViewData {
                component_id: "weather.Component.CloudLayer".to_string(),
                display_name: "Cloud Layer".to_string(),
                plugin_id: "weather".to_string(),
                drawer_available: true,
                diagnostic: None,
                properties: vec![InspectorPluginComponentPropertyViewData {
                    field_id: "weather.Component.CloudLayer.coverage".to_string(),
                    name: "coverage".to_string(),
                    label: "Coverage".to_string(),
                    value: "0.75".to_string(),
                    value_kind: "scalar".to_string(),
                    editable: true,
                }],
            },
            InspectorPluginComponentViewData {
                component_id: "particles.Component.Emitter".to_string(),
                display_name: "Emitter".to_string(),
                plugin_id: "particles".to_string(),
                drawer_available: false,
                diagnostic: Some(
                    "Plugin component drawer unavailable for `particles.Component.Emitter`; serialized data stays protected until the plugin reloads."
                        .to_string(),
                ),
                properties: vec![InspectorPluginComponentPropertyViewData {
                    field_id: "particles.Component.Emitter.rate".to_string(),
                    name: "rate".to_string(),
                    label: "Rate".to_string(),
                    value: "12".to_string(),
                    value_kind: "integer".to_string(),
                    editable: false,
                }],
            },
        ];

        let data = to_host_contract_inspector_pane_from_host_pane(
            &pane,
            PaneContentSize::new(360.0, 320.0),
        );

        let coverage = find_node(
            &data.nodes,
            "DynamicComponentField:weather.Component.CloudLayer.coverage",
        );
        assert_eq!(coverage.role.as_str(), "NumberField");
        assert_eq!(coverage.value_text.as_str(), "0.75");
        assert_eq!(
            coverage.edit_action_id.as_str(),
            "InspectorView/DynamicComponentField:weather.Component.CloudLayer.coverage"
        );
        assert!(!coverage.disabled);

        let degraded = find_node(
            &data.nodes,
            "DynamicComponentField:particles.Component.Emitter.rate",
        );
        assert!(degraded.disabled);
        assert_eq!(degraded.validation_level.as_str(), "warning");
        assert!(degraded
            .validation_message
            .as_str()
            .contains("serialized data stays protected"));
    }

    fn find_node(
        nodes: &ModelRc<host_contract::TemplatePaneNodeData>,
        control_id: &str,
    ) -> host_contract::TemplatePaneNodeData {
        (0..nodes.row_count())
            .filter_map(|row| nodes.row_data(row))
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("{control_id} node should be projected"))
    }

    fn inspector_pane_fixture(info: &str) -> PaneData {
        PaneData {
            id: "editor.inspector#1".into(),
            slot: "right".into(),
            kind: "Inspector".into(),
            title: "Inspector".into(),
            icon_key: "inspector".into(),
            subtitle: "Selection".into(),
            info: info.into(),
            show_empty: false,
            empty_title: "".into(),
            empty_body: "".into(),
            primary_action_label: "".into(),
            primary_action_id: "".into(),
            secondary_action_label: "".into(),
            secondary_action_id: "".into(),
            secondary_hint: "".into(),
            show_toolbar: false,
            viewport: blank_viewport_chrome(),
            native_body: PaneNativeBodyData {
                inspector: InspectorPaneViewData {
                    nodes: model_rc(Vec::new()),
                    info: info.into(),
                    inspector_name: "Camera".into(),
                    inspector_parent: "Root".into(),
                    inspector_x: "1.25".into(),
                    inspector_y: "2.50".into(),
                    inspector_z: "3.75".into(),
                    delete_enabled: true,
                    plugin_components: Vec::new(),
                },
                ..PaneNativeBodyData::default()
            },
            pane_presentation: None,
        }
    }
}

#[cfg(test)]
mod module_plugin_pane_tests {
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::layouts::views::blank_viewport_chrome;
    use crate::ui::layouts::windows::workbench_host_window::{
        ModulePluginStatusViewData, ModulePluginsPaneViewData, PaneData, PaneNativeBodyData,
    };
    use slint::Model;

    #[test]
    fn module_plugins_pane_projects_visual_rows_and_action_buttons() {
        let pane = module_plugins_pane_fixture();
        let data = to_host_contract_module_plugins_pane_from_host_pane(
            &pane,
            PaneContentSize::new(480.0, 260.0),
        );

        assert_eq!(data.plugins.row_count(), 1);
        assert_eq!(
            data.plugins
                .row_data(0)
                .map(|plugin| plugin.plugin_id.to_string()),
            Some("physics".to_string())
        );

        let action_ids = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .filter(|node| node.control_id.as_str() == "ModulePluginAction")
            .map(|node| node.action_id.to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            action_ids,
            vec![
                "Plugin.Disable.physics",
                "Plugin.Feature.Enable.physics.physics.raycast_queries",
                "Plugin.Packaging.Next.physics",
                "Plugin.TargetModes.Next.physics",
                "Plugin.Unload.physics",
                "Plugin.HotReload.physics",
            ]
        );

        let row_node = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "ModulePluginRow.physics")
            .expect("module plugin row should be projected");
        assert_eq!(row_node.actions.row_count(), 6);
        let feature_node = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "ModulePluginFeatures.physics")
            .expect("module plugin optional feature summary should be projected");
        assert_eq!(feature_node.text.to_string(), "Ray Cast Queries [ready]");
    }

    fn module_plugins_pane_fixture() -> PaneData {
        let module_plugins = module_plugins_fixture();
        PaneData {
            id: "editor.module_plugins#1".into(),
            slot: "left_bottom".into(),
            kind: "ModulePlugins".into(),
            title: "Plugin Manager".into(),
            icon_key: "plugin".into(),
            subtitle: "Project Plugins".into(),
            info: "Builtin and native plugin packages".into(),
            show_empty: false,
            empty_title: "".into(),
            empty_body: "".into(),
            primary_action_label: "".into(),
            primary_action_id: "".into(),
            secondary_action_label: "".into(),
            secondary_action_id: "".into(),
            secondary_hint: "".into(),
            show_toolbar: false,
            viewport: blank_viewport_chrome(),
            native_body: PaneNativeBodyData {
                module_plugins,
                ..PaneNativeBodyData::default()
            },
            pane_presentation: None,
        }
    }

    fn module_plugins_fixture() -> ModulePluginsPaneViewData {
        ModulePluginsPaneViewData {
            plugins: model_rc(vec![module_plugin_status_fixture()]),
            diagnostics: "plugin catalog ready".into(),
        }
    }

    fn module_plugin_status_fixture() -> ModulePluginStatusViewData {
        ModulePluginStatusViewData {
            plugin_id: "physics".into(),
            display_name: "Physics".into(),
            package_source: "builtin".into(),
            load_state: "loaded".into(),
            enabled: true,
            required: false,
            target_modes: "editor, runtime".into(),
            packaging: "linked".into(),
            runtime_crate: "zircon_plugins_physics_runtime".into(),
            editor_crate: "zircon_plugins_physics_editor".into(),
            runtime_capabilities: "simulation".into(),
            editor_capabilities: "inspector".into(),
            optional_features: "Ray Cast Queries [ready]".into(),
            feature_action_label: "Enable Feature".into(),
            feature_action_id: "Plugin.Feature.Enable.physics.physics.raycast_queries".into(),
            diagnostics: "".into(),
            primary_action_label: "Disable".into(),
            primary_action_id: "Plugin.Disable.physics".into(),
            packaging_action_label: "Cycle linked".into(),
            packaging_action_id: "Plugin.Packaging.Next.physics".into(),
            target_modes_action_label: "Cycle targets".into(),
            target_modes_action_id: "Plugin.TargetModes.Next.physics".into(),
            unload_action_label: "Unload".into(),
            unload_action_id: "Plugin.Unload.physics".into(),
            hot_reload_action_label: "Hot Reload".into(),
            hot_reload_action_id: "Plugin.HotReload.physics".into(),
        }
    }
}

#[cfg(test)]
mod build_export_pane_tests {
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::layouts::views::blank_viewport_chrome;
    use crate::ui::layouts::windows::workbench_host_window::{
        BuildExportPaneViewData, BuildExportTargetViewData, PaneData, PaneNativeBodyData,
    };
    use slint::Model;

    #[test]
    fn build_export_pane_projects_desktop_target_rows() {
        let pane = PaneData {
            id: "editor.build_export_desktop#1".into(),
            slot: "bottom_right".into(),
            kind: "BuildExport".into(),
            title: "Desktop Export".into(),
            icon_key: "build-export".into(),
            subtitle: "Desktop Targets".into(),
            info: "Windows, Linux, and macOS export plans".into(),
            show_empty: false,
            empty_title: "".into(),
            empty_body: "".into(),
            primary_action_label: "".into(),
            primary_action_id: "".into(),
            secondary_action_label: "".into(),
            secondary_action_id: "".into(),
            secondary_hint: "".into(),
            show_toolbar: false,
            viewport: blank_viewport_chrome(),
            native_body: PaneNativeBodyData {
                build_export: BuildExportPaneViewData {
                    targets: model_rc(vec![BuildExportTargetViewData {
                        profile_name: "desktop_windows".into(),
                        platform: "Windows".into(),
                        target_mode: "ClientRuntime".into(),
                        strategies: "SourceTemplate, LibraryEmbed, NativeDynamic".into(),
                        status: "Ready".into(),
                        enabled_plugins: "2".into(),
                        linked_runtime_crates: "1".into(),
                        native_dynamic_packages: "1".into(),
                        generated_files: "5".into(),
                        diagnostics: "native plugin package ready".into(),
                        fatal: false,
                    }]),
                    diagnostics: "export ready".into(),
                },
                ..PaneNativeBodyData::default()
            },
            pane_presentation: None,
        };
        let data = to_host_contract_build_export_pane_from_host_pane(
            &pane,
            PaneContentSize::new(520.0, 180.0),
        );

        assert_eq!(data.targets.row_count(), 1);
        let row_node = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "BuildExportRow.windows")
            .expect("desktop export target row should be projected");
        assert_eq!(row_node.text.to_string(), "desktop_windows");
        assert_eq!(row_node.actions.row_count(), 1);
        assert_eq!(
            row_node.actions.row_data(0).map(|action| action.action_id),
            Some("BuildExport.Execute.desktop_windows".into())
        );
        let counts = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "BuildExportCounts.windows")
            .expect("desktop export target counts should be projected");
        assert!(counts.text.to_string().contains("native 1"));
        let action = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "BuildExportAction")
            .expect("desktop export target action should be projected");
        assert_eq!(
            action.action_id.as_str(),
            "BuildExport.Execute.desktop_windows"
        );
        assert!(!action.disabled);
    }

    #[test]
    fn build_export_running_target_projects_cancel_action() {
        let pane = PaneData {
            id: "editor.build_export_desktop#1".into(),
            slot: "bottom_right".into(),
            kind: "BuildExport".into(),
            title: "Desktop Export".into(),
            icon_key: "build-export".into(),
            subtitle: "Desktop Targets".into(),
            info: "Windows, Linux, and macOS export plans".into(),
            show_empty: false,
            empty_title: "".into(),
            empty_body: "".into(),
            primary_action_label: "".into(),
            primary_action_id: "".into(),
            secondary_action_label: "".into(),
            secondary_action_id: "".into(),
            secondary_hint: "".into(),
            show_toolbar: false,
            viewport: blank_viewport_chrome(),
            native_body: PaneNativeBodyData {
                build_export: BuildExportPaneViewData {
                    targets: model_rc(vec![BuildExportTargetViewData {
                        profile_name: "desktop_linux".into(),
                        platform: "Linux".into(),
                        target_mode: "ClientRuntime".into(),
                        strategies: "SourceTemplate, LibraryEmbed, NativeDynamic".into(),
                        status: "Running".into(),
                        enabled_plugins: "2".into(),
                        linked_runtime_crates: "1".into(),
                        native_dynamic_packages: "1".into(),
                        generated_files: "5".into(),
                        diagnostics: "Progress: export backend is running".into(),
                        fatal: false,
                    }]),
                    diagnostics: "export ready".into(),
                },
                ..PaneNativeBodyData::default()
            },
            pane_presentation: None,
        };
        let data = to_host_contract_build_export_pane_from_host_pane(
            &pane,
            PaneContentSize::new(520.0, 180.0),
        );

        let row_node = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "BuildExportRow.linux")
            .expect("running desktop export target row should be projected");
        assert_eq!(row_node.actions.row_count(), 1);
        let row_action = row_node
            .actions
            .row_data(0)
            .expect("running row should expose cancel action");
        assert_eq!(row_action.label.as_str(), "Cancel");
        assert_eq!(
            row_action.action_id.as_str(),
            "BuildExport.Cancel.desktop_linux"
        );
        let button = (0..data.nodes.row_count())
            .filter_map(|row| data.nodes.row_data(row))
            .find(|node| node.control_id.as_str() == "BuildExportAction")
            .expect("running desktop export action should be projected");
        assert_eq!(button.text.as_str(), "Cancel");
        assert_eq!(
            button.action_id.as_str(),
            "BuildExport.Cancel.desktop_linux"
        );
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
            | Some("ModulePluginListPanel")
            | Some("ModulePluginListSlotAnchor")
            | Some("BuildExportTargetsPanel")
            | Some("BuildExportTargetsSlotAnchor")
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
