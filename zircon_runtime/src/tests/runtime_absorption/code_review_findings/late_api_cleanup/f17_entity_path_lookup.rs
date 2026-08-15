#[test]
fn review_f17_entity_path_option_lookup_uses_get_verb() {
    let path_resolution =
        include_str!("../../../../scene/world/property_access/path_resolution.rs");
    let runtime_compiled = include_str!("../../../../animation/sequence/compiled.rs");
    let runtime_target = include_str!("../../../../animation/sequence/target.rs");
    let plugin_runtime =
        include_str!("../../../../../../zircon_plugins/animation/runtime/src/lib.rs");
    let property_paths_runtime_mutation =
        include_str!("../../../../scene/tests/property_paths/runtime_mutation.rs");
    let property_paths_read = include_str!("../../../../scene/tests/property_paths/read_paths.rs");
    let review_findings = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let convention = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
    );
    let runtime_08 = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/08/2026-07-09-ecs-kernel-data-alignment-output-records.md"
    );
    let runtime_index = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );
    let ecs_doc = include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md");
    let animation_doc = include_str!(
        "../../../../../../docs/assets-and-rendering/runtime-physics-animation-assets.md"
    );
    let editor_boundary_doc = include_str!(
        "../../../../../../docs/editor-and-tooling/runtime-editor-boundary-cleanup.md"
    );
    let f17_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F17 |"))
        .expect("F17 review findings top row");

    assert!(
        f17_row.ends_with("| convention + Runtime 08 / review closed |"),
        "F17 top row should record lookup review closed status"
    );
    assert!(
        review_findings
            .contains("f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred"),
        "F17 numbered review output should retain the closed-status evidence anchor"
    );

    let old_option_lookup = ["resolve", "entity", "path"].join("_");
    assert!(
        path_resolution
            .contains("pub fn get_entity_by_path(&self, path: &EntityPath) -> Option<EntityId>"),
        "F17 entity path Option lookup should use get_* naming"
    );
    assert!(
        !path_resolution.contains(&old_option_lookup),
        "F17 should hard-cut the old resolve-verb entity path Option API"
    );

    for (name, source) in [
        ("runtime animation compiled projection", runtime_compiled),
        ("runtime animation target", runtime_target),
        (
            "property path runtime mutation tests",
            property_paths_runtime_mutation,
        ),
        ("property path read tests", property_paths_read),
    ] {
        assert!(
            source.contains("get_entity_by_path("),
            "F17 consumer `{name}` should use get_entity_by_path"
        );
        assert!(
            !source.contains(&old_option_lookup),
            "F17 consumer `{name}` should not keep the old resolve-verb entity path lookup"
        );
    }

    assert!(
        plugin_runtime.contains("pub use zircon_runtime::animation::{")
            && plugin_runtime.contains("apply_compiled_sequence_to_world")
            && plugin_runtime.contains("compile_sequence_for_world")
            && !plugin_runtime.contains("apply_sequence_to_world"),
        "the animation plugin must expose the canonical compiled runtime sequence API without reviving per-frame text dispatch"
    );

    for doc_anchor in [
        "F17 entity path Option lookup verb rename",
        "runtime_08_entity_path_lookup_getter_rename_coremin_check_passed",
        "f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred",
        "review_f17_entity_path_option_lookup_uses_get_verb",
        "get_entity_by_path",
        "old resolve-verb entity path method absent",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || runtime_08.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor)
                || animation_doc.contains(doc_anchor)
                || editor_boundary_doc.contains(doc_anchor),
            "F17 docs should record `{doc_anchor}`"
        );
    }
}
