#[test]
fn review_f5_scene_property_access_uses_scene_error() {
    let world_error = include_str!("../../../../../scene/world/error.rs");
    let read = include_str!("../../../../../scene/world/property_access/read.rs");
    let write = include_str!("../../../../../scene/world/property_access/write.rs");
    let write_animation =
        include_str!("../../../../../scene/world/property_access/write/animation.rs");
    let write_camera = include_str!("../../../../../scene/world/property_access/write/camera.rs");
    let write_lighting =
        include_str!("../../../../../scene/world/property_access/write/lighting.rs");
    let write_mesh = include_str!("../../../../../scene/world/property_access/write/mesh.rs");
    let write_physics = include_str!("../../../../../scene/world/property_access/write/physics.rs");
    let value_conversion_facade =
        include_str!("../../../../../scene/world/property_access/value_conversion.rs");
    let value_conversion_compiled =
        include_str!("../../../../../scene/world/property_access/value_conversion/compiled.rs");
    let value_conversion_domain =
        include_str!("../../../../../scene/world/property_access/value_conversion/domain.rs");
    let value_conversion_errors =
        include_str!("../../../../../scene/world/property_access/value_conversion/errors.rs");
    let value_conversion_identifiers =
        include_str!("../../../../../scene/world/property_access/value_conversion/identifiers.rs");
    let value_conversion_values =
        include_str!("../../../../../scene/world/property_access/value_conversion/values.rs");
    let read_path_tests = include_str!("../../../../../scene/tests/property_paths/read_paths.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../../../docs/zircon_runtime/scene/ecs.md");

    for required in [
        "PropertyUnavailable",
        "PropertySegmentCount",
        "UnknownProperty",
        "MissingPropertyComponent",
        "PropertyTypeMismatch",
        "UnknownPropertyAxis",
        "ZeroLengthQuaternion",
        "NonFinitePropertyValue",
        "InvalidPropertyResourceId",
        "UnsupportedPropertyValue",
        "ReadOnlyProperty",
        "InvalidPropertyIndex",
    ] {
        assert!(
            world_error.contains(required),
            "F5 scene property access SceneError should expose `{required}`"
        );
    }

    for (label, source) in [
        ("property read", read),
        ("property write", write),
        ("property animation write", write_animation),
        ("property camera write", write_camera),
        ("property lighting write", write_lighting),
        ("property mesh write", write_mesh),
        ("property physics write", write_physics),
        ("property conversion facade", value_conversion_facade),
        ("property compiled conversion", value_conversion_compiled),
        ("property domain conversion", value_conversion_domain),
        ("property error conversion", value_conversion_errors),
        (
            "property identifier conversion",
            value_conversion_identifiers,
        ),
        ("property value conversion", value_conversion_values),
    ] {
        for forbidden in [
            "Result<ScenePropertyValue, String>",
            "Result<bool, String>",
            "Result<(), String>",
            "Err(format!(",
            "map_err(|error| error.to_string())",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String property error branch `{forbidden}`"
            );
        }
    }

    for required in [
        ") -> SceneResult<ScenePropertyValue>",
        "SceneError::PropertyUnavailable",
    ] {
        assert!(
            read.contains(required),
            "property read should contain typed error anchor `{required}`"
        );
    }
    let property_write_owners = [
        write,
        write_animation,
        write_camera,
        write_lighting,
        write_mesh,
        write_physics,
    ];
    for required in [
        "pub fn set_property(",
        ") -> SceneResult<bool>",
        "SceneError::missing_entity(\"update\", entity)",
        "SceneError::ReadOnlyProperty",
        "SceneError::InvalidPropertyIndex",
        "self.set_dynamic_component_property(entity, property_path, value)",
    ] {
        assert!(
            property_write_owners
                .iter()
                .any(|source| source.contains(required)),
            "property writer should contain typed error anchor `{required}`"
        );
    }
    let property_conversion_owners = [
        value_conversion_facade,
        value_conversion_compiled,
        value_conversion_domain,
        value_conversion_errors,
        value_conversion_identifiers,
        value_conversion_values,
        write_physics,
    ];
    for required in [
        "fn expect_segment_count",
        "SceneError::PropertySegmentCount",
        "SceneError::PropertyTypeMismatch",
        "SceneError::UnknownPropertyAxis",
        "SceneError::InvalidPropertyResourceId",
        "SceneError::UnsupportedPropertyValue",
        ") -> SceneResult<bool>",
    ] {
        assert!(
            property_conversion_owners
                .iter()
                .any(|source| source.contains(required)),
            "property conversion should contain typed error anchor `{required}`"
        );
    }
    assert!(
        read_path_tests.contains("SceneError::PropertyUnavailable")
            && read_path_tests.contains(") -> SceneResult<ScenePropertyValue>"),
        "property read path source guard should be updated to the typed SceneError contract"
    );

    for doc_anchor in [
        "F5 scene property access typed errors",
        "runtime_15_scene_property_access_typed_errors_static_passed_cargo_deferred",
        "review_f5_scene_property_access_uses_scene_error",
        "SceneError::PropertyUnavailable",
        "World::property",
        "World::set_property",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor),
            "F5 property access docs should record `{doc_anchor}`"
        );
    }
}
