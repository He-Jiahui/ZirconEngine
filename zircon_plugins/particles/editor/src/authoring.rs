use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
    AssetTypePresentation, ThumbnailProviderDescriptor,
};
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_extension::{
    ComponentDrawerDescriptor, EditorExtensionRegistry, EditorExtensionRegistryError,
    EditorMenuItemDescriptor, EditorUiTemplateDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_plugin_particles_runtime::PARTICLE_SYSTEM_COMPONENT_TYPE;

use crate::capability::PARTICLES_AUTHORING_CAPABILITY;
use crate::extension_ids::{
    PARTICLES_AUTHORING_VIEW_ID, PARTICLES_COMPONENT_DRAWER_ID,
    PARTICLES_CPU_SPRITE_TEMPLATE_DOCUMENT, PARTICLES_CPU_SPRITE_TEMPLATE_ID, PARTICLES_DRAWER_ID,
    PARTICLES_PREVIEW_TEMPLATE_ID, PARTICLES_PREVIEW_VIEW_ID, PARTICLES_SYSTEM_ASSET_KIND,
    PARTICLES_TEMPLATE_ID,
};

pub(crate) fn register_particles_authoring_extensions(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    register_authoring_extensions(
        registry,
        EditorAuthoringExtensions {
            drawer_id: PARTICLES_DRAWER_ID,
            drawer_display_name: "Particles Tools",
            template_id: PARTICLES_TEMPLATE_ID,
            template_document: "plugins://particles/editor/authoring.zui",
            surfaces: &[
                EditorAuthoringSurface::new(
                    PARTICLES_AUTHORING_VIEW_ID,
                    "Particles",
                    "Effects",
                    "Plugins/Particles/Authoring",
                ),
                EditorAuthoringSurface::new(
                    PARTICLES_PREVIEW_VIEW_ID,
                    "Particle Preview",
                    "Effects",
                    "Plugins/Particles/Preview",
                ),
            ],
        },
    )?;
    register_particles_component_drawers(registry)?;
    let asset_type = AssetTypeId::parse(PARTICLES_SYSTEM_ASSET_KIND)?;
    register_authoring_contribution_batch(registry, particles_authoring_batch(asset_type))
}

fn register_particles_component_drawers(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        PARTICLES_PREVIEW_TEMPLATE_ID,
        "plugins://particles/editor/preview.zui",
    ))?;
    registry.register_component_drawer(ComponentDrawerDescriptor::new(
        PARTICLE_SYSTEM_COMPONENT_TYPE,
        "plugins://particles/editor/particle_system.drawer.zui",
        PARTICLES_COMPONENT_DRAWER_ID,
    ))?;
    Ok(())
}

fn particles_authoring_batch(asset_type: AssetTypeId) -> EditorAuthoringContributionBatch {
    let create_asset = operation("particles.authoring.create_cpu_sprite_asset");
    let add_component = operation("particles.authoring.add_component");
    let open_asset = operation("particles.authoring.open_asset");
    let add_emitter = operation("particles.authoring.add_emitter");
    let add_module = operation("particles.authoring.add_module");
    let edit_curve = operation("particles.authoring.edit_curve");
    let validate = operation("particles.authoring.validate_asset");
    let preview_play = operation("particles.preview.play");
    let preview_pause = operation("particles.preview.pause");
    let preview_stop = operation("particles.preview.stop");
    let preview_rewind = operation("particles.preview.rewind");
    let preview_warmup = operation("particles.preview.warmup");

    EditorAuthoringContributionBatch {
        commands: vec![
            EditorCommandDescriptor::pending_operation(
                create_asset.clone(),
                "Create CPU Sprite Particle Asset",
            )
            .with_menu_path("Plugins/Particles/Create CPU Sprite Asset")
            .with_payload_schema_id("particles.create_cpu_sprite_asset.v1")
            .with_callable_from_remote(false)
            .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(
                add_component.clone(),
                "Add Particle System Component",
            )
            .with_menu_path("Plugins/Particles/Add Particle System Component")
            .with_payload_schema_id("particles.add_component.v1")
            .with_callable_from_remote(false)
            .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(open_asset.clone(), "Open Particle Asset")
                .with_menu_path("Plugins/Particles/Open Particle Asset")
                .with_payload_schema_id("particles.open_asset.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(add_emitter.clone(), "Add Particle Emitter")
                .with_menu_path("Plugins/Particles/Add Emitter")
                .with_payload_schema_id("particles.add_emitter.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(add_module.clone(), "Add Particle Module")
                .with_menu_path("Plugins/Particles/Add Module")
                .with_payload_schema_id("particles.add_module.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(edit_curve.clone(), "Edit Particle Curve")
                .with_menu_path("Plugins/Particles/Edit Curve")
                .with_payload_schema_id("particles.edit_curve.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(validate.clone(), "Validate Particle Asset")
                .with_menu_path("Plugins/Particles/Validate Asset")
                .with_payload_schema_id("particles.validate_asset.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(
                preview_play.clone(),
                "Play Particle Preview",
            )
            .with_menu_path("Plugins/Particles/Preview/Play")
            .with_payload_schema_id("particles.preview_play.v1")
            .with_callable_from_remote(false)
            .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(
                preview_pause.clone(),
                "Pause Particle Preview",
            )
            .with_menu_path("Plugins/Particles/Preview/Pause")
            .with_payload_schema_id("particles.preview_pause.v1")
            .with_callable_from_remote(false)
            .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(
                preview_stop.clone(),
                "Stop Particle Preview",
            )
            .with_menu_path("Plugins/Particles/Preview/Stop")
            .with_payload_schema_id("particles.preview_stop.v1")
            .with_callable_from_remote(false)
            .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(
                preview_rewind.clone(),
                "Rewind Particle Preview",
            )
            .with_menu_path("Plugins/Particles/Preview/Rewind")
            .with_payload_schema_id("particles.preview_rewind.v1")
            .with_callable_from_remote(false)
            .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::pending_operation(
                preview_warmup.clone(),
                "Warm Up Particle Preview",
            )
            .with_menu_path("Plugins/Particles/Preview/Warmup")
            .with_payload_schema_id("particles.preview_warmup.v1")
            .with_callable_from_remote(false)
            .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
        ],
        menu_items: vec![
            menu_item("Plugins/Particles/Create CPU Sprite Asset", &create_asset),
            menu_item(
                "Plugins/Particles/Add Particle System Component",
                &add_component,
            ),
            menu_item("Plugins/Particles/Open Particle Asset", &open_asset),
            menu_item("Plugins/Particles/Add Emitter", &add_emitter),
            menu_item("Plugins/Particles/Add Module", &add_module),
            menu_item("Plugins/Particles/Edit Curve", &edit_curve),
            menu_item("Plugins/Particles/Validate Asset", &validate),
            menu_item("Plugins/Particles/Preview/Play", &preview_play),
            menu_item("Plugins/Particles/Preview/Pause", &preview_pause),
            menu_item("Plugins/Particles/Preview/Stop", &preview_stop),
            menu_item("Plugins/Particles/Preview/Rewind", &preview_rewind),
            menu_item("Plugins/Particles/Preview/Warmup", &preview_warmup),
        ],
        asset_type_contributions: vec![AssetTypeContribution::define(
            asset_type,
            AssetTypePresentation::new(
                "Particle System",
                "FX",
                "asset-particle-system",
                "asset.particles",
            ),
            ThumbnailProviderDescriptor::Icon("asset-particle-system".to_owned()),
        )
        .with_toolkit(
            AssetToolkitDescriptor::new(PARTICLES_AUTHORING_VIEW_ID, open_asset.clone())
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
        )
        .with_creation_template(
            AssetCreationTemplateDescriptor::new(
                PARTICLES_CPU_SPRITE_TEMPLATE_ID,
                "CPU Sprite Particle System",
                create_asset,
            )
            .with_default_document(PARTICLES_CPU_SPRITE_TEMPLATE_DOCUMENT)
            .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
        )],
        ..Default::default()
    }
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid particles operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone())
        .with_enabled(false)
        .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY])
}
