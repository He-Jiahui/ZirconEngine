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
                let empty_arguments: [crate::core::framework::script::ScriptHostValue; 0] = [];
                let argument_source =
                    crate::core::framework::script::ScriptHostOwnedArgumentSource::new(
                        &empty_arguments,
                    );
                let frame = ScriptHostCallFrame::new(
                    "zr.gameplay",
                    "entity",
                    crate::core::framework::script::ScriptHostArguments::new(&argument_source),
                    &[],
                    Some(active),
                );
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
    let source = include_str!("../runtime_context.rs");

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
