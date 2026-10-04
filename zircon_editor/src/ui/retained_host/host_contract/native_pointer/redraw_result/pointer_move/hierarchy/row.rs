use crate::ui::retained_host::host_contract::data::{FrameRect, HostPaneInteractionStateData};
use crate::ui::retained_host::host_contract::frame_geometry::{union_frame, union_optional_frames};

use super::super::super::super::routing::PanePointerRoute;
use super::super::super::hierarchy::hierarchy_row_damage_with_metrics;

pub(super) fn hierarchy_row_move_damage(
    pointer: &PanePointerRoute,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
    template_damage: Option<FrameRect>,
) -> FrameRect {
    let damage = union_optional_frames(
        hierarchy_row_damage_with_metrics(
            &pointer.frame,
            before.hovered_hierarchy_index,
            before.hierarchy_scroll_px,
            pointer.hierarchy_row_metrics.unwrap_or_else(
                crate::ui::retained_host::hierarchy_pointer::current_hierarchy_row_metrics,
            ),
        ),
        hierarchy_row_damage_with_metrics(
            &pointer.frame,
            after.hovered_hierarchy_index,
            after.hierarchy_scroll_px,
            pointer.hierarchy_row_metrics.unwrap_or_else(
                crate::ui::retained_host::hierarchy_pointer::current_hierarchy_row_metrics,
            ),
        ),
    )
    .unwrap_or_else(|| pointer.frame.clone());
    template_damage
        .map(|template| union_frame(&template, &damage))
        .unwrap_or(damage)
}
