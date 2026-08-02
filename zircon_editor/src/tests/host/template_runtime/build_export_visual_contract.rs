#[test]
fn build_export_body_uses_shared_dense_spacing() {
    let template = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/host/build_export_desktop_body.zui"),
    )
    .expect("build export template should be readable");

    assert_eq!(
        template
            .matches("gap = \"$editor.density.gap.small\"")
            .count(),
        2,
        "the build export root and header must share the dense spacing token"
    );
    assert!(
        !template.contains("gap = 6.0"),
        "build export must not carry a local six-pixel spacing override"
    );
    assert!(
        template.contains(
            "props = { label = \"Focus Export\", icon = \"editor_pages/build_plugins/package/package.svg\", icon_placement = \"leading\" }"
        ),
        "the labeled build export action must declare its icon-and-text composition"
    );
}
