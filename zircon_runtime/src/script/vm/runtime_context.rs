//! 同步宿主调用把 Core/Level/实体上下文动态限定在当前线程和调用栈；回调退出即撤销借用，反射世界访问令牌不能外逃。
use std::cell::Cell;

use crate::core::framework::script::{
    ScriptHostCallFrame, ScriptHostError, ScriptHostHotPathMetrics, ScriptHostValue,
};
use crate::core::{CoreHandle, CoreWeak};
use crate::scene::{EntityId, LevelSystem, World};

#[derive(Debug)]
pub(crate) struct ScriptRuntimeCallContext {
    pub(crate) core: CoreWeak,
    pub(crate) level: LevelSystem,
    pub(crate) entity: EntityId,
    pub(crate) delta_seconds: f32,
}

/// Runtime-issued authority that can create a short-lived reflection operation ticket.
///
/// The persistent token cannot borrow a `World`. Only an active runtime script
/// scope can produce the HRTB-bound operation passed to the supplied closure.
#[derive(Clone, Debug)]
pub struct VmReflectionWorldAccess {
    _runtime_issued: (),
}

impl VmReflectionWorldAccess {
    pub(crate) const fn new() -> Self {
        Self {
            _runtime_issued: (),
        }
    }

    /// Starts one synchronous reflection operation in the active runtime call scope.
    ///
    /// `VmReflectionWorldOperation` cannot outlive this closure, so a backend may
    /// retain this capability token but cannot retain a `World` borrow or reuse an
    /// operation ticket from an unrelated callback.
    pub fn with_reflection_operation<R>(
        &self,
        operation: impl for<'operation> FnOnce(VmReflectionWorldOperation<'operation>) -> R,
    ) -> Option<R> {
        with_active_script_runtime_call_context(|context| {
            context.map(|context| {
                ScriptHostHotPathMetrics::record_world_scope_entry();
                operation(VmReflectionWorldOperation {
                    context,
                    _runtime_issued: self,
                })
            })
        })
    }
}

/// A runtime-issued reflection operation that exists for one synchronous callback only.
#[derive(Debug)]
pub struct VmReflectionWorldOperation<'operation> {
    context: &'operation ScriptRuntimeCallContext,
    _runtime_issued: &'operation VmReflectionWorldAccess,
}

impl VmReflectionWorldOperation<'_> {
    pub fn with_world<R>(&self, operation: impl FnOnce(&World) -> R) -> R {
        self.context.level.with_world(operation)
    }

    pub fn with_world_mut<R>(&self, operation: impl FnOnce(&mut World) -> R) -> R {
        self.context.level.with_world_mut(operation)
    }
}

thread_local! {
    static SCRIPT_RUNTIME_CALL_CONTEXT: Cell<Option<*const ScriptRuntimeCallContext>> =
        const { Cell::new(None) };
}

pub(crate) fn with_script_runtime_call_context<R>(
    context: ScriptRuntimeCallContext,
    call: impl FnOnce() -> R,
) -> R {
    struct ContextResetGuard(Option<*const ScriptRuntimeCallContext>);

    impl Drop for ContextResetGuard {
        fn drop(&mut self) {
            let previous = self.0.take();
            SCRIPT_RUNTIME_CALL_CONTEXT.with(|slot| slot.set(previous));
        }
    }

    let context = context;
    let previous = SCRIPT_RUNTIME_CALL_CONTEXT.with(|slot| slot.replace(Some(&context)));
    let _guard = ContextResetGuard(previous);
    call()
}

/// Borrows the runtime context into one synchronous host-call operation.
///
/// The pointer is installed only by `with_script_runtime_call_context` and is cleared by its
/// drop guard before the owning stack frame can end. The closure signature prevents this borrow
/// from escaping the guest-to-host boundary.
pub(crate) fn with_active_script_runtime_call_context<R>(
    operation: impl FnOnce(Option<&ScriptRuntimeCallContext>) -> R,
) -> R {
    let context = SCRIPT_RUNTIME_CALL_CONTEXT.with(|slot| {
        slot.get().map(|context| {
            // The dynamically scoped guard above keeps this stack allocation alive for the
            // complete synchronous guest-to-host call.
            // SAFETY: 指针仅指向当前线程动态作用域的栈值；拥有者退出前 drop guard 恢复旧指针，本次同步操作不改变该作用域。
            unsafe { &*context }
        })
    });
    operation(context)
}

/// Borrows the runtime payload carried by a gameplay host call frame.
pub(crate) fn runtime_context_for_frame<'a>(
    frame: &'a ScriptHostCallFrame<'a>,
) -> Result<&'a ScriptRuntimeCallContext, ScriptHostError> {
    frame
        .runtime_context::<ScriptRuntimeCallContext>()
        .ok_or_else(|| ScriptHostError::new("script runtime context is not active"))
}

impl ScriptRuntimeCallContext {
    pub fn core_handle(&self) -> Result<CoreHandle, ScriptHostError> {
        self.core
            .upgrade()
            .ok_or_else(|| ScriptHostError::new("script runtime core handle is no longer active"))
    }
}

/// Explicit external-fixture entry point; production callers cannot construct or install a
/// script runtime context outside this crate.
#[cfg(feature = "test-support")]
#[derive(Debug)]
pub struct ScriptRuntimeTestContext {
    context: ScriptRuntimeCallContext,
}

#[cfg(feature = "test-support")]
impl ScriptRuntimeTestContext {
    pub fn new(core: CoreWeak, level: LevelSystem, entity: EntityId, delta_seconds: f32) -> Self {
        Self {
            context: ScriptRuntimeCallContext {
                core,
                level,
                entity,
                delta_seconds,
            },
        }
    }
}

#[cfg(feature = "test-support")]
pub fn with_script_runtime_test_context<R>(
    context: ScriptRuntimeTestContext,
    call: impl FnOnce() -> R,
) -> R {
    with_script_runtime_call_context(context.context, call)
}

pub fn script_float(value: f32) -> ScriptHostValue {
    ScriptHostValue::Float(f64::from(value))
}

#[cfg(test)]
#[path = "tests/runtime_context.rs"]
mod tests;
