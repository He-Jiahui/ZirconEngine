use zr_rhi::{DeviceAdmissionError, DeviceFaultKind, DiagnosticReadbackTerminal, SubmissionStatus};

pub(crate) const fn diagnostic_terminal_status(
    error: DeviceAdmissionError,
) -> DiagnosticReadbackTerminal {
    match error {
        DeviceAdmissionError::Faulted {
            kind: DeviceFaultKind::DeviceLostUnknown | DeviceFaultKind::DeviceDestroyed,
        } => DiagnosticReadbackTerminal::DeviceLost,
        DeviceAdmissionError::FaultRecording | DeviceAdmissionError::Faulted { .. } => {
            DiagnosticReadbackTerminal::Shutdown
        }
    }
}

pub(crate) const fn submission_terminal_status(error: DeviceAdmissionError) -> SubmissionStatus {
    match error {
        DeviceAdmissionError::Faulted {
            kind: DeviceFaultKind::DeviceLostUnknown | DeviceFaultKind::DeviceDestroyed,
        } => SubmissionStatus::DeviceLost,
        DeviceAdmissionError::FaultRecording | DeviceAdmissionError::Faulted { .. } => {
            SubmissionStatus::Failed
        }
    }
}

#[cfg(test)]
mod tests {
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
}
