//! 设备故障在此统一折算为提交与诊断终态，避免同一故障给两个消费者不同结果。
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
#[path = "tests/fault_terminal.rs"]
mod tests;
