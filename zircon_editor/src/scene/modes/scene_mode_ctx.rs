use crate::scene::selection::SelectionModel;
use crate::scene::viewport::SceneViewportSettings;

pub struct SceneModeCtx<'a> {
    selection: &'a mut SelectionModel,
    settings: &'a SceneViewportSettings,
}

impl<'a> SceneModeCtx<'a> {
    pub fn new(selection: &'a mut SelectionModel, settings: &'a SceneViewportSettings) -> Self {
        Self {
            selection,
            settings,
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
}
