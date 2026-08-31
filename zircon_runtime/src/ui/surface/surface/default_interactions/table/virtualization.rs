use zircon_runtime_interface::ui::{
    binding::{UiBindingUpdateReport, UiEventKind},
    component::UiComponentEvent,
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    surface::UiPointerRoute,
    tree::{UiTemplateNodeMetadata, UiTreeError},
};

use crate::ui::surface::surface::{UiSurface, UiVirtualWindowState};

const DEFAULT_ROW_EXTENT: f64 = 24.0;
const DEFAULT_VIEWPORT_COUNT: i64 = 20;

impl UiSurface {
    pub(super) fn apply_default_table_virtual_scroll(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<super::UiDefaultTablePointerActionReport, UiTreeError> {
        let Some(window) = self.default_table_virtual_scroll_window(route)? else {
            return Ok(super::UiDefaultTablePointerActionReport::default());
        };
        if !self.apply_table_virtual_window(&window, events, binding_reports)? {
            return Ok(super::UiDefaultTablePointerActionReport::default());
        }

        Ok(super::UiDefaultTablePointerActionReport {
            handled_by: Some(window.owner_id),
            damage_node: Some(window.owner_id),
            ..super::UiDefaultTablePointerActionReport::default()
        })
    }

    fn default_table_virtual_scroll_window(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiVirtualWindowState>, UiTreeError> {
        if route.captured.is_some() || route.scroll_delta == 0.0 {
            return Ok(None);
        }
        let Some(owner_id) = self.table_virtual_scroll_owner(route)? else {
            return Ok(None);
        };
        let owner = self
            .tree
            .node(owner_id)
            .ok_or(UiTreeError::MissingNode(owner_id))?;
        let Some(metadata) = owner.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(owner_id, owner, metadata)
            || table_virtualization_disabled(metadata)
        {
            return Ok(None);
        }

        let total_count = table_total_count(metadata).max(0);
        if total_count == 0 {
            return Ok(None);
        }
        let row_extent = table_row_extent(metadata);
        let viewport_count = table_viewport_count(
            metadata,
            f64::from(owner.layout_cache.frame.height),
            row_extent,
        )
        .max(0)
        .min(total_count);
        if viewport_count == 0 {
            return Ok(None);
        }

        let current_start = table_viewport_start(metadata, row_extent)
            .clamp(0, total_count.saturating_sub(viewport_count));
        let next_start = (current_start + scroll_rows_delta(route.scroll_delta, row_extent))
            .clamp(0, total_count.saturating_sub(viewport_count));
        if next_start == current_start {
            return Ok(None);
        }

        let visible_end = next_start.saturating_add(viewport_count).min(total_count);
        let overscan = table_overscan(metadata);
        let requested_start = next_start.saturating_sub(overscan);
        let requested_end = visible_end.saturating_add(overscan).min(total_count);
        Ok(Some(UiVirtualWindowState {
            owner_id,
            total_count,
            viewport_start: next_start,
            viewport_count,
            visible_end,
            requested_start,
            requested_count: requested_end.saturating_sub(requested_start),
            overscan,
            scroll_offset: next_start as f64 * row_extent,
        }))
    }

    fn table_virtual_scroll_owner(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        for node_id in route.hit_candidates() {
            let node = self
                .tree
                .node(node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if node
                .template_metadata
                .as_ref()
                .is_some_and(super::is_table_owner)
            {
                return Ok(Some(node_id));
            }
        }
        Ok(None)
    }

    fn apply_table_virtual_window(
        &mut self,
        window: &UiVirtualWindowState,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(binding_report) = self.mutate_virtual_window(window)? else {
            return Ok(false);
        };
        binding_reports.push(binding_report);

        self.push_pointer_component_events_for_component_event_kind(
            events,
            window.owner_id,
            UiEventKind::Change,
            UiComponentEvent::SetVisibleRange {
                start: window.viewport_start,
                count: window.viewport_count,
            },
            UiPointerComponentEventReason::DirectBinding,
        )?;
        Ok(true)
    }
}

fn table_total_count(metadata: &UiTemplateNodeMetadata) -> i64 {
    int_attribute_any(
        metadata,
        &[
            "total_count",
            "row_count",
            "rowCount",
            "item_count",
            "itemCount",
        ],
    )
    .or_else(|| array_len_attribute(metadata, "rows"))
    .or_else(|| array_len_attribute(metadata, "items"))
    .unwrap_or_default()
}

fn table_viewport_start(metadata: &UiTemplateNodeMetadata, row_extent: f64) -> i64 {
    int_attribute_any(
        metadata,
        &["viewport_start", "visible_start", "visibleStart"],
    )
    .or_else(|| {
        float_attribute_any(metadata, &["scroll_offset", "scrollTop"])
            .map(|offset| (offset / row_extent).round() as i64)
    })
    .unwrap_or_default()
}

fn table_viewport_count(
    metadata: &UiTemplateNodeMetadata,
    frame_height: f64,
    row_extent: f64,
) -> i64 {
    int_attribute_any(
        metadata,
        &["viewport_count", "visible_count", "visibleCount"],
    )
    .filter(|count| *count > 0)
    .unwrap_or_else(|| {
        if frame_height > 0.0 {
            (frame_height / row_extent).ceil().max(1.0) as i64
        } else {
            DEFAULT_VIEWPORT_COUNT
        }
    })
}

fn table_row_extent(metadata: &UiTemplateNodeMetadata) -> f64 {
    float_attribute_any(
        metadata,
        &["row_height", "rowHeight", "item_extent", "itemSize"],
    )
    .filter(|extent| *extent > 0.0)
    .unwrap_or(DEFAULT_ROW_EXTENT)
}

fn table_overscan(metadata: &UiTemplateNodeMetadata) -> i64 {
    int_attribute_any(metadata, &["overscan", "overscan_count", "overscanCount"])
        .unwrap_or_default()
        .max(0)
}

fn table_virtualization_disabled(metadata: &UiTemplateNodeMetadata) -> bool {
    super::bool_attribute(metadata, "disable_virtualization")
        || super::bool_attribute(metadata, "disableVirtualization")
}

fn scroll_rows_delta(scroll_delta: f32, row_extent: f64) -> i64 {
    let rows = f64::from(scroll_delta) / row_extent;
    if rows.is_sign_negative() {
        rows.floor() as i64
    } else {
        rows.ceil() as i64
    }
}

fn int_attribute_any(metadata: &UiTemplateNodeMetadata, properties: &[&str]) -> Option<i64> {
    properties.iter().find_map(|property| {
        metadata
            .attributes
            .get(*property)
            .and_then(toml::Value::as_integer)
    })
}

fn float_attribute_any(metadata: &UiTemplateNodeMetadata, properties: &[&str]) -> Option<f64> {
    properties.iter().find_map(|property| {
        metadata
            .attributes
            .get(*property)
            .and_then(super::toml_number)
    })
}

fn array_len_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> Option<i64> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_array)
        .map(|items| items.len() as i64)
}
