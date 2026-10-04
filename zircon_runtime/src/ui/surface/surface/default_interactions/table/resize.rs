use zircon_runtime_interface::ui::{
    binding::{UiBindingUpdateReport, UiEventKind},
    component::UiComponentEvent,
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    surface::UiPointerRoute,
    tree::UiTreeError,
};

#[cfg(test)]
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use crate::ui::surface::UiSurface;

use super::{columns, is_table_owner, UiDefaultTablePointerActionReport};

struct UiDefaultTableColumnResizeStart {
    owner_id: UiNodeId,
    field: String,
    start_width: f64,
    min_width: f64,
}

impl UiSurface {
    pub(super) fn apply_default_table_column_resize_press(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(start) = self.default_table_column_resize_start(route)? else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        self.capture_pointer(start.owner_id)?;
        let drag = self.input.begin_pointer_drag_with_resize(
            start.owner_id,
            route.point,
            &start.field,
            start.start_width,
            start.min_width,
        );
        // Keep the serialized property for older snapshots; live moves use the typed token.
        self.input.set_pointer_drag_property(
            start.owner_id,
            Some(columns::encode_table_column_resize_drag(
                start.start_width,
                start.min_width,
                &start.field,
            )),
        );
        self.push_pointer_component_events_with_drag_metrics(
            events,
            start.owner_id,
            UiEventKind::DragBegin,
            UiComponentEvent::BeginDrag {
                property: "column_width".to_string(),
            },
            UiPointerComponentEventReason::PressBegin,
            Some(drag),
        )?;
        Ok(UiDefaultTablePointerActionReport {
            handled_by: Some(start.owner_id),
            captured_by: Some(start.owner_id),
            damage_node: Some(start.owner_id),
            ..UiDefaultTablePointerActionReport::default()
        })
    }

    pub(super) fn apply_default_table_column_resize_drag(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(owner_id) = route.captured else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        let Some(token) = self
            .input
            .pointer_drag_resize(owner_id)
            .cloned()
            .or_else(|| {
                self.input
                    .pointer_drag_property(owner_id)
                    .and_then(columns::decode_table_column_resize_drag)
            })
        else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        let drag = self.input.update_pointer_drag(owner_id, route.point);
        let next_width = (token.start_width + f64::from(drag.delta.x)).max(token.min_width);
        if let Some(delta) = self.apply_default_table_column_width(
            owner_id,
            token.field.as_ref(),
            next_width,
            events,
            binding_reports,
            UiPointerComponentEventReason::DirectBinding,
        )? {
            self.push_pointer_component_events_with_drag_metrics(
                events,
                owner_id,
                UiEventKind::DragUpdate,
                UiComponentEvent::DragDelta {
                    property: "column_width".to_string(),
                    delta,
                },
                UiPointerComponentEventReason::DirectBinding,
                Some(drag),
            )?;
        }
        Ok(UiDefaultTablePointerActionReport {
            handled_by: Some(owner_id),
            damage_node: Some(owner_id),
            ..UiDefaultTablePointerActionReport::default()
        })
    }

    pub(super) fn apply_default_table_column_resize_release(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(owner_id) = route.captured.or(route.pressed) else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        let Some(token) = self
            .input
            .pointer_drag_resize(owner_id)
            .cloned()
            .or_else(|| {
                self.input
                    .pointer_drag_property(owner_id)
                    .and_then(columns::decode_table_column_resize_drag)
            })
        else {
            return Ok(UiDefaultTablePointerActionReport::default());
        };
        let drag = self.input.end_pointer_drag(owner_id, route.point);
        let next_width = (token.start_width + f64::from(drag.delta.x)).max(token.min_width);
        let _ = self.apply_default_table_column_width(
            owner_id,
            token.field.as_ref(),
            next_width,
            events,
            binding_reports,
            UiPointerComponentEventReason::DefaultClick,
        )?;
        self.push_pointer_component_events_with_drag_metrics(
            events,
            owner_id,
            UiEventKind::DragEnd,
            UiComponentEvent::EndDrag {
                property: "column_width".to_string(),
            },
            UiPointerComponentEventReason::PressEnd,
            Some(drag),
        )?;
        Ok(UiDefaultTablePointerActionReport {
            handled_by: Some(owner_id),
            released_capture: Some(owner_id),
            damage_node: Some(owner_id),
            ..UiDefaultTablePointerActionReport::default()
        })
    }

    fn default_table_column_resize_start(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiDefaultTableColumnResizeStart>, UiTreeError> {
        let Some(handle_id) = route.bubble_route().find(|node_id| {
            self.tree
                .node(*node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .is_some_and(columns::is_table_column_resize_handle)
        }) else {
            return Ok(None);
        };
        let Some(handle_metadata) = self
            .tree
            .node(handle_id)
            .and_then(|node| node.template_metadata.as_ref())
        else {
            return Ok(None);
        };
        let Some(field) = columns::table_column_field(handle_metadata) else {
            return Ok(None);
        };
        let Some(owner_id) = route.bubble_route().find(|node_id| {
            *node_id != handle_id
                && self
                    .tree
                    .node(*node_id)
                    .and_then(|node| node.template_metadata.as_ref())
                    .is_some_and(is_table_owner)
        }) else {
            return Ok(None);
        };
        let Some(owner) = self.tree.node(owner_id) else {
            return Ok(None);
        };
        let Some(owner_metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(owner_id, owner, owner_metadata)
            || columns::table_column_resize_disabled(owner_metadata)
        {
            return Ok(None);
        }
        let Some(start_width) = columns::table_column_width(owner_metadata, &field) else {
            return Ok(None);
        };
        let min_width = columns::table_min_column_width(owner_metadata, &field);
        Ok(Some(UiDefaultTableColumnResizeStart {
            owner_id,
            field,
            start_width,
            min_width,
        }))
    }

    fn apply_default_table_column_width(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        width: f64,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
        reason: UiPointerComponentEventReason,
    ) -> Result<Option<f64>, UiTreeError> {
        let (previous_width, projection_is_current) = self
            .template_metadata(owner_id)
            .ok()
            .map(|metadata| {
                (
                    columns::table_column_width(metadata, field).unwrap_or(width),
                    columns::table_column_width_projection_is_current(metadata, field, width),
                )
            })
            .unwrap_or((width, false));
        if !table_column_width_changed(previous_width, width) && projection_is_current {
            return Ok(None);
        }
        let changed =
            match self.apply_table_column_width_batch(owner_id, field, width, binding_reports)? {
                Some(changed) => changed,
                None => {
                    let mut fallback_changed = false;
                    fallback_changed |= self.apply_table_column_widths_mutation(
                        owner_id,
                        field,
                        width,
                        binding_reports,
                    )?;
                    fallback_changed |= self.apply_table_columns_width_mutation(
                        owner_id,
                        field,
                        width,
                        binding_reports,
                    )?;
                    fallback_changed
                }
            };
        if !changed {
            return Ok(None);
        }

        self.push_pointer_component_events(
            events,
            owner_id,
            UiEventKind::Change,
            UiComponentEvent::ValueChanged {
                property: "column_width".to_string(),
                value: columns::column_width_payload(field, width),
            },
            reason,
        )?;
        Ok(Some(width - previous_width))
    }
}

fn table_column_width_changed(previous: f64, next: f64) -> bool {
    previous != next
}

#[cfg(test)]
#[path = "tests/resize_optimization_tests.rs"]
mod optimization_tests;
