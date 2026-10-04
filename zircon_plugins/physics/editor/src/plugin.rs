use std::sync::{Arc, Mutex};

use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
    AssetTypePresentation, ThumbnailProviderDescriptor,
};
use zircon_editor::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};
use zircon_editor::core::editor_event::{
    EditorEvent, EditorViewportEvent, MenuAction, ViewDescriptorId,
};
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorUiTemplateDescriptor,
    ViewDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry;
use zircon_editor::{EditorPlugin, EditorPluginDescriptor, EditorPluginRegistrationReport};
use zircon_plugin_editor_support::{
    register_authoring_extensions, register_authoring_surface, EditorAuthoringExtensions,
    EditorAuthoringSurface,
};
use zircon_plugin_sdk::EditorPluginDeclaration;
use zircon_runtime::plugin::{PluginMaturity, PluginPackageManifest};

use crate::capability::{EDITOR_CAPABILITIES, PLUGIN_ID};
use crate::extension_ids::{
    PHYSICS_AUTHORING_VIEW_ID, PHYSICS_CREATE_RAGDOLL_PROFILE_OPERATION, PHYSICS_DEBUG_VIEW_ID,
    PHYSICS_DIAGNOSTICS_VIEW_ID, PHYSICS_DRAWER_ID, PHYSICS_RAGDOLL_PROFILE_VIEW_ID,
    PHYSICS_TEMPLATE_ID, PHYSICS_TOGGLE_OVERLAY_OPERATION, RAGDOLL_PROFILE_ASSET_KIND,
};
use crate::overlay::PHYSICS_OVERLAY_PROVIDER_ID;
use crate::runtime_mirror::{physics_runtime_event_consumers_with_mirror, PhysicsPieMirror};
use crate::viewport_overlay_provider::register_physics_viewport_overlay_provider;

#[derive(Clone, Debug)]
pub struct PhysicsEditorPlugin {
    declaration: EditorPluginDeclaration,
    pie_mirror: Arc<Mutex<PhysicsPieMirror>>,
}

impl Default for PhysicsEditorPlugin {
    fn default() -> Self {
        let pie_mirror = Arc::new(Mutex::new(PhysicsPieMirror::default()));
        let declaration = physics_runtime_event_consumers_with_mirror(pie_mirror.clone())
            .into_iter()
            .fold(
                EditorPluginDeclaration::new(PLUGIN_ID, "Physics", "zircon_plugin_physics_editor")
                    .with_category("runtime")
                    .with_description("Physics editor authoring extensions.")
                    .with_maturity(PluginMaturity::Experimental)
                    .mirrors_runtime_manifest(zircon_plugin_physics_runtime::package_manifest())
                    .with_capabilities(EDITOR_CAPABILITIES.iter().copied()),
                |declaration, registration| {
                    declaration.with_runtime_event_consumer_registration(registration)
                },
            );
        Self {
            declaration,
            pie_mirror,
        }
    }
}

impl PhysicsEditorPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declaration(&self) -> &EditorPluginDeclaration {
        &self.declaration
    }

    pub fn pie_mirror(&self) -> Arc<Mutex<PhysicsPieMirror>> {
        self.pie_mirror.clone()
    }

    pub fn package_manifest(&self) -> PluginPackageManifest {
        self.declaration.package_manifest()
    }

    pub fn editor_capabilities(&self) -> Vec<String> {
        self.declaration.capabilities().to_vec()
    }

    pub fn registration_report(&self) -> EditorPluginRegistrationReport {
        self.declaration.registration_report(self)
    }
}

impl EditorPlugin for PhysicsEditorPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        self.declaration.descriptor()
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_physics_authoring_extensions(registry, self.pie_mirror())
    }

    fn runtime_event_consumers(&self) -> EditorRuntimeEventConsumerRegistry {
        self.declaration.runtime_event_consumers()
    }
}

pub fn editor_plugin_declaration() -> EditorPluginDeclaration {
    editor_plugin().declaration().clone()
}

fn register_physics_authoring_extensions(
    registry: &mut EditorExtensionRegistry,
    pie_mirror: Arc<Mutex<PhysicsPieMirror>>,
) -> Result<(), EditorExtensionRegistryError> {
    register_authoring_extensions(
        registry,
        EditorAuthoringExtensions {
            drawer_id: PHYSICS_DRAWER_ID,
            drawer_display_name: "Physics Tools",
            template_id: PHYSICS_TEMPLATE_ID,
            template_document: "plugins://physics/editor/authoring.zui",
            surfaces: &[EditorAuthoringSurface::new(
                PHYSICS_AUTHORING_VIEW_ID,
                "Physics",
                "World",
            )],
        },
    )?;
    register_physics_debug_overlay(registry)?;
    register_physics_viewport_overlay_provider(registry, pie_mirror)?;
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        PHYSICS_DIAGNOSTICS_VIEW_ID,
        "plugins://physics/editor/diagnostics.zui",
    ))?;
    register_authoring_surface(
        registry,
        EditorAuthoringSurface::new(
            PHYSICS_DIAGNOSTICS_VIEW_ID,
            "Physics Diagnostics",
            "Diagnostics",
        ),
    )?;
    register_ragdoll_profile_editor(registry)
}

fn register_physics_debug_overlay(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let operation = parse_operation(PHYSICS_TOGGLE_OVERLAY_OPERATION)?;
    registry.register_view(ViewDescriptor::new(
        PHYSICS_DEBUG_VIEW_ID,
        "Physics Debug Overlay",
        "World",
    ))?;
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        PHYSICS_DEBUG_VIEW_ID,
        "plugins://physics/editor/debug_overlay.zui",
    ))?;
    registry.register_command(
        EditorCommandDescriptor::operation(operation.clone())
            .with_menu_path(EditorCommandMenuPath::builtin(
                &operation,
                "view",
                &["debug_overlays"],
            ))
            .with_callable_from_remote(false)
            .with_required_capabilities([crate::capability::PHYSICS_AUTHORING_CAPABILITY])
            .with_event(EditorEvent::Viewport(
                EditorViewportEvent::ToggleOverlayProvider {
                    provider_id: PHYSICS_OVERLAY_PROVIDER_ID.to_owned(),
                },
            )),
    )?;
    Ok(())
}

fn register_ragdoll_profile_editor(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        PHYSICS_RAGDOLL_PROFILE_VIEW_ID,
        "plugins://physics/editor/ragdoll_profile.zui",
    ))?;
    register_authoring_surface(
        registry,
        EditorAuthoringSurface::new(
            PHYSICS_RAGDOLL_PROFILE_VIEW_ID,
            "Ragdoll Profile",
            "Physics",
        ),
    )?;
    let open_operation = parse_operation(&format!("view.{PHYSICS_RAGDOLL_PROFILE_VIEW_ID}.open"))?;
    let asset_type = AssetTypeId::parse(RAGDOLL_PROFILE_ASSET_KIND)?;

    let create_operation = parse_operation(PHYSICS_CREATE_RAGDOLL_PROFILE_OPERATION)?;
    registry.register_command(
        EditorCommandDescriptor::operation(create_operation.clone())
            .with_callable_from_remote(false)
            .with_required_capabilities([crate::capability::PHYSICS_AUTHORING_CAPABILITY])
            .with_event(EditorEvent::WorkbenchMenu(MenuAction::OpenView(
                ViewDescriptorId::new(PHYSICS_RAGDOLL_PROFILE_VIEW_ID),
            ))),
    )?;
    registry.register_asset_type_contribution(
        AssetTypeContribution::define(
            asset_type,
            AssetTypePresentation::new(
                "Ragdoll Profile",
                "RAG",
                "asset-ragdoll-profile",
                "asset.physics",
            ),
            ThumbnailProviderDescriptor::Icon("asset-ragdoll-profile".to_owned()),
        )
        .with_toolkit(
            AssetToolkitDescriptor::new(PHYSICS_RAGDOLL_PROFILE_VIEW_ID, open_operation)
                .with_required_capabilities([crate::capability::PHYSICS_AUTHORING_CAPABILITY]),
        )
        .with_creation_template(
            AssetCreationTemplateDescriptor::new(
                "physics.ragdoll_profile.from_skeleton",
                "Ragdoll Profile From Skeleton",
                create_operation,
            )
            .with_default_document("plugins://physics/editor/ragdoll_profile.zui")
            .with_required_capabilities([crate::capability::PHYSICS_AUTHORING_CAPABILITY]),
        ),
    )
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::OperationPath)
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    editor_plugin_declaration().descriptor().clone()
}

pub fn editor_plugin() -> PhysicsEditorPlugin {
    PhysicsEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    editor_plugin().declaration().package_manifest()
}

pub fn editor_capabilities() -> Vec<String> {
    editor_plugin().declaration().capabilities().to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    let plugin = editor_plugin();
    plugin.declaration().registration_report(&plugin)
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}
