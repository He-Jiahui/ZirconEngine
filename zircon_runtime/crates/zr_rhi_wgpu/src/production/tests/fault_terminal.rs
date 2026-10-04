use super::*;

#[test]
fn terminal_status_distinguishes_device_loss_from_other_faults() {
    assert_eq!(
        submission_terminal_status(DeviceAdmissionError::Faulted {
            kind: DeviceFaultKind::DeviceDestroyed,
        }),
        SubmissionStatus::DeviceLost
    );
    assert_eq!(
        submission_terminal_status(DeviceAdmissionError::FaultRecording),
        SubmissionStatus::Failed
    );
    assert_eq!(
        diagnostic_terminal_status(DeviceAdmissionError::Faulted {
            kind: DeviceFaultKind::DeviceDestroyed,
        }),
        DiagnosticReadbackTerminal::DeviceLost
    );
}
