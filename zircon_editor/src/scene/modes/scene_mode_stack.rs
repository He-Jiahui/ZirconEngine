use crate::core::commands::CommandEvalCtx;
use crate::core::editor_message::SceneModeId;
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::ViewportInput;

use super::{
    EditorSceneMode, InputOutcome, SceneModeCtx, SceneModeStackError, ViewportOverlayBuilder,
};

pub struct SceneModeStack {
    base: Box<dyn EditorSceneMode>,
    overlays: Vec<Box<dyn EditorSceneMode>>,
}

impl SceneModeStack {
    pub fn new(
        mut base: Box<dyn EditorSceneMode>,
        ctx: &mut SceneModeCtx<'_>,
    ) -> Result<Self, SceneModeStackError> {
        enter_mode(base.as_mut(), ctx)?;
        Ok(Self {
            base,
            overlays: Vec::new(),
        })
    }

    pub fn active_mode_id(&self) -> &SceneModeId {
        self.overlays
            .last()
            .map_or_else(|| self.base.id(), |mode| mode.id())
    }

    pub fn base_mode_id(&self) -> &SceneModeId {
        self.base.id()
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
    ) -> Result<(), SceneModeStackError> {
        if self.contains(mode.id()) {
            return Err(SceneModeStackError::DuplicateMode {
                mode_id: mode.id().clone(),
            });
        }
        enter_mode(mode.as_mut(), ctx)?;
        self.overlays.push(mode);
        Ok(())
    }

    pub fn replace_base(
        &mut self,
        mut mode: Box<dyn EditorSceneMode>,
        ctx: &mut SceneModeCtx<'_>,
    ) -> Result<(), SceneModeStackError> {
        if self
            .overlays
            .iter()
            .any(|overlay| overlay.id() == mode.id())
        {
            return Err(SceneModeStackError::DuplicateMode {
                mode_id: mode.id().clone(),
            });
        }

        enter_mode(mode.as_mut(), ctx)?;
        self.base.exit(ctx);
        self.base = mode;
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
            let checkpoint = ctx.checkpoint();
            if mode.handle_input(input, ctx) == InputOutcome::Consumed {
                return InputOutcome::Consumed;
            }
            ctx.restore_after_pass_through(checkpoint);
        }

        let checkpoint = ctx.checkpoint();
        let outcome = self.base.handle_input(input, ctx);
        if outcome == InputOutcome::PassThrough {
            ctx.restore_after_pass_through(checkpoint);
        }
        outcome
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

    pub fn shutdown(&mut self, ctx: &mut SceneModeCtx<'_>) {
        while let Some(mut mode) = self.overlays.pop() {
            mode.exit(ctx);
        }
        self.base.exit(ctx);
    }

    fn contains(&self, id: &SceneModeId) -> bool {
        self.base.id() == id || self.overlays.iter().any(|mode| mode.id() == id)
    }
}

fn enter_mode(
    mode: &mut dyn EditorSceneMode,
    ctx: &mut SceneModeCtx<'_>,
) -> Result<(), SceneModeStackError> {
    let mode_id = mode.id().clone();
    mode.enter(ctx);
    if let Some(message) = mode.take_boundary_failure() {
        mode.exit(ctx);
        return Err(SceneModeStackError::EnterFailure { mode_id, message });
    }
    Ok(())
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
