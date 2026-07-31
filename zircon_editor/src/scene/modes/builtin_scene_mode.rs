use crate::core::editor_message::SceneModeId;
use crate::scene::viewport::ViewportInput;

use super::{
    EditorSceneMode, InputOutcome, SceneModeCtx, SceneModeInputEffect, ViewportOverlayBuilder,
};

pub(crate) struct SelectSceneMode {
    id: SceneModeId,
}

impl SelectSceneMode {
    pub(crate) fn new() -> Self {
        Self {
            id: SceneModeId::new("scene.select"),
        }
    }
}

impl EditorSceneMode for SelectSceneMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(&mut self, input: &ViewportInput, ctx: &mut SceneModeCtx<'_>) -> InputOutcome {
        emit_primary_pointer_effect(input, false, ctx)
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

pub(crate) struct TransformSceneMode {
    id: SceneModeId,
}

impl TransformSceneMode {
    pub(crate) fn new() -> Self {
        Self {
            id: SceneModeId::new("scene.transform"),
        }
    }
}

impl EditorSceneMode for TransformSceneMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(&mut self, input: &ViewportInput, ctx: &mut SceneModeCtx<'_>) -> InputOutcome {
        emit_primary_pointer_effect(input, true, ctx)
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

fn emit_primary_pointer_effect(
    input: &ViewportInput,
    allow_handle_drag: bool,
    ctx: &mut SceneModeCtx<'_>,
) -> InputOutcome {
    let effect = match input {
        ViewportInput::PointerMoved(position) => SceneModeInputEffect::PointerMoved(*position),
        ViewportInput::LeftPressed {
            position,
            selection_mutation,
        } => SceneModeInputEffect::PrimaryPressed {
            position: *position,
            allow_handle_drag,
            selection_mutation: *selection_mutation,
        },
        ViewportInput::LeftReleased => SceneModeInputEffect::PrimaryReleased,
        _ => return InputOutcome::PassThrough,
    };
    ctx.push_input_effect(effect);
    InputOutcome::Consumed
}
