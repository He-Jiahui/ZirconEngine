use super::super::*;
use crate::core::commands::EditorCommandDescriptor;

#[test]
fn viewport_overlay_provider_registration_routes_toggle_and_capability_lifecycle_to_packets() {
    use std::sync::Arc;

    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_event::{EditorEvent, EditorViewportEvent};
    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
        ViewportOverlayProvider, ViewportOverlayProviderContext,
        ViewportOverlayProviderRegistration,
    };
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use crate::ui::host::EditorManager;
    use zircon_runtime::core::framework::render::{SceneGizmoKind, SceneGizmoOverlayExtract};

    const CAPABILITY: &str = "editor.extension.weather_overlay";
    const MODE_ID: &str = "weather.viewport.overlay";
    const PROVIDER_ID: &str = "weather.viewport.overlay.provider";

    struct WeatherOverlayProvider;

    impl ViewportOverlayProvider for WeatherOverlayProvider {
        fn extract(
            &self,
            _context: &ViewportOverlayProviderContext<'_>,
        ) -> Vec<SceneGizmoOverlayExtract> {
            vec![SceneGizmoOverlayExtract {
                owner: 404,
                kind: SceneGizmoKind::NavigationMesh,
                selected: false,
                lines: Vec::new(),
                wire_shapes: Vec::new(),
                icons: Vec::new(),
                pick_shapes: Vec::new(),
            }]
        }
    }

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_viewport_overlay_provider",
        &[],
    );
    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager is available");
    manager
        .set_editor_capabilities_enabled(&[CAPABILITY.to_string()], true)
        .expect("overlay capability enables before registration");

    let toggle = EditorOperationPath::parse("weather.viewport.overlay.toggle").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_scene_mode(crate::tests::support::pass_through_scene_mode_registration(
            SceneModeDescriptor::new(MODE_ID, "Weather Overlay", "weather.debug", toggle.clone())
                .with_overlay_provider_id(PROVIDER_ID)
                .with_required_capabilities([CAPABILITY]),
        ))
        .expect("scene mode accepts its provider id");
    extension
        .register_viewport_overlay_provider(
            ViewportOverlayProviderRegistration::new(
                PROVIDER_ID,
                || -> Arc<dyn ViewportOverlayProvider> { Arc::new(WeatherOverlayProvider) },
            )
            .with_required_capabilities([CAPABILITY]),
        )
        .expect("provider registration is unique");
    extension
        .register_command(
            EditorCommandDescriptor::operation(toggle.clone(), "Toggle Weather Overlay")
                .with_event(EditorEvent::Viewport(
                    EditorViewportEvent::ToggleOverlayProvider {
                        provider_id: PROVIDER_ID.to_string(),
                    },
                ))
                .with_required_capabilities([CAPABILITY]),
        )
        .expect("toggle operation is an ordinary viewport event");
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new("View/Debug Overlays/Weather", toggle.clone())
                .with_required_capabilities([CAPABILITY]),
        )
        .expect("toggle menu is capability-gated");

    runtime
        .runtime
        .register_editor_extension_with_required_capabilities(
            extension.into_contribution_batch().unwrap(),
            vec![CAPABILITY.to_string()],
        )
        .expect("host installs overlay provider factory");
    runtime.runtime.refresh_reflection();

    let disabled_packet = runtime
        .runtime
        .shell()
        .lock()
        .state
        .render_snapshot()
        .expect("default fixture renders a scene");
    assert!(disabled_packet.overlays.scene_gizmos.is_empty());

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(toggle),
        )
        .expect("capability-enabled menu operation toggles the provider");
    let enabled_packet = runtime
        .runtime
        .shell()
        .lock()
        .state
        .render_snapshot()
        .expect("default fixture renders a scene");
    assert!(enabled_packet
        .overlays
        .scene_gizmos
        .iter()
        .any(|gizmo| gizmo.owner == 404 && gizmo.kind == SceneGizmoKind::NavigationMesh));

    manager
        .set_editor_capabilities_enabled(&[CAPABILITY.to_string()], false)
        .expect("overlay capability disables");
    runtime.runtime.refresh_reflection();
    let cleared_packet = runtime
        .runtime
        .shell()
        .lock()
        .state
        .render_snapshot()
        .expect("default fixture renders a scene");
    assert!(cleared_packet.overlays.scene_gizmos.is_empty());

    manager
        .set_editor_capabilities_enabled(&[CAPABILITY.to_string()], true)
        .expect("overlay capability re-enables");
    runtime.runtime.refresh_reflection();
    let restarted_packet = runtime
        .runtime
        .shell()
        .lock()
        .state
        .render_snapshot()
        .expect("default fixture renders a scene");
    assert!(
        restarted_packet.overlays.scene_gizmos.is_empty(),
        "capability restart must not resurrect a previously enabled provider"
    );

    let mut duplicate = EditorExtensionRegistry::default();
    duplicate
        .register_viewport_overlay_provider(ViewportOverlayProviderRegistration::new(
            PROVIDER_ID,
            || -> Arc<dyn ViewportOverlayProvider> { Arc::new(WeatherOverlayProvider) },
        ))
        .expect("a distinct extension can declare its provider before host validation");
    let error = runtime
        .runtime
        .register_editor_extension(duplicate.into_contribution_batch().unwrap())
        .expect_err("the host must preflight duplicate provider ids before installation");
    assert!(matches!(
        error,
        EditorExtensionRegistryError::ViewportOverlayProvider(message)
            if message == format!("duplicate viewport overlay provider `{PROVIDER_ID}`")
    ));
}

#[test]
fn editor_runtime_prepares_each_scene_mode_factory_once_before_host_commit() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_message::SceneModeId;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::scene::modes::SceneModeRegistration;

    const MODE_ID: &str = "test.scene.single-host-factory-call";

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_scene_mode_single_factory_call",
        &[],
    );
    let factory_calls = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&factory_calls);
    let descriptor = SceneModeDescriptor::new(
        MODE_ID,
        "Single Host Factory Call",
        "test.scene",
        EditorOperationPath::parse("test.scene.single_host_factory_call.activate").unwrap(),
    );
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_scene_mode(SceneModeRegistration::new(descriptor, move || {
            let call = calls.fetch_add(1, Ordering::SeqCst);
            crate::tests::support::pass_through_scene_mode(SceneModeId::new(if call == 0 {
                MODE_ID
            } else {
                "test.scene.unstable-host-factory"
            }))
        }))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("prepared scene mode registry commits without a second factory invocation");

    assert_eq!(factory_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn editor_runtime_rejects_invalid_scene_mode_factory_without_partial_host_commit() {
    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorExtensionRegistryError, ViewDescriptor,
    };
    use crate::core::editor_message::SceneModeId;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::scene::modes::SceneModeRegistration;

    const VIEW_ID: &str = "test.scene.atomic_registration";
    const MODE_ID: &str = "test.scene.atomic_registration.mode";

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_scene_mode_atomic_registration",
        &[],
    );
    let initial_extension_count = runtime.runtime.shell().lock().contributions.len();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(VIEW_ID, "Atomic Registration", "Tests"))
        .unwrap();
    extension
        .register_scene_mode(SceneModeRegistration::new(
            SceneModeDescriptor::new(
                MODE_ID,
                "Atomic Registration Mode",
                VIEW_ID,
                EditorOperationPath::parse("test.scene.atomic_registration.activate").unwrap(),
            ),
            || {
                crate::tests::support::pass_through_scene_mode(SceneModeId::new(
                    "test.scene.atomic_registration.wrong",
                ))
            },
        ))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect_err("invalid scene mode factory must reject the whole extension batch");

    assert!(matches!(error, EditorExtensionRegistryError::SceneMode(_)));
    assert_eq!(
        runtime.runtime.shell().lock().contributions.len(),
        initial_extension_count
    );
    assert!(runtime
        .runtime
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != VIEW_ID));
    assert!(runtime
        .runtime
        .commands()
        .lock()
        .command("view.test.scene.atomic_registration.open")
        .is_none());
}

#[test]
fn rejected_plugin_extension_does_not_publish_runtime_event_consumers() {
    use std::convert::Infallible;
    use std::sync::{Arc, Mutex};

    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_message::SceneModeId;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::plugin::EditorPluginRegistrationReport;
    use crate::core::runtime_event_consumer::{
        EditorRuntimeEventConsumerManifest, EditorRuntimeEventConsumerRegistration,
        EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerState,
    };
    use crate::scene::modes::SceneModeRegistration;
    use zircon_runtime::plugin::PluginPackageManifest;

    struct NoopConsumer;

    impl EditorRuntimeEventConsumerState for NoopConsumer {
        type Payload = ();
        type Error = Infallible;

        fn begin_session(&mut self, _play_session_id: u64) {}

        fn consume(
            &mut self,
            _play_session_id: u64,
            _sequence: u64,
            _payload: Self::Payload,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn end_session(&mut self, _play_session_id: u64) {}
    }

    fn consumer_registry() -> EditorRuntimeEventConsumerRegistry {
        let mut registry = EditorRuntimeEventConsumerRegistry::default();
        registry
            .register(EditorRuntimeEventConsumerRegistration::typed(
                EditorRuntimeEventConsumerManifest::new(
                    "test.scene.atomic.consumer",
                    "test.scene.atomic.event",
                    "test.scene.atomic.event.v1",
                ),
                Arc::new(Mutex::new(NoopConsumer)),
            ))
            .unwrap();
        registry
    }

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_plugin_registration_atomicity",
        &[],
    );
    let descriptor = SceneModeDescriptor::new(
        "plugin.test.scene.atomic.plugin.mode",
        "Atomic Plugin Mode",
        "test.scene",
        EditorOperationPath::parse("plugin.test.scene.atomic.plugin.mode.activate").unwrap(),
    );
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_scene_mode(SceneModeRegistration::new(descriptor, || {
            crate::tests::support::pass_through_scene_mode(SceneModeId::new(
                "plugin.test.scene.atomic.plugin.mode.wrong",
            ))
        }))
        .unwrap();

    runtime
        .runtime
        .register_editor_plugin_registration(EditorPluginRegistrationReport {
            package_manifest: PluginPackageManifest::new(
                "test.scene.atomic.plugin",
                "Atomic Plugin",
            ),
            capabilities: Vec::new(),
            extensions: extension,
            lifecycle: Default::default(),
            successful_lifecycle_stages: Vec::new(),
            failed_lifecycle_stages: Vec::new(),
            runtime_event_consumers: consumer_registry(),
            diagnostics: Vec::new(),
        })
        .expect_err("invalid extension must reject the complete plugin registration");

    runtime
        .runtime
        .register_runtime_event_consumers(consumer_registry())
        .expect("rejected plugin registration must not retain its consumer");
}

#[test]
fn editor_runtime_contains_overlay_provider_factory_panic_without_partial_host_commit() {
    use std::sync::Arc;

    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorExtensionRegistryError, ViewDescriptor,
        ViewportOverlayProvider, ViewportOverlayProviderRegistration,
    };

    const VIEW_ID: &str = "test.overlay.atomic_registration";

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_overlay_provider_atomic_registration",
        &[],
    );
    let initial_extension_count = runtime.runtime.shell().lock().contributions.len();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(VIEW_ID, "Atomic Overlay", "Tests"))
        .unwrap();
    extension
        .register_viewport_overlay_provider(ViewportOverlayProviderRegistration::new(
            "test.overlay.atomic_registration.provider",
            || -> Arc<dyn ViewportOverlayProvider> { panic!("provider factory panic") },
        ))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect_err("provider panic must reject the whole extension batch");

    assert!(matches!(
        error,
        EditorExtensionRegistryError::ViewportOverlayProvider(message)
            if message.contains("provider factory panic")
    ));
    assert_eq!(
        runtime.runtime.shell().lock().contributions.len(),
        initial_extension_count
    );
    assert!(runtime
        .runtime
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != VIEW_ID));
    assert!(runtime
        .runtime
        .commands()
        .lock()
        .command("view.test.overlay.atomic_registration.open")
        .is_none());
}

#[test]
fn editor_runtime_contains_overlay_provider_extract_panic() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use crate::core::editor_extension::{
        EditorExtensionRegistry, ViewportOverlayProvider, ViewportOverlayProviderContext,
        ViewportOverlayProviderRegistration,
    };
    use crate::ui::binding::ViewportCommand;
    use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;

    const PROVIDER_ID: &str = "test.overlay.panicking_extract";

    struct PanickingOverlayProvider {
        calls: Arc<AtomicUsize>,
    }

    impl ViewportOverlayProvider for PanickingOverlayProvider {
        fn extract(
            &self,
            _context: &ViewportOverlayProviderContext<'_>,
        ) -> Vec<SceneGizmoOverlayExtract> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            panic!("provider extract panic");
        }
    }

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_overlay_provider_extract_containment",
        &[],
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let mut extension = EditorExtensionRegistry::default();
    let provider_calls = Arc::clone(&calls);
    extension
        .register_viewport_overlay_provider(ViewportOverlayProviderRegistration::new(
            PROVIDER_ID,
            move || -> Arc<dyn ViewportOverlayProvider> {
                Arc::new(PanickingOverlayProvider {
                    calls: Arc::clone(&provider_calls),
                })
            },
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .unwrap();
    {
        let mut shell = runtime.runtime.shell().lock();
        shell
            .state
            .apply_viewport_command(&ViewportCommand::ToggleOverlayProvider {
                provider_id: PROVIDER_ID.to_string(),
            })
            .unwrap();
    }

    let packet = runtime
        .runtime
        .shell()
        .lock()
        .state
        .render_snapshot()
        .expect("panicking provider is isolated from render extraction");

    assert!(packet.overlays.scene_gizmos.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    {
        let mut shell = runtime.runtime.shell().lock();
        let error = shell
            .state
            .apply_viewport_command(&ViewportCommand::ToggleOverlayProvider {
                provider_id: PROVIDER_ID.to_string(),
            })
            .expect_err("a faulted provider must remain quarantined");
        assert!(error.contains("quarantined after callback failure"));
        assert!(error.contains("provider extract panic"));
        shell
            .state
            .apply_viewport_command(&ViewportCommand::Resized {
                width: 1280,
                height: 720,
            })
            .unwrap();
    }
    let packet = runtime
        .runtime
        .shell()
        .lock()
        .state
        .render_snapshot()
        .expect("a quarantined provider cannot poison later extraction");
    assert!(packet.overlays.scene_gizmos.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn editor_runtime_rejects_scene_mode_with_an_unregistered_overlay_provider() {
    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorExtensionRegistryError};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_missing_viewport_overlay_provider",
        &[],
    );
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_scene_mode(crate::tests::support::pass_through_scene_mode_registration(
            SceneModeDescriptor::new(
                "missing.viewport.overlay",
                "Missing Overlay",
                "weather.debug",
                EditorOperationPath::parse("missing.viewport.overlay.toggle").unwrap(),
            )
            .with_overlay_provider_id("missing.viewport.overlay.provider"),
        ))
        .expect("the descriptor accepts its provider id before host validation");

    let error = runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect_err("the host must reject a descriptor without its registered provider");
    assert!(matches!(
        error,
        EditorExtensionRegistryError::MissingViewportOverlayProvider { provider_id }
            if provider_id == "missing.viewport.overlay.provider"
    ));
}
