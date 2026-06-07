use toml::Value;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    binding::UiEventKind, template::UiBindingRef, tree::UiTemplateNodeMetadata,
};

use super::super::virtual_rows::{
    TemplateBridgeVirtualRowContext, TemplateBridgeVirtualRowSequence,
};
use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const COMPONENT_PROPERTY_CONTAINER: &str = "WorkbenchInspectorMesh";
const VIRTUAL_ROW_EDIT_BINDING_ID: &str = "Inspector/ComponentProperty04Edit";
const VIRTUAL_ROW_EDIT_ROUTE: &str = "inspector.component_property_04.edit";
const VIRTUAL_ROW_COMMIT_BINDING_ID: &str = "Inspector/ComponentProperty04Commit";
const VIRTUAL_ROW_COMMIT_ROUTE: &str = "inspector.component_property_04.commit";

const PROPERTY_FIELD_ID: &str = "inspector_property_field_id";
const PROPERTY_NAME: &str = "inspector_property_name";
const PROPERTY_LABEL: &str = "inspector_property_label";
const PROPERTY_VALUE_KIND: &str = "inspector_property_value_kind";
const PROPERTY_EDITABLE: &str = "inspector_property_editable";

pub(super) const COMPONENT_PROPERTY_STATIC_CONTROLS: &[&str] = &[
    "WorkbenchMeshRow",
    "WorkbenchMaterialRow",
    "WorkbenchComponentPropertySlot03Row",
    "WorkbenchComponentPropertySlot04Row",
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn reconcile_component_property_row_capacity(
        &mut self,
        property_count: usize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        component_property_virtual_rows(&self.template_surface.surface)?
            .reconcile(
                &mut self.template_surface.surface,
                property_count,
                virtual_metadata_from_prototype,
            )
            .map_err(BuiltinHostWindowTemplateBridgeError::from)
    }

    pub(super) fn component_property_row_control_ids(
        &self,
    ) -> Result<Vec<String>, BuiltinHostWindowTemplateBridgeError> {
        let mut controls = COMPONENT_PROPERTY_STATIC_CONTROLS
            .iter()
            .map(|control_id| (*control_id).to_string())
            .collect::<Vec<_>>();
        controls.extend(
            component_property_virtual_rows(&self.template_surface.surface)?
                .virtual_control_ids(&self.template_surface.surface),
        );
        Ok(controls)
    }

    pub(crate) fn is_component_property_row(
        &self,
        control_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        Ok(COMPONENT_PROPERTY_STATIC_CONTROLS.contains(&control_id)
            || component_property_virtual_rows(&self.template_surface.surface)?
                .contains_control(&self.template_surface.surface, control_id))
    }
}

fn component_property_virtual_rows(
    surface: &UiSurface,
) -> Result<TemplateBridgeVirtualRowSequence, BuiltinHostWindowTemplateBridgeError> {
    TemplateBridgeVirtualRowSequence::from_surface_repeat(surface, COMPONENT_PROPERTY_CONTAINER)
        .map_err(BuiltinHostWindowTemplateBridgeError::from)
}

fn virtual_metadata_from_prototype(
    mut metadata: UiTemplateNodeMetadata,
    context: &TemplateBridgeVirtualRowContext,
) -> UiTemplateNodeMetadata {
    let label = format!("Property {:02}", context.row_number);
    metadata
        .attributes
        .insert("text".to_string(), Value::String(label.clone()));
    metadata
        .attributes
        .insert("value".to_string(), Value::String(String::new()));
    metadata
        .attributes
        .insert("value_text".to_string(), Value::String("-".to_string()));
    metadata.attributes.insert(
        "visibility".to_string(),
        Value::String("collapsed".to_string()),
    );
    metadata
        .attributes
        .insert(PROPERTY_FIELD_ID.to_string(), Value::String(String::new()));
    metadata
        .attributes
        .insert(PROPERTY_NAME.to_string(), Value::String(String::new()));
    metadata
        .attributes
        .insert(PROPERTY_LABEL.to_string(), Value::String(label));
    metadata.attributes.insert(
        PROPERTY_VALUE_KIND.to_string(),
        Value::String(String::new()),
    );
    metadata
        .attributes
        .insert(PROPERTY_EDITABLE.to_string(), Value::Boolean(false));
    metadata.bindings = vec![
        UiBindingRef {
            id: VIRTUAL_ROW_EDIT_BINDING_ID.to_string(),
            event: UiEventKind::Change,
            route: Some(VIRTUAL_ROW_EDIT_ROUTE.to_string()),
            action: None,
            targets: Vec::new(),
        },
        UiBindingRef {
            id: VIRTUAL_ROW_COMMIT_BINDING_ID.to_string(),
            event: UiEventKind::Submit,
            route: Some(VIRTUAL_ROW_COMMIT_ROUTE.to_string()),
            action: None,
            targets: Vec::new(),
        },
    ];
    metadata
}
