use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_runtime_ui_dead_code_surface_is_test_support() {
    let ui_mod = read_runtime_src("ui/mod.rs");
    let public_runtime_frame = read_runtime_src("ui/public_runtime_frame.rs");
    let viewport_conversion =
        read_runtime_src("graphics/types/viewport_render_frame_from_public_runtime.rs");
    let runtime_ui_support_mod = read_runtime_src("ui/tests/runtime_ui_support/mod.rs");
    let runtime_ui_manager = read_runtime_src("ui/tests/runtime_ui_support/runtime_ui_manager.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "runtime UI production frame surface",
        &ui_mod,
        &[
            "mod public_runtime_frame;",
            "pub(crate) use public_runtime_frame::PublicRuntimeFrame;",
            "#[cfg(test)]",
            "#[path = \"tests/runtime_ui_support/mod.rs\"]",
            "mod runtime_ui_support;",
            "pub(crate) use runtime_ui_support::{RuntimeUiFixture, RuntimeUiManager};",
        ],
    );
    assert!(
        !ui_mod.contains("#[allow(dead_code)]"),
        "ui root should not hide runtime UI dead code behind allow(dead_code)"
    );
    assert!(
        !ui_mod.contains("mod runtime_ui;"),
        "runtime UI manager support should not remain a production ui::runtime_ui module"
    );
    assert!(
        !runtime_src_path("ui/runtime_ui/mod.rs").exists(),
        "old production ui/runtime_ui module directory should be removed"
    );

    assert_contains_all(
        "public runtime frame owner",
        &public_runtime_frame,
        &[
            "pub(crate) struct PublicRuntimeFrame",
            "pub extract: RenderFrameExtract",
            "pub viewport_size: UVec2",
            "pub ui: Option<UiRenderExtract>",
        ],
    );
    assert_contains_all(
        "graphics public runtime frame conversion",
        &viewport_conversion,
        &[
            "use crate::ui::PublicRuntimeFrame;",
            "impl From<PublicRuntimeFrame> for ViewportRenderFrame",
            "extract: Arc::new(frame.extract)",
        ],
    );
    assert_contains_all(
        "runtime UI test support owner",
        &runtime_ui_support_mod,
        &[
            "mod runtime_ui_fixture;",
            "mod runtime_ui_manager;",
            "pub(crate) use runtime_ui_fixture::RuntimeUiFixture;",
            "pub(crate) use runtime_ui_manager::RuntimeUiManager;",
        ],
    );
    assert_contains_all(
        "runtime UI manager test support frame import",
        &runtime_ui_manager,
        &[
            "use crate::ui::PublicRuntimeFrame;",
            "pub(crate) fn build_frame(&self) -> PublicRuntimeFrame",
        ],
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
                "Runtime 15 runtime UI dead-code support split",
                "runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed",
                "runtime_15_runtime_ui_dead_code_surface_is_test_support",
            ],
        );
    }
}

#[test]
fn runtime_15_runtime_owned_dead_code_suppression_cleanup() {
    let asset_worker_pool = read_runtime_src("asset/pipeline/worker_pool.rs");
    let asset_worker_pool_tests = read_runtime_src("asset/tests/pipeline/worker_pool.rs");
    let module_entry = read_runtime_src("core/runtime/state/module_entry.rs");
    let runtime_devtools = read_runtime_src("core/runtime/diagnostics/devtools.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("asset worker pool", asset_worker_pool.as_str()),
        ("core runtime module entry", module_entry.as_str()),
    ] {
        assert!(
            !source.contains("#[allow(dead_code)]"),
            "{label} should expose live code or test-only reads instead of dead-code suppression"
        );
    }

    assert_contains_all(
        "asset worker pool test-only receiver guard",
        &asset_worker_pool,
        &[
            "request_rx_guard: Option<ChannelReceiver<AssetRequest>>",
            "pub(crate) fn request_channel_guard_is_alive_for_test(&self) -> bool",
            "self.request_rx_guard.is_some()",
        ],
    );
    assert_contains_all(
        "asset worker pool tests read the guard",
        &asset_worker_pool_tests,
        &["pool.request_channel_guard_is_alive_for_test()"],
    );
    assert_contains_all(
        "module entry descriptor is a live diagnostics source",
        &module_entry,
        &[
            "pub(crate) descriptor: ModuleDescriptor",
            "pub(crate) fn descriptor(&self) -> &ModuleDescriptor",
            "&self.descriptor",
        ],
    );
    assert_contains_all(
        "runtime devtools consumes the module descriptor accessor",
        &runtime_devtools,
        &[
            "let descriptor = entry.descriptor();",
            "name: descriptor.name.clone()",
            "driver_count: descriptor.drivers.len()",
        ],
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
                "Runtime 15 F12 runtime-owned dead-code suppression cleanup",
                "runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed",
                "runtime_15_runtime_owned_dead_code_suppression_cleanup",
            ],
        );
    }
}

#[test]
fn runtime_15_script_host_value_descriptors_do_not_suppress_dead_code() {
    let builtin_host_modules = read_runtime_src("script/vm/host/builtin_host_modules.rs");
    let script_host_ledger = read_repo("docs/zircon_runtime/script/vm/host/function_ledger.md");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert!(
        !builtin_host_modules.contains("#[allow(dead_code)]"),
        "script host descriptors should keep value layouts live without dead-code suppression"
    );
    assert_contains_all(
        "script host value descriptor layout sentinel",
        &builtin_host_modules,
        &[
            "struct Vec3",
            "struct ColorRgba",
            "derive(crate::ZirconScriptType)",
            "const _: ((f64, f64, f64), (f64, f64, f64, f64))",
            "(vec3.x, vec3.y, vec3.z)",
            "(color.r, color.g, color.b, color.a)",
            "vec3_length",
            "vec3_dot",
        ],
    );
    assert_contains_all(
        "script host ledger keeps descriptor counts stable",
        &script_host_ledger,
        &[
            "6 host modules, 52 fixed host functions, and 2 fixed script type descriptors",
            "Type `Vec3`",
            "Type `ColorRgba`",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script host ledger", script_host_ledger.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 script host value descriptor dead-code cleanup",
                "runtime_15_script_host_value_descriptors_coremin_check_passed",
                "runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
            ],
        );
    }
}

#[test]
fn runtime_15_runtime_dead_code_guard_is_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let child =
        read_runtime_src("tests/runtime_absorption/structure_convention/runtime_dead_code.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs",
    );

    assert_contains_all(
        "structure convention parent runtime dead-code mount",
        &parent,
        &[
            "#[path = \"structure_convention/runtime_dead_code.rs\"]",
            "mod runtime_dead_code;",
        ],
    );

    for moved_guard in [
        "fn runtime_15_runtime_ui_dead_code_surface_is_test_support",
        "fn runtime_15_runtime_owned_dead_code_suppression_cleanup",
        "fn runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "top-level structure_convention.rs should mount runtime dead-code guards instead of defining {moved_guard}"
        );
        assert!(
            child.contains(moved_guard),
            "runtime_dead_code.rs should own moved guard {moved_guard}"
        );
    }

    let parent_lines = parent.lines().count();
    assert!(
        parent_lines < 180,
        "structure_convention.rs should remain a thin aggregator after runtime dead-code split; got {parent_lines} lines"
    );
    let child_lines = child.lines().count();
    assert!(
        child_lines < 700,
        "runtime_dead_code.rs should stay below the local guard module limit; got {child_lines} lines"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 runtime dead-code guard module split",
                "runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
                "structure_convention/runtime_dead_code.rs",
                "runtime_15_runtime_dead_code_guard_is_folder_backed",
                "runtime_15_runtime_ui_dead_code_surface_is_test_support",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
