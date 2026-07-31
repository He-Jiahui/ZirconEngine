use std::cell::{Cell, RefCell};

use crate::core::editor_message::SceneModeId;
use crate::core::plugin::run_editor_plugin_boundary;
use crate::scene::viewport::ViewportInput;

use super::{EditorSceneMode, InputOutcome, SceneModeCtx, ViewportOverlayBuilder};

pub(super) struct IsolatedSceneMode {
    owner_id: String,
    mode_id: SceneModeId,
    inner: Option<Box<dyn EditorSceneMode>>,
    faulted: Cell<bool>,
    entered: bool,
    last_failure: RefCell<Option<String>>,
}

impl IsolatedSceneMode {
    pub(super) fn new(
        owner_id: String,
        mode_id: SceneModeId,
        inner: Box<dyn EditorSceneMode>,
    ) -> Self {
        Self {
            owner_id,
            mode_id,
            inner: Some(inner),
            faulted: Cell::new(false),
            entered: false,
            last_failure: RefCell::new(None),
        }
    }

    pub(super) fn validate_inner_id(&self) -> Result<SceneModeId, String> {
        let owner_id = self.owner_id.clone();
        let result = run_editor_plugin_boundary(&owner_id, "scene mode id", || {
            Ok(self
                .inner
                .as_deref()
                .expect("isolated scene mode owns its inner mode")
                .id()
                .clone())
        });
        result.map_err(|error| {
            let message = error.to_string();
            self.record_failure(message.clone());
            message
        })
    }

    fn run_with_ctx(
        &mut self,
        operation: &'static str,
        run_when_faulted: bool,
        ctx: &mut SceneModeCtx<'_>,
        callback: impl FnOnce(&mut dyn EditorSceneMode, &mut SceneModeCtx<'_>),
    ) -> bool {
        if self.faulted.get() && !run_when_faulted {
            return false;
        }
        let checkpoint = ctx.checkpoint();
        let owner_id = self.owner_id.clone();
        let result = run_editor_plugin_boundary(&owner_id, operation, || {
            callback(
                self.inner
                    .as_deref_mut()
                    .expect("isolated scene mode owns its inner mode"),
                ctx,
            );
            Ok(())
        });
        if let Err(error) = result {
            ctx.restore(checkpoint);
            ctx.invalidate_overlay();
            self.record_failure(error.to_string());
            return false;
        }
        true
    }

    fn record_failure(&self, message: String) {
        self.faulted.set(true);
        self.last_failure.replace(Some(message));
    }
}

impl EditorSceneMode for IsolatedSceneMode {
    fn id(&self) -> &SceneModeId {
        &self.mode_id
    }

    fn enter(&mut self, ctx: &mut SceneModeCtx<'_>) {
        self.entered = true;
        self.run_with_ctx("scene mode enter", false, ctx, |mode, ctx| mode.enter(ctx));
    }

    fn exit(&mut self, ctx: &mut SceneModeCtx<'_>) {
        if !self.entered {
            return;
        }
        self.run_with_ctx("scene mode exit", true, ctx, |mode, ctx| mode.exit(ctx));
        self.entered = false;
    }

    fn handle_input(&mut self, input: &ViewportInput, ctx: &mut SceneModeCtx<'_>) -> InputOutcome {
        let mut outcome = InputOutcome::PassThrough;
        self.run_with_ctx("scene mode input", false, ctx, |mode, ctx| {
            outcome = mode.handle_input(input, ctx);
        });
        outcome
    }

    fn update(&mut self, ctx: &mut SceneModeCtx<'_>) {
        self.run_with_ctx("scene mode update", false, ctx, |mode, ctx| {
            mode.update(ctx)
        });
    }

    fn build_overlay(&self, out: &mut ViewportOverlayBuilder) {
        if self.faulted.get() {
            return;
        }
        let checkpoint = out.checkpoint();
        let result = run_editor_plugin_boundary(&self.owner_id, "scene mode overlay", || {
            self.inner
                .as_deref()
                .expect("isolated scene mode owns its inner mode")
                .build_overlay(out);
            Ok(())
        });
        if let Err(error) = result {
            out.restore(checkpoint);
            self.record_failure(error.to_string());
        }
    }

    fn take_boundary_failure(&mut self) -> Option<String> {
        self.last_failure.get_mut().take()
    }
}

impl Drop for IsolatedSceneMode {
    fn drop(&mut self) {
        let Some(inner) = self.inner.take() else {
            return;
        };
        let owner_id = self.owner_id.clone();
        let _ = run_editor_plugin_boundary(&owner_id, "scene mode drop", move || {
            drop(inner);
            Ok(())
        });
    }
}
