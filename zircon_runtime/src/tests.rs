use crate::builtin_runtime_modules;
use zircon_core::CoreRuntime;
use zircon_module::EngineModule;

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
        crate::graphics::GRAPHICS_MODULE_NAME,
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
    let legacy_graphics_lib = runtime_root.join("../zircon_graphics/src/lib.rs");
    let legacy_host_mod = runtime_root.join("../zircon_graphics/src/host/mod.rs");

    let graphics_entry_source = std::fs::read_to_string(&graphics_entry).unwrap_or_default();
    let graphics_mod_source = std::fs::read_to_string(&graphics_mod).unwrap_or_default();
    let legacy_graphics_lib_source =
        std::fs::read_to_string(&legacy_graphics_lib).unwrap_or_default();

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
    assert!(
        !legacy_graphics_lib_source.contains("pub struct GraphicsModule"),
        "zircon_graphics root should stop owning GraphicsModule after host absorption"
    );
    assert!(
        !legacy_host_mod.exists(),
        "legacy zircon_graphics/src/host/mod.rs should be removed after host absorption"
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
    let host_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/host/mod.rs")).unwrap_or_default();
    let module_host_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/host/module_host/mod.rs"))
            .unwrap_or_default();
    let create_mod_source =
        std::fs::read_to_string(runtime_root.join("src/graphics/host/module_host/create/mod.rs"))
            .unwrap_or_default();
    let rendering_manager_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/host/module_host/rendering_manager/mod.rs"),
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
fn graphics_runtime_surface_re_exports_module_descriptor_and_owner_type() {
    let descriptor = crate::graphics::module_descriptor();
    assert_eq!(descriptor.name, crate::graphics::GRAPHICS_MODULE_NAME);

    let module = crate::graphics::GraphicsModule;
    assert_eq!(module.module_name(), crate::graphics::GRAPHICS_MODULE_NAME);
    assert_eq!(module.descriptor().name, descriptor.name);
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
    let legacy_ui_lib_source = std::fs::read_to_string(&legacy_ui_lib).unwrap_or_default();

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
        !legacy_ui_lib_source.contains("pub struct UiModule"),
        "zircon_ui root should stop owning UiModule after runtime UI absorption"
    );
}

#[test]
fn runtime_ui_surface_keeps_template_and_layout_specialists_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    for required in [
        "pub use zircon_ui::layout;",
        "pub use zircon_ui::surface;",
        "pub use zircon_ui::template;",
        "pub use zircon_ui::tree;",
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
fn runtime_ui_surface_keeps_dispatch_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub use zircon_ui::dispatch;"),
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
        ui_mod_source.contains("pub use zircon_ui::binding;"),
        "zircon_runtime::ui should expose the binding namespace directly"
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
        ui_mod_source.contains("pub use zircon_ui::event_ui;"),
        "zircon_runtime::ui should expose the event_ui namespace directly"
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
        .resolve_manager::<zircon_ui::event_ui::UiEventManager>(crate::ui::UI_EVENT_MANAGER_NAME)
        .unwrap();
}

#[test]
fn runtime_ui_host_surface_is_absorbed_into_runtime_ui_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();
    let runtime_ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/runtime_ui/mod.rs")).unwrap_or_default();
    let legacy_graphics_runtime_mod_source =
        std::fs::read_to_string(runtime_root.join("../zircon_graphics/src/runtime/mod.rs"))
            .unwrap_or_default();
    let legacy_graphics_lib_source =
        std::fs::read_to_string(runtime_root.join("../zircon_graphics/src/lib.rs"))
            .unwrap_or_default();

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
            !legacy_graphics_runtime_mod_source.contains(forbidden)
                && !legacy_graphics_lib_source.contains(forbidden),
            "zircon_graphics should stop exporting runtime UI host surface `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces() {
    let viewport_size = zircon_math::UVec2::new(1280, 720);
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

    use zircon_framework::render::{
        RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
    };

    let viewport_size = zircon_math::UVec2::new(640, 360);
    let asset_manager = Arc::new(crate::asset::ProjectAssetManager::default());
    let server = zircon_graphics::WgpuRenderFramework::new(asset_manager).unwrap();
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
        .submit_runtime_frame(viewport, manager.build_frame())
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

    use zircon_framework::render::{
        RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
    };

    let viewport_size = zircon_math::UVec2::new(960, 540);
    let asset_manager = Arc::new(crate::asset::ProjectAssetManager::default());
    let server = zircon_graphics::WgpuRenderFramework::new(asset_manager).unwrap();
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
        .submit_runtime_frame(viewport, manager.build_frame())
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
    viewport_size: zircon_math::UVec2,
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
    let legacy_asset_lib_source = std::fs::read_to_string(&legacy_asset_lib).unwrap_or_default();

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
        !legacy_asset_lib_source.contains("pub struct AssetModule"),
        "zircon_asset root should stop owning AssetModule after runtime asset absorption"
    );
}

#[test]
fn runtime_asset_surface_keeps_project_and_watch_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_mod_source =
        std::fs::read_to_string(runtime_root.join("src/asset/mod.rs")).unwrap_or_default();

    for required in [
        "pub use zircon_asset::artifact;",
        "pub use zircon_asset::assets;",
        "pub use zircon_asset::editor;",
        "pub use zircon_asset::importer;",
        "pub use zircon_asset::pipeline;",
        "pub use zircon_asset::project;",
        "pub use zircon_asset::watch;",
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
    let legacy_scene_lib_source = std::fs::read_to_string(&legacy_scene_lib).unwrap_or_default();

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
        !legacy_scene_lib_source.contains("pub struct SceneModule"),
        "zircon_scene root should stop owning SceneModule after runtime scene absorption"
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
    let legacy_scene_lib_source = std::fs::read_to_string(&legacy_scene_lib).unwrap_or_default();

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
        !legacy_scene_lib_source.contains("pub use level_system"),
        "zircon_scene root should stop exporting LevelSystem after runtime absorption"
    );
}

#[test]
fn runtime_scene_surface_keeps_scene_domain_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_mod_source =
        std::fs::read_to_string(runtime_root.join("src/scene/mod.rs")).unwrap_or_default();

    for required in [
        "pub use zircon_scene::components;",
        "pub use zircon_scene::semantics;",
        "pub use zircon_scene::serializer;",
        "pub use zircon_scene::world;",
    ] {
        assert!(
            scene_mod_source.contains(required),
            "zircon_runtime::scene should expose namespace surface `{required}`"
        );
    }

    for forbidden in [
        "pub use zircon_scene::{",
        "default_render_layer_mask",
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
        std::fs::read_to_string(runtime_root.join("src/builtin.rs")).unwrap_or_default();

    assert!(
        extensions_mod.exists(),
        "expected zircon_runtime/src/extensions/mod.rs to own optional extension module registration"
    );

    for (legacy_lib, module_name) in [
        ("../zircon_physics/src/lib.rs", "PhysicsModule"),
        ("../zircon_sound/src/lib.rs", "SoundModule"),
        ("../zircon_texture/src/lib.rs", "TextureModule"),
        ("../zircon_net/src/lib.rs", "NetModule"),
        ("../zircon_navigation/src/lib.rs", "NavigationModule"),
        ("../zircon_particles/src/lib.rs", "ParticlesModule"),
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
