//! 启动窗口物理位置与显示器选择的转换。
//! 缺少显示器几何时交给后端自动摆放；大尺寸和负原点保持有界结果。

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::monitor::MonitorHandle;
use zircon_runtime::core::framework::window::{
    WindowMonitorSelection, WindowPosition, WindowResolution,
};

use super::monitor::{selected_monitor, WindowMonitorContext};

/// 有可靠显示器几何才计算居中；否则让 Winit 采用自动位置。
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
#[path = "tests/position.rs"]
mod tests;
