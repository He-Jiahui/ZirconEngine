use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, InspectorPluginComponentPropertySnapshot,
    InspectorPluginComponentSnapshot, InspectorSnapshot, SceneEntry,
};

use super::{
    component_property_rows::COMPONENT_PROPERTY_STATIC_CONTROLS,
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const TREE_ROW_INDENT_STEP: f64 = 20.0;
const INSPECTOR_TITLE: &str = "WorkbenchInspectorTitle";
const INSPECTOR_TAGS: &str = "WorkbenchInspectorTags";
const INSPECTOR_TRANSFORM: &str = "WorkbenchInspectorTransform";
const TRANSFORM_POSITION: &str = "WorkbenchTransformPosition";
const TRANSFORM_POSITION_X: &str = "WorkbenchTransformPositionX";
const TRANSFORM_POSITION_Y: &str = "WorkbenchTransformPositionY";
const TRANSFORM_POSITION_Z: &str = "WorkbenchTransformPositionZ";
const INSPECTOR_MESH: &str = "WorkbenchInspectorMesh";
const MESH_LABEL: &str = "WorkbenchMeshLabel";
const MESH_ROW: &str = "WorkbenchMeshRow";
const MATERIAL_ROW: &str = "WorkbenchMaterialRow";
const ADD_COMPONENT: &str = "WorkbenchAddComponent";
const PROPERTY_FIELD_ID: &str = "inspector_property_field_id";
const PROPERTY_NAME: &str = "inspector_property_name";
const PROPERTY_LABEL: &str = "inspector_property_label";
const PROPERTY_VALUE_KIND: &str = "inspector_property_value_kind";
const PROPERTY_EDITABLE: &str = "inspector_property_editable";
const ROW_BACKGROUND_COLOR: &str = "background_color";
const ROW_BORDER_COLOR: &str = "border_color";
const ROW_VALUE_COLOR: &str = "value_color";
const CAST_SHADOWS_FIELD_ID: &str = "cast_shadows";
const CAST_SHADOWS_SELECT_BACKGROUND: &str = "#282e32";
const CAST_SHADOWS_SELECT_BORDER: &str = "#343d43";
const CAST_SHADOWS_SELECT_VALUE: &str = "#b5c0c5";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn sync_from_chrome(
        &mut self,
        chrome: &EditorChromeSnapshot,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.sync_scene_and_inspector(&chrome.scene_entries, chrome.inspector.as_ref())
    }

    pub(crate) fn sync_scene_and_inspector(
        &mut self,
        scene_entries: &[SceneEntry],
        inspector: Option<&InspectorSnapshot>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.sync_scene_entries(scene_entries)?;
        self.sync_inspector(inspector)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }

    fn sync_scene_entries(
        &mut self,
        scene_entries: &[SceneEntry],
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.reconcile_scene_tree_row_capacity(scene_entries.len())?;
        let controls = self.scene_tree_control_ids()?;
        for (index, control_id) in controls.iter().enumerate() {
            let Some(entry) = scene_entries.get(index) else {
                self.set_selected(control_id, false)?;
                self.set_visible(control_id, false)?;
                continue;
            };

            self.set_visible(control_id, true)?;
            self.mutate_control_property(
                control_id,
                "text",
                UiValue::String(non_empty_label(&entry.name, "Entity")),
            )?;
            self.mutate_control_property(
                control_id,
                "tree_depth",
                UiValue::Int(entry.depth as i64),
            )?;
            self.mutate_control_property(
                control_id,
                "tree_indent_px",
                UiValue::Float(entry.depth as f64 * TREE_ROW_INDENT_STEP),
            )?;
            self.mutate_control_property(
                control_id,
                "scene_node_id",
                UiValue::Int(entry.id.min(i64::MAX as u64) as i64),
            )?;
            self.mutate_control_property(
                control_id,
                "expanded",
                UiValue::Bool(row_has_visible_child(scene_entries, index)),
            )?;
            self.set_selected(control_id, entry.selected)?;
        }
        Ok(())
    }

    fn sync_inspector(
        &mut self,
        inspector: Option<&InspectorSnapshot>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(inspector) = inspector else {
            self.mutate_control_property(
                INSPECTOR_TITLE,
                "text",
                UiValue::String("No Selection".to_string()),
            )?;
            self.hide_component_property_rows()?;
            self.set_visible(INSPECTOR_TAGS, false)?;
            self.set_visible(INSPECTOR_TRANSFORM, false)?;
            self.set_visible(INSPECTOR_MESH, false)?;
            self.set_visible(ADD_COMPONENT, false)?;
            return Ok(());
        };

        self.mutate_control_property(
            INSPECTOR_TITLE,
            "text",
            UiValue::String(non_empty_label(&inspector.name, "Entity")),
        )?;
        self.set_visible(INSPECTOR_TAGS, true)?;
        self.set_visible(INSPECTOR_TRANSFORM, true)?;
        self.set_visible(ADD_COMPONENT, true)?;
        self.mutate_control_property(
            TRANSFORM_POSITION,
            "value",
            UiValue::String(format_transform_position(inspector)),
        )?;
        self.sync_transform_position_axis_values(inspector)?;

        let Some(component) = inspector.plugin_components.first() else {
            self.hide_component_property_rows()?;
            self.set_visible(INSPECTOR_MESH, false)?;
            return Ok(());
        };

        self.set_visible(INSPECTOR_MESH, true)?;
        self.mutate_control_property(
            MESH_LABEL,
            "text",
            UiValue::String(non_empty_label(&component.display_name, "Component")),
        )?;
        self.sync_component_property_rows(component)?;
        Ok(())
    }

    fn sync_component_property_rows(
        &mut self,
        component: &InspectorPluginComponentSnapshot,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.reconcile_component_property_row_capacity(component.properties.len())?;
        for (index, control_id) in self
            .component_property_row_control_ids()?
            .iter()
            .enumerate()
        {
            let Some(property) = component.properties.get(index) else {
                self.sync_component_property_row_metadata(control_id, None)?;
                self.mutate_control_property(
                    control_id,
                    "text",
                    UiValue::String(component_property_fallback_label(component, index)),
                )?;
                self.mutate_control_property(
                    control_id,
                    "value_text",
                    UiValue::String(component_property_fallback_value(component, index)),
                )?;
                self.set_visible(control_id, false)?;
                continue;
            };

            self.set_visible(control_id, true)?;
            self.mutate_control_property(
                control_id,
                "text",
                UiValue::String(format_component_property_label(property)),
            )?;
            self.mutate_control_property(
                control_id,
                "value_text",
                UiValue::String(format_component_property_value(property)),
            )?;
            self.sync_component_property_row_metadata(control_id, Some(property))?;
        }
        Ok(())
    }

    fn hide_component_property_rows(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.reconcile_component_property_row_capacity(0)?;
        for control_id in self.component_property_row_control_ids()? {
            self.sync_component_property_row_metadata(&control_id, None)?;
            self.mutate_control_property(&control_id, "text", UiValue::String(String::new()))?;
            self.mutate_control_property(
                &control_id,
                "value_text",
                UiValue::String(String::new()),
            )?;
            self.set_visible(&control_id, false)?;
        }
        Ok(())
    }

    fn sync_component_property_row_metadata(
        &mut self,
        control_id: &str,
        property: Option<&InspectorPluginComponentPropertySnapshot>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let (field_id, name, label, value_kind, editable, value) = property
            .map(|property| {
                (
                    property.field_id.as_str(),
                    property.name.as_str(),
                    property.label.as_str(),
                    property.value_kind.as_str(),
                    property.editable,
                    property.value.as_str(),
                )
            })
            .unwrap_or(("", "", "", "", false, ""));
        self.mutate_control_property(
            control_id,
            PROPERTY_FIELD_ID,
            UiValue::String(field_id.to_string()),
        )?;
        self.mutate_control_property(control_id, PROPERTY_NAME, UiValue::String(name.to_string()))?;
        self.mutate_control_property(
            control_id,
            PROPERTY_LABEL,
            UiValue::String(label.to_string()),
        )?;
        self.mutate_control_property(
            control_id,
            PROPERTY_VALUE_KIND,
            UiValue::String(value_kind.to_string()),
        )?;
        self.mutate_control_property(control_id, PROPERTY_EDITABLE, UiValue::Bool(editable))?;
        self.mutate_control_property(control_id, "value", UiValue::String(value.to_string()))?;
        self.sync_component_property_row_visual_style(control_id, field_id)?;
        Ok(())
    }

    fn sync_component_property_row_visual_style(
        &mut self,
        control_id: &str,
        field_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let (background, border, value) = if field_id == CAST_SHADOWS_FIELD_ID {
            (
                CAST_SHADOWS_SELECT_BACKGROUND,
                CAST_SHADOWS_SELECT_BORDER,
                CAST_SHADOWS_SELECT_VALUE,
            )
        } else {
            ("", "", "")
        };
        self.mutate_control_property(
            control_id,
            ROW_BACKGROUND_COLOR,
            UiValue::String(background.to_string()),
        )?;
        self.mutate_control_property(
            control_id,
            ROW_BORDER_COLOR,
            UiValue::String(border.to_string()),
        )?;
        self.mutate_control_property(
            control_id,
            ROW_VALUE_COLOR,
            UiValue::String(value.to_string()),
        )?;
        Ok(())
    }

    fn sync_transform_position_axis_values(
        &mut self,
        inspector: &InspectorSnapshot,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for (control_id, value) in [
            (TRANSFORM_POSITION_X, inspector.translation[0].as_str()),
            (TRANSFORM_POSITION_Y, inspector.translation[1].as_str()),
            (TRANSFORM_POSITION_Z, inspector.translation[2].as_str()),
        ] {
            self.mutate_control_property(control_id, "value", UiValue::String(value.to_string()))?;
        }
        Ok(())
    }
}

fn row_has_visible_child(scene_entries: &[SceneEntry], index: usize) -> bool {
    let Some(entry) = scene_entries.get(index) else {
        return false;
    };
    scene_entries
        .get(index + 1)
        .map(|next| next.depth > entry.depth)
        .unwrap_or(false)
}

fn format_transform_position(inspector: &InspectorSnapshot) -> String {
    format!(
        "X {}   Y {}   Z {}",
        inspector.translation[0], inspector.translation[1], inspector.translation[2]
    )
}

fn component_property_fallback_label(
    _component: &InspectorPluginComponentSnapshot,
    index: usize,
) -> String {
    match COMPONENT_PROPERTY_STATIC_CONTROLS.get(index).copied() {
        Some(MESH_ROW) => "Component".to_string(),
        Some(MATERIAL_ROW) => "Plugin".to_string(),
        _ => format!("Property {:02}", index + 1),
    }
}

fn component_property_fallback_value(
    component: &InspectorPluginComponentSnapshot,
    index: usize,
) -> String {
    match COMPONENT_PROPERTY_STATIC_CONTROLS.get(index).copied() {
        Some(MESH_ROW) => non_empty_label(&component.component_id, "-"),
        Some(MATERIAL_ROW) => non_empty_label(&component.plugin_id, "-"),
        _ => "-".to_string(),
    }
}

fn format_component_property_label(property: &InspectorPluginComponentPropertySnapshot) -> String {
    non_empty_label(&property.label, &property.name)
}

fn format_component_property_value(property: &InspectorPluginComponentPropertySnapshot) -> String {
    non_empty_label(&property.value, "-")
}

fn non_empty_label(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}
