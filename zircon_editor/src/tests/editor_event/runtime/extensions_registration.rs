use super::*;
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
    use crate::ui::host::EditorManager;
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
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
    assert!(
        enabled_packet
            .overlays
            .scene_gizmos
            .iter()
            .any(|gizmo| gizmo.owner == 404 && gizmo.kind == SceneGizmoKind::NavigationMesh)
    );

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
        Arc,
        atomic::{AtomicUsize, Ordering},
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
    assert!(
        runtime
            .runtime
            .descriptors()
            .iter()
            .all(|descriptor| descriptor.descriptor_id.0 != VIEW_ID)
    );
    assert!(
        runtime
            .runtime
            .commands()
            .lock()
            .command("view.test.scene.atomic_registration.open")
            .is_none()
    );
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
    assert!(
        runtime
            .runtime
            .descriptors()
            .iter()
            .all(|descriptor| descriptor.descriptor_id.0 != VIEW_ID)
    );
    assert!(
        runtime
            .runtime
            .commands()
            .lock()
            .command("view.test.overlay.atomic_registration.open")
            .is_none()
    );
}

#[test]
fn editor_runtime_contains_overlay_provider_extract_panic() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

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

#[test]
fn editor_runtime_folds_menu_capabilities_into_the_shared_command_descriptor() {
    use crate::core::commands::{CommandEvalCtx, WhenClause};
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_menu_capability_command_when");
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let capability = "editor.extension.weather_authoring";
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(EditorCommandDescriptor::operation(
            operation_path.clone(),
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new(
                "Tools/Weather/Refresh Cloud Layers",
                operation_path.clone(),
            )
            .with_required_capabilities([capability]),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("menu capability should fold into its extension-owned command");

    let descriptor = runtime
        .runtime
        .commands()
        .lock()
        .command(operation_path.as_str())
        .cloned()
        .expect("shared command descriptor");
    assert_eq!(
        descriptor.required_capabilities(),
        &[capability.to_string()]
    );
    assert_eq!(
        descriptor.effective_when(),
        WhenClause::Capability(capability.to_string())
    );
    assert!(!descriptor.is_enabled(&CommandEvalCtx::interactive()));
    assert!(descriptor.is_enabled(&CommandEvalCtx::interactive().with_capabilities([capability])));
}

#[test]
fn editor_runtime_consumes_plugin_command_descriptors_into_the_shared_registry() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_operation");
    let operation_path = EditorOperationPath::parse("weather.tools.reset_layout").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Weather Reset Layout")
                .with_menu_path("Tools/Weather/Reset Layout")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");
    {
        let shell = runtime.runtime.shell().lock();
        let stored = shell.contributions.snapshot();
        assert!(
            stored
                .commands(&crate::core::extension::CapabilitySet::default())
                .any(|command| command.id() == &operation_path)
        );
    }
    let record = runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

    assert_eq!(
        record.operation_id.as_deref(),
        Some("weather.tools.reset_layout")
    );
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("weather.tools.reset_layout")
    );
}

#[test]
fn explicit_plugin_operation_keeps_its_identity_without_synthetic_history() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_operation_stack_identity");
    let operation_path = EditorOperationPath::parse("zzz.tools.reset_layout").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Plugin Reset Layout")
                .with_menu_path("Tools/Zzz/Reset Layout")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("zzz.tools.reset_layout")
    );
}

#[test]
fn editor_runtime_projects_plugin_menu_operations_into_remote_callable_reflection() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_menu_operation");
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new("Tools/Weather/Refresh Cloud Layers", operation_path)
                .with_priority(10)
                .with_shortcut("Ctrl+Alt+R"),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");
    runtime.runtime.refresh_reflection();

    let menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/tools/weather.cloud_layer.refresh"),
        });
    assert!(matches!(
        menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Refresh Cloud Layers"
                && node.actions["workbench.menu.item.click"].binding_symbol == "EditorOperation"
                && node.actions["workbench.menu.item.click"].callable_from_remote
                && node.properties["operation_path"].reflected_value
                    == json!("weather.cloud_layer.refresh")
                && node.properties["shortcut"].reflected_value == json!("Ctrl+Alt+R")
    ));

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/tools/weather.cloud_layer.refresh"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result)
            if result.error.is_none()
                && result.binding
                    .as_ref()
                    .and_then(|binding| binding.action.as_ref())
                    .map(|call| call.symbol.as_str())
                    == Some("EditorOperation")
    ));
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("weather.cloud_layer.refresh")
    );
}

#[test]
fn editor_operation_ui_binding_arguments_are_preserved_in_journal() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_menu_operation_arguments");
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path,
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: "External.OperationAudit".to_string(),
            display_name: "Operation Audit".to_string(),
        },
    );
    runtime.runtime.refresh_reflection();

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/tools/weather.cloud_layer.refresh"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: vec![
                UiBindingValue::String("storm".to_string()),
                UiBindingValue::Unsigned(7),
                UiBindingValue::Bool(true),
            ],
        });

    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result)
            if result.error.is_none()
                && result.binding
                    .as_ref()
                    .and_then(|binding| binding.action.as_ref())
                    .map(|call| call.arguments.len())
                    == Some(4)
    ));
    let journal = runtime.runtime.journal();
    let record = &journal.records()[0];
    assert_eq!(
        record.operation_id.as_deref(),
        Some("weather.cloud_layer.refresh")
    );
    assert_eq!(
        record.operation_arguments.as_ref(),
        Some(&json!(["storm", 7, true]))
    );
    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries {
            listener_id: "External.OperationAudit".to_string(),
        },
    );
    assert_eq!(
        deliveries.value["deliveries"][0]["operation_arguments"],
        json!(["storm", 7, true])
    );
}

#[test]
fn editor_runtime_registers_plugin_views_as_activity_descriptors() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_view_descriptor");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");
    runtime.runtime.refresh_reflection();

    let descriptor = runtime
        .runtime
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "weather.cloud_layers")
        .expect("plugin view descriptor registered");
    assert_eq!(descriptor.default_title, "Cloud Layers");
    assert_eq!(descriptor.icon_key, "weather.cloud_layers");
    assert!(
        runtime
            .runtime
            .activity_view_descriptor("weather.cloud_layers")
            .is_some()
    );
}

#[test]
fn editor_runtime_projects_plugin_views_into_view_menu_operations() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_view_menu_operation");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");
    runtime.runtime.refresh_reflection();

    let menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/view/view.weather.cloud_layers.open"),
        });
    assert!(matches!(
        menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Cloud Layers"
                && node.properties["operation_path"].reflected_value
                    == json!("view.weather.cloud_layers.open")
                && node.actions["workbench.menu.item.click"].binding_symbol == "EditorOperation"
                && node.actions["workbench.menu.item.click"].callable_from_remote
    ));

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/view/view.weather.cloud_layers.open"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result) if result.error.is_none()
    ));
    assert!(
        runtime
            .runtime
            .current_view_instances()
            .iter()
            .any(|instance| instance.descriptor_id.0 == "weather.cloud_layers")
    );
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("view.weather.cloud_layers.open")
    );
}

#[test]
fn editor_runtime_consumes_plugin_registration_reports_with_capability_gate() {
    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorMenuItemDescriptor, ViewDescriptor,
    };
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    };
    use crate::core::extension::InspectorCustomizationDescriptor;
    use crate::core::plugin::EditorPluginRegistrationReport;
    use crate::ui::host::EditorManager;
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
    use zircon_runtime::{plugin::PluginModuleManifest, plugin::PluginPackageManifest};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_plugin_registration_gate",
        &[],
    );
    let capability = "editor.extension.weather_authoring".to_string();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "plugin.weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    let operation_path = EditorOperationPath::parse("plugin.weather.cloud_layer.refresh").unwrap();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path.clone(),
        ))
        .unwrap();
    let component_type = "plugin.weather.CloudLayer";
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
                component_type,
                "plugins://weather/editor/cloud_layer.inspector.zui",
                "plugin.weather.CloudLayerInspectorController",
            )
            .with_id("plugin.weather.cloud_layer")
            .with_binding("plugin.weather.cloud_layer.refresh"),
        )
        .unwrap();
    let selected_node = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("default selection")
        .id;
    {
        let shell = runtime.runtime.shell().lock();
        shell.state.world.with_world_mut(|scene| {
            scene
                .register_component_type(
                    ComponentTypeDescriptor::new(component_type, "weather", "Cloud Layer")
                        .with_property("coverage", "scalar", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected_node, component_type, json!({ "coverage": 0.75 }))
                .unwrap();
        });
    }

    runtime
        .runtime
        .register_editor_plugin_registration(EditorPluginRegistrationReport {
            package_manifest: PluginPackageManifest::new("weather", "Weather").with_editor_module(
                PluginModuleManifest::editor("weather.editor", "zircon_plugin_weather_editor")
                    .with_capabilities([capability.clone()]),
            ),
            capabilities: vec![capability.clone()],
            extensions: extension,
            lifecycle: crate::core::plugin::sdk::lifecycle::EditorPluginLifecycleReport::default(),
            successful_lifecycle_stages: Vec::new(),
            failed_lifecycle_stages: Vec::new(),
            runtime_event_consumers:
                crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry::default(),
            diagnostics: Vec::new(),
        })
        .expect("register editor plugin report");
    runtime.runtime.refresh_reflection();

    let disabled_component = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("plugin component snapshot while disabled");
    assert!(!disabled_component.customization_available);
    assert_eq!(disabled_component.customization_ui_document, None);

    assert!(
        runtime
            .runtime
            .descriptors()
            .iter()
            .all(|descriptor| descriptor.descriptor_id.0 != "plugin.weather.cloud_layers")
    );
    let disabled_menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new(
                "editor/workbench/menu/view/view.plugin.weather.cloud_layers.open",
            ),
        });
    assert!(matches!(disabled_menu, UiControlResponse::Node(None)));
    let disabled_operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    assert!(
        !disabled_operations
            .value
            .as_ref()
            .and_then(|value| value.get("operations"))
            .and_then(serde_json::Value::as_array)
            .expect("operations array")
            .iter()
            .any(|operation| operation
                .get("operation_id")
                .and_then(serde_json::Value::as_str)
                == Some("plugin.weather.cloud_layer.refresh"))
    );
    let disabled_invoke = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path.clone(),
        )),
    );
    assert_eq!(
        disabled_invoke.error.as_deref(),
        Some(
            "editor command plugin.weather.cloud_layer.refresh requires disabled capabilities: editor.extension.weather_authoring"
        )
    );

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .set_editor_capabilities_enabled(&[capability.clone()], true)
        .unwrap();
    runtime.runtime.refresh_reflection();

    let enabled_component = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("plugin component snapshot while enabled");
    assert!(enabled_component.customization_available);
    assert_eq!(
        enabled_component.customization_ui_document.as_deref(),
        Some("plugins://weather/editor/cloud_layer.inspector.zui")
    );
    assert_eq!(
        enabled_component.customization_controller.as_deref(),
        Some("plugin.weather.CloudLayerInspectorController")
    );

    let descriptor = runtime
        .runtime
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "plugin.weather.cloud_layers")
        .expect("enabled plugin view descriptor registered");
    assert_eq!(
        descriptor.required_capabilities,
        vec!["editor.extension.weather_authoring"]
    );
    let enabled_menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new(
                "editor/workbench/menu/view/view.plugin.weather.cloud_layers.open",
            ),
        });
    assert!(matches!(
        enabled_menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Cloud Layers"
                && node.properties["operation_path"].reflected_value
                    == json!("view.plugin.weather.cloud_layers.open")
    ));
    let enabled_operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    let enabled_operations = enabled_operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array");
    let weather_operation = enabled_operations
        .iter()
        .find(|operation| {
            operation
                .get("operation_id")
                .and_then(serde_json::Value::as_str)
                == Some("plugin.weather.cloud_layer.refresh")
        })
        .expect("weather operation is discoverable when capability is enabled");
    assert_eq!(
        weather_operation.get("required_capabilities"),
        Some(&json!(["editor.extension.weather_authoring"]))
    );
    assert!(enabled_operations.iter().any(|operation| {
        operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some("plugin.weather.cloud_layer.refresh")
    }));
    let enabled_invoke = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path,
        )),
    );
    assert!(enabled_invoke.error.is_none());

    manager
        .set_editor_capabilities_enabled(&[capability], false)
        .unwrap();
    runtime.runtime.refresh_reflection();
    let disabled_again_component = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("plugin component snapshot after capability revocation");
    assert!(!disabled_again_component.customization_available);
    assert_eq!(disabled_again_component.customization_ui_document, None);
}

#[test]
fn editor_runtime_snapshots_enabled_plugin_templates_by_owner_and_capability() {
    use std::{collections::BTreeMap, sync::Arc};

    use crate::core::asset::AssetTypeRegistry;
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorUiTemplateDescriptor};
    use crate::core::plugin::EditorPluginRegistrationReport;
    use crate::ui::host::EditorManager;
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use zircon_runtime::{plugin::PluginModuleManifest, plugin::PluginPackageManifest};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_plugin_template_snapshot",
        &[],
    );
    let capability = "editor.extension.weather_authoring".to_string();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.weather.cloud_layer.inspector",
            "plugins://weather/editor/cloud_layer.inspector.zui",
        ))
        .expect("template descriptor should be accepted");

    runtime
        .runtime
        .register_editor_plugin_registration(EditorPluginRegistrationReport {
            package_manifest: PluginPackageManifest::new("weather", "Weather").with_editor_module(
                PluginModuleManifest::editor("weather.editor", "zircon_plugin_weather_editor")
                    .with_capabilities([capability.clone()]),
            ),
            capabilities: vec![capability.clone()],
            extensions: extension,
            lifecycle: crate::core::plugin::sdk::lifecycle::EditorPluginLifecycleReport::default(),
            successful_lifecycle_stages: Vec::new(),
            failed_lifecycle_stages: Vec::new(),
            runtime_event_consumers:
                crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry::default(),
            diagnostics: Vec::new(),
        })
        .expect("register plugin template descriptor");

    let (registered_generation, disabled_capabilities, disabled_templates) =
        runtime.runtime.enabled_plugin_template_descriptors();
    let (disabled_revision, disabled_revision_capabilities) =
        runtime.runtime.plugin_template_revision();
    assert!(registered_generation > 0);
    assert_eq!(disabled_revision, registered_generation);
    assert!(!disabled_capabilities.contains(&capability));
    assert_eq!(disabled_revision_capabilities, disabled_capabilities);
    assert!(!disabled_templates.contains_key("weather"));

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager should be available");
    manager
        .set_editor_capabilities_enabled(&[capability.clone()], true)
        .expect("enable plugin template capability");

    let (enabled_generation, enabled_capabilities, enabled_templates) =
        runtime.runtime.enabled_plugin_template_descriptors();
    let (enabled_revision, enabled_revision_capabilities) =
        runtime.runtime.plugin_template_revision();
    assert_eq!(enabled_generation, registered_generation);
    assert_eq!(enabled_revision, registered_generation);
    assert!(enabled_capabilities.contains(&capability));
    assert_eq!(enabled_revision_capabilities, enabled_capabilities);
    assert_eq!(
        enabled_templates
            .get("weather")
            .expect("enabled plugin owner should expose templates")
            .iter()
            .map(|descriptor| descriptor.id())
            .collect::<Vec<_>>(),
        vec!["plugin.weather.cloud_layer.inspector"]
    );

    let unknown_owner_error = runtime
        .runtime
        .replace_editor_plugin_ui_template_contributions(
            "unknown.weather",
            std::iter::empty::<EditorUiTemplateDescriptor>(),
            BTreeMap::new(),
        )
        .expect_err("template replacement must not register an unknown extension owner");
    assert!(matches!(
        unknown_owner_error,
        crate::core::editor_extension::EditorExtensionRegistryError::UnknownExtensionOwner {
            ref owner_id
        } if owner_id == "unknown.weather"
    ));
    assert_eq!(
        runtime.runtime.plugin_template_revision().0,
        enabled_generation,
        "rejected replacement must not advance the template generation"
    );

    {
        let mut shell = runtime.runtime.shell().lock();
        shell
            .asset_type_registry_cache
            .store(Vec::new(), Arc::new(AssetTypeRegistry::default()));
    }

    runtime
        .runtime
        .replace_editor_plugin_ui_template_contributions(
            "weather",
            [EditorUiTemplateDescriptor::new(
                "plugin.weather.cloud_layer.inspector",
                "plugins://weather/editor/cloud_layer.inspector.reloaded.zui",
            )],
            BTreeMap::new(),
        )
        .expect("registered plugin templates should support an atomic replacement");

    let asset_cache_counts = {
        let mut shell = runtime.runtime.shell().lock();
        assert!(
            shell.asset_type_registry_cache.get(&[]).is_some(),
            "template replacement must not invalidate the unrelated asset-type cache"
        );
        shell.asset_type_registry_cache.counts()
    };
    assert_eq!(asset_cache_counts, (1, 1));

    let (reloaded_generation, _, reloaded_templates) =
        runtime.runtime.enabled_plugin_template_descriptors();
    assert!(reloaded_generation > enabled_generation);
    assert_eq!(
        reloaded_templates
            .get("weather")
            .and_then(|templates| templates.first())
            .map(|descriptor| descriptor.ui_document()),
        Some("plugins://weather/editor/cloud_layer.inspector.reloaded.zui")
    );

    manager
        .set_editor_capabilities_enabled(&[capability.clone()], false)
        .expect("disable plugin template capability after reload");
    assert!(
        !runtime
            .runtime
            .enabled_plugin_template_descriptors()
            .2
            .contains_key("weather")
    );

    manager
        .set_editor_capabilities_enabled(&[capability], true)
        .expect("re-enable plugin template capability after reload");
    assert_eq!(
        runtime
            .runtime
            .enabled_plugin_template_descriptors()
            .2
            .get("weather")
            .and_then(|templates| templates.first())
            .map(|descriptor| descriptor.ui_document()),
        Some("plugins://weather/editor/cloud_layer.inspector.reloaded.zui")
    );
}

#[test]
fn editor_runtime_exposes_plugin_inspector_customization_surface_for_inspector_lookup() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorUiTemplateDescriptor};
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::extension::{InspectorCustomization, InspectorCustomizationDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_inspector_customization");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(EditorCommandDescriptor::operation(
            EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap(),
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "weather.cloud_layer.inspector",
            "asset://weather/editor/cloud_layer.inspector.zui",
        ))
        .unwrap();
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
                "weather.Component.CloudLayer",
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_template_id("weather.cloud_layer.inspector")
            .with_data_root("inspector.plugin_components.weather.Component.CloudLayer")
            .with_binding("weather.cloud_layer.refresh"),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");

    let customization = runtime
        .runtime
        .inspector_customization("weather.Component.CloudLayer")
        .expect("inspector customization registered");
    let surface = customization.surface().expect("customization UI surface");
    assert_eq!(
        surface.ui_document(),
        "asset://weather/editor/cloud_layer.inspector.zui"
    );
    assert_eq!(
        surface.controller(),
        "weather.editor.CloudLayerInspectorController"
    );
    assert_eq!(surface.template_id(), Some("weather.cloud_layer.inspector"));
    assert_eq!(
        surface.data_root(),
        Some("inspector.plugin_components.weather.Component.CloudLayer")
    );
    assert_eq!(surface.bindings(), ["weather.cloud_layer.refresh"]);

    let template = runtime
        .runtime
        .ui_template_descriptor("weather.cloud_layer.inspector")
        .expect("ui template registered");
    assert_eq!(
        template.ui_document(),
        "asset://weather/editor/cloud_layer.inspector.zui"
    );
}

#[test]
fn editor_snapshot_resolves_enabled_inspector_customization_for_selected_dynamic_component() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::extension::InspectorCustomizationDescriptor;
    use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_inspector_customization_snapshot");
    let component_type = "weather.Component.CloudLayer";
    let selected_node = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("default selection")
        .id;

    {
        let shell = runtime.runtime.shell().lock();
        shell.state.world.with_world_mut(|scene| {
            scene
                .register_component_type(
                    ComponentTypeDescriptor::new(component_type, "weather", "Cloud Layer")
                        .with_property("coverage", "scalar", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected_node, component_type, json!({ "coverage": 0.75 }))
                .unwrap();
        });
    }

    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(EditorCommandDescriptor::operation(
            operation_path,
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
                component_type,
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_template_id("weather.cloud_layer.inspector")
            .with_data_root("inspector.plugin_components.weather.Component.CloudLayer")
            .with_binding("weather.cloud_layer.refresh"),
        )
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");

    let snapshot = runtime.runtime.editor_snapshot();
    let component = snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");

    assert!(component.customization_available);
    assert_eq!(
        component.customization_ui_document.as_deref(),
        Some("asset://weather/editor/cloud_layer.inspector.zui")
    );
    assert_eq!(
        component.customization_controller.as_deref(),
        Some("weather.editor.CloudLayerInspectorController")
    );
    assert_eq!(
        component.customization_template_id.as_deref(),
        Some("weather.cloud_layer.inspector")
    );
    assert_eq!(
        component.customization_bindings,
        ["weather.cloud_layer.refresh"]
    );
    assert_eq!(component.diagnostic, None);
    assert_eq!(
        component.properties[0].field_id,
        "weather.Component.CloudLayer.coverage"
    );
}

#[test]
fn editor_snapshot_resolves_plugin_field_editors_from_active_contributions() {
    use crate::core::extension::{
        ContributionBatch, FieldEditorDefinition, FieldEditorInstance, FieldEditorKind,
    };
    use crate::ui::host::EditorManager;
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_plugin_field_editor_snapshot",
        &[],
    );
    let capability = "editor.extension.weather_authoring".to_string();
    let component_type = "plugin.weather.CloudLayer";
    let selected_node = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("default selection")
        .id;
    {
        let shell = runtime.runtime.shell().lock();
        shell.state.world.with_world_mut(|scene| {
            scene
                .register_component_type(
                    ComponentTypeDescriptor::new(component_type, "weather", "Cloud Layer")
                        .with_property("coverage", "plugin.weather.CloudCoverage", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected_node, component_type, json!({ "coverage": 0.75 }))
                .unwrap();
        });
    }

    let mut batch = ContributionBatch::default().with_required_capabilities([capability.clone()]);
    batch
        .register_field_editor(FieldEditorDefinition::new(
            "plugin.weather.CloudCoverage",
            |_| FieldEditorInstance::new(FieldEditorKind::Color),
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(batch)
        .expect("register field editor contribution");

    let disabled_snapshot = runtime.runtime.editor_snapshot();
    let disabled = disabled_snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");
    assert_eq!(
        disabled.properties[0].field_editor.kind(),
        FieldEditorKind::Auto
    );

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .set_editor_capabilities_enabled(&[capability], true)
        .expect("enable field editor capability");
    assert_eq!(
        disabled.properties[0].field_editor.kind(),
        FieldEditorKind::Auto,
        "published editor snapshots retain resolved field metadata after capability changes"
    );
    let enabled_snapshot = runtime.runtime.editor_snapshot();
    let enabled = enabled_snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");
    assert_eq!(
        enabled.properties[0].field_editor.kind(),
        FieldEditorKind::Color
    );
    manager
        .set_editor_capabilities_enabled(&[capability], false)
        .expect("disable field editor capability");
    assert_eq!(
        enabled.properties[0].field_editor.kind(),
        FieldEditorKind::Color,
        "published snapshots must retain resolved field metadata after capability removal"
    );
    let revoked_snapshot = runtime.runtime.editor_snapshot();
    let revoked = revoked_snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");
    assert_eq!(
        revoked.properties[0].field_editor.kind(),
        FieldEditorKind::Auto,
        "new snapshots must fall back when the field editor contribution is no longer active"
    );
}

#[test]
fn editor_snapshot_hides_inspector_customization_when_extension_capability_is_disabled() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::extension::InspectorCustomizationDescriptor;
    use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_inspector_customization_disabled");
    let component_type = "weather.Component.CloudLayer";
    let selected_node = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("default selection")
        .id;

    {
        let shell = runtime.runtime.shell().lock();
        shell.state.world.with_world_mut(|scene| {
            scene
                .register_component_type(
                    ComponentTypeDescriptor::new(component_type, "weather", "Cloud Layer")
                        .with_property("coverage", "scalar", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected_node, component_type, json!({ "coverage": 0.75 }))
                .unwrap();
        });
    }

    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(EditorCommandDescriptor::operation(
            operation_path,
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
                component_type,
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_binding("weather.cloud_layer.refresh"),
        )
        .unwrap();
    runtime
        .runtime
        .register_editor_extension_with_required_capabilities(
            extension.into_contribution_batch().unwrap(),
            vec!["editor.extension.weather_authoring".to_string()],
        )
        .expect("register disabled extension");

    let snapshot = runtime.runtime.editor_snapshot();
    let component = snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");

    assert!(!component.customization_available);
    assert_eq!(component.customization_ui_document, None);
    assert_eq!(component.customization_controller, None);
    assert!(component.diagnostic.as_deref().is_some_and(|diagnostic| {
        diagnostic.contains("enabled editor extension registers a customization")
    }));
}
