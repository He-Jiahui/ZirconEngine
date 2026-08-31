use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
    AssetTypePresentation, ThumbnailProviderDescriptor,
};
use zircon_editor::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorUiTemplateDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::core::extension::InspectorCustomizationDescriptor;
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
                EditorAuthoringSurface::new(PARTICLES_AUTHORING_VIEW_ID, "Particles", "Effects"),
                EditorAuthoringSurface::new(
                    PARTICLES_PREVIEW_VIEW_ID,
                    "Particle Preview",
                    "Effects",
                ),
            ],
        },
    )?;
    register_particles_inspector_customizations(registry)?;
    let asset_type = AssetTypeId::parse(PARTICLES_SYSTEM_ASSET_KIND)?;
    register_authoring_contribution_batch(registry, particles_authoring_batch(asset_type))
}

fn register_particles_inspector_customizations(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        PARTICLES_PREVIEW_TEMPLATE_ID,
        "plugins://particles/editor/preview.zui",
    ))?;
    registry.register_inspector_customization(InspectorCustomizationDescriptor::new(
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
            EditorCommandDescriptor::operation(create_asset.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &create_asset,
                    "plugins",
                    &["particles"],
                ))
                .with_payload_schema_id("particles.create_cpu_sprite_asset.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(add_component.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &add_component,
                    "plugins",
                    &["particles"],
                ))
                .with_payload_schema_id("particles.add_component.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(open_asset.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &open_asset,
                    "plugins",
                    &["particles"],
                ))
                .with_payload_schema_id("particles.open_asset.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(add_emitter.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &add_emitter,
                    "plugins",
                    &["particles"],
                ))
                .with_payload_schema_id("particles.add_emitter.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(add_module.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &add_module,
                    "plugins",
                    &["particles"],
                ))
                .with_payload_schema_id("particles.add_module.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(edit_curve.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &edit_curve,
                    "plugins",
                    &["particles"],
                ))
                .with_payload_schema_id("particles.edit_curve.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(validate.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &validate,
                    "plugins",
                    &["particles"],
                ))
                .with_payload_schema_id("particles.validate_asset.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(preview_play.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &preview_play,
                    "plugins",
                    &["particles", "preview"],
                ))
                .with_payload_schema_id("particles.preview_play.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(preview_pause.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &preview_pause,
                    "plugins",
                    &["particles", "preview"],
                ))
                .with_payload_schema_id("particles.preview_pause.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(preview_stop.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &preview_stop,
                    "plugins",
                    &["particles", "preview"],
                ))
                .with_payload_schema_id("particles.preview_stop.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(preview_rewind.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &preview_rewind,
                    "plugins",
                    &["particles", "preview"],
                ))
                .with_payload_schema_id("particles.preview_rewind.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
            EditorCommandDescriptor::operation(preview_warmup.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &preview_warmup,
                    "plugins",
                    &["particles", "preview"],
                ))
                .with_payload_schema_id("particles.preview_warmup.v1")
                .with_callable_from_remote(false)
                .with_required_capabilities([PARTICLES_AUTHORING_CAPABILITY]),
        ],
        menu_items: Vec::new(),
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
