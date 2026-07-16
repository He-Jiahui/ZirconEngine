use super::*;

#[test]
fn runtime_15_no_duplicated_provider_boilerplate() {
    let provider_mod = read_runtime_src("graphics/runtime_provider/mod.rs");
    let registration = read_runtime_src("graphics/runtime_provider/registration.rs");
    let update = read_runtime_src("graphics/runtime_provider/update.rs");
    let feedback = read_runtime_src("graphics/runtime_provider/feedback.rs");
    let prepare_input = read_runtime_src("graphics/runtime_provider/prepare_input.rs");
    let particle_feedback =
        read_runtime_src("graphics/particle_runtime_provider/runtime_feedback.rs");
    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let render_index = runtime_15_plan.clone();
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "runtime provider shared boilerplate owners",
        &provider_mod,
        &[
            "mod registration;",
            "mod update;",
            "mod feedback;",
            "mod prepare_input;",
            "RuntimeProviderRegistration",
            "RuntimeProviderUpdate",
            "RuntimeProviderFeedback",
            "RuntimeProviderPrepareInput",
        ],
    );
    assert_contains_all(
        "runtime provider registration shared owner",
        &registration,
        &[
            "pub(crate) struct RuntimeProviderRegistration<P: ?Sized>",
            "define_runtime_provider_registration",
            "registration: RuntimeProviderRegistration<dyn $provider_trait>",
        ],
    );
    assert_contains_all(
        "runtime provider update shared owner",
        &update,
        &[
            "pub(crate) struct RuntimeProviderUpdate<S>",
            "define_runtime_provider_update",
            "update: RuntimeProviderUpdate<$stats_ty>",
        ],
    );
    assert_contains_all(
        "runtime provider feedback shared owner",
        &feedback,
        &[
            "pub(crate) struct RuntimeProviderFeedback<G, V>",
            "gpu_completion: Option<G>",
            "visibility_feedback: Option<V>",
        ],
    );
    assert_contains_all(
        "runtime provider prepare input shared owner",
        &prepare_input,
        &[
            "pub(crate) struct RuntimeProviderPrepareInput<'a, E>",
            "extract: Option<&'a E>",
            "generation: u64",
        ],
    );

    for (label, source) in [
        (
            "hybrid GI provider registration",
            read_runtime_src("graphics/hybrid_gi_runtime_provider/provider_registration.rs"),
        ),
        (
            "virtual geometry provider registration",
            read_runtime_src("graphics/virtual_geometry_runtime_provider/provider_registration.rs"),
        ),
        (
            "solari provider registration",
            read_runtime_src("graphics/solari_runtime_provider/provider_registration.rs"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "define_runtime_provider_registration!",
                "RuntimeProviderRegistration",
            ],
        );
        assert_not_contains_any(
            label,
            &source,
            &[
                "provider_id: String",
                "priority: i32",
                "provider: Arc<",
                "impl std::fmt::Debug for",
                "impl fmt::Debug for",
            ],
        );
    }

    for (label, source) in [
        (
            "hybrid GI provider update",
            read_runtime_src("graphics/hybrid_gi_runtime_provider/runtime_update.rs"),
        ),
        (
            "virtual geometry provider update",
            read_runtime_src("graphics/virtual_geometry_runtime_provider/runtime_update.rs"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &["define_runtime_provider_update!", "RuntimeProviderUpdate"],
        );
        assert_not_contains_any(label, &source, &["pub fn new(", "pub fn stats(&self)"]);
    }

    for (label, source) in [
        (
            "hybrid GI provider feedback",
            read_runtime_src("graphics/hybrid_gi_runtime_provider/runtime_feedback.rs"),
        ),
        (
            "virtual geometry provider feedback",
            read_runtime_src("graphics/virtual_geometry_runtime_provider/runtime_feedback.rs"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "RuntimeProviderFeedback",
                "feedback.gpu_completion()",
                "feedback.visibility_feedback()",
            ],
        );
        assert_not_contains_any(
            label,
            &source,
            &[
                "\n    gpu_completion: Option<",
                "\n    visibility_feedback: Option<",
            ],
        );
    }

    for (label, source) in [
        (
            "hybrid GI provider prepare input",
            read_runtime_src("graphics/hybrid_gi_runtime_provider/prepare_input.rs"),
        ),
        (
            "virtual geometry provider prepare input",
            read_runtime_src("graphics/virtual_geometry_runtime_provider/prepare_input.rs"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "RuntimeProviderPrepareInput",
                "input.extract()",
                "input.generation()",
            ],
        );
        assert_not_contains_any(
            label,
            &source,
            &["\n    extract: Option<&'a ", "\n    generation: u64,"],
        );
    }

    assert_contains_all(
        "particle provider feedback remains feature-specific",
        &particle_feedback,
        &[
            "pub struct ParticleRuntimeFeedback",
            "gpu_feedback: Option<ParticleGpuFeedback>",
            "into_gpu_feedback(self)",
        ],
    );
    assert!(
        !particle_feedback.contains("visibility_feedback"),
        "particle feedback should not be forced through the dual-payload feedback owner because it has no visibility feedback payload"
    );

    let f13_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F13 |"))
        .expect("F13 review findings top row");
    assert!(
        f13_row.contains(
            "f13_f14_provider_diagnostics_top_row_closed_status_static_passed_cargo_deferred"
        ),
        "F13 top row should record the provider/diagnostics closed-status sync anchor"
    );
    assert!(
        f13_row.ends_with("| convention + Runtime 15 / review closed |"),
        "F13 top row should end with the closed Runtime 15 review status"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync",
                "f13_f14_provider_diagnostics_top_row_closed_status_static_passed_cargo_deferred",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F13 full provider boilerplate audit",
                "runtime_15_provider_boilerplate_full_audit_coremin_check_passed",
                "runtime_15_no_duplicated_provider_boilerplate",
            ],
        );
    }
}
