use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_provider_prepare_input_uses_shared_extract_generation_owner() {
    let shared_prepare = read_runtime_src("graphics/runtime_provider/prepare_input.rs");
    let runtime_provider_mod = read_runtime_src("graphics/runtime_provider/mod.rs");
    let hybrid_prepare = read_runtime_src("graphics/hybrid_gi_runtime_provider/prepare_input.rs");
    let virtual_geometry_prepare =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/prepare_input.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
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

#[test]
fn runtime_15_provider_registration_uses_shared_owner() {
    let shared_registration = read_runtime_src("graphics/runtime_provider/registration.rs");
    let graphics_mod = read_runtime_src("graphics/mod.rs");
    let hybrid_registration =
        read_runtime_src("graphics/hybrid_gi_runtime_provider/provider_registration.rs");
    let virtual_geometry_registration =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/provider_registration.rs");
    let solari_registration =
        read_runtime_src("graphics/solari_runtime_provider/provider_registration.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let provider_doc = read_repo("docs/zircon_runtime/graphics/runtime_provider/registration.md");

    assert_contains_all(
        "shared runtime provider registration owner",
        &shared_registration,
        &[
            "pub(crate) struct RuntimeProviderRegistration<P: ?Sized>",
            "provider_id: String",
            "priority: i32",
            "provider: Arc<P>",
            "debug_name: &'static str",
            "macro_rules! define_runtime_provider_registration",
        ],
    );
    assert_contains_all(
        "graphics runtime provider owner wiring",
        &graphics_mod,
        &["pub(crate) mod runtime_provider;"],
    );

    for (label, source, provider_type) in [
        (
            "hybrid GI provider registration",
            hybrid_registration.as_str(),
            "HybridGiRuntimeProvider",
        ),
        (
            "virtual geometry provider registration",
            virtual_geometry_registration.as_str(),
            "VirtualGeometryRuntimeProvider",
        ),
        (
            "Solari provider registration",
            solari_registration.as_str(),
            "SolariRuntimeProvider",
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "define_runtime_provider_registration!",
                provider_type,
                "RuntimeProviderRegistration",
            ],
        );
        for duplicated_field in [
            "provider_id: String",
            "priority: i32",
            "provider: Arc<dyn",
            "impl fmt::Debug for",
        ] {
            assert!(
                !source.contains(duplicated_field),
                "{label} should delegate registration storage/debug boilerplate to shared owner instead of containing `{duplicated_field}`"
            );
        }
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime provider doc", provider_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F13 provider registration shared owner",
                "runtime_15_provider_registration_shared_owner_coremin_check_passed",
                "runtime_15_provider_registration_uses_shared_owner",
            ],
        );
    }
}

#[test]
fn runtime_15_provider_update_uses_shared_stats_owner() {
    let shared_update = read_runtime_src("graphics/runtime_provider/update.rs");
    let runtime_provider_mod = read_runtime_src("graphics/runtime_provider/mod.rs");
    let hybrid_update = read_runtime_src("graphics/hybrid_gi_runtime_provider/runtime_update.rs");
    let virtual_geometry_update =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/runtime_update.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let provider_doc = read_repo("docs/zircon_runtime/graphics/runtime_provider/update.md");

    assert_contains_all(
        "shared runtime provider update owner",
        &shared_update,
        &[
            "pub(crate) struct RuntimeProviderUpdate<S>",
            "stats: S",
            "macro_rules! define_runtime_provider_update",
            "=> copy",
            "=> ref",
        ],
    );
    assert_contains_all(
        "runtime provider owner exports",
        &runtime_provider_mod,
        &[
            "mod update;",
            "define_runtime_provider_update",
            "RuntimeProviderUpdate",
        ],
    );

    for (label, source, stats_type) in [
        (
            "hybrid GI provider update",
            hybrid_update.as_str(),
            "HybridGiRuntimeStats",
        ),
        (
            "virtual geometry provider update",
            virtual_geometry_update.as_str(),
            "VirtualGeometryRuntimeStats",
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "define_runtime_provider_update!",
                stats_type,
                "RuntimeProviderUpdate",
            ],
        );
        let duplicated_stats_field = format!("stats: {stats_type},");
        assert!(
            !source.contains(&duplicated_stats_field),
            "{label} should delegate update stats storage to RuntimeProviderUpdate instead of declaring `{duplicated_stats_field}`"
        );
        assert!(
            !source.contains("Self { stats }"),
            "{label} should delegate update construction to RuntimeProviderUpdate instead of constructing a local stats field"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime provider doc", provider_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F13 provider update shared stats owner",
                "runtime_15_provider_update_shared_stats_owner_coremin_check_passed",
                "runtime_15_provider_update_uses_shared_stats_owner",
            ],
        );
    }
}

#[test]
fn runtime_15_provider_feedback_uses_shared_payload_owner() {
    let shared_feedback = read_runtime_src("graphics/runtime_provider/feedback.rs");
    let runtime_provider_mod = read_runtime_src("graphics/runtime_provider/mod.rs");
    let hybrid_feedback =
        read_runtime_src("graphics/hybrid_gi_runtime_provider/runtime_feedback.rs");
    let virtual_geometry_feedback =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/runtime_feedback.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let provider_doc = read_repo("docs/zircon_runtime/graphics/runtime_provider/feedback.md");

    assert_contains_all(
        "shared runtime provider feedback owner",
        &shared_feedback,
        &[
            "pub(crate) struct RuntimeProviderFeedback<G, V>",
            "gpu_completion: Option<G>",
            "visibility_feedback: Option<V>",
            "gpu_completion(&self) -> Option<&G>",
            "visibility_feedback(&self) -> Option<&V>",
        ],
    );
    assert_contains_all(
        "runtime provider owner exports",
        &runtime_provider_mod,
        &["mod feedback;", "RuntimeProviderFeedback"],
    );

    for (label, source, owner_type, gpu_type, visibility_type) in [
        (
            "hybrid GI provider feedback",
            hybrid_feedback.as_str(),
            "RuntimeProviderFeedback<HybridGiGpuCompletion, VisibilityHybridGiFeedback>",
            "HybridGiGpuCompletion",
            "VisibilityHybridGiFeedback",
        ),
        (
            "virtual geometry provider feedback",
            virtual_geometry_feedback.as_str(),
            "RuntimeProviderFeedback<VirtualGeometryGpuCompletion, VisibilityVirtualGeometryFeedback>",
            "VirtualGeometryGpuCompletion",
            "VisibilityVirtualGeometryFeedback",
        ),
    ] {
        assert_contains_all(label, source, &["RuntimeProviderFeedback", owner_type]);
        for duplicated_field in [
            format!("\n    gpu_completion: Option<{gpu_type}>,"),
            format!("\n    visibility_feedback: Option<{visibility_type}>,"),
        ] {
            assert!(
                !source.contains(&duplicated_field),
                "{label} should delegate common feedback payload storage to RuntimeProviderFeedback instead of declaring `{duplicated_field}`"
            );
        }
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime provider doc", provider_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F13 provider feedback shared payload owner",
                "runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed",
                "runtime_15_provider_feedback_uses_shared_payload_owner",
            ],
        );
    }
}

#[test]
fn runtime_15_no_duplicated_provider_boilerplate() {
    let provider_mod = read_runtime_src("graphics/runtime_provider/mod.rs");
    let registration = read_runtime_src("graphics/runtime_provider/registration.rs");
    let update = read_runtime_src("graphics/runtime_provider/update.rs");
    let feedback = read_runtime_src("graphics/runtime_provider/feedback.rs");
    let prepare_input = read_runtime_src("graphics/runtime_provider/prepare_input.rs");
    let particle_feedback =
        read_runtime_src("graphics/particle_runtime_provider/runtime_feedback.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
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

#[test]
fn runtime_15_provider_boilerplate_guard_is_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let provider_boilerplate =
        read_runtime_src("tests/runtime_absorption/structure_convention/provider_boilerplate.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "structure convention provider child module mount",
        &parent,
        &[
            "#[path = \"structure_convention/provider_boilerplate.rs\"]",
            "mod provider_boilerplate;",
        ],
    );

    for moved_guard in [
        "fn runtime_15_provider_registration_uses_shared_owner",
        "fn runtime_15_provider_update_uses_shared_stats_owner",
        "fn runtime_15_provider_feedback_uses_shared_payload_owner",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "provider boilerplate guard `{moved_guard}` should live in provider_boilerplate.rs, not the structure_convention.rs aggregator"
        );
        assert!(
            provider_boilerplate.contains(moved_guard),
            "provider_boilerplate.rs should own provider boilerplate guard `{moved_guard}`"
        );
    }

    let parent_line_count = parent.lines().count();
    assert!(
        parent_line_count < 700,
        "structure_convention.rs should stay a small aggregator after provider guard split, found {parent_line_count} lines"
    );
    let provider_line_count = provider_boilerplate.lines().count();
    assert!(
        provider_line_count < 900,
        "provider_boilerplate.rs should stay below the near-large-file split threshold, found {provider_line_count} lines"
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
                "Runtime 15 M3 provider boilerplate guard module split",
                "runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked",
                "structure_convention/provider_boilerplate.rs",
                "runtime_15_provider_boilerplate_guard_is_folder_backed",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    let path = runtime_src_path(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("runtime source should exist at {}: {error}", path.display())
    })
}

fn read_repo(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("repo file should exist at {}: {error}", path.display()))
}

fn assert_not_contains_any(label: &str, source: &str, unexpected: &[&str]) {
    for token in unexpected {
        assert!(
            !source.contains(token),
            "{label} should not contain duplicated provider boilerplate token `{token}`"
        );
    }
}
