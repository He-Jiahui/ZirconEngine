use winit::monitor::{MonitorHandle, VideoMode};
use zircon_runtime::core::framework::window::{WindowVideoMode, WindowVideoModeSelection};

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
mod tests {
    use super::*;
    use std::num::{NonZeroU16, NonZeroU32};
    use winit::dpi::PhysicalSize;

    #[test]
    fn video_mode_matching_treats_unspecified_fields_as_wildcards() {
        let candidate = VideoMode::new(
            PhysicalSize::new(1920, 1080),
            NonZeroU16::new(32),
            NonZeroU32::new(60_000),
        );

        assert!(video_mode_matches(
            &candidate,
            WindowVideoMode::new(1920, 1080)
        ));
        assert!(video_mode_matches(
            &candidate,
            WindowVideoMode::new(1920, 1080)
                .with_bit_depth(32)
                .with_refresh_rate_millihertz(60_000)
        ));
        assert!(!video_mode_matches(
            &candidate,
            WindowVideoMode::new(1280, 720)
        ));
        assert!(!video_mode_matches(
            &candidate,
            WindowVideoMode::new(1920, 1080).with_bit_depth(24)
        ));
        assert!(!video_mode_matches(
            &candidate,
            WindowVideoMode::new(1920, 1080).with_refresh_rate_millihertz(59_940)
        ));
    }
}
