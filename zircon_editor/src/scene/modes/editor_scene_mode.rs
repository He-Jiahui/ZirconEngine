use crate::core::editor_message::SceneModeId;
use crate::scene::viewport::ViewportInput;

use super::{InputOutcome, SceneModeCtx, ViewportOverlayBuilder};

pub trait EditorSceneMode: Send {
    fn id(&self) -> &SceneModeId;
    fn enter(&mut self, ctx: &mut SceneModeCtx<'_>);
    fn exit(&mut self, ctx: &mut SceneModeCtx<'_>);
    fn handle_input(&mut self, input: &ViewportInput, ctx: &mut SceneModeCtx<'_>) -> InputOutcome;

    fn update(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}

    #[doc(hidden)]
    fn take_boundary_failure(&mut self) -> Option<String> {
        None
    }
}
