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
    surface::property_mutation::mutate_tree_metadata_properties,
};

use super::UiSurface;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::surface::surface) struct UiVirtualWindowState {
    pub(in crate::ui::surface::surface) owner_id: UiNodeId,
    pub(in crate::ui::surface::surface) total_count: i64,
    pub(in crate::ui::surface::surface) viewport_start: i64,
    pub(in crate::ui::surface::surface) viewport_count: i64,
    pub(in crate::ui::surface::surface) visible_end: i64,
    pub(in crate::ui::surface::surface) requested_start: i64,
    pub(in crate::ui::surface::surface) requested_count: i64,
    pub(in crate::ui::surface::surface) overscan: i64,
    pub(in crate::ui::surface::surface) scroll_offset: f64,
}

impl UiVirtualWindowState {
    fn properties(&self) -> [(&'static str, UiValue); 18] {
        [
            ("total_count", UiValue::Int(self.total_count)),
            ("item_count", UiValue::Int(self.total_count)),
            ("itemCount", UiValue::Int(self.total_count)),
            ("row_count", UiValue::Int(self.total_count)),
            ("rowCount", UiValue::Int(self.total_count)),
            ("viewport_start", UiValue::Int(self.viewport_start)),
            ("viewport_count", UiValue::Int(self.viewport_count)),
            ("visible_end", UiValue::Int(self.visible_end)),
            ("visibleEnd", UiValue::Int(self.visible_end)),
            ("requested_start", UiValue::Int(self.requested_start)),
            ("requestedStart", UiValue::Int(self.requested_start)),
            ("requested_count", UiValue::Int(self.requested_count)),
            ("requestedCount", UiValue::Int(self.requested_count)),
            ("overscan", UiValue::Int(self.overscan)),
            ("overscan_count", UiValue::Int(self.overscan)),
            ("overscanCount", UiValue::Int(self.overscan)),
            ("scroll_offset", UiValue::Float(self.scroll_offset)),
            ("scrollTop", UiValue::Float(self.scroll_offset)),
        ]
    }
}

impl UiSurface {
    pub(in crate::ui::surface::surface) fn mutate_virtual_window(
        &mut self,
        window: &UiVirtualWindowState,
    ) -> Result<Option<UiBindingUpdateReport>, UiTreeError> {
        let batch = mutate_tree_metadata_properties(
            &mut self.tree,
            window.owner_id,
            window.properties(),
            UiBindingSourceKind::WidgetBehavior,
        )?;
        if batch.changes.is_empty() {
            return Ok(None);
        }

        let mut combined_dirty = batch.dirty;
        let mut binding_updates = Vec::with_capacity(batch.changes.len() * 2);
        for (change, mut reflected_update) in batch.changes.into_iter().zip(batch.reflected_updates)
        {
            let previous_component_value = self
                .component_states
                .get(window.owner_id)
                .and_then(|state| state.value(change.property.as_str()).cloned());
            let _ = self.runtime_style.set_base_attribute(
                window.owner_id,
                change.property.clone(),
                change.value.to_toml(),
            );
            let component_change = self.component_states.sync_from_property(
                window.owner_id,
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
            binding_updates.push(reflected_update);
            if component_change.any_changed() {
                binding_updates.push(component_state_value_update(
                    window.owner_id,
                    change.property,
                    previous_component_value,
                    change.value,
                    dirty,
                    UiBindingUpdateStatus::Applied,
                ));
            }
        }

        self.mark_node_dirty(window.owner_id, combined_dirty)?;
        Ok(Some(UiBindingUpdateReport::from_updates(binding_updates)))
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
