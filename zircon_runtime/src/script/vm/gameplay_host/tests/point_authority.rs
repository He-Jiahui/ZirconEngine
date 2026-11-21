//! The Script host must leave mutability/missing-component precedence to Scene.
use std::sync::{Arc, Mutex};

use super::{register_gameplay_host_module, GAMEPLAY_MODULE};
use crate::core::framework::scene::WorldHandle;
use crate::core::framework::script::ScriptHostValue;
use crate::core::math::{Mat4, Transform, Vec3};
use crate::core::CoreRuntime;
use crate::scene::components::{LocalTransform, WorldMatrix};
use crate::scene::{LevelMetadata, LevelSystem, Mobility, SceneError, World};
use crate::script::{
    with_script_runtime_call_context, CapabilitySet, HostExportRegistry, HostRegistry,
    ScriptRuntimeCallContext, VmError,
};

#[test]
fn non_transform_entity_keeps_the_scene_owners_error_order() {
    let mut world = World::empty();
    // Two legitimate ECS allocations, not a forbidden removal of LocalTransform.
    let a = world.commands().spawn_empty().into_deferred_entity();
    let b = world.commands().spawn_empty().into_deferred_entity();
    let report = world.apply_deferred();
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.applied_count(), 2);
    let mutable = report.resolve(&a).unwrap();
    let immobile = report.resolve(&b).unwrap();
    assert!(world.set_mobility(immobile, Mobility::Static).unwrap());
    world.flush_pending_scene_systems();

    let mut expectations = Vec::new();
    for entity in [mutable, immobile] {
        assert!(world.get::<LocalTransform>(entity).is_none());
        // Scene supports this projection fallback, not a writable authored pose.
        assert_eq!(world.world_matrix(entity), Some(Mat4::IDENTITY));
        let direct = world
            .update_transform(entity, Transform::from_translation(Vec3::X))
            .unwrap_err();
        let valid_error = match &direct {
            SceneError::MissingRequiredComponent {
                entity: affected,
                component: "LocalTransform",
                ..
            } => entity == mutable && *affected == entity,
            SceneError::StaticTransformMutation { entity: affected } => {
                entity == immobile && *affected == entity
            }
            _ => false,
        };
        assert!(valid_error, "invalid control: {direct}");
        expectations.push((entity, direct, world.get::<WorldMatrix>(entity).copied()));
    }
    let generation = world.world_generation();
    let level = LevelSystem::new(
        WorldHandle::new(513),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let core = CoreRuntime::new();
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");
    let mut observed = Vec::new();
    for (entity, expected, cache) in expectations {
        let error = with_script_runtime_call_context(
            ScriptRuntimeCallContext {
                core: core.weak(),
                level: level.clone(),
                entity,
                delta_seconds: 0.016,
            },
            || {
                exports.call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "set_position",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(1.0),
                        ScriptHostValue::Float(0.0),
                        ScriptHostValue::Float(0.0),
                    ],
                    &capabilities,
                )
            },
        )
        .unwrap_err();
        let VmError::Operation(message) = error else {
            panic!("expected registered host operation error: {error:?}");
        };
        level.with_world(|world| {
            assert!(world.get::<LocalTransform>(entity).is_none());
            assert_eq!(world.world_matrix(entity), Some(Mat4::IDENTITY));
            assert_eq!(world.get::<WorldMatrix>(entity).copied(), cache);
            assert_eq!(world.world_generation(), generation);
        });
        eprintln!("SCRIPT_POINT_AUTHORITY entity={entity} expected={expected:?} host={message}");
        observed.push((
            message,
            format!("host export call failed: {GAMEPLAY_MODULE}.set_position: {expected}"),
        ));
    }
    for (message, expected) in observed {
        assert_eq!(
            message, expected,
            "Script must preserve Scene's error order"
        );
    }
}
