use super::*;

#[test]
fn view_template_projection_is_hard_cut_to_zui_prototype_store() {
    let view_projection = source_file(&["src", "ui", "layouts", "views", "view_projection.rs"]);

    for required in [
        "UiV2PrototypeStoreFileCache",
        ".load_store(",
        "UiV2SurfaceBuilder::build_surface_from_compiled_document",
        "NonV2AssetPath",
    ] {
        assert_contains("view_projection.rs", &view_projection, required);
    }
    for forbidden in [
        "EditorTemplateRuntimeService",
        "load_document_file",
        "compile_document_with_import_maps",
        "try_load_flat_store",
        "UiPrototypeStoreFileCache",
        "UiDocumentCompiler",
        "UiTemplateSurfaceBuilder",
        "compile_prototype_asset(",
    ] {
        assert_does_not_contain("view_projection.rs", &view_projection, forbidden);
    }
}
