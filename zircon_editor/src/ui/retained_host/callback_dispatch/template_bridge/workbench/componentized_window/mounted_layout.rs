use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use super::super::error::BuiltinHostWindowTemplateBridgeError;
use super::{refresh_layout, BuiltinWorkbenchWindowTemplateSurfaceBridge};

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mount_frame = super::normalized_mount_frame(UiFrame::new(
            0.0,
            0.0,
            shell_size.width,
            shell_size.height,
        ));
        refresh_layout::recompute(self, shell_size)
    }

    pub(crate) fn recompute_layout_at_mount(
        &mut self,
        mount_frame: UiFrame,
    ) -> Result<UiSize, BuiltinHostWindowTemplateBridgeError> {
        let shell_size = self.prepare_layout_at_mount(mount_frame);
        refresh_layout::recompute(self, shell_size)?;
        Ok(shell_size)
    }

    pub(in crate::ui::retained_host::callback_dispatch::template_bridge::workbench) fn prepare_layout_at_mount(
        &mut self,
        mount_frame: UiFrame,
    ) -> UiSize {
        self.prepare_layout_at_mount_with_scale(mount_frame, 1.0)
    }

    pub(in crate::ui::retained_host::callback_dispatch::template_bridge::workbench) fn prepare_layout_at_mount_with_scale(
        &mut self,
        mount_frame: UiFrame,
        scale_factor: f32,
    ) -> UiSize {
        self.mount_frame = super::normalized_mount_frame(mount_frame);
        self.presentation_scale_factor = super::normalized_presentation_scale_factor(scale_factor);
        UiSize::new(
            self.mount_frame.width / self.presentation_scale_factor,
            self.mount_frame.height / self.presentation_scale_factor,
        )
    }
}
