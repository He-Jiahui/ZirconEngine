use super::super::*;

use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use crate::core::asset::{AssetCreationTemplateDescriptor, AssetTypeContribution, AssetTypeId};
use crate::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};
use crate::core::editing::operation::{
    EditOperationTarget, OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
    OperationCommandFactoryRegistration,
};
use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_event::{EditorEvent, EditorViewportEvent};
use crate::core::editor_extension::{
    EditorExtensionRegistry, EditorMenuItemDescriptor, ViewDescriptor, ViewportOverlayProvider,
    ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
};
use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};
use crate::core::extension::CapabilitySet;
use crate::core::plugin::EditorPluginRegistrationReport;
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerRegistration, EditorRuntimeEventConsumerRegistry,
    EditorRuntimeEventConsumerState,
};
use crate::core::tools::{
    AcquireOutcome, ToolDefinitionId, ToolOwnerGeneration, ToolResourceChannelPolicy,
    ToolResourceKey, ToolResourceKindDeclaration, ToolResourceKindId, ToolResourceSet, ToolScope,
    ToolScopeKind,
};
use crate::scene::modes::{SELECT_SCENE_MODE_ID, SceneModeActivation};
use zircon_runtime::core::framework::render::SceneGizmoOverlayExtract;
use zircon_runtime::plugin::{PluginModuleManifest, PluginPackageManifest};
use zircon_runtime_interface::resource::ResourceKind;

struct RejectingOperationFactory;

struct NoopRuntimeConsumer;

struct NoopOverlayProvider;

impl ViewportOverlayProvider for NoopOverlayProvider {
    fn extract(
        &self,
        _context: &ViewportOverlayProviderContext<'_>,
    ) -> Vec<SceneGizmoOverlayExtract> {
        Vec::new()
    }
}

impl EditorRuntimeEventConsumerState for NoopRuntimeConsumer {
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

impl OperationCommandFactory for RejectingOperationFactory {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        Err(OperationCommandFactoryError::InvalidArguments {
            operation: invocation.operation_id.clone(),
            reason: "ticket ownership fixture is not executable".to_string(),
        })
    }
}

fn plugin_registration(
    plugin_id: &str,
    command_id: &str,
    view_id: &str,
    asset_kind: ResourceKind,
) -> EditorPluginRegistrationReport {
    let mut extensions = EditorExtensionRegistry::default();
    let command_id =
        EditorOperationPath::parse(command_id).expect("plugin command id should be valid");
    extensions
        .register_command(EditorCommandDescriptor::operation(command_id.clone()))
        .expect("plugin command should register in its batch");
    extensions
        .register_menu_item(EditorMenuItemDescriptor::new(
            EditorCommandMenuPath::builtin(&command_id, "tools", &[plugin_id]),
            command_id,
        ))
        .expect("plugin menu item should register in its batch");
    extensions
        .register_view(ViewDescriptor::new(
            view_id,
            format!("{plugin_id} View"),
            "Plugins",
        ))
        .expect("plugin view should register in its batch");
    let factory_operation = plugin_operation(plugin_id, "apply");
    extensions
        .register_operation_command(
            EditorCommandDescriptor::operation(factory_operation.clone()),
            OperationCommandFactoryRegistration::new(
                factory_operation,
                format!("Apply {plugin_id}"),
                EditOperationTarget::EditWorkspace,
                Arc::new(RejectingOperationFactory),
            ),
        )
        .expect("plugin operation factory should register in its batch");
    let asset_operation = plugin_operation(plugin_id, "create_asset");
    extensions
        .register_command(EditorCommandDescriptor::operation(asset_operation.clone()))
        .expect("plugin asset operation should register in its batch");
    extensions
        .register_asset_type_contribution(
            AssetTypeContribution::augment(AssetTypeId::from_resource_kind(asset_kind))
                .with_creation_template(AssetCreationTemplateDescriptor::new(
                    format!("plugin.{plugin_id}.asset"),
                    format!("{plugin_id} Asset"),
                    asset_operation,
                )),
        )
        .expect("plugin asset type contribution should register in its batch");

    registration_report(plugin_id, extensions)
}

fn conflicting_plugin_registration(
    plugin_id: &str,
    command_id: &str,
    view_id: &str,
) -> EditorPluginRegistrationReport {
    let mut extensions = EditorExtensionRegistry::default();
    extensions
        .register_command(EditorCommandDescriptor::operation(
            EditorOperationPath::parse(command_id).expect("plugin command id should be valid"),
        ))
        .expect("conflicting command should register in its private batch");
    extensions
        .register_view(ViewDescriptor::new(
            view_id,
            format!("{plugin_id} Secondary View"),
            "Plugins",
        ))
        .expect("secondary view should register in its private batch");
    registration_report(plugin_id, extensions)
}

fn plugin_registration_with_overlay(
    plugin_id: &str,
    command_id: &str,
    view_id: &str,
    asset_kind: ResourceKind,
    provider_id: &str,
) -> EditorPluginRegistrationReport {
    let mut registration = plugin_registration(plugin_id, command_id, view_id, asset_kind);
    registration
        .extensions
        .register_viewport_overlay_provider(ViewportOverlayProviderRegistration::new(
            provider_id,
            || Arc::new(NoopOverlayProvider) as Arc<dyn ViewportOverlayProvider>,
        ))
        .expect("plugin overlay provider should register in its batch");
    registration
}

fn plugin_registration_with_scene_mode(
    plugin_id: &str,
    command_id: &str,
    view_id: &str,
    asset_kind: ResourceKind,
    mode_id: &str,
) -> EditorPluginRegistrationReport {
    let mut registration = plugin_registration(plugin_id, command_id, view_id, asset_kind);
    let activation_operation = plugin_operation(plugin_id, "mode.activate");
    let activation =
        SceneModeActivation::Custom(crate::core::editor_message::SceneModeId::new(mode_id));
    registration
        .extensions
        .register_command(
            EditorCommandDescriptor::operation(activation_operation.clone()).with_event(
                EditorEvent::Viewport(EditorViewportEvent::ActivateSceneMode {
                    mode: activation.clone(),
                }),
            ),
        )
        .expect("scene-mode activation command should register in its batch");
    registration
        .extensions
        .register_scene_mode(crate::tests::support::pass_through_scene_mode_registration(
            SceneModeDescriptor::new(
                mode_id,
                format!("{plugin_id} Mode"),
                format!("plugin.{plugin_id}"),
                activation_operation,
            ),
        ))
        .expect("plugin scene mode should register in its batch");
    registration
}

fn registration_report(
    plugin_id: &str,
    extensions: EditorExtensionRegistry,
) -> EditorPluginRegistrationReport {
    EditorPluginRegistrationReport {
        package_manifest: PluginPackageManifest::new(plugin_id, plugin_id).with_editor_module(
            PluginModuleManifest::editor(
                format!("{plugin_id}.editor"),
                format!("zircon_plugin_{plugin_id}_editor"),
            ),
        ),
        capabilities: Vec::new(),
        extensions,
        lifecycle: crate::core::plugin::sdk::lifecycle::EditorPluginLifecycleReport::default(),
        successful_lifecycle_stages: Vec::new(),
        failed_lifecycle_stages: Vec::new(),
        runtime_event_consumers: plugin_consumer_registry(plugin_id),
        native_command_bindings: std::collections::BTreeMap::new(),
        diagnostics: Vec::new(),
    }
}

fn plugin_consumer_registry(plugin_id: &str) -> EditorRuntimeEventConsumerRegistry {
    let consumer_id = format!("plugin.{plugin_id}.events");
    let mut registry = EditorRuntimeEventConsumerRegistry::default();
    registry
        .register(EditorRuntimeEventConsumerRegistration::typed(
            crate::core::runtime_event_consumer::EditorRuntimeEventConsumerManifest::new(
                consumer_id.clone(),
                format!("{consumer_id}.event"),
                format!("{consumer_id}.event.v1"),
            ),
            Arc::new(Mutex::new(NoopRuntimeConsumer)),
        ))
        .expect("plugin consumer should register in its private batch");
    registry
}

fn plugin_operation(plugin_id: &str, suffix: &str) -> EditorOperationPath {
    EditorOperationPath::parse(format!("plugin.{plugin_id}.{suffix}"))
        .expect("plugin operation id should be valid")
}

fn has_plugin_menu(runtime: &EventRuntimeHarness, plugin_id: &str) -> bool {
    let snapshot = runtime.runtime.shell().lock().contributions.snapshot();
    let capabilities = CapabilitySet::default();
    snapshot
        .menu_items(&capabilities)
        .any(|item| item.path() == format!("Tools/{plugin_id}/Run"))
}

#[test]
fn ticketed_command_router_revoke_reprojects_every_executable_route() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("ticketed_command_router_revoke");
    let weather_command = "plugin.weather.refresh";
    let weather_view_command = "view.plugin.weather.panel.open";
    let lighting_command = "plugin.lighting.rebuild";
    let weather_factory = plugin_operation("weather", "apply");
    let lighting_factory = plugin_operation("lighting", "apply");
    let weather_asset_write = plugin_operation("weather", "create_asset");
    let lighting_asset_write = plugin_operation("lighting", "create_asset");
    let builtin_generation = runtime.runtime.commands().lock().generation();

    let weather_handle = runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration(
            "weather",
            weather_command,
            "plugin.weather.panel",
            ResourceKind::Material,
        ))
        .expect("weather contribution should register");
    let _lighting_handle = runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration(
            "lighting",
            lighting_command,
            "plugin.lighting.panel",
            ResourceKind::Texture,
        ))
        .expect("lighting contribution should register");

    let active_generations = runtime
        .runtime
        .context()
        .tools()
        .snapshot()
        .active_owner_generations()
        .to_vec();
    assert_eq!(active_generations.len(), 3);
    assert!(active_generations.contains(&ToolOwnerGeneration::BUILTIN));

    let manager = runtime
        .core
        .resolve_manager::<crate::ui::host::EditorManager>(
            crate::ui::host::module::EDITOR_MANAGER_NAME,
        )
        .expect("editor manager should resolve");
    let weather_instance = manager
        .open_view(
            crate::ui::workbench::view::ViewDescriptorId::new("plugin.weather.panel"),
            None,
        )
        .expect("weather extension view should open");
    assert!(
        manager
            .current_view_instances()
            .iter()
            .any(|instance| instance.instance_id == weather_instance)
    );

    {
        let commands = runtime.runtime.commands().lock();
        assert!(commands.command(weather_command).is_some());
        assert!(commands.command(weather_view_command).is_some());
        assert!(commands.command(lighting_command).is_some());
        assert!(commands.operation_factory(&weather_factory).is_some());
        assert!(commands.operation_factory(&lighting_factory).is_some());
        assert!(
            commands
                .command(&weather_asset_write)
                .and_then(|command| command.asset_write_target())
                .is_some()
        );
        assert!(
            commands
                .command(&lighting_asset_write)
                .and_then(|command| command.asset_write_target())
                .is_some()
        );
    }
    assert!(has_plugin_menu(&runtime, "weather"));
    assert!(has_plugin_menu(&runtime, "lighting"));
    let generation_before_revoke = runtime.runtime.commands().lock().generation();
    assert!(generation_before_revoke > builtin_generation);

    assert!(
        runtime
            .runtime
            .revoke_editor_plugin_contribution(&weather_handle)
            .expect("weather contribution should revoke")
    );

    let active_generations_after_revoke = runtime
        .runtime
        .context()
        .tools()
        .snapshot()
        .active_owner_generations()
        .to_vec();
    assert_eq!(active_generations_after_revoke.len(), 2);
    assert!(active_generations_after_revoke.contains(&ToolOwnerGeneration::BUILTIN));

    let commands = runtime.runtime.commands().lock();
    assert!(commands.command(weather_command).is_none());
    assert!(commands.command(weather_view_command).is_none());
    assert!(commands.command(lighting_command).is_some());
    assert!(commands.operation_factory(&weather_factory).is_none());
    assert!(commands.operation_factory(&lighting_factory).is_some());
    assert!(commands.command(&weather_asset_write).is_none());
    assert!(
        commands
            .command(&lighting_asset_write)
            .and_then(|command| command.asset_write_target())
            .is_some()
    );
    assert!(commands.command("file.documents.save_all").is_some());
    assert!(commands.generation() > generation_before_revoke);
    assert_eq!(
        commands.command_palette_catalog().generation(),
        commands.generation()
    );
    drop(commands);
    assert!(
        manager
            .current_view_instances()
            .iter()
            .all(|instance| instance.descriptor_id.0 != "plugin.weather.panel")
    );
    assert!(
        manager
            .descriptors()
            .iter()
            .all(|descriptor| descriptor.descriptor_id.0 != "plugin.weather.panel")
    );
    assert!(
        manager
            .descriptors()
            .iter()
            .any(|descriptor| descriptor.descriptor_id.0 == "plugin.lighting.panel")
    );
    assert!(!has_plugin_menu(&runtime, "weather"));
    assert!(has_plugin_menu(&runtime, "lighting"));
    runtime
        .runtime
        .register_runtime_event_consumers(plugin_consumer_registry("weather"))
        .expect("revoked ticket must release its runtime consumer id");
    let error = runtime
        .runtime
        .register_runtime_event_consumers(plugin_consumer_registry("lighting"))
        .expect_err("remaining ticket must retain its runtime consumer id");
    assert!(error.to_string().contains("plugin.lighting.events"));
}

#[test]
fn ticketed_command_router_rejected_candidate_publishes_neither_generation() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("ticketed_command_router_reject");
    let command_id = "plugin.weather.refresh";

    runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration(
            "weather",
            command_id,
            "plugin.weather.panel",
            ResourceKind::Material,
        ))
        .expect("initial weather contribution should register");
    let store_generation = runtime.runtime.extension_projection_revision().0;
    let command_generation = runtime.runtime.commands().lock().generation();
    let tool_snapshot = runtime.runtime.context().tools().snapshot();

    let error = runtime
        .runtime
        .register_editor_plugin_registration(conflicting_plugin_registration(
            "weather",
            command_id,
            "plugin.weather.secondary_panel",
        ))
        .expect_err("a second live ticket for one plugin owner must reject the whole candidate");

    assert!(error.to_string().contains("editor extension owner"));
    assert!(error.to_string().contains("weather"));
    assert_eq!(
        runtime.runtime.extension_projection_revision().0,
        store_generation
    );
    let commands = runtime.runtime.commands().lock();
    assert_eq!(commands.generation(), command_generation);
    assert!(commands.command(command_id).is_some());
    assert!(
        commands
            .command("view.plugin.weather.secondary_panel.open")
            .is_none()
    );
    assert_eq!(runtime.runtime.context().tools().snapshot(), tool_snapshot);
}

#[test]
fn stale_contribution_handle_cannot_revoke_a_reloaded_plugin_generation() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("ticketed_contribution_stale_handle");
    let command_id = "plugin.weather.refresh";

    let first_handle = runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration(
            "weather",
            command_id,
            "plugin.weather.panel",
            ResourceKind::Material,
        ))
        .expect("first weather contribution should register");
    let tool_definition = ToolDefinitionId::parse("plugin.weather.tool")
        .expect("plugin tool definition should be valid");
    let first_instance = runtime
        .runtime
        .allocate_editor_tool_instance(&first_handle, &tool_definition)
        .expect("current contribution handle should allocate a tool instance");
    assert!(
        runtime
            .runtime
            .revoke_editor_plugin_contribution(&first_handle)
            .expect("first weather contribution should revoke")
    );

    let second_handle = runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration(
            "weather",
            command_id,
            "plugin.weather.panel",
            ResourceKind::Material,
        ))
        .expect("reloaded weather contribution should register");
    let store_generation = runtime.runtime.extension_projection_revision();
    let command_generation = runtime.runtime.commands().lock().generation();
    let tool_snapshot = runtime.runtime.context().tools().snapshot();

    let stale_allocation = runtime
        .runtime
        .allocate_editor_tool_instance(&first_handle, &tool_definition)
        .expect_err("stale contribution handle must not allocate for the reloaded plugin");
    assert!(matches!(
        stale_allocation,
        crate::core::editor_extension::EditorExtensionRegistryError::StaleContributionHandle {
            owner_id
        } if owner_id == "weather"
    ));
    let second_instance = runtime
        .runtime
        .allocate_editor_tool_instance(&second_handle, &tool_definition)
        .expect("reloaded contribution handle should allocate a tool instance");
    assert_ne!(
        first_instance.owner_generation(),
        second_instance.owner_generation()
    );

    assert!(
        !runtime
            .runtime
            .revoke_editor_plugin_contribution(&first_handle)
            .expect("stale contribution handle should be an idempotent no-op")
    );
    assert_eq!(
        runtime.runtime.extension_projection_revision(),
        store_generation
    );
    assert_eq!(
        runtime.runtime.commands().lock().generation(),
        command_generation
    );
    assert_eq!(runtime.runtime.context().tools().snapshot(), tool_snapshot);
    assert!(
        runtime
            .runtime
            .commands()
            .lock()
            .command(command_id)
            .is_some()
    );

    assert!(
        runtime
            .runtime
            .revoke_editor_plugin_contribution(&second_handle)
            .expect("current contribution handle should revoke the reloaded plugin")
    );
}

#[test]
fn contributed_tool_resource_kind_is_bound_to_the_exact_owner_generation() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("ticketed_tool_resource_kind_owner");
    let kind = ToolResourceKindId::parse("plugin.weather.viewport-lock").unwrap();
    let mut registration = plugin_registration(
        "weather",
        "plugin.weather.refresh",
        "plugin.weather.panel",
        ResourceKind::Material,
    );
    registration
        .extensions
        .register_tool_resource_kind(
            ToolResourceKindDeclaration::new(
                kind.clone(),
                [ToolScopeKind::Viewport],
                ToolResourceChannelPolicy::Forbidden,
            )
            .unwrap(),
        )
        .unwrap();

    let handle = runtime
        .runtime
        .register_editor_plugin_registration(registration)
        .expect("plugin resource kind contribution should register atomically");
    let registered_kind = runtime
        .runtime
        .context()
        .tools()
        .snapshot()
        .resource_catalog()
        .iter()
        .find(|registration| registration.kind() == &kind)
        .expect("contributed resource kind should be visible in the authority snapshot")
        .clone();
    assert_eq!(
        registered_kind.owner_generation(),
        handle.owner_generation()
    );

    let instance = runtime
        .runtime
        .allocate_editor_tool_instance(
            &handle,
            &ToolDefinitionId::parse("plugin.weather.viewport-tool").unwrap(),
        )
        .unwrap();
    let resource = ToolResourceKey::new(
        kind.clone(),
        ToolScope::Viewport {
            viewport_id: crate::ui::workbench::view::ViewInstanceId::new("editor.scene#weather"),
        },
        None,
    )
    .unwrap();
    assert!(matches!(
        runtime
            .runtime
            .context()
            .tools()
            .acquire(instance, ToolResourceSet::single(resource))
            .unwrap()
            .outcome(),
        AcquireOutcome::Acquired { .. }
    ));

    assert!(
        runtime
            .runtime
            .revoke_editor_plugin_contribution(&handle)
            .unwrap()
    );
    assert!(
        runtime
            .runtime
            .context()
            .tools()
            .snapshot()
            .resource_catalog()
            .iter()
            .all(|registration| registration.kind() != &kind)
    );
}

#[test]
fn cross_plugin_tool_resource_namespace_is_rejected_without_publication() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("ticketed_tool_resource_kind_namespace_reject");
    let mut registration = plugin_registration(
        "weather",
        "plugin.weather.refresh",
        "plugin.weather.panel",
        ResourceKind::Material,
    );
    registration
        .extensions
        .register_tool_resource_kind(
            ToolResourceKindDeclaration::new(
                ToolResourceKindId::parse("plugin.lighting.viewport-lock").unwrap(),
                [ToolScopeKind::Viewport],
                ToolResourceChannelPolicy::Forbidden,
            )
            .unwrap(),
        )
        .unwrap();
    let store_generation = runtime.runtime.extension_projection_revision();
    let command_generation = runtime.runtime.commands().lock().generation();
    let tool_snapshot = runtime.runtime.context().tools().snapshot();

    let error = runtime
        .runtime
        .register_editor_plugin_registration(registration)
        .expect_err("plugin must not reserve another plugin's tool resource namespace");
    assert!(matches!(
        error,
        crate::core::editor_extension::EditorExtensionRegistryError::ToolResourceKindOwnerMismatch {
            owner_id,
            ..
        } if owner_id == "weather"
    ));
    assert_eq!(
        runtime.runtime.extension_projection_revision(),
        store_generation
    );
    assert_eq!(
        runtime.runtime.commands().lock().generation(),
        command_generation
    );
    assert_eq!(runtime.runtime.context().tools().snapshot(), tool_snapshot);
}

#[test]
fn ticketed_runtime_consumer_collision_publishes_no_store_or_router_generation() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("ticketed_runtime_consumer_reject");
    runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration(
            "weather",
            "plugin.weather.refresh",
            "plugin.weather.panel",
            ResourceKind::Material,
        ))
        .expect("initial weather contribution should register");
    let store_generation = runtime.runtime.extension_projection_revision().0;
    let command_generation = runtime.runtime.commands().lock().generation();
    let mut climate = plugin_registration(
        "climate",
        "plugin.climate.refresh",
        "plugin.climate.panel",
        ResourceKind::Shader,
    );
    climate.runtime_event_consumers = plugin_consumer_registry("weather");

    let error = runtime
        .runtime
        .register_editor_plugin_registration(climate)
        .expect_err("duplicate consumer must reject the complete plugin candidate");

    assert!(error.to_string().contains("plugin.weather.events"));
    assert_eq!(
        runtime.runtime.extension_projection_revision().0,
        store_generation
    );
    let commands = runtime.runtime.commands().lock();
    assert_eq!(commands.generation(), command_generation);
    assert!(commands.command("plugin.climate.refresh").is_none());
    assert!(commands.command("view.plugin.climate.panel.open").is_none());
}

#[test]
fn ticketed_revoke_removes_only_the_owned_viewport_overlay_provider() {
    use crate::ui::binding::viewport::ViewportCommand;

    const WEATHER_PROVIDER: &str = "plugin.weather.overlay";
    const LIGHTING_PROVIDER: &str = "plugin.lighting.overlay";

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("ticketed_overlay_provider_revoke");
    let weather_handle = runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration_with_overlay(
            "weather",
            "plugin.weather.refresh",
            "plugin.weather.panel",
            ResourceKind::Material,
            WEATHER_PROVIDER,
        ))
        .expect("weather contribution should register");
    runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration_with_overlay(
            "lighting",
            "plugin.lighting.rebuild",
            "plugin.lighting.panel",
            ResourceKind::Texture,
            LIGHTING_PROVIDER,
        ))
        .expect("lighting contribution should register");
    {
        let mut shell = runtime.runtime.shell().lock();
        assert!(
            shell
                .state
                .apply_viewport_command(&ViewportCommand::ToggleOverlayProvider {
                    provider_id: WEATHER_PROVIDER.to_string(),
                })
                .unwrap()
        );
        assert!(
            shell
                .state
                .apply_viewport_command(&ViewportCommand::ToggleOverlayProvider {
                    provider_id: LIGHTING_PROVIDER.to_string(),
                })
                .unwrap()
        );
    }

    assert!(
        runtime
            .runtime
            .revoke_editor_plugin_contribution(&weather_handle)
            .expect("weather contribution should revoke")
    );

    let mut shell = runtime.runtime.shell().lock();
    let error = shell
        .state
        .apply_viewport_command(&ViewportCommand::ToggleOverlayProvider {
            provider_id: WEATHER_PROVIDER.to_string(),
        })
        .expect_err("revoked provider must no longer be routable");
    assert!(matches!(
        error,
        crate::ui::workbench::state::EditorViewportStateError::ViewportController(
            crate::scene::viewport::SceneViewportControllerError::ViewportOverlayProvider(
                crate::scene::viewport::ViewportOverlayProviderError::UnknownProvider {
                    provider_id
                }
            )
        ) if provider_id == WEATHER_PROVIDER
    ));
    assert!(
        !shell
            .state
            .apply_viewport_command(&ViewportCommand::ToggleOverlayProvider {
                provider_id: LIGHTING_PROVIDER.to_string(),
            })
            .expect("the remaining provider keeps its enabled state")
    );
}

#[test]
fn ticketed_revoke_exits_an_active_scene_mode_before_removing_its_factory() {
    use crate::ui::binding::viewport::ViewportCommand;

    const WEATHER_MODE: &str = "plugin.weather.mode";
    const LIGHTING_MODE: &str = "plugin.lighting.mode";

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("ticketed_scene_mode_revoke");
    let weather_handle = runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration_with_scene_mode(
            "weather",
            "plugin.weather.refresh",
            "plugin.weather.panel",
            ResourceKind::Material,
            WEATHER_MODE,
        ))
        .expect("weather contribution should register");
    runtime
        .runtime
        .register_editor_plugin_registration(plugin_registration_with_scene_mode(
            "lighting",
            "plugin.lighting.rebuild",
            "plugin.lighting.panel",
            ResourceKind::Texture,
            LIGHTING_MODE,
        ))
        .expect("lighting contribution should register");
    {
        let mut shell = runtime.runtime.shell().lock();
        shell
            .state
            .apply_viewport_command(&ViewportCommand::ActivateSceneMode(
                SceneModeActivation::Custom(crate::core::editor_message::SceneModeId::new(
                    WEATHER_MODE,
                )),
            ))
            .unwrap();
    }

    assert!(
        runtime
            .runtime
            .revoke_editor_plugin_contribution(&weather_handle)
            .expect("weather contribution should revoke")
    );

    let mut shell = runtime.runtime.shell().lock();
    assert_eq!(
        shell
            .state
            .viewport_controller
            .active_scene_mode()
            .mode_id(),
        crate::core::editor_message::SceneModeId::new(SELECT_SCENE_MODE_ID)
    );
    let error = shell
        .state
        .apply_viewport_command(&ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Custom(crate::core::editor_message::SceneModeId::new(
                WEATHER_MODE,
            )),
        ))
        .expect_err("revoked scene mode factory must no longer be routable");
    assert!(matches!(
        error,
        crate::ui::workbench::state::EditorViewportStateError::ViewportController(
            crate::scene::viewport::SceneViewportControllerError::SceneModeRegistry(
                crate::scene::modes::SceneModeRegistryError::UnknownMode { mode_id }
            )
        ) if mode_id == crate::core::editor_message::SceneModeId::new(WEATHER_MODE)
    ));
    shell
        .state
        .apply_viewport_command(&ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Custom(crate::core::editor_message::SceneModeId::new(
                LIGHTING_MODE,
            )),
        ))
        .expect("the remaining scene mode stays routable");
}
