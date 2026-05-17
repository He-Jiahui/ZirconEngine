use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiAxis, UiContainerKind, UiFrame, UiScrollState},
    surface::{UiPointerActivationPhase, UiPointerRoute},
    tree::{UiTemplateNodeMetadata, UiTreeError},
    widget::UiWidgetBehavior,
};

use crate::ui::surface::UiSurface;
use crate::ui::tree::{UiRuntimeTreeAccessExt, UiRuntimeTreeScrollExt};

use super::widget_behavior;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::surface::surface) struct UiDefaultScrollbarPointerActionReport {
    pub handled_by: Option<UiNodeId>,
    pub damage_node: Option<UiNodeId>,
}

impl UiSurface {
    pub(in crate::ui::surface::surface) fn apply_default_scrollbar_pointer_action(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<UiDefaultScrollbarPointerActionReport, UiTreeError> {
        if route.activation_phase != UiPointerActivationPhase::PrimaryRelease {
            return Ok(UiDefaultScrollbarPointerActionReport::default());
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
            Ok(UiDefaultScrollbarPointerActionReport {
                handled_by: Some(scrollbar_id),
                damage_node: Some(target_id),
            })
        } else {
            Ok(UiDefaultScrollbarPointerActionReport::default())
        }
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
