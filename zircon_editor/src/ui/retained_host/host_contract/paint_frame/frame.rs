use super::super::data::{FrameRect, HostPaneInteractionStateData};
use super::pixels;
use super::recording::HostPaintRecording;

pub(in crate::ui::retained_host::host_contract) struct HostRgbaFrame {
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
    pub(in crate::ui::retained_host::host_contract) bytes: Vec<u8>,
    pub(in crate::ui::retained_host::host_contract) paint_clip: Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract) pane_interaction_state:
        Option<HostPaneInteractionStateData>,
    pub(in crate::ui::retained_host::host_contract) recording: Option<HostPaintRecording>,
}

impl HostRgbaFrame {
    pub(in crate::ui::retained_host::host_contract) fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: Vec::new(),
            paint_clip: None,
            pane_interaction_state: None,
            recording: None,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn filled(
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Self {
        let mut bytes = vec![0; width as usize * height as usize * 4];
        pixels::fill_pixel_span(&mut bytes, color);
        Self {
            width,
            height,
            bytes,
            paint_clip: None,
            pane_interaction_state: None,
            recording: None,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn recording_only(
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            width,
            height,
            bytes: Vec::new(),
            paint_clip: None,
            pane_interaction_state: None,
            recording: Some(HostPaintRecording::record_only()),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn replace_paint_clip(
        &mut self,
        paint_clip: Option<FrameRect>,
    ) -> Option<FrameRect> {
        std::mem::replace(&mut self.paint_clip, paint_clip)
    }

    pub(in crate::ui::retained_host::host_contract) fn paint_clip(&self) -> Option<&FrameRect> {
        self.paint_clip.as_ref()
    }

    pub(in crate::ui::retained_host::host_contract) fn set_pane_interaction_state(
        &mut self,
        interaction: &HostPaneInteractionStateData,
    ) {
        self.pane_interaction_state = Some(interaction.clone());
    }

    pub(in crate::ui::retained_host::host_contract) fn pane_interaction_state(
        &self,
    ) -> Option<&HostPaneInteractionStateData> {
        self.pane_interaction_state.as_ref()
    }

    pub(in crate::ui::retained_host::host_contract) fn width(&self) -> u32 {
        self.width
    }

    pub(in crate::ui::retained_host::host_contract) fn height(&self) -> u32 {
        self.height
    }

    pub(in crate::ui::retained_host::host_contract) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::ui::retained_host::host_contract) fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub(in crate::ui::retained_host::host_contract) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}
