#[cfg(feature = "profiling")]
use zircon_runtime_interface::ProfileControlCommand;

#[cfg(feature = "profiling")]
const PERFORMANCE_TIMELINE_START_CAPTURE_ACTION: &str =
    "workbench.performance_timeline.capture.start";
#[cfg(feature = "profiling")]
const PERFORMANCE_TIMELINE_STOP_CAPTURE_ACTION: &str =
    "workbench.performance_timeline.capture.stop";
#[cfg(feature = "profiling")]
const PERFORMANCE_TIMELINE_EXPORT_REPORT_ACTION: &str =
    "workbench.performance_timeline.report.export";
#[cfg(feature = "profiling")]
const PERFORMANCE_TIMELINE_RESET_ACTION: &str = "workbench.performance_timeline.reset";

#[cfg(feature = "profiling")]
pub(super) fn profile_command_for_action(action_id: &str) -> Option<ProfileControlCommand> {
    match action_id {
        PERFORMANCE_TIMELINE_START_CAPTURE_ACTION => Some(ProfileControlCommand::StartCapture),
        PERFORMANCE_TIMELINE_STOP_CAPTURE_ACTION => Some(ProfileControlCommand::StopCapture),
        PERFORMANCE_TIMELINE_EXPORT_REPORT_ACTION => Some(ProfileControlCommand::ExportReport),
        PERFORMANCE_TIMELINE_RESET_ACTION => Some(ProfileControlCommand::Reset),
        _ => None,
    }
}

#[cfg(all(test, feature = "profiling"))]
mod tests {
    use super::*;

    #[test]
    fn performance_timeline_actions_map_to_profile_control_commands() {
        assert_eq!(
            profile_command_for_action("workbench.performance_timeline.capture.start"),
            Some(ProfileControlCommand::StartCapture)
        );
        assert_eq!(
            profile_command_for_action("workbench.performance_timeline.capture.stop"),
            Some(ProfileControlCommand::StopCapture)
        );
        assert_eq!(
            profile_command_for_action("workbench.performance_timeline.report.export"),
            Some(ProfileControlCommand::ExportReport)
        );
        assert_eq!(
            profile_command_for_action("workbench.performance_timeline.reset"),
            Some(ProfileControlCommand::Reset)
        );
        assert_eq!(
            profile_command_for_action("workbench.performance_timeline.unknown"),
            None
        );
    }
}
