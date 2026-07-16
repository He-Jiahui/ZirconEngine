use crate::core::commands::CommandEvalCtx;
use crate::core::editor_message::SceneModeId;
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::ViewportInput;

use super::{
    DuplicateSceneModeError, EditorSceneMode, InputOutcome, SceneModeCtx, ViewportOverlayBuilder,
};

pub struct SceneModeStack {
    base: Box<dyn EditorSceneMode>,
    overlays: Vec<Box<dyn EditorSceneMode>>,
}

impl SceneModeStack {
    pub fn new(mut base: Box<dyn EditorSceneMode>, ctx: &mut SceneModeCtx<'_>) -> Self {
        base.enter(ctx);
        Self {
            base,
            overlays: Vec::new(),
        }
    }

    pub fn active_mode_id(&self) -> &SceneModeId {
        self.overlays
            .last()
            .map_or_else(|| self.base.id(), |mode| mode.id())
    }

    pub fn project_command_eval_ctx(
        &self,
        context: CommandEvalCtx,
        selection: &SelectionModel,
    ) -> CommandEvalCtx {
        context
            .with_scene_mode(self.active_mode_id().clone())
            .with_selection_count(selection.active_items().len())
    }

    pub fn push(
        &mut self,
        mut mode: Box<dyn EditorSceneMode>,
        ctx: &mut SceneModeCtx<'_>,
    ) -> Result<(), DuplicateSceneModeError> {
        if self.contains(mode.id()) {
            return Err(DuplicateSceneModeError::new(mode.id().clone()));
        }
        mode.enter(ctx);
        self.overlays.push(mode);
        Ok(())
    }

    pub fn pop(&mut self, ctx: &mut SceneModeCtx<'_>) -> Option<SceneModeId> {
        let mut mode = self.overlays.pop()?;
        let id = mode.id().clone();
        mode.exit(ctx);
        Some(id)
    }

    pub fn handle_input(
        &mut self,
        input: &ViewportInput,
        ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        for mode in self.overlays.iter_mut().rev() {
            if mode.handle_input(input, ctx) == InputOutcome::Consumed {
                return InputOutcome::Consumed;
            }
        }
        self.base.handle_input(input, ctx)
    }

    pub fn update(&mut self, ctx: &mut SceneModeCtx<'_>) {
        self.base.update(ctx);
        for mode in &mut self.overlays {
            mode.update(ctx);
        }
    }

    pub fn build_overlay(&self, out: &mut ViewportOverlayBuilder) {
        self.base.build_overlay(out);
        for mode in &self.overlays {
            mode.build_overlay(out);
        }
    }

    pub fn shutdown(mut self, ctx: &mut SceneModeCtx<'_>) {
        while let Some(mut mode) = self.overlays.pop() {
            mode.exit(ctx);
        }
        self.base.exit(ctx);
    }

    fn contains(&self, id: &SceneModeId) -> bool {
        self.base.id() == id || self.overlays.iter().any(|mode| mode.id() == id)
    }
}

impl std::fmt::Debug for SceneModeStack {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SceneModeStack")
            .field("base", self.base.id())
            .field(
                "overlays",
                &self
                    .overlays
                    .iter()
                    .map(|mode| mode.id())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
