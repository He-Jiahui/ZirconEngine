use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{UiRenderFrameCommandRef, UiSurfaceFrame};

use super::super::super::data::FrameRect;
use super::super::geometry::visible_frame;
use super::model::{
    HostRecordedFrame, HostRecordedPaintCommand, HostRecordedPaintKind, HostRenderCommandSource,
};
use super::source_table::{HostRenderSourceKey, HostRenderSourceTable};

#[derive(Clone, Debug, Default)]
pub(in crate::ui::retained_host::host_contract) struct HostPaintRecording {
    commands: Vec<HostRecordedPaintCommand>,
    next_z_index: i32,
    record_only: bool,
    render_sources: HostRenderSourceTable,
    current_source_surface: Option<HostRenderSourceKey>,
    current_source_command: Option<UiRenderFrameCommandRef>,
    source_fragment_counts: HashMap<(HostRenderSourceKey, UiRenderFrameCommandRef), u32>,
}

const MAX_REUSABLE_COMMANDS: usize = 32_768;
thread_local! {
    // One scratch set per paint thread, shared sequentially by its native presenters.
    // Only cleared containers enter these slots; frame/source ownership stays with the consumer.
    static RECORDING_SCRATCH: RefCell<Option<HostPaintRecording>> = const { RefCell::new(None) };
    static COMMAND_SCRATCH: RefCell<Vec<HostRecordedPaintCommand>> = const { RefCell::new(Vec::new()) };
    static SOURCE_SCRATCH: RefCell<HostRenderSourceTable> = RefCell::new(HostRenderSourceTable::default());
}

impl HostPaintRecording {
    pub(in crate::ui::retained_host::host_contract) fn record_only() -> Self {
        let mut recording = RECORDING_SCRATCH
            .with_borrow_mut(Option::take)
            .unwrap_or_default();
        recording.commands = COMMAND_SCRATCH.with_borrow_mut(std::mem::take);
        recording.render_sources = SOURCE_SCRATCH.with_borrow_mut(std::mem::take);
        recording.record_only = true;
        recording
    }

    pub(in crate::ui::retained_host::host_contract) fn command_count(&self) -> usize {
        self.commands.len()
    }
    pub(in crate::ui::retained_host::host_contract) fn commands_since(
        &self,
        start: usize,
    ) -> &[HostRecordedPaintCommand] {
        self.commands.get(start..).unwrap_or_default()
    }
    pub(in crate::ui::retained_host::host_contract) fn is_record_only(&self) -> bool {
        self.record_only
    }

    pub(in crate::ui::retained_host::host_contract) fn into_commands(
        self,
    ) -> Vec<HostRecordedPaintCommand> {
        self.into_frame().commands
    }

    pub(in crate::ui::retained_host::host_contract) fn into_frame(mut self) -> HostRecordedFrame {
        let frame = HostRecordedFrame {
            commands: std::mem::take(&mut self.commands),
            render_sources: std::mem::take(&mut self.render_sources),
        };
        self.next_z_index = 0;
        self.record_only = false;
        self.current_source_surface = None;
        self.current_source_command = None;
        self.source_fragment_counts.clear();
        if self.source_fragment_counts.capacity() <= MAX_REUSABLE_COMMANDS {
            RECORDING_SCRATCH.with_borrow_mut(|slot| *slot = Some(self));
        }
        frame
    }

    pub(in crate::ui::retained_host::host_contract) fn recycle_commands(
        mut commands: Vec<HostRecordedPaintCommand>,
    ) {
        commands.clear();
        if commands.capacity() <= MAX_REUSABLE_COMMANDS {
            COMMAND_SCRATCH.with_borrow_mut(|slot| {
                if commands.capacity() > slot.capacity() {
                    *slot = commands;
                }
            });
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn recycle_sources(
        mut sources: HostRenderSourceTable,
    ) {
        sources.clear();
        SOURCE_SCRATCH.with_borrow_mut(|slot| *slot = sources);
    }

    pub(in crate::ui::retained_host::host_contract) fn replace_source_surface(
        &mut self,
        source_frame: Option<&Arc<UiSurfaceFrame>>,
    ) -> Option<HostRenderSourceKey> {
        let source_key = source_frame.and_then(|frame| self.render_sources.register(frame));
        std::mem::replace(&mut self.current_source_surface, source_key)
    }

    pub(in crate::ui::retained_host::host_contract) fn restore_source_surface(
        &mut self,
        source_key: Option<HostRenderSourceKey>,
    ) {
        self.current_source_surface = source_key;
    }

    pub(in crate::ui::retained_host::host_contract) fn replace_source_command(
        &mut self,
        command_ref: Option<UiRenderFrameCommandRef>,
    ) -> Option<UiRenderFrameCommandRef> {
        std::mem::replace(&mut self.current_source_command, command_ref)
    }

    pub(in crate::ui::retained_host::host_contract) fn restore_source_command(
        &mut self,
        command_ref: Option<UiRenderFrameCommandRef>,
    ) {
        self.current_source_command = command_ref;
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
        let source = self.next_render_command_source();
        self.commands.push(HostRecordedPaintCommand {
            frame,
            clip_frame,
            z_index,
            source,
            kind,
        });
    }

    fn next_render_command_source(&mut self) -> Option<HostRenderCommandSource> {
        let (surface_key, command_ref) = self
            .current_source_surface
            .zip(self.current_source_command)?;
        let next_fragment_index = self
            .source_fragment_counts
            .entry((surface_key, command_ref))
            .or_default();
        let fragment_index = u16::try_from(*next_fragment_index).ok()?;
        *next_fragment_index = next_fragment_index.saturating_add(1);
        Some(HostRenderCommandSource {
            surface_key,
            command_ref,
            fragment_index,
        })
    }
}

#[cfg(test)]
mod scratch_tests {
    use super::*;
    #[test]
    fn recording_scratch_reuses_capacity_without_replaying_previous_commands() {
        let mut first = HostPaintRecording::record_only();
        first.record_command(
            FrameRect {
                width: 2.0,
                height: 2.0,
                ..Default::default()
            },
            None,
            HostRecordedPaintKind::Quad {
                color: [1; 4],
                corner_radius: 0.0,
            },
        );
        let frame = first.into_frame();
        let capacity = frame.commands.capacity();
        HostPaintRecording::recycle_commands(frame.commands);
        HostPaintRecording::recycle_sources(frame.render_sources);
        let next = HostPaintRecording::record_only();
        assert!(next.commands.capacity() >= capacity);
        assert_eq!(next.command_count(), 0);
        assert_eq!(next.next_z_index, 0);
        assert!(next.source_fragment_counts.is_empty());
        assert!(next.current_source_surface.is_none());
    }
}
