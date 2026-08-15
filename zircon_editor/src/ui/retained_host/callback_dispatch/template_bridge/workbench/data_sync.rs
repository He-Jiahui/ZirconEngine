use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, InspectorPluginComponentPropertySnapshot,
    InspectorPluginComponentSnapshot, InspectorSnapshot, SceneEntries,
};

use super::{
    component_property_rows::COMPONENT_PROPERTY_STATIC_CONTROLS,
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const INSPECTOR_TITLE: &str = "WorkbenchInspectorTitle";
const INSPECTOR_TAGS: &str = "WorkbenchInspectorTags";
const INSPECTOR_TRANSFORM: &str = "WorkbenchInspectorTransform";
const TRANSFORM_POSITION: &str = "WorkbenchTransformPosition";
const TRANSFORM_POSITION_X: &str = "WorkbenchTransformPositionX";
const TRANSFORM_POSITION_Y: &str = "WorkbenchTransformPositionY";
const TRANSFORM_POSITION_Z: &str = "WorkbenchTransformPositionZ";
const TRANSFORM_SCALE: &str = "WorkbenchTransformScale";
const TRANSFORM_SCALE_X: &str = "WorkbenchTransformScaleX";
const TRANSFORM_SCALE_Y: &str = "WorkbenchTransformScaleY";
const TRANSFORM_SCALE_Z: &str = "WorkbenchTransformScaleZ";
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
// These aliases outrank static background_color/border_color style overrides
// when a snapshot marks a property row as selected.
const ROW_BACKGROUND_COLOR: &str = "background";
const ROW_BORDER_COLOR: &str = "border";
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
        self.prepare_chrome_state_for_layout(chrome)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }

    pub(crate) fn prepare_chrome_state_for_layout(
        &mut self,
        chrome: &EditorChromeSnapshot,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.sync_status_bar(chrome)?;
        self.prepare_scene_and_inspector_state(&chrome.scene_entries, chrome.inspector.as_ref())
    }

    pub(crate) fn sync_scene_and_inspector(
        &mut self,
        scene_entries: &SceneEntries,
        inspector: Option<&InspectorSnapshot>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.prepare_scene_and_inspector_state(scene_entries, inspector)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }

    fn prepare_scene_and_inspector_state(
        &mut self,
        scene_entries: &SceneEntries,
        inspector: Option<&InspectorSnapshot>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.sync_scene_entries(scene_entries, Some(0))?;
        self.sync_inspector(inspector)
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
        self.mutate_control_property(
            TRANSFORM_SCALE,
            "value",
            UiValue::String(format_transform_scale(inspector)),
        )?;
        self.sync_transform_scale_axis_values(inspector)?;

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
                self.sync_component_property_row_metadata(control_id, None, false)?;
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
            self.sync_component_property_row_metadata(
                control_id,
                Some(property),
                component.customization_available,
            )?;
        }
        Ok(())
    }

    fn hide_component_property_rows(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.reconcile_component_property_row_capacity(0)?;
        for control_id in self.component_property_row_control_ids()? {
            self.sync_component_property_row_metadata(&control_id, None, false)?;
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
        customization_available: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let (field_id, name, label, value_kind, editable, value) = property
            .map(|property| {
                (
                    property.field_id.as_str(),
                    property.name.as_str(),
                    property.label.as_str(),
                    property.value_kind.as_str(),
                    property.editable && customization_available,
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

    fn sync_transform_scale_axis_values(
        &mut self,
        inspector: &InspectorSnapshot,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for (control_id, value) in [
            (TRANSFORM_SCALE_X, inspector.scale[0].as_str()),
            (TRANSFORM_SCALE_Y, inspector.scale[1].as_str()),
            (TRANSFORM_SCALE_Z, inspector.scale[2].as_str()),
        ] {
            self.mutate_control_property(control_id, "value", UiValue::String(value.to_string()))?;
        }
        Ok(())
    }
}

fn format_transform_position(inspector: &InspectorSnapshot) -> String {
    format!(
        "X {}   Y {}   Z {}",
        inspector.translation[0], inspector.translation[1], inspector.translation[2]
    )
}

fn format_transform_scale(inspector: &InspectorSnapshot) -> String {
    format!(
        "X {}   Y {}   Z {}",
        inspector.scale[0], inspector.scale[1], inspector.scale[2]
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
