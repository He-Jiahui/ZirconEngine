use super::super::HostRgbaFrame;
use super::super::recording::{HostPaintRecording, HostRecordedPaintCommand};

impl HostRgbaFrame {
    pub(in crate::ui::retained_host::host_contract) fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    pub(in crate::ui::retained_host::host_contract) fn record_only(&self) -> bool {
        self.recording
            .as_ref()
            .is_some_and(HostPaintRecording::is_record_only)
    }

    pub(in crate::ui::retained_host::host_contract) fn into_recorded_commands(
        self,
    ) -> Vec<HostRecordedPaintCommand> {
        self.recording
            .map(HostPaintRecording::into_commands)
            .unwrap_or_default()
    }
}
