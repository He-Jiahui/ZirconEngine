use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::monitor::MonitorHandle;
use zircon_runtime::core::framework::window::{
    WindowMonitorSelection, WindowPosition, WindowResolution,
};

use super::monitor::{selected_monitor, WindowMonitorContext};

pub(super) fn runtime_window_position(
    position: WindowPosition,
    resolution: &WindowResolution,
    monitor_context: &WindowMonitorContext,
) -> Option<PhysicalPosition<i32>> {
    match position {
        WindowPosition::Automatic => None,
        WindowPosition::Centered => centered_window_position_for_selection(
            resolution,
            monitor_context,
            WindowMonitorSelection::Primary,
        ),
        WindowPosition::CenteredOn(monitor) => {
            centered_window_position_for_selection(resolution, monitor_context, monitor)
        }
        WindowPosition::At { x, y } => Some(PhysicalPosition::new(x, y)),
    }
}

fn centered_window_position_for_selection(
    resolution: &WindowResolution,
    monitor_context: &WindowMonitorContext,
    selection: WindowMonitorSelection,
) -> Option<PhysicalPosition<i32>> {
    let monitor = selected_monitor(monitor_context, selection)?;
    centered_window_position(resolution, &monitor)
}

fn centered_window_position(
    resolution: &WindowResolution,
    monitor: &MonitorHandle,
) -> Option<PhysicalPosition<i32>> {
    let monitor_position = monitor.position()?;
    // winit 0.31 exposes monitor dimensions through the current video mode.
    let monitor_size = monitor.current_video_mode()?.size();
    let window_size = resolution.physical_size();

    Some(centered_physical_position(
        monitor_position,
        monitor_size,
        PhysicalSize::new(window_size.x, window_size.y),
    ))
}

fn centered_physical_position(
    monitor_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    window_size: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    let x = i64::from(monitor_position.x)
        + i64::from(monitor_size.width.saturating_sub(window_size.width)) / 2;
    let y = i64::from(monitor_position.y)
        + i64::from(monitor_size.height.saturating_sub(window_size.height)) / 2;

    PhysicalPosition::new(saturating_i64_to_i32(x), saturating_i64_to_i32(y))
}

fn saturating_i64_to_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_physical_position_uses_monitor_origin_and_size() {
        let position = centered_physical_position(
            PhysicalPosition::new(-1920, 100),
            PhysicalSize::new(1920, 1080),
            PhysicalSize::new(800, 600),
        );

        assert_eq!(position, PhysicalPosition::new(-1360, 340));
    }

    #[test]
    fn centered_physical_position_keeps_oversized_windows_at_monitor_origin() {
        let position = centered_physical_position(
            PhysicalPosition::new(12, -34),
            PhysicalSize::new(640, 480),
            PhysicalSize::new(800, 600),
        );

        assert_eq!(position, PhysicalPosition::new(12, -34));
    }

    #[test]
    fn centered_physical_position_saturates_output_coordinates() {
        let position = centered_physical_position(
            PhysicalPosition::new(i32::MAX, i32::MIN),
            PhysicalSize::new(u32::MAX, u32::MAX),
            PhysicalSize::new(1, 1),
        );

        assert_eq!(position, PhysicalPosition::new(i32::MAX, -1));
    }
}
