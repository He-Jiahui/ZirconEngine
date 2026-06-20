use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};

use super::super::{FrameRect, UiHostWindow};
use super::target::NativeFloatingWindowTarget;

pub(crate) fn configure_native_floating_window_presentation(
    ui: &UiHostWindow,
    target: &NativeFloatingWindowTarget,
) {
    let mut host_presentation = ui.get_host_presentation();
    host_presentation.host_shell.native_floating_window_mode = true;
    host_presentation.host_shell.native_floating_window_id = target.window_id.0.clone().into();
    host_presentation.host_shell.native_surface_tree_id = target.surface_tree_id.0.clone().into();
    host_presentation.host_shell.native_window_title = target.title.clone().into();
    host_presentation.host_shell.native_window_bounds = FrameRect {
        x: target.bounds[0],
        y: target.bounds[1],
        width: target.bounds[2],
        height: target.bounds[3],
    };
    host_presentation
        .native_floating_surface_data
        .native_floating_window_id = target.window_id.0.clone().into();
    host_presentation
        .native_floating_surface_data
        .native_surface_tree_id = target.surface_tree_id.0.clone().into();
    host_presentation
        .native_floating_surface_data
        .native_window_bounds = host_presentation.host_shell.native_window_bounds.clone();
    ui.set_host_presentation(host_presentation);
    ui.window().set_position(PhysicalPosition::new(
        target.bounds[0].round() as i32,
        target.bounds[1].round() as i32,
    ));
    ui.window().set_size(PhysicalSize::new(
        target.bounds[2].max(1.0).round() as u32,
        target.bounds[3].max(1.0).round() as u32,
    ));
}
