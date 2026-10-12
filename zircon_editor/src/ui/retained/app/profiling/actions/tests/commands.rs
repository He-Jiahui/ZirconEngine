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
