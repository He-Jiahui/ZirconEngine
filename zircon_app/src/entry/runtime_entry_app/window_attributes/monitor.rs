//! 一次启动窗口创建所需的最小显示器上下文。
//! Current 留给后端，索引仅对本次枚举有效；两个请求方共享同一快照。

use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
use zircon_runtime::core::framework::window::{WindowMode, WindowMonitorSelection, WindowPosition};

const INDEXED_MONITOR_SELECTION_CAPACITY: usize = 2;

/// 本次窗口创建使用的显示器快照；索引只对应当前事件循环的枚举次序。
pub(super) struct WindowMonitorContext {
    primary_monitor: Option<MonitorHandle>,
    indexed_monitors: [Option<(usize, MonitorHandle)>; INDEXED_MONITOR_SELECTION_CAPACITY],
}

impl WindowMonitorContext {
    /// 按位置与全屏请求获取最多两个索引，避免无需求时枚举所有显示器。
    pub(super) fn for_event_loop(
        event_loop: &dyn ActiveEventLoop,
        position: WindowPosition,
        mode: WindowMode,
    ) -> Self {
        let requested_indices = requested_monitor_indices(position, mode);
        let primary_monitor = event_loop.primary_monitor();
        let mut indexed_monitors = std::array::from_fn(|_| None);
        if let Some(last_requested_index) = requested_indices.iter().flatten().max().copied() {
            for (index, monitor) in event_loop.available_monitors().enumerate() {
                if let Some(slot) = requested_indices
                    .iter()
                    .position(|requested| *requested == Some(index))
                {
                    indexed_monitors[slot] = Some((index, monitor));
                }
                if index == last_requested_index {
                    break;
                }
            }
        }
        Self {
            primary_monitor,
            indexed_monitors,
        }
    }
}

fn requested_monitor_indices(
    position: WindowPosition,
    mode: WindowMode,
) -> [Option<usize>; INDEXED_MONITOR_SELECTION_CAPACITY] {
    let position_index = match position {
        WindowPosition::CenteredOn(WindowMonitorSelection::Index(index)) => Some(index),
        WindowPosition::Automatic
        | WindowPosition::Centered
        | WindowPosition::CenteredOn(
            WindowMonitorSelection::Current | WindowMonitorSelection::Primary,
        )
        | WindowPosition::At { .. } => None,
    };
    let mode_index = match mode {
        WindowMode::BorderlessFullscreenOn(WindowMonitorSelection::Index(index))
        | WindowMode::FullscreenOn {
            monitor: WindowMonitorSelection::Index(index),
            ..
        } => Some(index),
        WindowMode::Windowed
        | WindowMode::BorderlessFullscreen
        | WindowMode::BorderlessFullscreenOn(
            WindowMonitorSelection::Current | WindowMonitorSelection::Primary,
        )
        | WindowMode::Fullscreen
        | WindowMode::FullscreenOn {
            monitor: WindowMonitorSelection::Current | WindowMonitorSelection::Primary,
            ..
        } => None,
    };
    [
        position_index,
        if mode_index == position_index {
            None
        } else {
            mode_index
        },
    ]
}

/// 首窗没有 Current 上下文；返回 None 表示由 Winit 决定目标显示器。
pub(super) fn selected_monitor(
    monitor_context: &WindowMonitorContext,
    selection: WindowMonitorSelection,
) -> Option<MonitorHandle> {
    match selection {
        WindowMonitorSelection::Current => None,
        WindowMonitorSelection::Primary => monitor_context.primary_monitor.clone(),
        WindowMonitorSelection::Index(index) => monitor_context
            .indexed_monitors
            .iter()
            .flatten()
            .find(|(candidate, _)| *candidate == index)
            .map(|(_, monitor)| monitor.clone()),
    }
}

#[cfg(test)]
impl WindowMonitorContext {
    pub(super) fn primary_only(primary_monitor: Option<MonitorHandle>) -> Self {
        Self {
            primary_monitor,
            indexed_monitors: std::array::from_fn(|_| None),
        }
    }
}

#[cfg(test)]
#[path = "tests/monitor.rs"]
mod tests;
