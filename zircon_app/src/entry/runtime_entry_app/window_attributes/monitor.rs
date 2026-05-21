use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
use zircon_runtime::core::framework::window::WindowMonitorSelection;

pub(super) struct WindowMonitorContext {
    primary_monitor: Option<MonitorHandle>,
    available_monitors: Vec<MonitorHandle>,
}

impl WindowMonitorContext {
    pub(super) fn for_event_loop(event_loop: &dyn ActiveEventLoop) -> Self {
        Self {
            primary_monitor: event_loop.primary_monitor(),
            available_monitors: event_loop.available_monitors().collect::<Vec<_>>(),
        }
    }

    #[cfg(test)]
    pub(super) fn primary_only(primary_monitor: Option<MonitorHandle>) -> Self {
        Self {
            primary_monitor,
            available_monitors: Vec::new(),
        }
    }
}

pub(super) fn selected_monitor(
    monitor_context: &WindowMonitorContext,
    selection: WindowMonitorSelection,
) -> Option<MonitorHandle> {
    match selection {
        WindowMonitorSelection::Current => None,
        WindowMonitorSelection::Primary => monitor_context.primary_monitor.clone(),
        WindowMonitorSelection::Index(index) => {
            monitor_context.available_monitors.get(index).cloned()
        }
    }
}
