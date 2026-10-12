use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{UiRenderFrameCommandRef, UiSurfaceFrame};

use super::super::HostRgbaFrame;

impl HostRgbaFrame {
    pub(in crate::ui::retained_host::host_contract) fn with_render_source_frame<T>(
        &mut self,
        source_frame: Option<&Arc<UiSurfaceFrame>>,
        draw: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self.recording.as_mut().map(|recording| {
            let surface = recording.replace_source_surface(source_frame);
            let command = recording.replace_source_command(None);
            (surface, command)
        });
        let result = draw(self);
        if let Some((surface, command)) = previous {
            let recording = self
                .recording
                .as_mut()
                .expect("recording state cannot disappear inside a paint scope");
            recording.restore_source_command(command);
            recording.restore_source_surface(surface);
        }
        result
    }

    pub(in crate::ui::retained_host::host_contract) fn with_render_source_command<T>(
        &mut self,
        command_ref: Option<UiRenderFrameCommandRef>,
        draw: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self
            .recording
            .as_mut()
            .map(|recording| recording.replace_source_command(command_ref));
        let result = draw(self);
        if let Some(previous) = previous {
            self.recording
                .as_mut()
                .expect("recording state cannot disappear inside a paint scope")
                .restore_source_command(previous);
        }
        result
    }
}
