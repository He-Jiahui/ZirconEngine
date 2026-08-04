use super::*;

#[test]
fn runtime_15_runtime_plugin_lifecycle_fixture_owner_is_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/runtime_plugin_lifecycle.rs");
    let fixtures =
        read_runtime_src("tests/plugin_extensions/runtime_plugin_lifecycle/lifecycle_fixtures.rs");
    let capability_projection = read_runtime_src(
        "tests/plugin_extensions/runtime_plugin_lifecycle/capability_projection.rs",
    );
    let kernel_lifecycle =
        read_runtime_src("tests/plugin_extensions/runtime_plugin_lifecycle/kernel_lifecycle.rs");
    let native_projection =
        read_runtime_src("tests/plugin_extensions/runtime_plugin_lifecycle/native_projection.rs");
    let registration_order =
        read_runtime_src("tests/plugin_extensions/runtime_plugin_lifecycle/registration_order.rs");
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "runtime plugin lifecycle parent mounts lifecycle fixture child owner",
        &parent,
        &[
            "#[path = \"runtime_plugin_lifecycle/lifecycle_fixtures.rs\"]",
            "mod lifecycle_fixtures;",
            "use lifecycle_fixtures::*;",
        ],
    );

    for moved_fixture in [
        "struct RecordingModuleLifecycle",
        "struct KernelLifecyclePlugin",
    ] {
        assert!(
            !parent.contains(moved_fixture),
            "runtime plugin lifecycle parent should mount the fixture child instead of defining {moved_fixture}"
        );
    }

    assert_contains_all(
        "runtime plugin lifecycle fixture child owns lifecycle helper types",
        &fixtures,
        &[
            "pub(super) struct RecordingModuleLifecycle",
            "pub(super) struct KernelLifecyclePlugin",
            "impl ModuleLifecycle for RecordingModuleLifecycle",
            "impl RuntimePlugin for KernelLifecyclePlugin",
        ],
    );

    assert_eq!(
        [
            parent.as_str(),
            fixtures.as_str(),
            capability_projection.as_str(),
            kernel_lifecycle.as_str(),
            native_projection.as_str(),
            registration_order.as_str(),
        ]
        .into_iter()
        .map(|source| source.matches("#[test]").count())
        .sum::<usize>(),
        8,
        "runtime plugin lifecycle owner tree should preserve the current 8 tests"
    );
    assert_eq!(
        fixtures.matches("#[test]").count(),
        0,
        "runtime plugin lifecycle fixture child should only own helper types"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/runtime_plugin_lifecycle.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/runtime_plugin_lifecycle/lifecycle_fixtures.rs",
            fixtures.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
