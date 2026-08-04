use super::*;

#[test]
fn runtime_15_core_runtime_state_module_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let state_dir = manifest_root.join("src/core/runtime/state");
    let retired_runtime_inner = state_dir.join("runtime_inner.rs");
    let state_mod = read_text(
        &state_dir.join("mod.rs"),
        "core runtime state module entry should be readable",
    );
    let core_runtime_state = read_text(
        &state_dir.join("core_runtime_state.rs"),
        "core runtime state owner should be readable",
    );
    let core_handle = read_text(
        &manifest_root.join("src/core/runtime/handle/core_handle.rs"),
        "core runtime handle owner should be readable",
    );
    let core_runtime = read_text(
        &manifest_root.join("src/core/runtime/runtime.rs"),
        "core runtime facade should be readable",
    );
    let registration_structure = read_text(
        &manifest_root.join("src/core/runtime/tests/registration/structure/mod.rs"),
        "registration structure source fixture should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let core_state_doc = read_repo_text(manifest_root, "docs/zircon_runtime/core/state.md");

    assert!(
        !retired_runtime_inner.exists(),
        "core runtime state owner should not keep banned-name module file {:?}",
        retired_runtime_inner
    );
    assert_contains_all(
        "core runtime state mod entry",
        &state_mod,
        &[
            "mod core_runtime_state;",
            "pub(crate) use core_runtime_state::CoreRuntimeInner;",
        ],
    );
    assert!(
        !state_mod.contains("runtime_inner"),
        "core/runtime/state/mod.rs should not preserve the banned runtime_inner module name"
    );
    assert_contains_all(
        "core runtime state owner",
        &core_runtime_state,
        &[
            "pub(crate) struct CoreRuntimeInner",
            "HashMap<RegistryName, ServiceEntry>",
            "runtime_module_lifecycle_observer",
            "RuntimeModuleLifecycleObserver",
        ],
    );
    assert_contains_all(
        "core runtime lifecycle observer access owner",
        &core_handle,
        &[
            "fn lock_runtime_module_lifecycle_observer(",
            "self.inner.runtime_module_lifecycle_observer",
            "RuntimeModuleLifecycleObserver",
        ],
    );
    assert_contains_all(
        "core runtime lifecycle observer install facade",
        &core_runtime,
        &[
            "pub fn install_runtime_module_lifecycle_observer(",
            "observer: Arc<dyn RuntimeModuleLifecycleObserver>",
            ".install_runtime_module_lifecycle_observer(observer)",
        ],
    );
    for retired_plugin_owner in [
        "plugin_bridge_lifecycle",
        "RuntimePluginBridgeLifecycleState",
        "install_plugin_bridge_lifecycle_state",
    ] {
        assert!(
            !core_runtime_state.contains(retired_plugin_owner)
                && !core_handle.contains(retired_plugin_owner)
                && !core_runtime.contains(retired_plugin_owner),
            "core runtime owners should not restore plugin-specific lifecycle state `{retired_plugin_owner}`"
        );
    }
    assert_contains_all(
        "core runtime registration structure fixture",
        &registration_structure,
        &[
            "pub(super) runtime_state: &'static str",
            "include_str!(\"../../../state/core_runtime_state.rs\")",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("core state doc", core_state_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 core runtime state module naming hard cutover",
                "runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred",
                "core/runtime/state/core_runtime_state.rs",
                "runtime_15_core_runtime_state_module_uses_owner_name",
            ],
        );
    }
}
