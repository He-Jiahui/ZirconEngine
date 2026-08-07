use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};

use super::super::{FrameRect, UiHostWindow};
use super::target::NativeFloatingWindowTarget;

pub(crate) fn configure_native_floating_window_presentation(
    ui: &UiHostWindow,
    target: &NativeFloatingWindowTarget,
) {
    let bounds = FrameRect {
        x: target.bounds[0],
        y: target.bounds[1],
        width: target.bounds[2],
        height: target.bounds[3],
    };
    let generation = ui.get_host_presentation_generation();
    if !native_floating_presentation_matches(generation.structure(), target, &bounds) {
        ui.update_host_presentation(|host_presentation| {
            host_presentation.host_shell.native_floating_window_mode = true;
            host_presentation.host_shell.native_floating_window_id = target.window_id.0.clone();
            host_presentation.host_shell.native_surface_tree_id = target.surface_tree_id.0.clone();
            host_presentation.host_shell.native_window_title = target.title.clone();
            host_presentation.host_shell.native_window_bounds = bounds.clone();
            host_presentation
                .native_floating_surface_data
                .native_floating_window_id = target.window_id.0.clone();
            host_presentation
                .native_floating_surface_data
                .native_surface_tree_id = target.surface_tree_id.0.clone();
            host_presentation
                .native_floating_surface_data
                .native_window_bounds = bounds.clone();
        });
    }

    let position = PhysicalPosition::new(
        target.bounds[0].round() as i32,
        target.bounds[1].round() as i32,
    );
    let size = PhysicalSize::new(
        target.bounds[2].max(1.0).round() as u32,
        target.bounds[3].max(1.0).round() as u32,
    );
    if ui.window().position() != position {
        ui.window().set_position(position);
    }
    if ui.window().size() != size {
        ui.window().set_size(size);
    }
}

fn native_floating_presentation_matches(
    presentation: &HostWindowPresentationData,
    target: &NativeFloatingWindowTarget,
    bounds: &FrameRect,
) -> bool {
    let shell = &presentation.host_shell;
    let surface = &presentation.native_floating_surface_data;
    shell.native_floating_window_mode
        && shell.native_floating_window_id == target.window_id.0
        && shell.native_surface_tree_id == target.surface_tree_id.0
        && shell.native_window_title == target.title
        && shell.native_window_bounds == *bounds
        && surface.native_floating_window_id == target.window_id.0
        && surface.native_surface_tree_id == target.surface_tree_id.0
        && surface.native_window_bounds == *bounds
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::event_ui::UiTreeId;

    use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
    use crate::ui::workbench::layout::MainPageId;

    use super::{native_floating_presentation_matches, FrameRect, NativeFloatingWindowTarget};

    #[test]
    fn unchanged_native_target_does_not_require_a_presentation_rebuild() {
        let target = NativeFloatingWindowTarget {
            window_id: MainPageId("window-1".to_owned()),
            title: "Tools".to_owned(),
            bounds: [12.0, 24.0, 640.0, 480.0],
            surface_tree_id: UiTreeId("tree-1".to_owned()),
        };
        let bounds = FrameRect {
            x: 12.0,
            y: 24.0,
            width: 640.0,
            height: 480.0,
        };
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_shell.native_floating_window_mode = true;
        presentation.host_shell.native_floating_window_id = target.window_id.0.clone();
        presentation.host_shell.native_surface_tree_id = target.surface_tree_id.0.clone();
        presentation.host_shell.native_window_title = target.title.clone();
        presentation.host_shell.native_window_bounds = bounds.clone();
        presentation
            .native_floating_surface_data
            .native_floating_window_id = target.window_id.0.clone();
        presentation
            .native_floating_surface_data
            .native_surface_tree_id = target.surface_tree_id.0.clone();
        presentation
            .native_floating_surface_data
            .native_window_bounds = bounds.clone();

        assert!(native_floating_presentation_matches(
            &presentation,
            &target,
            &bounds
        ));

        presentation.host_shell.native_window_title = "Changed".to_owned();
        assert!(!native_floating_presentation_matches(
            &presentation,
            &target,
            &bounds
        ));
    }
}
