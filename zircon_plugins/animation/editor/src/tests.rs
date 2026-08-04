use super::*;
use zircon_editor::EditorPlugin;

#[test]
fn animation_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&ANIMATION_AUTHORING_CAPABILITY.to_string()));
    assert_eq!(
        editor_plugin().declaration().mirrored_runtime_package_id(),
        Some(PLUGIN_ID)
    );
    assert!(registration
        .package_manifest
        .capabilities
        .contains(&zircon_plugin_animation_runtime::ANIMATION_RUNTIME_CAPABILITY.to_string()));
    assert!(registration
        .package_manifest
        .capabilities
        .contains(&ANIMATION_AUTHORING_CAPABILITY.to_string()));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == ANIMATION_AUTHORING_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == ANIMATION_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == ANIMATION_TEMPLATE_ID));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "view.animation.authoring.open"));
    assert!(registration
        .extensions
        .commands()
        .commands()
        .any(|operation| operation.id().as_str() == "view.animation.authoring.open"));
}

#[test]
fn blend_space_and_avatar_mask_asset_drawers_are_owned_by_animation_editor() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("animation asset authoring registration");

    for (component_type, ui_document) in [
        (
            "animation.Asset.BlendSpace1D",
            "plugins://animation/editor/blend_space_1d.zui",
        ),
        (
            "animation.Asset.BlendSpace2D",
            "plugins://animation/editor/blend_space_2d.zui",
        ),
        (
            "animation.Asset.AvatarMask",
            "plugins://animation/editor/avatar_mask_bone_tree.zui",
        ),
    ] {
        assert!(registry.inspector_customizations().iter().any(|customization| {
            customization.target_type() == component_type
                && customization.surface().ui_document() == ui_document
        }));
    }
}
