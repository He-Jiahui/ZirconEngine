use std::cell::RefCell;

use crate::core::framework::script::{ScriptHostError, ScriptHostValue};
use crate::core::{CoreHandle, CoreWeak};
use crate::scene::{EntityId, LevelSystem};

#[derive(Clone, Debug)]
pub struct ScriptRuntimeCallContext {
    pub core: CoreWeak,
    pub level: LevelSystem,
    pub entity: EntityId,
    pub delta_seconds: f32,
}

thread_local! {
    static SCRIPT_RUNTIME_CALL_CONTEXT: RefCell<Option<ScriptRuntimeCallContext>> =
        const { RefCell::new(None) };
}

pub fn with_script_runtime_call_context<R>(
    context: ScriptRuntimeCallContext,
    call: impl FnOnce() -> R,
) -> R {
    struct ContextResetGuard(Option<ScriptRuntimeCallContext>);

    impl Drop for ContextResetGuard {
        fn drop(&mut self) {
            let previous = self.0.take();
            SCRIPT_RUNTIME_CALL_CONTEXT.with(|slot| {
                *slot.borrow_mut() = previous;
            });
        }
    }

    let previous = SCRIPT_RUNTIME_CALL_CONTEXT.with(|slot| slot.replace(Some(context)));
    let _guard = ContextResetGuard(previous);
    call()
}

pub fn current_script_runtime_call_context() -> Result<ScriptRuntimeCallContext, ScriptHostError> {
    SCRIPT_RUNTIME_CALL_CONTEXT.with(|slot| {
        slot.borrow()
            .clone()
            .ok_or_else(|| ScriptHostError::new("script runtime context is not active"))
    })
}

impl ScriptRuntimeCallContext {
    pub fn core_handle(&self) -> Result<CoreHandle, ScriptHostError> {
        self.core
            .upgrade()
            .ok_or_else(|| ScriptHostError::new("script runtime core handle is no longer active"))
    }
}

pub fn script_float(value: f32) -> ScriptHostValue {
    ScriptHostValue::Float(f64::from(value))
}
