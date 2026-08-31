use crate::core::editor_message::SceneModeId;
use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::ViewportInput;
use zircon_runtime_interface::math::Vec2;

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
        emit_primary_pointer_effect(input, selection_primary_pressed, ctx)
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
        emit_primary_pointer_effect(input, transform_primary_pressed, ctx)
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

fn emit_primary_pointer_effect(
    input: &ViewportInput,
    primary_pressed: fn(Vec2, SelectionMutation) -> SceneModeInputEffect,
    ctx: &mut SceneModeCtx<'_>,
) -> InputOutcome {
    let effect = match input {
        ViewportInput::PointerMoved(position) => SceneModeInputEffect::PointerMoved(*position),
        ViewportInput::LeftPressed {
            position,
            selection_mutation,
        } => primary_pressed(*position, *selection_mutation),
        ViewportInput::LeftReleased => SceneModeInputEffect::PrimaryReleased,
        _ => return InputOutcome::PassThrough,
    };
    ctx.push_input_effect(effect);
    InputOutcome::Consumed
}

fn selection_primary_pressed(
    position: Vec2,
    selection_mutation: SelectionMutation,
) -> SceneModeInputEffect {
    SceneModeInputEffect::SelectionPrimaryPressed {
        position,
        selection_mutation,
    }
}

fn transform_primary_pressed(
    position: Vec2,
    selection_mutation: SelectionMutation,
) -> SceneModeInputEffect {
    SceneModeInputEffect::TransformPrimaryPressed {
        position,
        selection_mutation,
    }
}
