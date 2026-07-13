use super::super::render::ScreenSpaceUiTextBatch;
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

#[derive(Clone, Debug, Default)]
pub(super) struct ResolvedScreenSpaceUiTextBatches {
    pub(super) native_texts: Vec<ScreenSpaceUiTextBatch>,
    pub(super) sdf_texts: Vec<ScreenSpaceUiTextBatch>,
}

impl ResolvedScreenSpaceUiTextBatches {
    pub(super) fn from_explicit_batches(
        native_texts: &[ScreenSpaceUiTextBatch],
        sdf_texts: &[ScreenSpaceUiTextBatch],
    ) -> Self {
        Self {
            native_texts: native_texts.to_vec(),
            sdf_texts: sdf_texts.to_vec(),
        }
    }

    pub(super) fn push_resolved_auto_text(
        &mut self,
        text: ScreenSpaceUiTextBatch,
        resolved_mode: UiTextRenderMode,
    ) {
        match resolved_mode {
            UiTextRenderMode::Auto | UiTextRenderMode::Native => self.native_texts.push(text),
            UiTextRenderMode::Sdf | UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf => {
                self.sdf_texts.push(text)
            }
        }
    }

    pub(super) fn native_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.native_texts
    }

    pub(super) fn sdf_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.sdf_texts
    }
}
