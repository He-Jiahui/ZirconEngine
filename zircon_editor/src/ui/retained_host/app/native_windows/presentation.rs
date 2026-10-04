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
    ui.set_native_floating_window_presentation(
        target.window_id.0.as_str(),
        target.surface_tree_id.0.as_str(),
        target.title.as_str(),
        &bounds,
    );

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

#[cfg(test)]
fn apply_native_floating_presentation_data(
    presentation: &mut HostWindowPresentationData,
    target: &NativeFloatingWindowTarget,
    bounds: &FrameRect,
) {
    presentation.host_shell.native_floating_window_mode = true;
    presentation
        .host_shell
        .native_floating_window_id
        .clone_from(&target.window_id.0);
    presentation
        .host_shell
        .native_surface_tree_id
        .clone_from(&target.surface_tree_id.0);
    presentation
        .host_shell
        .native_window_title
        .clone_from(&target.title);
    presentation.host_shell.native_window_bounds = bounds.clone();
    presentation
        .native_floating_surface_data
        .native_floating_window_id
        .clone_from(&target.window_id.0);
    presentation
        .native_floating_surface_data
        .native_surface_tree_id
        .clone_from(&target.surface_tree_id.0);
    presentation
        .native_floating_surface_data
        .native_window_bounds = bounds.clone();
}

#[cfg(test)]
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
#[path = "tests/presentation.rs"]
mod tests;

#[cfg(test)]
#[path = "presentation/tests/reused_string_tests.rs"]
mod reused_string_tests;
