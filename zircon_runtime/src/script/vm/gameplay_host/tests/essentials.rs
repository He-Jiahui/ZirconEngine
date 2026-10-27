use crate::core::framework::script::ScriptHostValueKind;
use crate::core::math::EulerRot;
use crate::script::VmError;

use super::*;

struct EssentialsHost {
    core: CoreRuntime,
    level: LevelSystem,
    entity: u64,
    exports: HostExportRegistry,
}

impl EssentialsHost {
    fn new() -> Self {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .update_transform(
                entity,
                Transform {
                    translation: Vec3::new(3.0, -2.0, 8.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(2.0, 3.0, 4.0),
                },
            )
            .unwrap();
        let exports = HostExportRegistry::default();
        register_gameplay_host_module(&exports).unwrap();
        Self {
            core: CoreRuntime::new(),
            level: LevelSystem::new(
                WorldHandle::new(93),
                Arc::new(Mutex::new(world)),
                LevelMetadata::default(),
            ),
            entity,
            exports,
        }
    }

    fn call(
        &self,
        function: &str,
        arguments: Vec<ScriptHostValue>,
    ) -> Result<ScriptHostValue, VmError> {
        with_script_runtime_call_context(
            ScriptRuntimeCallContext {
                core: self.core.weak(),
                level: self.level.clone(),
                entity: self.entity,
                delta_seconds: 0.016,
            },
            || {
                self.exports.call_with_capabilities(
                    GAMEPLAY_MODULE,
                    function,
                    arguments,
                    &CapabilitySet::default().with("gameplay.entity"),
                )
            },
        )
    }

    fn rotation(&self, angles: [f64; 3]) -> Result<ScriptHostValue, VmError> {
        self.call(
            "set_rotation_euler",
            vec![
                ScriptHostValue::Int(self.entity as i64),
                ScriptHostValue::Float(angles[0]),
                ScriptHostValue::Float(angles[1]),
                ScriptHostValue::Float(angles[2]),
            ],
        )
    }

    fn number(
        &self,
        entity: u64,
        component: &str,
        fallback: ScriptHostValue,
    ) -> Result<ScriptHostValue, VmError> {
        self.call(
            "component_number",
            vec![
                ScriptHostValue::Int(entity as i64),
                ScriptHostValue::String(component.to_string()),
                fallback,
            ],
        )
    }
}

#[test]
fn gameplay_host_essentials_descriptors_enforce_types_arity_and_capabilities() {
    use ScriptHostValueKind::{Bool, Float, Int, String as Text};
    let host = EssentialsHost::new();
    let module = host.exports.module(GAMEPLAY_MODULE).unwrap().descriptor;
    let builtin_capabilities = crate::script::builtin_host_capabilities();
    for (name, parameters, result, capability) in [
        ("random_range", vec![Float, Float], Float, "gameplay.random"),
        ("mouse_pressed", vec![Int], Bool, "gameplay.input"),
        ("mouse_x", vec![], Float, "gameplay.input"),
        ("mouse_y", vec![], Float, "gameplay.input"),
        ("mouse_delta_x", vec![], Float, "gameplay.input"),
        ("mouse_delta_y", vec![], Float, "gameplay.input"),
        ("mouse_wheel_x", vec![], Float, "gameplay.input"),
        ("mouse_wheel_y", vec![], Float, "gameplay.input"),
        (
            "set_rotation_euler",
            vec![Int, Float, Float, Float],
            Bool,
            "gameplay.entity",
        ),
        (
            "component_number",
            vec![Int, Text, Float],
            Float,
            "gameplay.entity",
        ),
    ] {
        assert!(
            builtin_capabilities.contains(capability),
            "{name} has a builtin host grant"
        );
        let descriptor = module
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap();
        assert_eq!(descriptor.min_argument_count, parameters.len(), "{name}");
        assert_eq!(descriptor.max_argument_count, parameters.len(), "{name}");
        assert_eq!(
            descriptor
                .parameters
                .iter()
                .map(|parameter| parameter.value_kind)
                .collect::<Vec<_>>(),
            parameters,
            "{name}"
        );
        assert_eq!(descriptor.return_value_kind, result, "{name}");
        assert_eq!(descriptor.required_capabilities, [capability], "{name}");
        let arguments = parameters
            .iter()
            .map(|_| ScriptHostValue::Null)
            .collect::<Vec<_>>();
        let denied = host
            .exports
            .call_with_capabilities(GAMEPLAY_MODULE, name, arguments, &CapabilitySet::default())
            .unwrap_err();
        assert!(
            denied
                .to_string()
                .contains(&format!("missing capability {capability}")),
            "{denied}"
        );
        assert!(
            host.exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    name,
                    vec![ScriptHostValue::Null; parameters.len() + 1],
                    &CapabilitySet::default().with(capability)
                )
                .is_err(),
            "{name} rejects extra arguments"
        );
    }
}

#[test]
fn gameplay_host_component_number_reads_integer_and_float_components_without_f32_narrowing() {
    let host = EssentialsHost::new();
    for value in [
        serde_json::json!(-3.125),
        serde_json::json!(16_777_217),
        serde_json::json!(1.23456789012345),
        serde_json::json!(f64::MAX),
    ] {
        host.level
            .with_world_mut(|world| {
                world.set_dynamic_component(host.entity, "gameplay.number", value.clone())
            })
            .unwrap();
        assert_eq!(
            host.number(
                host.entity,
                "gameplay.number",
                ScriptHostValue::Float(-99.0)
            )
            .unwrap(),
            ScriptHostValue::Float(value.as_f64().unwrap())
        );
    }
}

#[test]
fn gameplay_host_component_number_uses_fallback_for_missing_and_non_numeric_components() {
    let host = EssentialsHost::new();
    for value in [
        serde_json::json!("12.5"),
        serde_json::json!(true),
        serde_json::Value::Null,
        serde_json::json!([4]),
        serde_json::json!({"number":4}),
    ] {
        host.level
            .with_world_mut(|world| {
                world.set_dynamic_component(host.entity, "gameplay.number", value)
            })
            .unwrap();
        assert_eq!(
            host.number(
                host.entity,
                "gameplay.number",
                ScriptHostValue::Float(-1.23456789012345)
            )
            .unwrap(),
            ScriptHostValue::Float(-1.23456789012345)
        );
    }
    for (entity, component) in [(host.entity, "missing"), (u64::MAX / 2, "gameplay.number")] {
        assert_eq!(
            host.number(entity, component, ScriptHostValue::Int(7))
                .unwrap(),
            ScriptHostValue::Float(7.0)
        );
    }
    for fallback in [
        ScriptHostValue::Float(f64::NAN),
        ScriptHostValue::Float(f64::INFINITY),
        ScriptHostValue::Float(f64::NEG_INFINITY),
        ScriptHostValue::Bool(true),
        ScriptHostValue::String("7".to_string()),
    ] {
        assert!(host.number(host.entity, "missing", fallback).is_err());
    }
}

#[test]
fn gameplay_host_rotation_euler_sets_world_xyz_radians_and_preserves_position_and_scale() {
    let host = EssentialsHost::new();
    let before = host
        .level
        .with_world(|world| world.world_transform(host.entity).unwrap());
    assert_eq!(
        host.rotation([0.3, -0.4, 0.7]).unwrap(),
        ScriptHostValue::Bool(true)
    );
    let after = host
        .level
        .with_world(|world| world.world_transform(host.entity).unwrap());
    assert_quat_close(
        after.rotation,
        Quat::from_euler(EulerRot::XYZ, 0.3, -0.4, 0.7),
    );
    assert_vec3_close(after.translation, before.translation);
    assert_vec3_close(after.scale, before.scale);
    host.rotation([0.0, std::f64::consts::FRAC_PI_2, 0.0])
        .unwrap();
    let direction = host
        .level
        .with_world(|world| world.world_transform(host.entity).unwrap().rotation * -Vec3::Z);
    assert_vec3_close(direction, -Vec3::X);
}

#[test]
fn gameplay_host_rotation_euler_converts_world_rotation_through_a_rotated_parent() {
    let host = EssentialsHost::new();
    host.level.with_world_mut(|world| {
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .update_transform(
                parent,
                Transform {
                    translation: Vec3::new(8.0, 1.0, -5.0),
                    rotation: Quat::from_rotation_y(0.4),
                    scale: Vec3::splat(2.0),
                },
            )
            .unwrap();
        world.set_parent_checked(host.entity, Some(parent)).unwrap();
    });
    let before = host
        .level
        .with_world(|world| world.world_transform(host.entity).unwrap());
    host.rotation([0.2, -0.3, 0.5]).unwrap();
    let after = host
        .level
        .with_world(|world| world.world_transform(host.entity).unwrap());
    assert_quat_close(
        after.rotation,
        Quat::from_euler(EulerRot::XYZ, 0.2, -0.3, 0.5),
    );
    assert_vec3_close(after.translation, before.translation);
    assert_vec3_close(after.scale, before.scale);
}

#[test]
fn gameplay_host_rotation_euler_rejects_invalid_angles_and_missing_entities_without_mutation() {
    let host = EssentialsHost::new();
    let before = host
        .level
        .with_world(|world| world.world_matrix(host.entity).unwrap());
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
        for axis in 0..3 {
            let mut angles = [0.0; 3];
            angles[axis] = invalid;
            assert!(host.rotation(angles).is_err());
            assert_eq!(
                host.level
                    .with_world(|world| world.world_matrix(host.entity).unwrap()),
                before
            );
        }
    }
    assert!(host
        .call(
            "set_rotation_euler",
            vec![
                ScriptHostValue::Int(host.entity as i64),
                ScriptHostValue::Bool(true),
                ScriptHostValue::Float(0.0),
                ScriptHostValue::Float(0.0)
            ]
        )
        .is_err());
    let missing = host
        .call(
            "set_rotation_euler",
            vec![
                ScriptHostValue::Int(i64::MAX),
                ScriptHostValue::Float(0.0),
                ScriptHostValue::Float(0.0),
                ScriptHostValue::Float(0.0),
            ],
        )
        .unwrap_err();
    assert!(missing.to_string().contains("missing entity"));
    assert_eq!(
        host.level
            .with_world(|world| world.world_matrix(host.entity).unwrap()),
        before
    );
}

#[test]
fn gameplay_host_rotation_euler_preserves_world_when_parent_would_require_local_shear() {
    let host = EssentialsHost::new();
    host.level.with_world_mut(|world| {
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .update_transform(
                parent,
                Transform {
                    scale: Vec3::new(2.0, 1.0, 1.0),
                    ..Transform::identity()
                },
            )
            .unwrap();
        world.set_parent_checked(host.entity, Some(parent)).unwrap();
    });
    let before = host
        .level
        .with_world(|world| world.world_matrix(host.entity).unwrap());
    let error = host
        .rotation([0.0, std::f64::consts::FRAC_PI_4, 0.0])
        .unwrap_err();
    assert!(error.to_string().contains("local TRS"), "{error}");
    assert_eq!(
        host.level
            .with_world(|world| world.world_matrix(host.entity).unwrap()),
        before
    );
}
