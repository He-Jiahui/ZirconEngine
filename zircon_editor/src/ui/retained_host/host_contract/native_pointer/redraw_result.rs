use crate::ui::retained_host::hierarchy_pointer::constants::{
    ROW_GAP, ROW_HEIGHT, ROW_WIDTH_INSET, ROW_X, ROW_Y,
};

use super::super::data::{
    FrameRect, HostDragStateData, HostPaneInteractionStateData, HostWindowPresentationData,
};
use super::super::frame_geometry::{union_frame, union_optional_frames};
use super::super::redraw::NativePointerDispatchResult;
use super::chrome_damage::chrome_press_damage_frame;
use super::resize_damage::resize_damage_frame;
use super::routing::{ChromePointerRoute, PanePointerRoute, PanePointerTarget};
use super::tab_drag_damage::tab_drag_release_damage_frame;
use super::template_hover_damage::template_hover_damage;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;

pub(in crate::ui::retained_host::host_contract) fn pointer_move_redraw(
    pointer: &PanePointerRoute,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> NativePointerDispatchResult {
    if matches!(&pointer.target, PanePointerTarget::Viewport(_)) || before == after {
        if before == after {
            return NativePointerDispatchResult::idle();
        }
        return template_hover_damage(before, after)
            .map(NativePointerDispatchResult::region)
            .unwrap_or_else(NativePointerDispatchResult::idle);
    }

    let template_damage = template_hover_damage(before, after);
    if matches!(&pointer.target, PanePointerTarget::Hierarchy) {
        if (before.hierarchy_scroll_px - after.hierarchy_scroll_px).abs() > f32::EPSILON {
            let damage = template_damage
                .map(|template| union_frame(&template, &pointer.frame))
                .unwrap_or_else(|| pointer.frame.clone());
            return NativePointerDispatchResult::region(damage);
        }
        let damage = union_optional_frames(
            hierarchy_row_damage(
                &pointer.frame,
                before.hovered_hierarchy_index,
                before.hierarchy_scroll_px,
            ),
            hierarchy_row_damage(
                &pointer.frame,
                after.hovered_hierarchy_index,
                after.hierarchy_scroll_px,
            ),
        )
        .unwrap_or_else(|| pointer.frame.clone());
        let damage = template_damage
            .map(|template| union_frame(&template, &damage))
            .unwrap_or(damage);
        return NativePointerDispatchResult::region(damage);
    }

    if let Some(template_damage) = template_damage {
        return match &pointer.target {
            PanePointerTarget::AssetTree(_)
            | PanePointerTarget::AssetContent(_)
            | PanePointerTarget::AssetReference(_, _)
            | PanePointerTarget::Welcome => {
                NativePointerDispatchResult::region(union_frame(&template_damage, &pointer.frame))
            }
            _ => NativePointerDispatchResult::region(template_damage),
        };
    }

    NativePointerDispatchResult::region(pointer.frame.clone())
}

pub(in crate::ui::retained_host::host_contract) fn workbench_template_node_move_redraw(
    hit: &TemplateNodePointerHit,
    before: &HostPaneInteractionStateData,
    after: &HostPaneInteractionStateData,
) -> NativePointerDispatchResult {
    if before == after {
        return NativePointerDispatchResult::idle();
    }
    template_hover_damage(before, after)
        .map(|template| union_frame(&template, &hit.frame))
        .map(NativePointerDispatchResult::region)
        .unwrap_or_else(|| NativePointerDispatchResult::region(hit.frame.clone()))
}

pub(in crate::ui::retained_host::host_contract) fn resize_pointer_redraw(
    presentation: &HostWindowPresentationData,
    extra_damage: Option<FrameRect>,
) -> NativePointerDispatchResult {
    match union_optional_frames(resize_damage_frame(presentation), extra_damage) {
        Some(frame) => NativePointerDispatchResult::region_with_frame_update(frame),
        None => NativePointerDispatchResult::full_frame(),
    }
}

pub(in crate::ui::retained_host::host_contract) fn chrome_press_redraw(
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
    extra_damage: Option<FrameRect>,
) -> NativePointerDispatchResult {
    let Some(frame) = chrome_press_damage_frame(presentation, route) else {
        return NativePointerDispatchResult::full_frame();
    };
    let damage = match extra_damage {
        Some(extra) => union_frame(&frame, &extra),
        None => frame,
    };
    NativePointerDispatchResult::region_with_frame_update(damage)
}

pub(in crate::ui::retained_host::host_contract) fn tab_drag_release_redraw(
    presentation: &HostWindowPresentationData,
    drag_state: &HostDragStateData,
) -> NativePointerDispatchResult {
    match tab_drag_release_damage_frame(presentation, drag_state) {
        Some(frame) => NativePointerDispatchResult::region_with_frame_update(frame),
        None => NativePointerDispatchResult::full_frame(),
    }
}

fn hierarchy_row_damage(frame: &FrameRect, row_index: i32, scroll_px: f32) -> Option<FrameRect> {
    if row_index < 0 {
        return None;
    }
    Some(FrameRect {
        x: frame.x + ROW_X,
        y: frame.y + ROW_Y + row_index as f32 * (ROW_HEIGHT + ROW_GAP) - scroll_px.max(0.0),
        width: (frame.width - ROW_WIDTH_INSET).max(1.0),
        height: ROW_HEIGHT,
    })
}
