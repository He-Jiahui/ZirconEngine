use super::super::super::data::FrameRect;
use super::super::geometry::visible_frame;
use super::model::{HostRecordedPaintCommand, HostRecordedPaintKind};

#[derive(Clone, Debug, Default)]
pub(in crate::ui::retained_host::host_contract) struct HostPaintRecording {
    commands: Vec<HostRecordedPaintCommand>,
    next_z_index: i32,
    record_only: bool,
}

impl HostPaintRecording {
    pub(in crate::ui::retained_host::host_contract) fn record_only() -> Self {
        Self {
            record_only: true,
            ..Self::default()
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn is_record_only(&self) -> bool {
        self.record_only
    }

    pub(in crate::ui::retained_host::host_contract) fn into_commands(
        self,
    ) -> Vec<HostRecordedPaintCommand> {
        self.commands
    }

    pub(in crate::ui::retained_host::host_contract) fn record_command(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        kind: HostRecordedPaintKind,
    ) {
        if !visible_frame(&frame) {
            return;
        }
        let z_index = self.next_z_index;
        self.next_z_index = self.next_z_index.saturating_add(1);
        self.commands.push(HostRecordedPaintCommand {
            frame,
            clip_frame,
            z_index,
            kind,
        });
    }
}
