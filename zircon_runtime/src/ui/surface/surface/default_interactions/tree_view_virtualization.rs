use zircon_runtime_interface::ui::{
    binding::{UiBindingUpdateReport, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute},
    tree::{UiTemplateNodeMetadata, UiTreeError},
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};

use super::{
    tree_view_support::{is_tree_view_owner, tree_node_ids},
    UiDefaultTreeViewPointerActionReport,
};

const DEFAULT_TREE_ROW_EXTENT: f64 = 24.0;
const DEFAULT_TREE_VIEWPORT_COUNT: i64 = 20;

struct UiDefaultTreeVirtualWindow {
    owner_id: UiNodeId,
    total_count: i64,
    viewport_start: i64,
    viewport_count: i64,
    visible_end: i64,
    requested_start: i64,
    requested_count: i64,
    overscan: i64,
    scroll_offset: f64,
}

impl UiSurface {
    pub(in crate::ui::surface::surface) fn apply_default_tree_view_virtual_scroll(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultTreeViewPointerActionReport, UiTreeError> {
        if route.activation_phase != UiPointerActivationPhase::Scroll
            || !matches!(route.kind, UiPointerEventKind::Scroll)
        {
            return Ok(UiDefaultTreeViewPointerActionReport::default());
        }
        let Some(window) = self.default_tree_view_virtual_scroll_window(route)? else {
            return Ok(UiDefaultTreeViewPointerActionReport::default());
        };
        if !self.apply_tree_view_virtual_window(&window, events, binding_reports)? {
            return Ok(UiDefaultTreeViewPointerActionReport::default());
        }

        Ok(UiDefaultTreeViewPointerActionReport {
            handled_by: Some(window.owner_id),
            damage_node: Some(window.owner_id),
        })
    }

    fn default_tree_view_virtual_scroll_window(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiDefaultTreeVirtualWindow>, UiTreeError> {
        if route.captured.is_some() || route.scroll_delta == 0.0 {
            return Ok(None);
        }
        let Some(owner_id) = self.tree_view_virtual_owner(route)? else {
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
            || tree_virtualization_disabled(metadata)
        {
            return Ok(None);
        }

        let total_count = tree_total_count(metadata).max(0);
        if total_count == 0 {
            return Ok(None);
        }
        let row_extent = tree_row_extent(metadata);
        let viewport_count = tree_viewport_count(
            metadata,
            f64::from(owner.layout_cache.frame.height),
            row_extent,
        )
        .max(0)
        .min(total_count);
        if viewport_count == 0 {
            return Ok(None);
        }

        let current_start = tree_viewport_start(metadata, row_extent)
            .clamp(0, total_count.saturating_sub(viewport_count));
        let next_start = (current_start + scroll_rows_delta(route.scroll_delta, row_extent))
            .clamp(0, total_count.saturating_sub(viewport_count));
        if next_start == current_start {
            return Ok(None);
        }

        Ok(Some(tree_virtual_window_for_start(
            owner_id,
            metadata,
            total_count,
            viewport_count,
            next_start,
            row_extent,
        )))
    }

    fn tree_view_virtual_owner(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        let hit_route = if route.stacked.is_empty() {
            route.bubbled.as_slice()
        } else {
            route.stacked.as_slice()
        };

        for node_id in hit_route {
            let node = self
                .tree
                .node(*node_id)
                .ok_or(UiTreeError::MissingNode(*node_id))?;
            if node
                .template_metadata
                .as_ref()
                .is_some_and(is_tree_view_owner)
            {
                return Ok(Some(*node_id));
            }
        }
        Ok(None)
    }

    pub(super) fn apply_tree_view_virtual_window_for_index(
        &mut self,
        owner_id: UiNodeId,
        visible_index: i64,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let owner = self
            .tree
            .node(owner_id)
            .ok_or(UiTreeError::MissingNode(owner_id))?;
        let Some(metadata) = owner.template_metadata.as_ref() else {
            return Ok(false);
        };
        if tree_virtualization_disabled(metadata) {
            return Ok(false);
        }
        let total_count = tree_total_count(metadata).max(0);
        if total_count == 0 {
            return Ok(false);
        }
        let row_extent = tree_row_extent(metadata);
        let viewport_count = tree_viewport_count(
            metadata,
            f64::from(owner.layout_cache.frame.height),
            row_extent,
        )
        .max(0)
        .min(total_count);
        if viewport_count == 0 {
            return Ok(false);
        }
        let next_start = visible_index.clamp(0, total_count.saturating_sub(viewport_count));
        let window = tree_virtual_window_for_start(
            owner_id,
            metadata,
            total_count,
            viewport_count,
            next_start,
            row_extent,
        );
        self.apply_tree_view_virtual_window(&window, events, binding_reports)
    }

    fn apply_tree_view_virtual_window(
        &mut self,
        window: &UiDefaultTreeVirtualWindow,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let mut changed = false;
        for (property, value) in [
            ("total_count", UiValue::Int(window.total_count)),
            ("item_count", UiValue::Int(window.total_count)),
            ("itemCount", UiValue::Int(window.total_count)),
            ("row_count", UiValue::Int(window.total_count)),
            ("rowCount", UiValue::Int(window.total_count)),
            ("viewport_start", UiValue::Int(window.viewport_start)),
            ("viewport_count", UiValue::Int(window.viewport_count)),
            ("visible_end", UiValue::Int(window.visible_end)),
            ("visibleEnd", UiValue::Int(window.visible_end)),
            ("requested_start", UiValue::Int(window.requested_start)),
            ("requestedStart", UiValue::Int(window.requested_start)),
            ("requested_count", UiValue::Int(window.requested_count)),
            ("requestedCount", UiValue::Int(window.requested_count)),
            ("overscan", UiValue::Int(window.overscan)),
            ("overscan_count", UiValue::Int(window.overscan)),
            ("overscanCount", UiValue::Int(window.overscan)),
            ("scroll_offset", UiValue::Float(window.scroll_offset)),
            ("scrollTop", UiValue::Float(window.scroll_offset)),
        ] {
            changed |= self.apply_tree_view_virtual_property(
                window.owner_id,
                property,
                value,
                binding_reports,
            )?;
        }
        if !changed {
            return Ok(false);
        }

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

    fn apply_tree_view_virtual_property(
        &mut self,
        owner_id: UiNodeId,
        property: &'static str,
        value: UiValue,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            owner_id,
            property.to_string(),
            value,
        ))?;
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            binding_reports.push(report.binding);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn tree_virtual_window_for_start(
    owner_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    total_count: i64,
    viewport_count: i64,
    viewport_start: i64,
    row_extent: f64,
) -> UiDefaultTreeVirtualWindow {
    let visible_end = viewport_start
        .saturating_add(viewport_count)
        .min(total_count);
    let overscan = tree_overscan(metadata);
    let requested_start = viewport_start.saturating_sub(overscan);
    let requested_end = visible_end.saturating_add(overscan).min(total_count);
    UiDefaultTreeVirtualWindow {
        owner_id,
        total_count,
        viewport_start,
        viewport_count,
        visible_end,
        requested_start,
        requested_count: requested_end.saturating_sub(requested_start),
        overscan,
        scroll_offset: viewport_start as f64 * row_extent,
    }
}

fn tree_total_count(metadata: &UiTemplateNodeMetadata) -> i64 {
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
    .unwrap_or_else(|| tree_node_ids(metadata).len() as i64)
}

fn tree_viewport_start(metadata: &UiTemplateNodeMetadata, row_extent: f64) -> i64 {
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

fn tree_viewport_count(
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
            DEFAULT_TREE_VIEWPORT_COUNT
        }
    })
}

fn tree_row_extent(metadata: &UiTemplateNodeMetadata) -> f64 {
    float_attribute_any(
        metadata,
        &["row_height", "rowHeight", "item_extent", "itemSize"],
    )
    .filter(|extent| *extent > 0.0)
    .unwrap_or(DEFAULT_TREE_ROW_EXTENT)
}

fn tree_overscan(metadata: &UiTemplateNodeMetadata) -> i64 {
    int_attribute_any(metadata, &["overscan", "overscan_count", "overscanCount"])
        .unwrap_or_default()
        .max(0)
}

fn tree_virtualization_disabled(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "disable_virtualization")
        || bool_attribute(metadata, "disableVirtualization")
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
    properties
        .iter()
        .find_map(|property| metadata.attributes.get(*property).and_then(toml_number))
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, property: &str) -> bool {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn toml_number(value: &toml::Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
}
