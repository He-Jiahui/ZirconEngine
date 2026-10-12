use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

use super::super::action_matches;

pub(super) fn demo_world_surface_input(
    action_id: &str,
) -> Option<UiComponentShowcaseDemoEventInput> {
    match action_id {
        action if action_matches(action, "world_space_surface_moved") => {
            Some(UiComponentShowcaseDemoEventInput::SetWorldTransform {
                position: [1.0, 2.0, 4.0],
                rotation: [0.0, 180.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            })
        }
        action if action_matches(action, "world_space_surface_configured") => {
            Some(UiComponentShowcaseDemoEventInput::SetWorldSurface {
                size: [2.5, 1.25],
                pixels_per_meter: 256.0,
                billboard: true,
                depth_test: true,
                render_order: 4,
                camera_target: "viewport-main".to_string(),
            })
        }
        _ => None,
    }
}
