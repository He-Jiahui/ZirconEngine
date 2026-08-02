use crate::scene::selection::SelectionModel;
use crate::scene::viewport::SceneViewportSettings;

use super::SceneModeInputEffect;

pub struct SceneModeCtx<'a> {
    selection: &'a mut SelectionModel,
    settings: &'a SceneViewportSettings,
    input_effect: Option<SceneModeInputEffect>,
    overlay_invalidated: bool,
}

pub(crate) struct SceneModeCtxCheckpoint {
    selection: SelectionModel,
    input_effect: Option<SceneModeInputEffect>,
    overlay_invalidated: bool,
}

impl<'a> SceneModeCtx<'a> {
    pub fn new(selection: &'a mut SelectionModel, settings: &'a SceneViewportSettings) -> Self {
        Self {
            selection,
            settings,
            input_effect: None,
            overlay_invalidated: false,
        }
    }

    pub fn selection(&self) -> &SelectionModel {
        self.selection
    }

    pub fn selection_mut(&mut self) -> &mut SelectionModel {
        self.selection
    }

    pub fn settings(&self) -> &SceneViewportSettings {
        self.settings
    }

    pub fn invalidate_overlay(&mut self) {
        self.overlay_invalidated = true;
    }

    pub(crate) fn take_overlay_invalidation(&mut self) -> bool {
        std::mem::take(&mut self.overlay_invalidated)
    }

    pub(crate) fn push_input_effect(&mut self, effect: SceneModeInputEffect) {
        assert!(
            self.input_effect.is_none(),
            "a scene mode input dispatch may emit only one effect"
        );
        self.input_effect = Some(effect);
    }

    pub(crate) fn take_input_effect(&mut self) -> Option<SceneModeInputEffect> {
        self.input_effect.take()
    }

    pub(crate) fn checkpoint(&self) -> SceneModeCtxCheckpoint {
        SceneModeCtxCheckpoint {
            selection: self.selection.clone(),
            input_effect: self.input_effect,
            overlay_invalidated: self.overlay_invalidated,
        }
    }

    pub(crate) fn restore(&mut self, checkpoint: SceneModeCtxCheckpoint) {
        *self.selection = checkpoint.selection;
        self.input_effect = checkpoint.input_effect;
        self.overlay_invalidated = checkpoint.overlay_invalidated;
    }

    pub(crate) fn restore_after_pass_through(&mut self, checkpoint: SceneModeCtxCheckpoint) {
        let overlay_invalidated = self.take_overlay_invalidation();
        self.restore(checkpoint);
        if overlay_invalidated {
            self.invalidate_overlay();
        }
    }
}
