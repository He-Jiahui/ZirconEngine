mod controls;

use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::profiling_artifacts::UiProfileNamedFrame;

use self::controls::collect_surface_frame_control_nodes;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn collect_surface_frame_controls(
    kind: &str,
    surface: &str,
    origin: &FrameRect,
    surface_frame: Option<&UiSurfaceFrame>,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    let Some(surface_frame) = surface_frame else {
        return;
    };
    collect_surface_frame_control_nodes(kind, surface, origin, surface_frame, out);
}

#[cfg(not(test))]
pub(super) fn collect_surface_frame_controls(
    kind: &str,
    surface: &str,
    origin: &FrameRect,
    surface_frame: Option<&UiSurfaceFrame>,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    let Some(surface_frame) = surface_frame else {
        return;
    };
    collect_surface_frame_control_nodes(kind, surface, origin, surface_frame, out);
}
