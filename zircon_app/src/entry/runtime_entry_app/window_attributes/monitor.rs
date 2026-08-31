use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
use zircon_runtime::core::framework::window::{WindowMode, WindowMonitorSelection, WindowPosition};

const INDEXED_MONITOR_SELECTION_CAPACITY: usize = 2;

pub(super) struct WindowMonitorContext {
    primary_monitor: Option<MonitorHandle>,
    indexed_monitors: [Option<(usize, MonitorHandle)>; INDEXED_MONITOR_SELECTION_CAPACITY],
}

impl WindowMonitorContext {
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
mod tests {
    use zircon_runtime::core::framework::window::WindowVideoModeSelection;

    use super::*;

    #[test]
    fn monitor_index_demand_keeps_two_distinct_descriptor_indices() {
        assert_eq!(
            requested_monitor_indices(
                WindowPosition::CenteredOn(WindowMonitorSelection::Index(7)),
                WindowMode::FullscreenOn {
                    monitor: WindowMonitorSelection::Index(11),
                    video_mode: WindowVideoModeSelection::Current,
                },
            ),
            [Some(7), Some(11)]
        );
    }

    #[test]
    fn monitor_index_demand_deduplicates_and_ignores_non_index_selections() {
        assert_eq!(
            requested_monitor_indices(
                WindowPosition::CenteredOn(WindowMonitorSelection::Index(4)),
                WindowMode::BorderlessFullscreenOn(WindowMonitorSelection::Index(4)),
            ),
            [Some(4), None]
        );
        assert_eq!(
            requested_monitor_indices(WindowPosition::Centered, WindowMode::Fullscreen),
            [None, None]
        );
        assert_eq!(
            requested_monitor_indices(
                WindowPosition::CenteredOn(WindowMonitorSelection::Current),
                WindowMode::BorderlessFullscreenOn(WindowMonitorSelection::Primary),
            ),
            [None, None]
        );
    }
}
