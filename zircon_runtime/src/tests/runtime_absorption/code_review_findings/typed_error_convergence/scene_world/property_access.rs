#[test]
fn review_f5_scene_property_access_uses_scene_error() {
    let world_error = include_str!("../../../../../scene/world/error.rs");
    let read = include_str!("../../../../../scene/world/property_access/read.rs");
    let write = include_str!("../../../../../scene/world/property_access/write.rs");
    let write_physics = include_str!("../../../../../scene/world/property_access/write/physics.rs");
    let value_conversion =
        include_str!("../../../../../scene/world/property_access/value_conversion.rs");
    let read_path_tests = include_str!("../../../../../scene/tests/property_paths/read_paths.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
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
        ("property physics write", write_physics),
        ("property conversion", value_conversion),
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
    for required in [
        "pub fn set_property(",
        ") -> SceneResult<bool>",
        "SceneError::missing_entity(\"update\", entity)",
        "SceneError::ReadOnlyProperty",
        "SceneError::InvalidPropertyIndex",
        "self.set_dynamic_component_property(entity, property_path, value)",
    ] {
        assert!(
            write.contains(required) || write_physics.contains(required),
            "property writer should contain typed error anchor `{required}`"
        );
    }
    for required in [
        "pub(super) fn expect_segment_count",
        "SceneError::PropertySegmentCount",
        "SceneError::PropertyTypeMismatch",
        "SceneError::UnknownPropertyAxis",
        "SceneError::InvalidPropertyResourceId",
        "SceneError::UnsupportedPropertyValue",
        ") -> SceneResult<bool>",
    ] {
        assert!(
            value_conversion.contains(required) || write_physics.contains(required),
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
