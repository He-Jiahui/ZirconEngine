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

    pub(in crate::ui::retained_host::host_contract) fn into_frame(self) -> HostRecordedFrame {
        HostRecordedFrame {
            commands: self.commands,
            render_sources: self.render_sources,
        }
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
