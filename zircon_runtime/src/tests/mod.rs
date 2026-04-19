use crate::builtin_runtime_modules;
use crate::core::CoreRuntime;

#[test]
fn runtime_absorption_does_not_keep_nested_compatibility_shells() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    assert!(
        !runtime_root.join("crates").exists(),
        "zircon_runtime should not keep nested compatibility crates after absorption"
    );
}

#[test]
fn builtin_runtime_modules_include_absorbed_high_level_subsystems_and_extensions() {
    let descriptors = builtin_runtime_modules()
        .into_iter()
        .map(|module| module.descriptor().name)
        .collect::<Vec<_>>();

    for expected in [
        crate::foundation::FOUNDATION_MODULE_NAME,
        crate::platform::PLATFORM_MODULE_NAME,
        crate::input::INPUT_MODULE_NAME,
        crate::asset::ASSET_MODULE_NAME,
        crate::scene::SCENE_MODULE_NAME,
        crate::script::SCRIPT_MODULE_NAME,
        crate::ui::UI_MODULE_NAME,
        "PhysicsModule",
        "SoundModule",
        "TextureModule",
        "NetModule",
        "NavigationModule",
        "ParticlesModule",
        "AnimationModule",
    ] {
        assert!(
            descriptors.iter().any(|name| name == expected),
            "missing runtime module {expected}"
        );
    }
}

#[test]
fn script_subsystem_is_physically_absorbed_into_runtime_crate() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script_entry = runtime_root.join("src/script.rs");
    let script_mod = runtime_root.join("src/script/mod.rs");
    let workspace_manifest = runtime_root.join("../Cargo.toml");
    let legacy_manifest = runtime_root.join("../zircon_script/Cargo.toml");

    let script_entry_source = std::fs::read_to_string(&script_entry).unwrap_or_default();
    let workspace_manifest_source =
        std::fs::read_to_string(&workspace_manifest).unwrap_or_default();

    assert!(
        script_mod.exists(),
        "expected zircon_runtime/src/script/mod.rs to own the absorbed script subsystem"
    );
    assert!(
        !script_entry_source.contains("pub use zircon_script::*"),
        "zircon_runtime/src/script.rs should stop re-exporting zircon_script after absorption"
    );
    assert!(
        !workspace_manifest_source.contains("\"zircon_script\""),
        "workspace Cargo.toml should stop listing zircon_script after absorption"
    );
    assert!(
        !legacy_manifest.exists(),
        "standalone zircon_script crate should be deleted after absorption"
    );
}

#[test]
fn graphics_module_host_is_absorbed_into_runtime_graphics_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let graphics_entry = runtime_root.join("src/graphics.rs");
    let graphics_mod = runtime_root.join("src/graphics/mod.rs");
    let graphics_entry_source = std::fs::read_to_string(&graphics_entry).unwrap_or_default();
    let graphics_mod_source = std::fs::read_to_string(&graphics_mod).unwrap_or_default();

    assert!(
        graphics_mod.exists(),
        "expected zircon_runtime/src/graphics/mod.rs to own the absorbed graphics module-host surface"
    );
    assert!(
        graphics_mod_source.contains("GraphicsModule"),
        "zircon_runtime::graphics should define GraphicsModule after host absorption"
    );
    assert!(
        !graphics_entry_source.contains("pub use zircon_graphics::*"),
        "zircon_runtime/src/graphics.rs should stop re-exporting the entire zircon_graphics crate"
    );
    for forbidden in [
        "create_render_service",
        "create_runtime_preview_renderer",
        "create_shared_texture_render_service",
        "WgpuDriver",
        "WgpuRenderingManager",
    ] {
        assert!(
            !graphics_mod_source.contains(forbidden),
            "zircon_runtime::graphics should stop publicly exporting `{forbidden}` after graphics boundary cleanup"
        );
    }
}

#[test]
fn graphics_runtime_host_no_longer_owns_legacy_preview_or_render_service_wiring() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let host_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/runtime_builtin_graphics/host/mod.rs"),
    )
    .unwrap_or_default();
    let module_host_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/runtime_builtin_graphics/host/module_host/mod.rs"),
    )
    .unwrap_or_default();
    let create_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/runtime_builtin_graphics/host/module_host/create/mod.rs"),
    )
    .unwrap_or_default();
    let rendering_manager_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/runtime_builtin_graphics/host/module_host/rendering_manager/mod.rs"),
    )
    .unwrap_or_default();

    for forbidden in [
        "create_render_service",
        "create_runtime_preview_renderer",
        "create_shared_texture_render_service",
        "WgpuDriver",
        "WgpuRenderingManager",
        "WGPU_DRIVER_NAME",
    ] {
        assert!(
            !host_mod_source.contains(forbidden),
            "runtime graphics host should stop publicly exporting `{forbidden}`"
        );
    }

    assert!(
        module_host_mod_source.contains("pub use module_registration::module_descriptor;"),
        "runtime graphics module host should keep module_descriptor on its own re-export line to avoid grouped-use name collisions"
    );
    assert!(
        !module_host_mod_source.contains("pub use module_registration::{"),
        "runtime graphics module host should not group module_descriptor with other module_registration re-exports"
    );

    for forbidden in [
        "create_render_service",
        "create_runtime_preview_renderer",
        "create_shared_texture_render_service",
    ] {
        assert!(
            !create_mod_source.contains(forbidden),
            "runtime graphics create surface should stop routing legacy helper `{forbidden}`"
        );
    }

    for forbidden in [
        "manager_create_runtime_preview_renderer",
        "manager_spawn_render_service",
        "manager_spawn_shared_texture_render_service",
    ] {
        assert!(
            !rendering_manager_mod_source.contains(forbidden),
            "runtime rendering manager should stop owning legacy helper module `{forbidden}`"
        );
    }
}


#[test]
fn ui_module_registration_is_absorbed_into_runtime_ui_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_entry = runtime_root.join("src/ui.rs");
    let ui_mod = runtime_root.join("src/ui/mod.rs");
    let ui_module = runtime_root.join("src/ui/module.rs");
    let legacy_ui_lib = runtime_root.join("../zircon_ui/src/lib.rs");

    let ui_entry_source = std::fs::read_to_string(&ui_entry).unwrap_or_default();
    let ui_mod_source = std::fs::read_to_string(&ui_mod).unwrap_or_default();
    let ui_module_source = std::fs::read_to_string(&ui_module).unwrap_or_default();

    assert!(
        ui_mod.exists(),
        "expected zircon_runtime/src/ui/mod.rs to own the absorbed UI module registration surface"
    );
    assert!(
        ui_mod_source.contains("UiModule"),
        "zircon_runtime::ui should define UiModule after UI module absorption"
    );
    assert!(
        !ui_entry_source.contains("pub use zircon_ui::*"),
        "zircon_runtime/src/ui.rs should stop re-exporting the entire zircon_ui crate after absorption"
    );
    assert!(
        !ui_mod_source.contains("pub use zircon_ui::*"),
        "zircon_runtime/src/ui/mod.rs should stop wildcard-re-exporting zircon_ui"
    );
    assert!(
        !ui_module_source.contains("stub_module_descriptor"),
        "zircon_runtime::ui module descriptor should stop using stub_module_descriptor"
    );
    assert!(
        !legacy_ui_lib.exists(),
        "standalone zircon_ui crate should be removed after merging into zircon_runtime::ui"
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

#[test]
fn runtime_ui_surface_keeps_surface_state_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod surface;"),
        "zircon_runtime::ui should expose the surface namespace directly"
    );

    for forbidden in ["UiFocusState", "UiNavigationState"] {
        assert!(
            !ui_mod_source.contains(forbidden),
            "zircon_runtime::ui should stop flattening surface state `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_surface_keeps_input_policy_under_tree_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod tree;"),
        "zircon_runtime::ui should expose the tree namespace directly"
    );

    assert!(
        !ui_mod_source.contains("UiInputPolicy"),
        "zircon_runtime::ui should stop flattening tree input policy `UiInputPolicy`"
    );
}

#[test]
fn runtime_ui_surface_keeps_dispatch_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod dispatch;"),
        "zircon_runtime::ui should expose the dispatch namespace directly"
    );

    for (forbidden, needle) in [
        ("UiNavigationDispatchContext", "UiNavigationDispatchContext"),
        ("UiNavigationDispatchEffect", "UiNavigationDispatchEffect"),
        (
            "UiNavigationDispatchInvocation",
            "UiNavigationDispatchInvocation",
        ),
        ("UiNavigationDispatchResult", "UiNavigationDispatchResult"),
        ("UiNavigationDispatcher", "UiNavigationDispatcher"),
        ("UiPointerDispatchContext", "UiPointerDispatchContext"),
        ("UiPointerDispatchEffect", "UiPointerDispatchEffect"),
        ("UiPointerDispatchInvocation", "UiPointerDispatchInvocation"),
        ("UiPointerDispatchResult", "UiPointerDispatchResult"),
        ("UiPointerDispatcher", "UiPointerDispatcher"),
        ("UiPointerEvent", "UiPointerEvent,"),
    ] {
        assert!(
            !ui_mod_source.contains(needle),
            "zircon_runtime::ui should stop flattening dispatch specialist `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_surface_keeps_binding_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod binding;"),
        "zircon_runtime::ui should expose the binding namespace directly"
    );
    assert!(
        !ui_mod_source.contains("pub use binding::*;"),
        "zircon_runtime::ui should stop wildcard-flattening the binding namespace"
    );

    for forbidden in [
        "UiBindingCall",
        "UiBindingParseError",
        "UiBindingValue",
        "UiEventBinding",
        "UiEventKind",
        "UiEventPath",
        "UiEventRouter",
    ] {
        assert!(
            !ui_mod_source.contains(forbidden),
            "zircon_runtime::ui should stop flattening binding specialist `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_surface_keeps_event_ui_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod event_ui;"),
        "zircon_runtime::ui should expose the event_ui namespace directly"
    );
    assert!(
        !ui_mod_source.contains("pub use event_ui::*;"),
        "zircon_runtime::ui should stop wildcard-flattening the event_ui namespace"
    );

    for forbidden in [
        "UiActionDescriptor",
        "UiBindingCodec",
        "UiControlRequest",
        "UiControlResponse",
        "UiEventManager",
        "UiInvocationContext",
        "UiInvocationError",
        "UiInvocationRequest",
        "UiInvocationResponse",
        "UiInvocationResult",
        "UiInvocationSource",
        "UiNodeDescriptor",
        "UiNodeId",
        "UiNodePath",
        "UiNotification",
        "UiParameterDescriptor",
        "UiPropertyDescriptor",
        "UiReflectionDiff",
        "UiReflectionSnapshot",
        "UiRouteId",
        "UiStateFlags",
        "UiSubscriptionId",
        "UiTreeId",
        "UiValueType",
    ] {
        assert!(
            !ui_mod_source.contains(forbidden),
            "zircon_runtime::ui should stop flattening event_ui specialist `{forbidden}`"
        );
    }
}

#[test]
fn ui_runtime_module_registers_real_driver_and_manager_services() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::ui::module_descriptor())
        .unwrap();
    runtime.activate_module(crate::ui::UI_MODULE_NAME).unwrap();

    let _driver = runtime
        .resolve_driver::<crate::ui::UiRuntimeDriver>(crate::ui::UI_RUNTIME_DRIVER_NAME)
        .unwrap();
    let _manager = runtime
        .resolve_manager::<crate::ui::event_ui::UiEventManager>(crate::ui::UI_EVENT_MANAGER_NAME)
        .unwrap();
}

#[test]
fn runtime_ui_host_surface_is_absorbed_into_runtime_ui_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();
    let runtime_ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/runtime_ui/mod.rs")).unwrap_or_default();
    let graphics_runtime_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/runtime/mod.rs")).unwrap_or_default();
    let graphics_lib_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/mod.rs")).unwrap_or_default();

    assert!(
        runtime_root.join("src/ui/runtime_ui/mod.rs").exists(),
        "runtime ui host subtree should live under zircon_runtime::ui::runtime_ui"
    );
    assert!(
        ui_mod_source.contains(
            "pub use runtime_ui::{RuntimeUiFixture, RuntimeUiManager, RuntimeUiManagerError};"
        ),
        "zircon_runtime::ui should publicly own the runtime UI host surface"
    );
    for required in [
        "RuntimeUiFixture",
        "RuntimeUiManager",
        "RuntimeUiManagerError",
    ] {
        assert!(
            runtime_ui_mod_source.contains(required),
            "runtime ui host subtree should own `{required}`"
        );
    }
    for forbidden in [
        "mod ui;",
        "RuntimeUiFixture",
        "RuntimeUiManager",
        "RuntimeUiManagerError",
    ] {
        assert!(
            !graphics_runtime_mod_source.contains(forbidden)
                && !graphics_lib_source.contains(forbidden),
            "zircon_runtime::graphics should not leak runtime UI host surface `{forbidden}` at the graphics crate root"
        );
    }
}

#[test]
fn runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces() {
    let viewport_size = crate::core::math::UVec2::new(1280, 720);
    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);

    for fixture in [
        crate::ui::RuntimeUiFixture::HudOverlay,
        crate::ui::RuntimeUiFixture::PauseMenu,
        crate::ui::RuntimeUiFixture::SettingsDialog,
        crate::ui::RuntimeUiFixture::InventoryList,
    ] {
        manager.load_builtin_fixture(fixture).unwrap();

        let surface = manager.surface();
        assert_eq!(surface.tree.roots.len(), 1);
        assert!(
            surface.render_extract.list.commands.len() >= 4,
            "expected fixture {fixture:?} to build a non-trivial shared visual tree"
        );
        assert_eq!(
            manager.build_frame().viewport_size,
            viewport_size,
            "runtime UI frame should preserve viewport size for {fixture:?}"
        );
        assert!(
            manager.build_frame().ui.is_some(),
            "runtime UI frame should carry a shared UI render extract for {fixture:?}"
        );
    }
}

#[cfg(feature = "runtime-ui-integration-tests")]
#[test]
fn render_framework_submits_runtime_ui_frames_and_renders_pause_menu_panels() {
    use std::sync::Arc;

    use crate::core::framework::render::{
        RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
    };

    let viewport_size = crate::core::math::UVec2::new(640, 360);
    let asset_manager = Arc::new(crate::asset::ProjectAssetManager::default());
    let server = crate::graphics::WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();

    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::PauseMenu)
        .unwrap();

    server
        .submit_runtime_frame(viewport, manager.build_frame().into())
        .unwrap();

    let stats = server.query_stats().unwrap();
    assert!(
        stats.last_ui_command_count >= 8,
        "expected runtime UI submission to report draw-list command stats"
    );
    assert!(
        stats.last_ui_quad_count >= 4,
        "expected pause menu fixture to contribute multiple quad-like UI draws"
    );

    let frame = server.capture_frame(viewport).unwrap().unwrap();
    assert!(
        count_non_background_pixels(&frame.rgba) > 4_096,
        "expected pause menu UI pass to contribute a visible screen-space footprint"
    );
    let center_red = average_region_channel(&frame.rgba, viewport_size, 0, 0.35, 0.25, 0.65, 0.75);
    let corner_red = average_region_channel(&frame.rgba, viewport_size, 0, 0.0, 0.0, 0.18, 0.18);
    assert!(
        center_red > corner_red + 24.0,
        "expected centered pause dialog to brighten the middle of the frame above the corner background; center_red={center_red:.2}, corner_red={corner_red:.2}"
    );
}

#[cfg(feature = "runtime-ui-integration-tests")]
#[test]
fn render_framework_reports_clipped_ui_commands_for_inventory_fixture() {
    use std::sync::Arc;

    use crate::core::framework::render::{
        RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
    };

    let viewport_size = crate::core::math::UVec2::new(960, 540);
    let asset_manager = Arc::new(crate::asset::ProjectAssetManager::default());
    let server = crate::graphics::WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-list")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();

    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::InventoryList)
        .unwrap();

    server
        .submit_runtime_frame(viewport, manager.build_frame().into())
        .unwrap();

    let stats = server.query_stats().unwrap();
    assert!(
        stats.last_ui_clipped_command_count >= 1,
        "expected inventory fixture to route at least one clipped UI command through the runtime UI pass"
    );
}

#[cfg(feature = "runtime-ui-integration-tests")]
fn count_non_background_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[0] > 8 || pixel[1] > 8 || pixel[2] > 8)
        .count()
}

#[cfg(feature = "runtime-ui-integration-tests")]
fn average_region_channel(
    rgba: &[u8],
    viewport_size: crate::core::math::UVec2,
    channel: usize,
    left_norm: f32,
    top_norm: f32,
    right_norm: f32,
    bottom_norm: f32,
) -> f32 {
    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let left = ((width as f32) * left_norm).floor() as usize;
    let top = ((height as f32) * top_norm).floor() as usize;
    let right = ((width as f32) * right_norm).ceil() as usize;
    let bottom = ((height as f32) * bottom_norm).ceil() as usize;

    let mut total = 0.0;
    let mut count = 0usize;
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..right.min(width) {
            let pixel_index = (y * width + x) * 4;
            total += rgba[pixel_index + channel] as f32;
            count += 1;
        }
    }

    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

#[test]
fn asset_module_registration_is_absorbed_into_runtime_asset_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_entry = runtime_root.join("src/asset.rs");
    let asset_mod = runtime_root.join("src/asset/mod.rs");
    let legacy_asset_lib = runtime_root.join("../zircon_asset/src/lib.rs");

    let asset_entry_source = std::fs::read_to_string(&asset_entry).unwrap_or_default();
    let asset_mod_source = std::fs::read_to_string(&asset_mod).unwrap_or_default();

    assert!(
        asset_mod.exists(),
        "expected zircon_runtime/src/asset/mod.rs to own the absorbed asset module registration surface"
    );
    assert!(
        asset_mod_source.contains("AssetModule"),
        "zircon_runtime::asset should define AssetModule after asset module absorption"
    );
    assert!(
        !asset_entry_source.contains("pub use zircon_asset::*"),
        "zircon_runtime/src/asset.rs should stop re-exporting the entire zircon_asset crate after absorption"
    );
    assert!(
        !asset_mod_source.contains("pub use zircon_asset::*"),
        "zircon_runtime/src/asset/mod.rs should stop wildcard-re-exporting zircon_asset"
    );
    assert!(
        !legacy_asset_lib.exists(),
        "standalone zircon_asset crate should be removed after merging into zircon_runtime::asset"
    );
}

#[test]
fn runtime_asset_surface_keeps_project_and_watch_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_mod_source =
        std::fs::read_to_string(runtime_root.join("src/asset/mod.rs")).unwrap_or_default();

    for required in [
        "pub mod artifact;",
        "pub mod assets;",
        "pub mod editor;",
        "pub mod importer;",
        "pub mod pipeline;",
        "pub mod project;",
        "pub mod watch;",
    ] {
        assert!(
            asset_mod_source.contains(required),
            "zircon_runtime::asset should expose namespace surface `{required}`"
        );
    }

    for forbidden in [
        "pub use zircon_asset::ArtifactStore;",
        "pub use zircon_asset::MaterialAsset;",
        "pub use zircon_asset::ProjectAssetManager;",
        "pub use zircon_asset::EditorAssetManager;",
        "pub use zircon_asset::AssetWorkerPool;",
        "pub use zircon_asset::AssetId;",
        "pub use zircon_asset::AssetKind;",
        "pub use zircon_asset::AssetReference;",
        "pub use zircon_asset::AssetUri;",
        "pub use zircon_asset::AssetUuid;",
        "pub use zircon_asset::project::{",
        "pub use zircon_asset::watch::{AssetChange, AssetChangeKind, AssetWatchEvent, AssetWatcher};",
        "pub use zircon_asset::{",
    ] {
        assert!(
            !asset_mod_source.contains(forbidden),
            "zircon_runtime::asset should stop flattening namespace-owned surface `{forbidden}`"
        );
    }
}

#[test]
fn scene_module_registration_is_absorbed_into_runtime_scene_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_entry = runtime_root.join("src/scene.rs");
    let scene_mod = runtime_root.join("src/scene/mod.rs");
    let legacy_scene_lib = runtime_root.join("../zircon_scene/src/lib.rs");

    let scene_entry_source = std::fs::read_to_string(&scene_entry).unwrap_or_default();
    let scene_mod_source = std::fs::read_to_string(&scene_mod).unwrap_or_default();

    assert!(
        scene_mod.exists(),
        "expected zircon_runtime/src/scene/mod.rs to own the absorbed scene module registration surface"
    );
    assert!(
        scene_mod_source.contains("SceneModule"),
        "zircon_runtime::scene should define SceneModule after scene module absorption"
    );
    assert!(
        !scene_entry_source.contains("pub use zircon_scene::*"),
        "zircon_runtime/src/scene.rs should stop re-exporting the entire zircon_scene crate after absorption"
    );
    assert!(
        !legacy_scene_lib.exists(),
        "standalone zircon_scene crate should be removed after merging into zircon_runtime::scene"
    );
}

#[test]
fn scene_runtime_orchestration_and_level_system_are_absorbed_into_runtime_scene_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_mod = runtime_root.join("src/scene/mod.rs");
    let level_system = runtime_root.join("src/scene/level_system.rs");
    let module_dir = runtime_root.join("src/scene/module/mod.rs");
    let legacy_scene_lib = runtime_root.join("../zircon_scene/src/lib.rs");

    let scene_mod_source = std::fs::read_to_string(&scene_mod).unwrap_or_default();

    assert!(
        level_system.exists(),
        "runtime scene should own LevelSystem"
    );
    assert!(
        module_dir.exists(),
        "runtime scene should own folder-backed module orchestration"
    );
    assert!(scene_mod_source.contains("LevelSystem"));
    assert!(scene_mod_source.contains("DefaultLevelManager"));
    assert!(scene_mod_source.contains("WorldDriver"));
    assert!(
        !scene_mod_source.contains("pub use zircon_scene::*"),
        "runtime scene root should stop wildcard-re-exporting zircon_scene"
    );
    assert!(
        !legacy_scene_lib.exists(),
        "standalone zircon_scene crate should be removed after merging into zircon_runtime::scene"
    );
}

#[test]
fn runtime_scene_surface_keeps_scene_domain_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_mod_source =
        std::fs::read_to_string(runtime_root.join("src/scene/mod.rs")).unwrap_or_default();

    for required in [
        "pub mod components;",
        "pub mod semantics;",
        "pub mod serializer;",
        "pub mod world;",
    ] {
        assert!(
            scene_mod_source.contains(required),
            "zircon_runtime::scene should expose namespace surface `{required}`"
        );
    }

    for forbidden in [
        "pub use zircon_scene::{",
        "CameraComponent",
        "ComponentData",
        "EntityIdentity",
        "SceneAssetSerializer",
        "SceneProjectError",
        "WorldMatrix",
    ] {
        assert!(
            !scene_mod_source.contains(forbidden),
            "zircon_runtime::scene should stop flattening scene-domain surface `{forbidden}`"
        );
    }
}

#[test]
fn optional_extension_module_registration_is_absorbed_into_runtime_extensions_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let extensions_mod = runtime_root.join("src/extensions/mod.rs");
    let builtin_source =
        std::fs::read_to_string(runtime_root.join("src/builtin/mod.rs")).unwrap_or_default();

    assert!(
        extensions_mod.exists(),
        "expected zircon_runtime/src/extensions/mod.rs to own optional extension module registration"
    );

    for (legacy_lib, module_name) in [
        ("../zircon_physics/src/lib.rs", "PhysicsModule"),
        ("../zircon_animation/src/lib.rs", "AnimationModule"),
    ] {
        let legacy_source =
            std::fs::read_to_string(runtime_root.join(legacy_lib)).unwrap_or_default();

        assert!(
            !legacy_source.contains(&format!("pub struct {module_name}")),
            "legacy extension crate {legacy_lib} should stop owning {module_name}"
        );
        assert!(
            !legacy_source.contains("pub fn module_descriptor()"),
            "legacy extension crate {legacy_lib} should stop owning module_descriptor()"
        );
        assert!(
            !builtin_source.contains(&format!("Arc::new(zircon_")),
            "builtin runtime module list should stop constructing optional extension modules from legacy crate roots"
        );
    }
}

#[test]
fn physics_extension_is_physically_absorbed_into_runtime() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let physics_mod = std::fs::read_to_string(runtime_root.join("src/physics/mod.rs"))
        .unwrap_or_default();
    let physics_interface =
        std::fs::read_to_string(runtime_root.join("src/physics/physics_interface.rs"))
            .unwrap_or_default();

    assert!(
        runtime_root.join("src/physics/mod.rs").exists(),
        "physics should live as a first-class subsystem at zircon_runtime/src/physics/mod.rs"
    );
    assert!(
        runtime_root
            .join("src/physics/physics_interface.rs")
            .exists(),
        "physics absorption should add a runtime-owned physics_interface surface"
    );
    assert!(
        !runtime_root.join("src/physics.rs").exists(),
        "zircon_runtime should stop keeping physics absorption in a flat root file"
    );
    assert!(
        physics_mod.contains("PhysicsDriver")
            && physics_mod.contains("PhysicsManager")
            && physics_mod.contains("PhysicsInterface"),
        "runtime physics subtree should own the driver, manager, and runtime-facing interface"
    );
    assert!(
        physics_interface.contains("pub trait PhysicsInterface"),
        "physics_interface should define a runtime-owned PhysicsInterface trait"
    );
    assert!(
        !runtime_manifest.contains("zircon_physics"),
        "zircon_runtime/Cargo.toml should stop depending on zircon_physics once physics is absorbed"
    );
    assert!(
        !workspace_manifest.contains("\"zircon_physics\""),
        "workspace Cargo.toml should stop listing zircon_physics after runtime absorption"
    );
    assert!(
        !runtime_root.join("../zircon_physics/Cargo.toml").exists(),
        "legacy zircon_physics crate should be removed instead of kept as a compatibility shell"
    );
}

#[test]
fn animation_extension_is_physically_absorbed_into_runtime() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let animation_mod =
        std::fs::read_to_string(runtime_root.join("src/animation/mod.rs"))
            .unwrap_or_default();
    let animation_interface = std::fs::read_to_string(
        runtime_root.join("src/animation/animation_interface.rs"),
    )
    .unwrap_or_default();

    assert!(
        runtime_root.join("src/animation/mod.rs").exists(),
        "animation should live as a first-class subsystem at zircon_runtime/src/animation/mod.rs"
    );
    assert!(
        runtime_root
            .join("src/animation/animation_interface.rs")
            .exists(),
        "animation absorption should add a runtime-owned animation_interface surface"
    );
    assert!(
        !runtime_root.join("src/animation.rs").exists(),
        "zircon_runtime should stop keeping animation absorption in a flat root file"
    );
    assert!(
        animation_mod.contains("AnimationDriver")
            && animation_mod.contains("AnimationManager")
            && animation_mod.contains("AnimationInterface"),
        "runtime animation subtree should own the driver, manager, and runtime-facing interface"
    );
    assert!(
        animation_interface.contains("pub trait AnimationInterface"),
        "animation_interface should define a runtime-owned AnimationInterface trait"
    );
    assert!(
        !runtime_manifest.contains("zircon_animation"),
        "zircon_runtime/Cargo.toml should stop depending on zircon_animation once animation is absorbed"
    );
    assert!(
        !workspace_manifest.contains("\"zircon_animation\""),
        "workspace Cargo.toml should stop listing zircon_animation after runtime absorption"
    );
    assert!(
        !runtime_root.join("../zircon_animation/Cargo.toml").exists(),
        "legacy zircon_animation crate should be removed instead of kept as a compatibility shell"
    );
}

#[test]
fn physics_and_animation_runtime_extensions_register_public_manager_handles() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let physics_mod_source =
        std::fs::read_to_string(runtime_root.join("src/physics/mod.rs"))
            .unwrap_or_default();
    let physics_service_source =
        std::fs::read_to_string(runtime_root.join("src/physics/service_types.rs"))
            .unwrap_or_default();
    let animation_mod_source =
        std::fs::read_to_string(runtime_root.join("src/animation/mod.rs"))
            .unwrap_or_default();
    let animation_service_source =
        std::fs::read_to_string(runtime_root.join("src/animation/service_types.rs"))
            .unwrap_or_default();

    for required in [
        "PhysicsManagerHandle",
        "crate::core::manager::PHYSICS_MANAGER_NAME",
        "DefaultPhysicsManager",
        "impl crate::core::framework::physics::PhysicsManager for DefaultPhysicsManager",
    ] {
        assert!(
            physics_mod_source.contains(required) || physics_service_source.contains(required),
            "runtime physics extension should expose framework-backed manager handle wiring `{required}`"
        );
    }

    for required in [
        "AnimationManagerHandle",
        "crate::core::manager::ANIMATION_MANAGER_NAME",
        "DefaultAnimationManager",
        "impl crate::core::framework::animation::AnimationManager for DefaultAnimationManager",
    ] {
        assert!(
            animation_mod_source.contains(required)
                || animation_service_source.contains(required),
            "runtime animation extension should expose framework-backed manager handle wiring `{required}`"
        );
    }
}

#[test]
fn physics_and_animation_managers_resolve_through_framework_facades() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::physics::module_descriptor())
        .unwrap();
    runtime
        .register_module(crate::animation::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::physics::PHYSICS_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(crate::animation::ANIMATION_MODULE_NAME)
        .unwrap();

    let physics = crate::core::manager::resolve_physics_manager(&runtime.handle()).unwrap();
    let animation = crate::core::manager::resolve_animation_manager(&runtime.handle()).unwrap();

    assert_eq!(physics.backend_name(), "unconfigured");
    assert_eq!(physics.settings().fixed_hz, 60);
    assert!(animation.playback_settings().enabled);
    assert!(animation.playback_settings().property_tracks);

    let track_path =
        crate::core::framework::animation::AnimationTrackPath::parse("root/child:transform.translation")
            .unwrap();
    assert_eq!(animation.normalize_track_path(&track_path), track_path);
}
