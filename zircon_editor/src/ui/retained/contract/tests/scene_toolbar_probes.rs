use std::sync::Arc;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
    surface::UiSurfaceFrame,
};

use super::{
    native_pointer::routing, paint_recording::record_host_frame_commands,
    HostPresentationGeneration,
};

pub(in crate::ui::retained_host) fn scene_toolbar_route_for_test(
    generation: &HostPresentationGeneration,
    point: UiPoint,
) -> (bool, Option<(String, String)>) {
    let presentation = generation.structure();
    let blocked = presentation.close_prompt.visible
        || presentation.asset_deletion_blocker.visible
        || generation.menu_state().open_menu_index >= 0
        || generation.dock_overflow_menu_state().open
        || generation.page_overflow_menu_state().open
        || routing::route_top_level_chrome(presentation, point.x, point.y).is_some()
        || routing::route_pointer_to_workbench_generation(generation, point.x, point.y).is_some();
    let route = routing::route_pointer_to_pane(
        presentation,
        generation.pane_interaction_state(),
        point.x,
        point.y,
    );
    let toolbar = route.and_then(|route| match route.target {
        routing::PanePointerTarget::ViewportToolbar {
            surface_key,
            source_control_id: Some(id),
            ..
        } => Some((surface_key.to_owned(), id)),
        _ => None,
    });
    (blocked, toolbar)
}

pub(in crate::ui::retained_host) fn record_scene_toolbar_control_ink_for_test(
    generation: &HostPresentationGeneration,
    width: u32,
    height: u32,
    surface: &Arc<UiSurfaceFrame>,
    node: UiNodeId,
) -> Vec<UiFrame> {
    let _paint_scope = generation.enter_paint_scope();
    let (recorded, _) = record_host_frame_commands(width, height, generation.structure(), None);
    recorded
        .commands
        .iter()
        .filter_map(|command| {
            let owner = command.source?;
            if owner.command_ref.node_id != node
                || !recorded
                    .render_sources
                    .resolve(owner.surface_key)
                    .is_some_and(|source| Arc::ptr_eq(source, surface))
            {
                return None;
            }
            let frame = UiFrame::new(
                command.frame.x,
                command.frame.y,
                command.frame.width,
                command.frame.height,
            );
            match command.clip_frame.as_ref() {
                Some(clip) => {
                    frame.intersection(UiFrame::new(clip.x, clip.y, clip.width, clip.height))
                }
                None => Some(frame),
            }
        })
        .collect()
}
