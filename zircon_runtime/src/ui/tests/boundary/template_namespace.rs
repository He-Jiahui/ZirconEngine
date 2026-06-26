use super::*;

#[test]
fn template_legacy_adapter_is_removed_from_formal_namespace_surface() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        lib_source.contains("pub mod template;"),
        "zircon_ui root should expose the template namespace directly"
    );

    assert!(
        interface_template_mod_source.contains("UiTemplateDocument"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiTemplateDocument`"
    );
    assert!(
        !template_mod_source.contains("UiTemplateDocument"),
        "zircon_ui::template should not re-export interface DTO `UiTemplateDocument`"
    );

    for required in ["UiTemplateLoader"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should expose runtime behavior service `{required}`"
        );
    }

    for forbidden in [
        "UiLegacyTemplateAdapter",
        "UiTemplateDocument",
        "UiTemplateLoader",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template boundary type `{forbidden}`"
        );
    }

    assert!(
        !template_mod_source.contains("UiLegacyTemplateAdapter"),
        "zircon_ui::template should drop the legacy template adapter from the formal surface"
    );
}

#[test]
fn template_compiler_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");

    for required in [
        "UiCompiledDocument",
        "UiDocumentCompiler",
        "UiStyleResolver",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in [
        "UiCompiledDocument",
        "UiDocumentCompiler",
        "UiStyleResolver",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template compiler type `{forbidden}`"
        );
    }
}

#[test]
fn template_runtime_builder_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in ["UiTemplateError"] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
        );
    }

    for required in [
        "UiTemplateBuildError",
        "UiTemplateSurfaceBuilder",
        "UiTemplateTreeBuilder",
        "UiTemplateValidator",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should expose runtime behavior service `{required}`"
        );
    }

    for forbidden in [
        "UiTemplateBuildError",
        "UiTemplateError",
        "UiTemplateSurfaceBuilder",
        "UiTemplateTreeBuilder",
        "UiTemplateValidator",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template runtime specialist `{forbidden}`"
        );
    }
}

#[test]
fn template_runtime_model_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        interface_template_mod_source.contains("UiTemplateNode"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiTemplateNode`"
    );
    assert!(
        !template_mod_source.contains("UiTemplateNode"),
        "zircon_ui::template should not re-export interface DTO `UiTemplateNode`"
    );

    for required in ["UiTemplateInstance"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should expose runtime behavior model `{required}`"
        );
    }

    for forbidden in ["UiTemplateInstance", "UiTemplateNode"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template runtime model `{forbidden}`"
        );
    }
}

#[test]
fn template_component_schema_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in [
        "UiComponentDefinition",
        "UiComponentParamSchema",
        "UiNamedSlotSchema",
        "UiStyleScope",
    ] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
        );
    }

    for forbidden in [
        "UiComponentDefinition",
        "UiComponentParamSchema",
        "UiNamedSlotSchema",
        "UiStyleScope",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template component-schema surface `{forbidden}`"
        );
    }
}

#[test]
fn template_selector_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in ["UiSelector", "UiSelectorToken"] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral selector DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface selector DTO `{required}`"
        );
    }

    for forbidden in ["UiSelector", "UiSelectorToken"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template selector surface `{forbidden}`"
        );
    }
}

#[test]
fn template_binding_model_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in [
        "UiActionRef",
        "UiBindingRef",
        "UiComponentTemplate",
        "UiSlotTemplate",
    ] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
        );
    }

    for forbidden in [
        "UiActionRef",
        "UiBindingRef",
        "UiComponentTemplate",
        "UiSlotTemplate",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template binding model `{forbidden}`"
        );
    }
}

#[test]
fn template_asset_metadata_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in ["UiAssetHeader", "UiAssetImports"] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
        );
    }

    for required in ["UiAssetNodeIter", "UiNodeParent"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should expose runtime document helper `{required}`"
        );
    }

    for forbidden in [
        "UiAssetHeader",
        "UiAssetImports",
        "UiAssetNodeIter",
        "UiNodeParent",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template asset metadata `{forbidden}`"
        );
    }
}

#[test]
fn template_asset_mount_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        interface_template_mod_source.contains("UiChildMount"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiChildMount`"
    );

    assert!(
        !template_mod_source.contains("UiChildMount"),
        "zircon_ui::template should not re-export interface DTO `UiChildMount`"
    );

    assert!(
        !lib_source.contains("UiChildMount"),
        "zircon_ui root should stop flattening template asset mount `UiChildMount`"
    );
}

#[test]
fn template_asset_loader_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");

    assert!(
        template_mod_source.contains("UiAssetLoader"),
        "zircon_ui::template should own `UiAssetLoader`"
    );

    assert!(
        !lib_source.contains("UiAssetLoader"),
        "zircon_ui root should stop flattening template asset loader `UiAssetLoader`"
    );
}

#[test]
fn template_asset_style_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in [
        "UiAssetError",
        "UiStyleDeclarationBlock",
        "UiStyleRule",
        "UiStyleSheet",
    ] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
        );
    }

    for forbidden in [
        "UiAssetError",
        "UiStyleDeclarationBlock",
        "UiStyleRule",
        "UiStyleSheet",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template asset style specialist `{forbidden}`"
        );
    }
}

#[test]
fn template_asset_node_definition_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in ["UiNodeDefinition", "UiNodeDefinitionKind"] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
        );
    }

    for forbidden in ["UiNodeDefinition", "UiNodeDefinitionKind"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template asset node specialist `{forbidden}`"
        );
    }
}

#[test]
fn template_asset_kind_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        interface_template_mod_source.contains("UiAssetKind"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiAssetKind`"
    );

    assert!(
        !template_mod_source.contains("UiAssetKind"),
        "zircon_ui::template should not re-export interface DTO `UiAssetKind`"
    );

    assert!(
        !lib_source.contains("UiAssetKind"),
        "zircon_ui root should stop flattening template asset kind `UiAssetKind`"
    );
}

#[test]
fn template_asset_document_api_moves_under_template_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let template_mod_source = include_str!("../../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        interface_template_mod_source.contains("UiAssetDocument"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiAssetDocument`"
    );

    assert!(
        !template_mod_source.contains("UiAssetDocument,"),
        "zircon_ui::template should not re-export interface DTO `UiAssetDocument`"
    );

    assert!(
        template_mod_source.contains("UiAssetDocumentRuntimeExt"),
        "zircon_ui::template should expose runtime document behavior `UiAssetDocumentRuntimeExt`"
    );

    assert!(
        !lib_source.contains("UiAssetDocument"),
        "zircon_ui root should stop flattening template asset document `UiAssetDocument`"
    );
}
