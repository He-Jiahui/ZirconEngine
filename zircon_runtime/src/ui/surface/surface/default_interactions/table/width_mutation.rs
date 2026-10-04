use zircon_runtime_interface::ui::{
    binding::{
        UiBindingDirtyDomain, UiBindingSourceKind, UiBindingUpdateReport, UiBindingUpdateStatus,
    },
    component::UiValue,
    event_ui::UiNodeId,
    tree::{UiDirtyFlags, UiTreeError},
};

use crate::ui::{
    binding::component_state_value_update,
    surface::{property_mutation::mutate_tree_metadata_properties, UiSurface},
};

use super::{columns, mutation::replace_table_map_value};

impl UiSurface {
    // None 表示此专用路径不可用并允许上层 fallback；Some(false) 表示已处理但无变化，不能再次执行旧路径。
    pub(super) fn apply_table_column_width_batch(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        width: f64,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<Option<bool>, UiTreeError> {
        let Some(metadata) = self.template_metadata(owner_id).ok() else {
            return Ok(None);
        };
        let properties = table_column_width_values(metadata, field, width);

        let batch = mutate_tree_metadata_properties(
            &mut self.tree,
            owner_id,
            properties,
            UiBindingSourceKind::WidgetBehavior,
        )?;
        if batch.changes.is_empty() {
            return Ok(Some(false));
        }

        let mut combined_dirty = batch.dirty;
        for (change, mut reflected_update) in batch.changes.into_iter().zip(batch.reflected_updates)
        {
            let previous_component_value = self
                .component_states
                .get(owner_id)
                .and_then(|state| state.value(change.property.as_str()).cloned());
            let _ = self.runtime_style.set_base_attribute(
                owner_id,
                change.property.clone(),
                change.value.to_toml(),
            );
            let component_change = self.component_states.sync_from_property(
                owner_id,
                change.property.as_str(),
                &change.value,
            );
            debug_assert!(!component_change.pseudo_state_changed);

            let mut dirty = change.dirty;
            if component_change.any_changed() {
                dirty.render = true;
                reflected_update.dirty = UiBindingDirtyDomain::from_dirty_flags(dirty);
            }
            merge_dirty_flags(&mut combined_dirty, dirty);
            let mut updates = vec![reflected_update];
            if component_change.any_changed() {
                updates.push(component_state_value_update(
                    owner_id,
                    change.property,
                    previous_component_value,
                    change.value,
                    dirty,
                    UiBindingUpdateStatus::Applied,
                ));
            }
            // Keep one report per retained aggregate, matching the legacy route's shape.
            binding_reports.push(UiBindingUpdateReport::from_updates(updates));
        }
        self.mark_node_dirty(owner_id, combined_dirty)?;
        Ok(Some(true))
    }
}

fn table_column_width_values(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    field: &str,
    width: f64,
) -> Vec<(&'static str, UiValue)> {
    let mut widths = metadata
        .attributes
        .get("column_widths")
        .map(UiValue::from_toml)
        .and_then(|value| match value {
            UiValue::Map(values) => Some(values),
            _ => None,
        })
        .unwrap_or_default();
    replace_table_map_value(&mut widths, field, UiValue::Float(width));

    let mut properties = vec![("column_widths", UiValue::Map(widths))];
    let Some(mut columns) = metadata
        .attributes
        .get("columns")
        .map(UiValue::from_toml)
        .and_then(|value| match value {
            UiValue::Array(columns) => Some(columns),
            _ => None,
        })
    else {
        return properties;
    };

    let found = columns.iter_mut().any(|column| {
        let UiValue::Map(values) = column else {
            return false;
        };
        if !columns::table_column_matches(values, field) {
            return false;
        }
        replace_table_map_value(values, "width", UiValue::Float(width));
        true
    });
    if found {
        properties.push(("columns", UiValue::Array(columns)));
    }
    properties
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
#[path = "tests/width_mutation_performance_tests.rs"]
mod performance_tests;
