use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner() {
    // The test name is the stable failure-reproduction anchor. Its assertions intentionally
    // validate the final derived-reflection owner after the former fixed adapter was retired.
    let lighting = read_runtime_src("scene/components/scene/lighting.rs");
    let registration = read_runtime_src("scene/reflect/builtin_reflection/registration.rs");
    let derived_adapter = read_runtime_src("scene/reflect/derived/component_adapter.rs");
    let reflect_root = read_runtime_src("scene/reflect/mod.rs");
    let fixed_module =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/scene/reflect/fixed/mod.rs");

    assert_contains_all(
        "light components own their reflected schemas through the derive",
        &lighting,
        &[
            "zircon_reflect_derive::ZrReflect",
            "zircon_runtime::scene::components::AmbientLight",
            "zircon_runtime::scene::components::DirectionalLight",
            "zircon_runtime::scene::components::PointLight",
            "zircon_runtime::scene::components::RectLight",
            "zircon_runtime::scene::components::SpotLight",
            "script_visibility = \"public\"",
        ],
    );
    assert_contains_all(
        "builtin light registration uses the unified derived adapter",
        &registration,
        &[
            "derived_component_registration::<AmbientLight>()",
            "derived_component_registration::<DirectionalLight>()",
            "derived_component_registration::<PointLight>()",
            "derived_component_registration::<RectLight>()",
            "derived_component_registration::<SpotLight>()",
        ],
    );
    assert_contains_all(
        "derived component writes preserve World-owned invariants",
        &derived_adapter,
        &[
            "let mut next = component::<T>(world, entity, type_path)?.clone();",
            "match world.insert(entity, next)",
            "Ok(_) => Ok(true)",
            "Err(error) => Err(ReflectError::UnsupportedConversion",
        ],
    );

    assert!(
        !fixed_module.exists()
            && !reflect_root.contains("mod fixed;")
            && !registration.contains("ReflectComponent::new"),
        "light reflection must not restore the retired manual fixed-adapter owner"
    );

    for (path, source) in [
        ("scene/components/scene/lighting.rs", lighting.as_str()),
        (
            "scene/reflect/builtin_reflection/registration.rs",
            registration.as_str(),
        ),
        (
            "scene/reflect/derived/component_adapter.rs",
            derived_adapter.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }
}
