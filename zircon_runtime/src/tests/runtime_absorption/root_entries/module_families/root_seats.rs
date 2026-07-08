#[test]
fn runtime_14_module_family_root_seats_match_documented_judgements() {
    let crate_root = include_str!("../../../../lib.rs");

    for module_name in ["animation", "navigation", "diagnostic_log", "engine_module"] {
        let declaration = format!("pub mod {module_name};");
        assert!(
            crate_root.contains(&declaration),
            "Runtime 14 keeps `{module_name}` as a crate-root module family; update docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md before moving it"
        );

        let flattened_reexport = format!("pub use {module_name}::{{");
        assert!(
            !crate_root.contains(&flattened_reexport),
            "Runtime 14 should keep `{module_name}` behind its namespace instead of flattening the family at crate root"
        );
    }

    let plan_doc = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
    );
    for required_anchor in [
        "animation / navigation / diagnostic_log / engine_module",
        "四族在 crate 根的席位与判词一致",
        "runtime_14_module_family_root_seats_match_documented_judgements",
    ] {
        assert!(
            plan_doc.contains(required_anchor),
            "Runtime 14 plan should record the crate-root family judgement anchor `{required_anchor}`"
        );
    }

    let animation_doc = include_str!("../../../../../../docs/zircon_runtime/animation/runtime.md");
    assert!(
        animation_doc.contains("should keep its crate-root seat"),
        "animation runtime doc should keep the crate-root seat judgement"
    );

    let navigation_doc =
        include_str!("../../../../../../docs/zircon_runtime/navigation/runtime.md");
    assert!(
        navigation_doc.contains("built-in fallback implementation"),
        "navigation runtime doc should keep the fallback root-seat judgement"
    );

    let diagnostic_log_doc =
        include_str!("../../../../../../docs/zircon_runtime/diagnostic_log/mod.md");
    assert!(
        diagnostic_log_doc.contains("Keep `diagnostic_log` at crate root."),
        "diagnostic_log doc should keep the crate-root process diagnostics judgement"
    );

    let engine_module_doc =
        include_str!("../../../../../../docs/zircon_runtime/engine_module/relationship.md");
    assert!(
        engine_module_doc.contains("Keep `engine_module` as a crate-root declaration family."),
        "engine_module relationship doc should keep the declared-layering root-seat judgement"
    );
}
