//! 独占全屏模式的配置匹配边界。
//! 未指定的深度或刷新率不限制候选；匹配失败交上层无边框降级。

use winit::monitor::{MonitorHandle, VideoMode};
use zircon_runtime::core::framework::window::{WindowVideoMode, WindowVideoModeSelection};

/// 为独占全屏选模式；返回 None 由调用方退到无边框。
pub(super) fn selected_video_mode(
    monitor: &MonitorHandle,
    selection: WindowVideoModeSelection,
) -> Option<VideoMode> {
    match selection {
        WindowVideoModeSelection::Current => monitor.current_video_mode(),
        WindowVideoModeSelection::Specific(requested) => monitor
            .video_modes()
            .find(|candidate| video_mode_matches(candidate, requested)),
    }
}

fn video_mode_matches(candidate: &VideoMode, requested: WindowVideoMode) -> bool {
    let size = candidate.size();
    size.width == requested.physical_size.x
        && size.height == requested.physical_size.y
        && optional_video_mode_field_matches(
            requested.bit_depth,
            candidate.bit_depth().map(|value| value.get()),
        )
        && optional_video_mode_field_matches(
            requested.refresh_rate_millihertz,
            candidate.refresh_rate_millihertz().map(|value| value.get()),
        )
}

fn optional_video_mode_field_matches<T: PartialEq>(
    requested: Option<T>,
    actual: Option<T>,
) -> bool {
    match requested {
        Some(requested) => actual == Some(requested),
        None => true,
    }
}

#[cfg(test)]
#[path = "tests/video_mode.rs"]
mod tests;
