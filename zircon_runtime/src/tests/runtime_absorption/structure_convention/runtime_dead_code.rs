use super::{assert_contains_all, repo_path, runtime_src_path};

const DEAD_CODE_ALLOW_ATTRIBUTE: &str = concat!("#[allow(", "dead_code", ")]");
const DEAD_CODE_ALLOW_CALL_PREFIX: &str = concat!("allow(", "dead_code");

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
        !ui_mod.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
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
            "use crate::ui::{dispatch::UiInputManager, PublicRuntimeFrame};",
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
                "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred",
            ],
        );
    }
    let f10_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F10 |"))
        .expect("F10 review findings top row");
    assert!(
        f10_row.contains(
            "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred"
        ) && f10_row.ends_with("| Runtime 09 + Runtime 15 / review closed |"),
        "F10 top row should record runtime surface review closed status"
    );
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
            !source.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
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
fn runtime_15_ui_text_edit_state_dead_code_suppression_cleanup() {
    let ui_text_mod = read_runtime_src("ui/text/mod.rs");
    let edit_state = read_runtime_src("ui/text/edit_state.rs");
    let text_input = read_runtime_src("ui/component/state_reducer/text_input.rs");
    let editable_text = read_runtime_src("ui/surface/input/editable_text.rs");
    let keyboard_clipboard = read_runtime_src("ui/surface/input/keyboard_clipboard.rs");
    let text_pointer = read_runtime_src("ui/surface/input/text_pointer.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_text_doc = read_repo("docs/zircon_runtime/ui/text.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
    );

    assert!(
        dead_code_suppression_lines(&ui_text_mod).is_empty(),
        "ui/text/mod.rs should keep edit_state live without cfg_attr/allow(dead_code)"
    );
    assert_contains_all(
        "ui text edit state production module",
        &ui_text_mod,
        &[
            "mod edit_state;",
            "pub(crate) use edit_state::apply_text_edit_action;",
        ],
    );
    assert_contains_all(
        "ui text edit state owner",
        &edit_state,
        &[
            "pub(crate) fn apply_text_edit_action",
            "UiTextEditAction::Insert { text }",
            "UiTextEditAction::SetComposition { range, text }",
            "replace_range_preserving_composition",
            "previous_grapheme_boundary",
            "next_grapheme_boundary",
        ],
    );
    assert_contains_all(
        "text input edit state reducer consumer",
        &text_input,
        &[
            "use crate::ui::text::apply_text_edit_action;",
            "let next_state = apply_text_edit_action(",
        ],
    );
    for (label, source) in [
        ("editable text input consumer", editable_text.as_str()),
        ("keyboard clipboard consumer", keyboard_clipboard.as_str()),
        ("text pointer consumer", text_pointer.as_str()),
    ] {
        assert_contains_all(label, source, &["apply_text_edit_action("]);
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI text doc", ui_text_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 UI text edit-state dead-code suppression cleanup",
                "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred",
                "ui/text/mod.rs",
                "ui/text/edit_state.rs",
                "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup",
            ],
        );
    }
    assert_contains_all(
        "Runtime 15 status map",
        &status_map,
        &[
            "Runtime 15 F12 UI text edit-state dead-code suppression cleanup",
            "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 date map",
        &date_map,
        &[
            "Runtime 15 F12 UI text edit-state dead-code suppression cleanup",
            "2026-06-27",
        ],
    );
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
        !builtin_host_modules.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
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
fn runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code() {
    let reflection_docs = read_runtime_src("script/vm/tests/reflection_docs.rs");
    let vm_tests_doc = read_repo("docs/zircon_runtime/script/vm/tests.md");
    let host_reflection_doc = read_repo("docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert!(
        !reflection_docs.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "reflection_docs macro fixtures should be exercised by test assertions instead of dead-code suppression"
    );
    assert_contains_all(
        "script reflection macro fixture live reads",
        &reflection_docs,
        &[
            "let test_vec3 = TestVec3",
            "test_vec3.x + test_vec3.y + test_vec3.z",
            "matches!(TestEnum::A, TestEnum::A)",
            "pub fn point_fixture_x() -> f64",
            "Point { x: 3.5 }.x",
            "macro_math::point_fixture_x()",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script VM tests doc", vm_tests_doc.as_str()),
        ("host reflection doc", host_reflection_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 script reflection macro fixture dead-code cleanup",
                "runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred",
                "runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code",
            ],
        );
    }
}

#[test]
fn runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed() {
    let child =
        read_runtime_src("tests/runtime_absorption/structure_convention/runtime_dead_code.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert!(
        !child.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "runtime dead-code guard should not embed the forbidden attribute as a source literal"
    );
    assert_contains_all(
        "runtime dead-code guard constant-backed forbidden attribute",
        &child,
        &[
            "const DEAD_CODE_ALLOW_ATTRIBUTE: &str = concat!(\"#[allow(\", \"dead_code\", \")]\");",
            "!ui_mod.contains(DEAD_CODE_ALLOW_ATTRIBUTE)",
            "!builtin_host_modules.contains(DEAD_CODE_ALLOW_ATTRIBUTE)",
            "runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
        ],
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
                "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup",
                "runtime_15_runtime_dead_code_guard_literal_cleanup_static_passed_cargo_deferred",
                "structure_convention/runtime_dead_code.rs",
                "runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
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
        "fn runtime_15_ui_text_edit_state_dead_code_suppression_cleanup",
        "fn runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
        "fn runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code",
        "fn runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
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

#[test]
fn runtime_15_production_sources_do_not_allow_dead_code_suppression() {
    let src_root = runtime_src_path("");
    let mut production_sources = Vec::new();
    collect_production_rust_sources(&src_root, &src_root, &mut production_sources);
    production_sources.sort();

    assert!(
        production_sources.len() > 100,
        "production dead-code scan should cover the runtime source tree; got {} files",
        production_sources.len()
    );

    let mut violations = Vec::new();
    for path in &production_sources {
        let source = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read production source `{path:?}`: {error}"));
        let suppression_lines = dead_code_suppression_lines(&source);
        if !suppression_lines.is_empty() {
            let relative = path
                .strip_prefix(&src_root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            violations.push(format!("{relative}: {suppression_lines:?}"));
        }
    }

    assert!(
        violations.is_empty(),
        "production runtime sources should not use dead-code suppression: {violations:?}"
    );

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M5 production dead-code suppression global gate",
                "runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred",
                "structure_convention/runtime_dead_code.rs",
                "runtime_15_production_sources_do_not_allow_dead_code_suppression",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status map",
        &status_map,
        &[
            "Runtime 15 M5 production dead-code suppression global gate",
            "runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 date map",
        &date_map,
        &["Runtime 15 M5 production dead-code suppression global gate"],
    );
}

fn dead_code_suppression_lines(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let compact: String = line.chars().filter(|ch| !ch.is_whitespace()).collect();
            if compact.contains(DEAD_CODE_ALLOW_CALL_PREFIX)
                && (compact.contains("#[") || compact.contains("#!["))
            {
                Some((index + 1, line.trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn collect_production_rust_sources(
    src_root: &std::path::Path,
    current_dir: &std::path::Path,
    sources: &mut Vec<std::path::PathBuf>,
) {
    for entry in std::fs::read_dir(current_dir)
        .unwrap_or_else(|error| panic!("failed to read directory `{current_dir:?}`: {error}"))
    {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read directory entry under `{current_dir:?}`: {error}")
        });
        let path = entry.path();
        if path.is_dir() {
            collect_production_rust_sources(src_root, &path, sources);
        } else if is_production_rust_source(src_root, &path) {
            sources.push(path);
        }
    }
}

fn is_production_rust_source(root: &std::path::Path, path: &std::path::Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return false;
    }

    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "tests.rs" || file_name.ends_with("_tests.rs") {
        return false;
    }

    let relative = path.strip_prefix(root).unwrap_or(path);
    !relative.components().any(|component| match component {
        std::path::Component::Normal(name) => name == std::ffi::OsStr::new("tests"),
        _ => false,
    })
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
