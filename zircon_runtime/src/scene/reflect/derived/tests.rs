use zircon_runtime_interface::reflect::{
    ReflectError, ReflectScriptVisibility, ReflectTypeKind, ReflectedValue,
};

use crate::scene::ecs::Component;

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
