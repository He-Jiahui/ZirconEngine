#[test]
fn runtime_ui_template_builders_live_under_build_namespace_without_bridge_folder() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let template_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/template/mod.rs")).unwrap_or_default();

    assert!(
        runtime_root.join("src/ui/template/build/mod.rs").exists(),
        "runtime ui template builders should live under the folder-backed build owner"
    );
    assert!(
        !runtime_root.join("src/ui/template/bridge").exists(),
        "runtime ui template should delete the legacy bridge folder after the hard cutover"
    );
    assert!(
        template_mod_source.contains("mod build;"),
        "runtime ui template root should wire the build owner directly"
    );
    assert!(
        !template_mod_source.contains("mod bridge;")
            && !template_mod_source.contains("pub use bridge::"),
        "runtime ui template root should not preserve bridge-based forwarding after the cutover"
    );
}

#[test]
fn runtime_ui_template_surface_removes_legacy_asset_migration_entrypoints() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let template_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/template/mod.rs")).unwrap_or_default();
    let asset_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/template/asset/mod.rs"))
            .unwrap_or_default();
    let loader_source =
        std::fs::read_to_string(runtime_root.join("src/ui/template/asset/loader.rs"))
            .unwrap_or_default();
    let legacy_source_path = runtime_root.join("src/ui/template/asset/legacy.rs");

    assert!(
        !template_mod_source.contains("UiLegacyTemplateAdapter")
            && !template_mod_source.contains("UiFlatAssetMigrationAdapter"),
        "runtime template root should drop legacy migration adapters from the formal surface"
    );
    assert!(
        !asset_mod_source.contains("mod legacy;")
            && !asset_mod_source.contains("UiLegacyTemplateAdapter")
            && !asset_mod_source.contains("UiFlatAssetMigrationAdapter"),
        "runtime template asset surface should stop wiring legacy migration entrypoints"
    );
    assert!(
        !loader_source.contains("UiFlatAssetMigrationAdapter")
            && !loader_source.contains("looks_like_flat"),
        "formal runtime asset loader should stay tree-only instead of silently canonicalizing flat documents"
    );
    assert!(
        !legacy_source_path.exists(),
        "runtime template asset legacy.rs entrypoint should be removed after the hard cutover"
    );
}

#[test]
fn runtime_ui_surface_keeps_template_and_layout_specialists_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    for required in [
        "pub mod layout;",
        "pub mod surface;",
        "pub mod template;",
        "pub mod tree;",
    ] {
        assert!(
            ui_mod_source.contains(required),
            "zircon_runtime::ui should expose namespace surface `{required}`"
        );
    }

    for forbidden in [
        "pub use zircon_ui::layout::{compute_layout_tree, compute_virtual_list_window, solve_axis_constraints};",
        "pub use zircon_ui::template::{UiCompiledDocument, UiDocumentCompiler, UiStyleResolver};",
        "pub use zircon_ui::{UiHitTestIndex, UiHitTestResult, UiLayoutCache, UiTemplateNodeMetadata, UiTreeError};",
        "pub use zircon_ui::{UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle, UiVisualAssetRef};",
        "UiComponentDefinition",
        "UiActionRef",
        "UiAssetHeader",
        "UiAssetImports",
        "UiAssetKind",
        "UiAssetLoader",
        "UiAssetRoot",
        "UiAssetError",
        "UiBindingRef",
        "UiChildMount",
        "UiComponentTemplate",
        "UiComponentParamSchema",
        "UiSelector",
        "UiSelectorToken",
        "UiSlotTemplate",
        "UiStyleDeclarationBlock",
        "UiStyleRule",
        "UiStyleSheet",
        "UiTemplateBuildError",
        "UiTemplateInstance",
        "UiNamedSlotSchema",
        "UiNodeDefinition",
        "UiNodeDefinitionKind",
        "UiTemplateNode",
        "UiTemplateSurfaceBuilder",
        "UiTemplateTreeBuilder",
        "UiTemplateValidator",
        "UiStyleScope",
    ] {
        assert!(
            !ui_mod_source.contains(forbidden),
            "zircon_runtime::ui should stop flattening namespace-owned surface `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_surface_keeps_layout_constraint_models_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod layout;"),
        "zircon_runtime::ui should expose the layout namespace directly"
    );

    for (forbidden, needle) in [
        ("AxisConstraint", " AxisConstraint,"),
        ("LayoutBoundary", " LayoutBoundary,"),
        ("StretchMode", " StretchMode,"),
    ] {
        assert!(
            !ui_mod_source.contains(needle),
            "zircon_runtime::ui should stop flattening layout constraint model `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_surface_keeps_asset_document_under_template_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod template;"),
        "zircon_runtime::ui should expose the template namespace directly"
    );

    assert!(
        !ui_mod_source.contains("UiAssetDocument"),
        "zircon_runtime::ui should stop flattening template asset document `UiAssetDocument`"
    );
}
