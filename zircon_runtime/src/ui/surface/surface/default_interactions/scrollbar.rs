use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiBindingUpdateReport, UiBindingUpdateStatus, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    layout::{UiAxis, UiContainerKind, UiFrame, UiScrollState},
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute},
    tree::{UiDirtyFlags, UiTemplateNodeMetadata, UiTreeError},
    widget::UiWidgetBehavior,
};

use crate::ui::binding::{binding_update_report, runtime_state_update_with_source_kind};
use crate::ui::surface::UiSurface;
use crate::ui::tree::{UiRuntimeTreeAccessExt, UiRuntimeTreeScrollExt};

use super::widget_behavior;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::surface::surface) struct UiDefaultScrollbarPointerActionReport {
    pub handled_by: Option<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
    pub released_capture: Option<UiNodeId>,
    pub damage_node: Option<UiNodeId>,
}

impl UiSurface {
    pub(in crate::ui::surface::surface) fn apply_default_scrollbar_pointer_action(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultScrollbarPointerActionReport, UiTreeError> {
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                return self.apply_default_scrollbar_thumb_press(route, events);
            }
            UiPointerActivationPhase::Hover if matches!(route.kind, UiPointerEventKind::Move) => {
                return self.apply_default_scrollbar_thumb_drag(route, events, binding_reports);
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(action) = self.apply_default_scrollbar_thumb_release(route, events)? {
                    return Ok(action);
                }
            }
            _ => return Ok(UiDefaultScrollbarPointerActionReport::default()),
        }

        let Some(scrollbar_id) = route.click_target else {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
        };
        let Some((target_id, axis, track_frame, scroll_state)) =
            self.default_scrollbar_track(scrollbar_id)?
        else {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
        };
        let Some(next_offset) = page_offset_for_track_click(route, track_frame, axis, scroll_state)
        else {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
        };

        if self.tree.set_scroll_offset(target_id, next_offset)? {
            self.push_scrollbar_runtime_state_report(
                binding_reports,
                scrollbar_id,
                "scroll_target",
                target_id,
                scroll_state.offset,
                next_offset,
            );
            Ok(UiDefaultScrollbarPointerActionReport {
                handled_by: Some(scrollbar_id),
                damage_node: Some(target_id),
                ..UiDefaultScrollbarPointerActionReport::default()
            })
        } else {
            Ok(UiDefaultScrollbarPointerActionReport::default())
        }
    }

    fn apply_default_scrollbar_thumb_press(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<UiDefaultScrollbarPointerActionReport, UiTreeError> {
        let Some(thumb_id) = route.target else {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
        };
        if self.default_scrollbar_thumb(thumb_id)?.is_none() {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
        }
        self.capture_pointer(thumb_id)?;
        self.push_pointer_component_events(
            events,
            thumb_id,
            UiEventKind::DragBegin,
            UiComponentEvent::BeginDrag {
                property: "scroll_offset".to_string(),
            },
            UiPointerComponentEventReason::PressBegin,
        )?;
        Ok(UiDefaultScrollbarPointerActionReport {
            handled_by: Some(thumb_id),
            captured_by: Some(thumb_id),
            damage_node: Some(thumb_id),
            ..UiDefaultScrollbarPointerActionReport::default()
        })
    }

    fn apply_default_scrollbar_thumb_drag(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<UiDefaultScrollbarPointerActionReport, UiTreeError> {
        let Some(thumb_id) = route.captured else {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
        };
        let Some(context) = self.default_scrollbar_thumb(thumb_id)? else {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
        };
        let Some(next_offset) = thumb_drag_offset(route, context) else {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
        };
        if !self
            .tree
            .set_scroll_offset(context.target_id, next_offset)?
        {
            return Ok(UiDefaultScrollbarPointerActionReport {
                handled_by: Some(thumb_id),
                damage_node: Some(thumb_id),
                ..UiDefaultScrollbarPointerActionReport::default()
            });
        }
        self.push_scrollbar_runtime_state_report(
            binding_reports,
            thumb_id,
            "scroll_thumb",
            context.target_id,
            context.scroll_state.offset,
            next_offset,
        );
        self.push_pointer_component_events(
            events,
            thumb_id,
            UiEventKind::DragUpdate,
            UiComponentEvent::DragDelta {
                property: "scroll_offset".to_string(),
                delta: f64::from(next_offset - context.scroll_state.offset),
            },
            UiPointerComponentEventReason::DirectBinding,
        )?;
        Ok(UiDefaultScrollbarPointerActionReport {
            handled_by: Some(thumb_id),
            damage_node: Some(context.target_id),
            ..UiDefaultScrollbarPointerActionReport::default()
        })
    }

    fn apply_default_scrollbar_thumb_release(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<Option<UiDefaultScrollbarPointerActionReport>, UiTreeError> {
        let Some(thumb_id) = route.captured.or(route.pressed) else {
            return Ok(None);
        };
        if self.default_scrollbar_thumb(thumb_id)?.is_none() {
            return Ok(None);
        }
        self.push_pointer_component_events(
            events,
            thumb_id,
            UiEventKind::DragEnd,
            UiComponentEvent::EndDrag {
                property: "scroll_offset".to_string(),
            },
            UiPointerComponentEventReason::PressEnd,
        )?;
        Ok(Some(UiDefaultScrollbarPointerActionReport {
            handled_by: Some(thumb_id),
            released_capture: Some(thumb_id),
            damage_node: Some(thumb_id),
            ..UiDefaultScrollbarPointerActionReport::default()
        }))
    }

    fn push_scrollbar_runtime_state_report(
        &self,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
        source_id: UiNodeId,
        source_property: &'static str,
        target_id: UiNodeId,
        previous_offset: f32,
        next_offset: f32,
    ) {
        let dirty = self
            .tree
            .node(target_id)
            .map(|node| node.dirty)
            .unwrap_or(UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            });
        binding_reports.push(binding_update_report(vec![
            runtime_state_update_with_source_kind(
                source_id,
                source_property,
                UiBindingSourceKind::WidgetBehavior,
                target_id,
                "scroll_offset",
                Some(UiValue::Float(f64::from(previous_offset))),
                UiValue::Float(f64::from(next_offset)),
                dirty,
                UiBindingUpdateStatus::Applied,
                None,
            ),
        ]));
    }

    fn default_scrollbar_track(
        &self,
        scrollbar_id: UiNodeId,
    ) -> Result<Option<(UiNodeId, UiAxis, UiFrame, UiScrollState)>, UiTreeError> {
        let scrollbar = self
            .tree
            .node(scrollbar_id)
            .ok_or(UiTreeError::MissingNode(scrollbar_id))?;
        let Some(metadata) = scrollbar.template_metadata.as_ref() else {
            return Ok(None);
        };
        if widget_behavior(metadata) != UiWidgetBehavior::Scrollbar
            || !self.widget_interaction_enabled(scrollbar_id, scrollbar, metadata)
        {
            return Ok(None);
        }
        let Some(target_id) = self.resolve_scrollbar_target(metadata) else {
            return Ok(None);
        };
        let Some(target) = self.tree.node(target_id) else {
            return Ok(None);
        };
        let Some(scroll_state) = target.scroll_state else {
            return Ok(None);
        };
        let axis = metadata
            .widget
            .scroll_axis
            .unwrap_or_else(|| target_scroll_axis(target.container));
        Ok(Some((
            target_id,
            axis,
            scrollbar.layout_cache.frame,
            scroll_state,
        )))
    }

    fn default_scrollbar_thumb(
        &self,
        thumb_id: UiNodeId,
    ) -> Result<Option<UiDefaultScrollbarThumbContext>, UiTreeError> {
        let thumb = self
            .tree
            .node(thumb_id)
            .ok_or(UiTreeError::MissingNode(thumb_id))?;
        let Some(thumb_metadata) = thumb.template_metadata.as_ref() else {
            return Ok(None);
        };
        if widget_behavior(thumb_metadata) != UiWidgetBehavior::ScrollbarThumb
            || !self.widget_interaction_enabled(thumb_id, thumb, thumb_metadata)
        {
            return Ok(None);
        }
        let Some(scrollbar_id) = thumb.parent else {
            return Ok(None);
        };
        let Some((target_id, axis, track_frame, scroll_state)) =
            self.default_scrollbar_track(scrollbar_id)?
        else {
            return Ok(None);
        };
        Ok(Some(UiDefaultScrollbarThumbContext {
            target_id,
            axis,
            track_frame,
            thumb_frame: thumb.layout_cache.frame,
            scroll_state,
        }))
    }

    fn resolve_scrollbar_target(&self, metadata: &UiTemplateNodeMetadata) -> Option<UiNodeId> {
        let reference = metadata
            .widget
            .scroll_target
            .as_deref()
            .or_else(|| string_attribute_value(metadata, "scroll_target"))
            .or_else(|| string_attribute_value(metadata, "target"))?;
        self.resolve_node_reference(reference)
    }

    fn resolve_node_reference(&self, reference: &str) -> Option<UiNodeId> {
        if let Some(node_id) = parse_node_reference(reference) {
            return Some(node_id);
        }
        self.tree.nodes.iter().find_map(|(node_id, node)| {
            let metadata = node.template_metadata.as_ref()?;
            (metadata.control_id.as_deref() == Some(reference) || node.node_path.0 == reference)
                .then_some(*node_id)
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct UiDefaultScrollbarThumbContext {
    target_id: UiNodeId,
    axis: UiAxis,
    track_frame: UiFrame,
    thumb_frame: UiFrame,
    scroll_state: UiScrollState,
}

fn page_offset_for_track_click(
    route: &UiPointerRoute,
    track_frame: UiFrame,
    axis: UiAxis,
    scroll_state: UiScrollState,
) -> Option<f32> {
    let track_extent = match axis {
        UiAxis::Horizontal => track_frame.width,
        UiAxis::Vertical => track_frame.height,
    };
    if track_extent <= f32::EPSILON
        || scroll_state.viewport_extent <= 0.0
        || scroll_state.content_extent <= scroll_state.viewport_extent
    {
        return None;
    }

    let point_offset = match axis {
        UiAxis::Horizontal => route.point.x - track_frame.x,
        UiAxis::Vertical => route.point.y - track_frame.y,
    }
    .clamp(0.0, track_extent);
    let clicked_content_offset = point_offset * scroll_state.content_extent / track_extent;
    let page_delta = if clicked_content_offset > scroll_state.offset {
        scroll_state.viewport_extent
    } else {
        -scroll_state.viewport_extent
    };
    Some(scroll_state.offset + page_delta)
}

fn thumb_drag_offset(
    route: &UiPointerRoute,
    context: UiDefaultScrollbarThumbContext,
) -> Option<f32> {
    let track_extent = axis_extent(context.axis, context.track_frame);
    let thumb_extent = axis_extent(context.axis, context.thumb_frame);
    let travel_extent = track_extent - thumb_extent;
    let scroll_extent = context.scroll_state.content_extent - context.scroll_state.viewport_extent;
    if travel_extent <= f32::EPSILON || scroll_extent <= f32::EPSILON {
        return None;
    }
    let track_start = axis_start(context.axis, context.track_frame);
    let pointer = axis_point(context.axis, route) - track_start;
    let thumb_origin = (pointer - thumb_extent * 0.5).clamp(0.0, travel_extent);
    Some((thumb_origin / travel_extent) * scroll_extent)
}

fn axis_extent(axis: UiAxis, frame: UiFrame) -> f32 {
    match axis {
        UiAxis::Horizontal => frame.width,
        UiAxis::Vertical => frame.height,
    }
}

fn axis_start(axis: UiAxis, frame: UiFrame) -> f32 {
    match axis {
        UiAxis::Horizontal => frame.x,
        UiAxis::Vertical => frame.y,
    }
}

fn axis_point(axis: UiAxis, route: &UiPointerRoute) -> f32 {
    match axis {
        UiAxis::Horizontal => route.point.x,
        UiAxis::Vertical => route.point.y,
    }
}

fn target_scroll_axis(container: UiContainerKind) -> UiAxis {
    match container {
        UiContainerKind::ScrollableBox(config) => config.axis,
        _ => UiAxis::Vertical,
    }
}

fn string_attribute_value<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    property: &str,
) -> Option<&'a str> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_str)
}

fn parse_node_reference(reference: &str) -> Option<UiNodeId> {
    reference
        .strip_prefix('#')
        .unwrap_or(reference)
        .parse::<u64>()
        .ok()
        .map(UiNodeId::new)
}
