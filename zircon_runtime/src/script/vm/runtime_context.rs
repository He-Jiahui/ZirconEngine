use std::cell::Cell;

use crate::core::framework::script::{ScriptHostCallFrame, ScriptHostError, ScriptHostValue};
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
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::core::framework::scene::WorldHandle;
    use crate::core::framework::script::ScriptHostCallFrame;
    use crate::core::CoreRuntime;
    use crate::scene::{LevelMetadata, LevelSystem, World};

    use super::{
        runtime_context_for_frame, with_active_script_runtime_call_context,
        with_script_runtime_call_context, ScriptRuntimeCallContext, VmReflectionWorldAccess,
    };

    #[test]
    fn runtime13_current_context_is_borrowed_by_the_host_call_frame() {
        let core = CoreRuntime::new();
        let level = LevelSystem::new(
            WorldHandle::new(13),
            Arc::new(Mutex::new(World::empty())),
            LevelMetadata::default(),
        );

        let borrowed = with_script_runtime_call_context(
            ScriptRuntimeCallContext {
                core: core.weak(),
                level,
                entity: 17,
                delta_seconds: 0.016,
            },
            || {
                with_active_script_runtime_call_context(|active| {
                    let active = active.expect("active context");
                    let frame =
                        ScriptHostCallFrame::new("zr.gameplay", "entity", &[], &[], Some(active));
                    std::ptr::eq(
                        active,
                        runtime_context_for_frame(&frame).expect("frame context"),
                    )
                })
            },
        );

        assert!(borrowed);
    }

    #[test]
    fn runtime13_reflection_world_ticket_is_limited_to_the_active_runtime_scope() {
        let core = CoreRuntime::new();
        let level = LevelSystem::new(
            WorldHandle::new(14),
            Arc::new(Mutex::new(World::empty())),
            LevelMetadata::default(),
        );
        let access = VmReflectionWorldAccess::new();

        let saw_world = with_script_runtime_call_context(
            ScriptRuntimeCallContext {
                core: core.weak(),
                level,
                entity: 18,
                delta_seconds: 0.016,
            },
            || access.with_reflection_operation(|ticket| ticket.with_world(|_| true)),
        );

        assert_eq!(saw_world, Some(true));
        assert_eq!(
            access.with_reflection_operation(|ticket| ticket.with_world(|_| true)),
            None,
            "a retained access token cannot mint a world operation outside the runtime scope"
        );
    }

    #[test]
    fn runtime13_production_context_and_raw_world_borrows_remain_crate_private() {
        let source = include_str!("runtime_context.rs");

        assert!(source.contains("pub(crate) struct ScriptRuntimeCallContext"));
        assert!(source.contains("pub(crate) fn with_script_runtime_call_context"));
        assert!(source.contains("pub struct VmReflectionWorldOperation"));
        assert!(source.contains("pub fn with_reflection_operation"));
        let persistent_access = source
            .split("impl VmReflectionWorldAccess")
            .nth(1)
            .and_then(|source| source.split("pub struct VmReflectionWorldOperation").next())
            .expect("persistent reflection access implementation");
        assert!(!persistent_access.contains("pub fn with_world"));
        assert!(!persistent_access.contains("pub fn with_world_mut"));
    }
}
