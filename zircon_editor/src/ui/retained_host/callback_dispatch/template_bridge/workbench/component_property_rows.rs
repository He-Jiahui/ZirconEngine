use toml::Value;
use zircon_runtime::ui::surface::{UiSurface, UiVirtualListItemKey};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    surface::{UiPointerEventKind, UiPointerRoute},
    template::UiBindingRef,
    tree::UiTemplateNodeMetadata,
};

use super::super::virtual_rows::{
    TemplateBridgeVirtualRowBinding, TemplateBridgeVirtualRowContext,
    TemplateBridgeVirtualRowReconcile, TemplateBridgeVirtualRowSequence,
};
use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};
use crate::ui::workbench::snapshot::InspectorPluginComponentPropertySnapshot;

const COMPONENT_PROPERTY_CONTAINER: &str = "WorkbenchInspectorMeshProperties";
const VIRTUAL_ROW_EDIT_BINDING_ID: &str = "Inspector/ComponentProperty04Edit";
const VIRTUAL_ROW_EDIT_ROUTE: &str = "inspector.component_property_04.edit";
const VIRTUAL_ROW_COMMIT_BINDING_ID: &str = "Inspector/ComponentProperty04Commit";
const VIRTUAL_ROW_COMMIT_ROUTE: &str = "inspector.component_property_04.commit";

const PROPERTY_FIELD_ID: &str = "inspector_property_field_id";
const PROPERTY_NAME: &str = "inspector_property_name";
const PROPERTY_LABEL: &str = "inspector_property_label";
const PROPERTY_VALUE_KIND: &str = "inspector_property_value_kind";
const PROPERTY_EDITABLE: &str = "inspector_property_editable";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn component_property_row_bindings(
        &mut self,
        item_keys: &[UiVirtualListItemKey],
    ) -> Result<Vec<TemplateBridgeVirtualRowBinding>, BuiltinHostWindowTemplateBridgeError> {
        let (rows, _) = self.reconcile_component_property_rows(item_keys)?;
        rows.bindings(&self.template_surface.surface)
            .map_err(BuiltinHostWindowTemplateBridgeError::from)
    }

    pub(crate) fn refresh_component_property_rows_after_scroll(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if route.kind != UiPointerEventKind::Scroll || route.scroll_delta == 0.0 {
            return Ok(false);
        }
        let properties = self.component_properties.clone();
        let item_keys = self.component_property_keys.clone();
        let customization_available = self.component_customization_available;
        let (rows, reconcile) = self.reconcile_component_property_rows(item_keys.as_ref())?;
        let bindings = rows
            .bindings_for_changes(&self.template_surface.surface, reconcile.changes.as_slice())?;
        for binding in &bindings {
            self.sync_component_property_binding(
                binding,
                properties.as_ref(),
                customization_available,
            )?;
        }
        if self
            .template_surface
            .surface
            .pending_invalidation_changed_node_count()
            == 0
        {
            return Ok(false);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.workbench.inspector.virtual_row_scroll_changed_slot_count",
            bindings.len()
        );
        Ok(true)
    }

    fn reconcile_component_property_rows(
        &mut self,
        item_keys: &[UiVirtualListItemKey],
    ) -> Result<
        (
            TemplateBridgeVirtualRowSequence,
            TemplateBridgeVirtualRowReconcile,
        ),
        BuiltinHostWindowTemplateBridgeError,
    > {
        let rows = component_property_virtual_rows(&self.template_surface.surface)?;
        let reconcile = rows.reconcile_with_keys(
            &mut self.template_surface.surface,
            item_keys.len(),
            |logical_index| item_keys[logical_index],
            virtual_metadata_from_prototype,
        )?;
        if reconcile.topology_changed {
            self.template_surface.refresh_control_node_index()?;
        }
        Ok((rows, reconcile))
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
    let label = format!("Property slot {:02}", context.slot_number);
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
            component_event: None,
            id: VIRTUAL_ROW_EDIT_BINDING_ID.to_string(),
            event: UiEventKind::Change,
            mode: Default::default(),
            route: Some(VIRTUAL_ROW_EDIT_ROUTE.to_string()),
            action: None,
            targets: Vec::new(),
        },
        UiBindingRef {
            component_event: None,
            id: VIRTUAL_ROW_COMMIT_BINDING_ID.to_string(),
            event: UiEventKind::Submit,
            mode: Default::default(),
            route: Some(VIRTUAL_ROW_COMMIT_ROUTE.to_string()),
            action: None,
            targets: Vec::new(),
        },
    ];
    metadata
}

fn component_property_item_key(
    property: &InspectorPluginComponentPropertySnapshot,
    logical_index: usize,
) -> UiVirtualListItemKey {
    const FNV_OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    const FNV_PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;
    let identity = if property.field_id.is_empty() {
        property.name.as_bytes()
    } else {
        property.field_id.as_bytes()
    };
    if identity.is_empty() {
        return UiVirtualListItemKey::new(logical_index as u128);
    }
    let mut hash = FNV_OFFSET;
    for byte in identity {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    UiVirtualListItemKey::new(hash)
}

pub(super) fn component_property_item_keys(
    properties: &[InspectorPluginComponentPropertySnapshot],
) -> Vec<UiVirtualListItemKey> {
    properties
        .iter()
        .enumerate()
        .map(|(logical_index, property)| component_property_item_key(property, logical_index))
        .collect()
}
