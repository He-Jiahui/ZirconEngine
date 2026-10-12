use super::*;

#[test]
fn runtime_15_provider_prepare_input_uses_shared_extract_generation_owner() {
    let shared_prepare = read_runtime_src("graphics/runtime_provider/prepare_input.rs");
    let runtime_provider_mod = read_runtime_src("graphics/runtime_provider/mod.rs");
    let hybrid_prepare = read_runtime_src("graphics/hybrid_gi_runtime_provider/prepare_input.rs");
    let virtual_geometry_prepare =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/prepare_input.rs");
    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let provider_doc = read_repo("docs/zircon_runtime/graphics/runtime_provider/prepare_input.md");

    assert_contains_all(
        "shared runtime provider prepare input owner",
        &shared_prepare,
        &[
            "pub(crate) struct RuntimeProviderPrepareInput<'a, E>",
            "extract: Option<&'a E>",
            "generation: u64",
            "extract(&self) -> Option<&'a E>",
            "generation(&self) -> u64",
        ],
    );
    assert_contains_all(
        "runtime provider owner exports",
        &runtime_provider_mod,
        &["mod prepare_input;", "RuntimeProviderPrepareInput"],
    );

    for (label, source, extract_type) in [
        (
            "hybrid GI provider prepare input",
            hybrid_prepare.as_str(),
            "RenderHybridGiExtract",
        ),
        (
            "virtual geometry provider prepare input",
            virtual_geometry_prepare.as_str(),
            "RenderVirtualGeometryExtract",
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "RuntimeProviderPrepareInput",
                "input.extract()",
                "input.generation()",
            ],
        );
        for duplicated_field in [
            format!("\n    extract: Option<&'a {extract_type}>,"),
            "\n    generation: u64,".to_string(),
        ] {
            assert!(
                !source.contains(&duplicated_field),
                "{label} should delegate common prepare input storage to RuntimeProviderPrepareInput instead of declaring `{duplicated_field}`"
            );
        }
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime provider doc", provider_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F13 provider prepare input shared frame owner",
                "runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed",
                "runtime_15_provider_prepare_input_uses_shared_extract_generation_owner",
            ],
        );
    }
}
