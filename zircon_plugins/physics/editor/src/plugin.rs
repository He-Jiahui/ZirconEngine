use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
    AssetTypePresentation, ThumbnailProviderDescriptor,
};
use zircon_editor::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};
use zircon_editor::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorUiTemplateDescriptor,
    ViewDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_editor_support::{
    register_authoring_extensions, register_authoring_surface, EditorAuthoringExtensions,
    EditorAuthoringSurface,
};
use zircon_plugin_sdk::{authoring_plugin, EditorPluginDeclaration};

use crate::capability::{EDITOR_CAPABILITIES, PLUGIN_ID};
use crate::extension_ids::{
    PHYSICS_AUTHORING_VIEW_ID, PHYSICS_CREATE_RAGDOLL_PROFILE_OPERATION, PHYSICS_DEBUG_VIEW_ID,
    PHYSICS_DIAGNOSTICS_VIEW_ID, PHYSICS_DRAWER_ID, PHYSICS_RAGDOLL_PROFILE_VIEW_ID,
    PHYSICS_TEMPLATE_ID, PHYSICS_TOGGLE_OVERLAY_OPERATION, RAGDOLL_PROFILE_ASSET_KIND,
};

authoring_plugin! {
    pub struct PhysicsEditorPlugin {
        package_id: PLUGIN_ID,
        display_name: "Physics",
        crate_name: "zircon_plugin_physics_editor",
        category: "runtime",
        description: "Physics editor authoring extensions.",
        maturity: zircon_runtime::plugin::PluginMaturity::Experimental,
        mirrors_runtime_manifest: zircon_plugin_physics_runtime::package_manifest(),
        capabilities: EDITOR_CAPABILITIES,
        register_extensions: register_physics_authoring_extensions,
    }
}

pub fn editor_plugin_declaration() -> EditorPluginDeclaration {
    editor_plugin().declaration().clone()
}

fn register_physics_authoring_extensions(
    registry: &mut EditorExtensionRegistry,
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
            .with_event(EditorEvent::WorkbenchMenu(MenuAction::OpenView(
                ViewDescriptorId::new(PHYSICS_DEBUG_VIEW_ID),
            ))),
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
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}
