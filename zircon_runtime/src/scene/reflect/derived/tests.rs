use zircon_runtime_interface::reflect::{
    ReflectError, ReflectScriptVisibility, ReflectTypeKind, ReflectedValue,
};

use crate::scene::ecs::Component;
use crate::scene::{NodeKind, RuntimeTypeRegistration, World};

use super::{derived_component_registration, ZrReflect};

#[derive(Clone, Debug, PartialEq, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(component, script_visibility = "public", display_name = "Health")]
struct Health {
    #[zr_reflect(editor_hint = "Scalar")]
    current: f32,
    #[zr_reflect(editor_hint = "Scalar", readonly)]
    maximum: f32,
}

impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct UnselectedStagingSentinel;

impl Component for UnselectedStagingSentinel {}

#[test]
fn derive_round_trips_reflect_type_info() {
    let registration =
        Health::reflect_type_registration().expect("derived type registration should be valid");

    assert_eq!(registration.type_path.short_type_path, "Health");
    assert_eq!(registration.display_name, "Health");
    assert_eq!(registration.type_info.kind, ReflectTypeKind::Struct);
    assert!(registration.is_component);
    assert!(!registration.remote_visible);
    assert_eq!(
        registration.script_visibility,
        ReflectScriptVisibility::Public
    );
    assert_eq!(registration.type_info.fields.len(), 2);
    assert_eq!(registration.type_info.fields[0].name, "current");
    assert!(registration.type_info.fields[0].editable);
    assert_eq!(registration.type_info.fields[1].name, "maximum");
    assert!(!registration.type_info.fields[1].editable);

    let runtime =
        derived_component_registration::<Health>().expect("derived component adapter should build");
    assert!(runtime.component.is_some());
}

#[test]
fn derive_generated_field_accessors_round_trip_values() {
    let mut health = Health {
        current: 25.0,
        maximum: 100.0,
    };

    assert_eq!(
        health
            .read_reflected_field("current")
            .expect("derived getter should read current health"),
        ReflectedValue::Scalar(25.0)
    );
    assert!(health
        .write_reflected_field("current", ReflectedValue::Scalar(40.0))
        .expect("derived setter should update current health"));
    assert_eq!(health.current, 40.0);
    assert!(!health
        .write_reflected_field("current", ReflectedValue::Scalar(40.0))
        .expect("writing the same reflected value should be a no-op"));
    assert_eq!(
        health
            .write_reflected_field("maximum", ReflectedValue::Scalar(120.0))
            .expect_err("readonly reflected fields must reject writes"),
        ReflectError::NonEditableField {
            type_path: format!("{}::Health", module_path!()),
            field_name: "maximum".to_string(),
        }
    );
}

#[test]
fn derived_component_stage_clone_moves_an_owned_value_into_preflight_world() {
    let mut source = World::empty();
    let entity = source.spawn_node(NodeKind::Empty);
    source
        .insert(
            entity,
            Health {
                current: 25.0,
                maximum: 100.0,
            },
        )
        .expect("source health component must attach");
    source
        .insert(entity, UnselectedStagingSentinel)
        .expect("unselected source component must attach");

    let registration =
        derived_component_registration::<Health>().expect("derived component adapter should build");
    let type_path = registration.registration.type_path.type_path.clone();
    source
        .type_registry_mut_for_tests()
        .register(registration)
        .expect("source derived component must register for reflection");

    let mut preflight = source.dynamic_scene_preflight_world([type_path.as_str()]);
    preflight
        .insert_owned_node_records(vec![source
            .node_record(entity)
            .expect("source node record")])
        .expect("preflight identity must be restored before component staging");

    source
        .stage_reflected_component_clone(entity, &type_path, &mut preflight)
        .expect("derived component staging must succeed");
    assert_eq!(
        preflight.get::<Health>(entity),
        Some(&Health {
            current: 25.0,
            maximum: 100.0,
        })
    );
    assert_eq!(preflight.get::<UnselectedStagingSentinel>(entity), None);
}

#[test]
fn reflected_component_stage_clone_rejects_registration_without_adapter() {
    let mut source = World::empty();
    let entity = source.spawn_node(NodeKind::Empty);
    let registration =
        Health::reflect_type_registration().expect("derived reflection metadata should build");
    let type_path = registration.type_path.type_path.clone();
    source
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration::metadata(registration))
        .expect("metadata-only registration must register");

    let error = source
        .stage_reflected_component_clone(entity, &type_path, &mut World::empty())
        .expect_err("metadata-only component registration must not stage silently");
    assert_eq!(
        error,
        ReflectError::InvalidRegistration {
            type_path,
            reason: "registered type has no component staging adapter".to_string(),
        }
    );
}

#[test]
fn reflected_component_stage_clone_rejects_adapter_without_callback() {
    let mut source = World::empty();
    let entity = source.spawn_node(NodeKind::Empty);
    let RuntimeTypeRegistration {
        registration,
        component,
        resource: _,
    } = derived_component_registration::<Health>().expect("derived component adapter should build");
    let type_path = registration.type_path.type_path.clone();
    let mut component = component.expect("derived registration must include its component adapter");
    component.stage_clone = None;
    source
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration {
            registration,
            component: Some(component),
            resource: None,
        })
        .expect("adapter without staging callback must register");

    let error = source
        .stage_reflected_component_clone(entity, &type_path, &mut World::empty())
        .expect_err("component adapter without staging callback must not stage silently");
    assert_eq!(
        error,
        ReflectError::InvalidRegistration {
            type_path,
            reason: "component reflection has no affected-row staging clone adapter".to_string(),
        }
    );
}
