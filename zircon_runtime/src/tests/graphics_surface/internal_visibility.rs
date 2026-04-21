#[test]
fn graphics_surface_keeps_viewport_frame_and_icon_source_internal() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_lib_source =
        std::fs::read_to_string(runtime_root.join("src/lib.rs")).unwrap_or_default();
    let graphics_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/mod.rs")).unwrap_or_default();
    let graphics_types_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/types/mod.rs")).unwrap_or_default();
    let graphics_scene_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/scene/mod.rs")).unwrap_or_default();
    let scene_renderer_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/scene/scene_renderer/mod.rs"))
            .unwrap_or_default();
    let overlay_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/scene/scene_renderer/overlay/mod.rs"),
    )
    .unwrap_or_default();
    let icon_source_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/scene/scene_renderer/overlay/icon_source/mod.rs"),
    )
    .unwrap_or_default();
    let icon_source_trait_source = std::fs::read_to_string(
        runtime_root
            .join("src/graphics/scene/scene_renderer/overlay/icon_source/viewport_icon_source.rs"),
    )
    .unwrap_or_default();
    let new_with_icon_source_source =
        std::fs::read_to_string(runtime_root.join(
            "src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs",
        ))
        .unwrap_or_default();

    assert!(
        !runtime_lib_source.contains("ViewportRenderFrame")
            && !runtime_lib_source.contains("ViewportIconSource"),
        "crate root should stop re-exporting graphics-internal viewport frame or icon-source seams"
    );
    assert!(
        graphics_mod_source.contains("pub(crate) use types::ViewportRenderFrame;"),
        "graphics root should keep viewport frame carrier as a crate-private seam"
    );
    assert!(
        !graphics_mod_source.contains("pub use scene::{SceneRenderer, ViewportIconSource};"),
        "graphics root should stop publicly exporting ViewportIconSource"
    );
    assert!(
        !graphics_mod_source.contains("ViewportIconSource"),
        "graphics root should stop re-exporting ViewportIconSource entirely"
    );
    assert!(
        !graphics_mod_source.contains("pub use types::{\r\n    ViewportRenderFrame,")
            && !graphics_mod_source.contains("pub use types::{\n    ViewportRenderFrame,"),
        "graphics root should stop publicly exporting ViewportRenderFrame"
    );
    assert!(
        graphics_types_mod_source
            .contains("pub(crate) use viewport_render_frame::ViewportRenderFrame;"),
        "graphics types namespace should keep ViewportRenderFrame crate-private"
    );
    assert!(
        !graphics_types_mod_source.contains("pub use viewport_render_frame::ViewportRenderFrame;"),
        "graphics types namespace should stop public re-export of ViewportRenderFrame"
    );
    assert!(
        !graphics_scene_mod_source
            .contains("pub use scene_renderer::{SceneRenderer, ViewportIconSource};"),
        "graphics scene surface should stop publicly exporting ViewportIconSource"
    );
    assert!(
        !graphics_scene_mod_source.contains("ViewportIconSource"),
        "graphics scene surface should stop re-exporting ViewportIconSource entirely"
    );
    assert!(
        !scene_renderer_mod_source.contains("pub use overlay::ViewportIconSource;")
            && !scene_renderer_mod_source.contains("pub(crate) use overlay::ViewportIconSource;"),
        "scene renderer root should stop re-exporting ViewportIconSource"
    );
    assert!(
        overlay_mod_source.contains("pub(crate) use icon_source::ViewportIconSource;"),
        "overlay root should keep viewport icon source crate-private"
    );
    assert!(
        icon_source_mod_source.contains("pub(crate) use viewport_icon_source::ViewportIconSource;"),
        "icon_source namespace should keep ViewportIconSource crate-private"
    );
    assert!(
        icon_source_trait_source.contains("pub(crate) trait ViewportIconSource"),
        "viewport icon source contract should be crate-private"
    );
    assert!(
        new_with_icon_source_source.contains("pub(crate) fn new_with_icon_source("),
        "SceneRenderer::new_with_icon_source should be crate-private"
    );
    assert!(
        !new_with_icon_source_source.contains("pub fn new_with_icon_source("),
        "SceneRenderer should stop exposing new_with_icon_source publicly"
    );
}
