//! Exercise the private menu writer through public linked-session input events.

use std::sync::{Arc, Mutex};

use serde_json::json;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginSelection;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::dynamic_api::{create_linked_runtime_session, zircon_runtime_get_api_v8};
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimePluginRegistrationReport,
};
use zircon_runtime::scene::{NodeKind, SceneError, SystemStage};
use zircon_runtime_interface::{
    ZrRuntimeApiV8, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
};

const PLUGIN_TYPE: &str = "navigation.Component.MenuTestTag";

struct SessionGuard(ZrRuntimeSessionHandle, &'static ZrRuntimeApiV8);

impl Drop for SessionGuard {
    fn drop(&mut self) {
        // SAFETY: this guard owns the live linked session and no borrowed ABI output is retained.
        let status = unsafe { self.1.destroy_session.unwrap()(self.0) };
        if !std::thread::panicking() {
            assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
        }
    }
}

#[test]
fn runtime_owned_menu_click_writes_start_and_retry_with_a_plugin_descriptor() {
    let observed = Arc::new(Mutex::new(None));
    let mut extensions = RuntimeExtensionRegistry::default();
    extensions
        .register_module(ModuleDescriptor::new("navigation.runtime", "Menu test"))
        .unwrap();
    let owner = extensions
        .intern_plugin_module("navigation.runtime")
        .unwrap();
    extensions
        .register_component_for_owner(
            owner,
            ComponentTypeDescriptor::new(PLUGIN_TYPE, "navigation", "MenuTestTag"),
        )
        .unwrap();
    let output = observed.clone();
    extensions
        .register_runtime_scene_system(
            owner,
            "navigation.test_gameplay_menu",
            SystemStage::Update,
            move || {
                let output = output.clone();
                let mut menu_entity = None;
                move |context| {
                    let (plugin_installed, result) = context.level.with_world_mut(|world| {
                        let plugin_installed =
                            world.component_type_descriptor(PLUGIN_TYPE).is_some();
                        let result = (|| -> Result<Option<String>, SceneError> {
                            let entity = match menu_entity {
                                Some(entity) => entity,
                                None => {
                                    let entity = world.spawn_node(NodeKind::Empty)?;
                                    world.set_dynamic_component(
                                        entity,
                                        "gameplay.menu_state",
                                        json!({"state": "start"}),
                                    )?;
                                    menu_entity = Some(entity);
                                    entity
                                }
                            };
                            let action = world
                                .dynamic_component(entity, "gameplay.control_state")
                                .and_then(serde_json::Value::as_str)
                                .map(str::to_owned);
                            if action.as_deref() == Some("start_game") {
                                world.set_dynamic_component(
                                    entity,
                                    "gameplay.menu_state",
                                    json!({"state": "game_over"}),
                                )?;
                            }
                            Ok(action)
                        })();
                        (plugin_installed, result.map_err(|error| error.to_string()))
                    });
                    *output.lock().unwrap() = Some((plugin_installed, result));
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    let report = RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new("navigation", "Menu test")
            .with_runtime_module(PluginModuleManifest::runtime(
                "navigation.runtime",
                "test_navigation_runtime",
            )),
        project_selection: ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Navigation,
            true,
            false,
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        extensions,
        diagnostics: Vec::new(),
    };
    // UI-capable linked sessions require an importer provider even without project assets.
    // This fixture supplies only its catalog identity; no importer or editor plugin is enabled.
    let mut importer_extensions = RuntimeExtensionRegistry::default();
    importer_extensions
        .register_module(ModuleDescriptor::new(
            "ui_document_importer.runtime",
            "Unused headless importer fixture",
        ))
        .unwrap();
    let importer = RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new("ui_document_importer", "Importer fixture")
            .with_runtime_module(PluginModuleManifest::runtime(
                "ui_document_importer.runtime",
                "test_ui_document_importer_runtime",
            )),
        project_selection: ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::UiDocumentImporter,
            true,
            true,
        ),
        extensions: importer_extensions,
        diagnostics: Vec::new(),
    };
    // SAFETY: a null host selects the runtime's supported built-in host table.
    let api = unsafe { zircon_runtime_get_api_v8(std::ptr::null()) };
    assert!(!api.is_null());
    // SAFETY: the linked runtime returns its non-null, process-lifetime API table.
    let api = unsafe { &*api };
    let session = SessionGuard(
        create_linked_runtime_session(b"headless", None, vec![report, importer]).unwrap(),
        api,
    );
    let tick = || {
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        // SAFETY: the session is live and demand points to an exclusive, initialized output.
        let status = unsafe { api.tick_frame.unwrap()(session.0, &mut demand) };
        assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    };
    tick();
    assert_eq!(*observed.lock().unwrap(), Some((true, Ok(None))));

    let viewport = ZrRuntimeViewportHandle::new(1);
    for expected_action in ["start_game", "retry_game"] {
        for event in [
            ZrRuntimeEventV1::viewport_resized(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport,
                ZrRuntimeViewportSizeV1::new(640, 360),
            ),
            ZrRuntimeEventV1::pointer_moved(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, 320.0, 248.0),
            ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport,
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                320.0,
                248.0,
            ),
            ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport,
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
                320.0,
                248.0,
            ),
        ] {
            // SAFETY: the live session consumes each valid input event synchronously; payload is empty.
            let status = unsafe { api.handle_event.unwrap()(session.0, event) };
            assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
        }
        tick();
        assert_eq!(
            *observed.lock().unwrap(),
            Some((true, Ok(Some(expected_action.to_owned()))))
        );
    }
}
