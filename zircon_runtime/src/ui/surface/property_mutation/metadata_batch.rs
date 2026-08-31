use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiBindingUpdate, UiBindingUpdateStatus},
    component::UiValue,
    event_ui::UiNodeId,
    tree::{UiDirtyFlags, UiTree, UiTreeError},
};

use crate::ui::binding::reflected_property_update_with_source_kind;

use super::metadata_dirty::metadata_attribute_dirty;

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
        let next = value.to_toml();
        if metadata.attributes.get(property_name) == Some(&next) {
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
mod tests {
    use std::collections::BTreeMap;

    use zircon_runtime_interface::ui::{
        event_ui::{UiNodePath, UiTreeId},
        tree::{UiTemplateNodeMetadata, UiTreeNode},
    };

    use super::*;

    #[test]
    fn virtual_window_metadata_batch_skips_unchanged_aliases_and_merges_dirty_once() {
        let node_id = UiNodeId::new(2);
        let mut tree = UiTree::new(UiTreeId::new("runtime.ui.virtual_window.batch"));
        tree.insert_root(
            UiTreeNode::new(node_id, UiNodePath::new("root/table")).with_template_metadata(
                UiTemplateNodeMetadata {
                    component: "Table".to_string(),
                    attributes: BTreeMap::from([
                        ("viewport_start".to_string(), toml::Value::Integer(0)),
                        ("viewport_count".to_string(), toml::Value::Integer(4)),
                    ]),
                    ..UiTemplateNodeMetadata::default()
                },
            ),
        );

        let batch = mutate_tree_metadata_properties(
            &mut tree,
            node_id,
            [
                ("viewport_start", UiValue::Int(2)),
                ("viewport_count", UiValue::Int(4)),
                ("visible_end", UiValue::Int(6)),
            ],
            UiBindingSourceKind::WidgetBehavior,
        )
        .expect("metadata batch should apply");

        assert_eq!(batch.changes.len(), 2);
        assert_eq!(batch.reflected_updates.len(), 2);
        assert!(
            batch
                .reflected_updates
                .iter()
                .all(|update| update.status == UiBindingUpdateStatus::Applied)
        );
        assert!(batch.dirty.layout);
        assert!(batch.dirty.hit_test);
        assert!(batch.dirty.render);
        assert!(batch.dirty.input);
        assert!(batch.dirty.visible_range);
        assert!(!tree.node(node_id).expect("node").dirty.any());
        assert_eq!(
            tree.node(node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .and_then(|metadata| metadata.attributes.get("viewport_start")),
            Some(&toml::Value::Integer(2))
        );
    }
}
