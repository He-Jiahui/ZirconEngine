use winit::monitor::Fullscreen;
use zircon_runtime::core::framework::window::{
    WindowMode, WindowMonitorSelection, WindowVideoModeSelection,
};

use super::monitor::{selected_monitor, WindowMonitorContext};
use super::video_mode::selected_video_mode;

pub(super) fn runtime_window_fullscreen(
    mode: WindowMode,
    monitor_context: &WindowMonitorContext,
) -> Option<Fullscreen> {
    match mode {
        WindowMode::Windowed => None,
        WindowMode::BorderlessFullscreen => Some(borderless_fullscreen_for_selection(
            monitor_context,
            WindowMonitorSelection::Primary,
        )),
        WindowMode::BorderlessFullscreenOn(monitor) => Some(borderless_fullscreen_for_selection(
            monitor_context,
            monitor,
        )),
        WindowMode::Fullscreen => Some(fullscreen_for_selection(
            monitor_context,
            WindowMonitorSelection::Primary,
            WindowVideoModeSelection::Current,
        )),
        WindowMode::FullscreenOn {
            monitor,
            video_mode,
        } => Some(fullscreen_for_selection(
            monitor_context,
            monitor,
            video_mode,
        )),
    }
}

fn borderless_fullscreen_for_selection(
    monitor_context: &WindowMonitorContext,
    selection: WindowMonitorSelection,
) -> Fullscreen {
    Fullscreen::Borderless(selected_monitor(monitor_context, selection))
}

fn fullscreen_for_selection(
    monitor_context: &WindowMonitorContext,
    monitor_selection: WindowMonitorSelection,
    video_mode_selection: WindowVideoModeSelection,
) -> Fullscreen {
    let monitor = selected_monitor(monitor_context, monitor_selection);
    if let Some(monitor) = monitor.clone() {
        if let Some(video_mode) = selected_video_mode(&monitor, video_mode_selection) {
            return Fullscreen::Exclusive(monitor, video_mode);
        }
    }

    Fullscreen::Borderless(monitor)
}
