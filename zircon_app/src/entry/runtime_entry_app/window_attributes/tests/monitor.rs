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
