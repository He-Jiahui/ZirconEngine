const D5_EDITOR_AUTHORING_MACRO_CRATES: &[(&str, &str, &str)] = &[
    (
        "animation",
        include_str!("../../../../../../zircon_plugins/animation/editor/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/animation/editor/src/tests.rs"),
    ),
    (
        "physics",
        include_str!("../../../../../../zircon_plugins/physics/editor/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/physics/editor/src/tests.rs"),
    ),
    (
        "net",
        include_str!("../../../../../../zircon_plugins/net/editor/src/plugin.rs"),
        include_str!(
            "../../../../../../zircon_plugins/net/editor/src/tests/authoring_extensions.rs"
        ),
    ),
];

#[test]
fn review_d5_editor_authoring_plugins_use_sdk_macro() {
    let review_findings = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let structure_convention = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
    );
    let plugins_12 = include_str!(
        "../../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let runtime_15 = crate::tests::runtime_absorption::current_source_fixture::RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT;
    let runtime_index = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let sdk_editor = include_str!("../../../../../../zircon_plugins/plugin_sdk/src/editor.rs");

    assert_eq!(
        D5_EDITOR_AUTHORING_MACRO_CRATES.len(),
        3,
        "D5 editor authoring macro guard should cover animation, physics, and net representative roots"
    );

    for &(label, plugin_source, test_source) in D5_EDITOR_AUTHORING_MACRO_CRATES {
        for required in [
            "authoring_plugin! {",
            "mirrors_runtime_manifest:",
            "capabilities: EDITOR_CAPABILITIES",
            "register_extensions:",
            "pub fn editor_plugin_declaration() -> EditorPluginDeclaration",
            "plugin.declaration().registration_report(&plugin)",
        ] {
            assert!(
                plugin_source.contains(required),
                "{label} editor plugin should use SDK authoring macro anchor `{required}`"
            );
        }
        for stale in [
            "impl zircon_editor::EditorPlugin for",
            "pub struct AnimationEditorPlugin {\n    declaration: EditorPluginDeclaration",
            "pub struct PhysicsEditorPlugin {\n    declaration: EditorPluginDeclaration",
            "pub struct NetEditorPlugin {\n    declaration: EditorPluginDeclaration",
            "EditorPluginDeclaration::new(",
        ] {
            assert!(
                !plugin_source.contains(stale),
                "{label} editor plugin should not keep hand-written editor boilerplate `{stale}`"
            );
        }
        for required in [
            "mirrored_runtime_package_id()",
            ".package_manifest",
            "_AUTHORING_CAPABILITY",
        ] {
            assert!(
                test_source.contains(required),
                "{label} editor tests should keep macro consumer evidence `{required}`"
            );
        }
    }

    for required in [
        "macro_rules! authoring_plugin",
        "mirrors_runtime_manifest: $runtime_manifest:expr",
        "let declaration = declaration.mirrors_runtime_manifest($runtime_manifest);",
        "pub fn declaration(&self) -> &$crate::editor::EditorPluginDeclaration",
        "pub fn registration_report",
    ] {
        assert!(
            sdk_editor.contains(required),
            "plugin SDK editor macro should own D5 helper anchor `{required}`"
        );
    }

    for required in [
        "editor authoring macro consumer guard",
        "animation/physics/net",
        "zircon_plugin_sdk::authoring_plugin!",
        "d5_editor_authoring_macro_consumers_static_passed_cargo_deferred",
        "review_d5_editor_authoring_plugins_use_sdk_macro",
    ] {
        assert!(
            review_findings.contains(required),
            "D5 review output should record editor authoring macro consumer convergence anchor `{required}`"
        );
    }

    for (doc_label, doc) in [
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("Plugins 12 plan", plugins_12),
        ("plugin SDK doc", plugin_sdk_doc),
        ("Runtime 15 plan", runtime_15),
        ("Runtime index", runtime_index),
        ("module convention", module_convention),
    ] {
        for required in [
            "D5 editor authoring macro consumer guard",
            "d5_editor_authoring_macro_consumers_static_passed_cargo_deferred",
            "review_d5_editor_authoring_plugins_use_sdk_macro",
            "zircon_plugin_sdk::authoring_plugin!",
        ] {
            assert!(
                doc.contains(required),
                "{doc_label} should record D5 editor authoring macro consumer anchor `{required}`"
            );
        }
    }
}
