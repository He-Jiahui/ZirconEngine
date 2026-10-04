use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiBindingUpdate, UiBindingUpdateStatus},
    component::UiValue,
    event_ui::UiNodeId,
    tree::{UiDirtyFlags, UiTree, UiTreeError},
};

use crate::ui::binding::reflected_property_update_with_source_kind;

use super::metadata_dirty::metadata_attribute_dirty;

#[cfg(test)]
#[path = "metadata_batch/tests/optimization_tests.rs"]
mod optimization_tests;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMetadataPropertyChange {
    pub(crate) property: String,
    pub(crate) value: UiValue,
    pub(crate) dirty: UiDirtyFlags,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiMetadataPropertyBatchMutation {
    pub(crate) changes: Vec<UiMetadataPropertyChange>,
    pub(crate) reflected_updates: Vec<UiBindingUpdate>,
    pub(crate) dirty: UiDirtyFlags,
}

pub(crate) fn mutate_tree_metadata_properties<P>(
    tree: &mut UiTree,
    node_id: UiNodeId,
    properties: impl IntoIterator<Item = (P, UiValue)>,
    source_kind: UiBindingSourceKind,
) -> Result<UiMetadataPropertyBatchMutation, UiTreeError>
where
    P: AsRef<str> + Into<String>,
{
    let node = tree
        .node_mut(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    let Some(metadata) = node.template_metadata.as_mut() else {
        return Ok(UiMetadataPropertyBatchMutation::default());
    };

    let mut batch = UiMetadataPropertyBatchMutation::default();
    for (property, value) in properties {
        let property_name = property.as_ref();
        metadata.localized_text_references.remove(property_name);
        // Caret/selection projections include the unchanged owned text value.
        // Compare borrowed bodies before allocating their TOML representation.
        let string_equal = match (&value, metadata.attributes.get(property_name)) {
            (UiValue::String(proposed), Some(toml::Value::String(current))) => {
                Some(current == proposed)
            }
            _ => None,
        };
        if string_equal == Some(true) {
            continue;
        }
        let next = value.to_toml();
        if string_equal.is_none() && metadata.attributes.get(property_name) == Some(&next) {
            continue;
        }

        let previous = metadata
            .attributes
            .get(property_name)
            .map(UiValue::from_toml);
        set_metadata_value(&mut metadata.attributes, property_name, next);
        let dirty =
            metadata_attribute_dirty(metadata.component.as_str(), property_name, value.kind());
        merge_dirty_flags(&mut batch.dirty, dirty);
        batch
            .reflected_updates
            .push(reflected_property_update_with_source_kind(
                node_id,
                property_name,
                source_kind,
                previous,
                value.clone(),
                dirty,
                UiBindingUpdateStatus::Applied,
                None,
            ));
        let property = property.into();
        batch.changes.push(UiMetadataPropertyChange {
            property,
            value,
            dirty,
        });
    }

    Ok(batch)
}

fn set_metadata_value(
    attributes: &mut BTreeMap<String, toml::Value>,
    property: &str,
    value: toml::Value,
) {
    if let Some(existing) = attributes.get_mut(property) {
        *existing = value;
    } else {
        attributes.insert(property.to_string(), value);
    }
}

fn merge_dirty_flags(target: &mut UiDirtyFlags, dirty: UiDirtyFlags) {
    target.layout |= dirty.layout;
    target.hit_test |= dirty.hit_test;
    target.render |= dirty.render;
    target.style |= dirty.style;
    target.text |= dirty.text;
    target.input |= dirty.input;
    target.visible_range |= dirty.visible_range;
}

#[cfg(test)]
#[path = "tests/metadata_batch.rs"]
mod tests;
