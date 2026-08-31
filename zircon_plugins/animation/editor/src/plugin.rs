use zircon_editor::core::extension::InspectorCustomizationDescriptor;
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_plugin_sdk::{authoring_plugin, EditorPluginDeclaration};

use crate::capability::{EDITOR_CAPABILITIES, PLUGIN_ID};
use crate::extension_ids::{
    ANIMATION_AUTHORING_VIEW_ID, ANIMATION_DRAWER_ID, ANIMATION_TEMPLATE_ID,
};

authoring_plugin! {
    pub struct AnimationEditorPlugin {
        package_id: PLUGIN_ID,
        display_name: "Animation",
        crate_name: "zircon_plugin_animation_editor",
        category: "runtime",
        description: "Animation editor authoring extensions.",
        maturity: zircon_runtime::plugin::PluginMaturity::Beta,
        mirrors_runtime_manifest: zircon_plugin_animation_runtime::package_manifest(),
        capabilities: EDITOR_CAPABILITIES,
        register_extensions: register_animation_authoring_extensions,
    }
}

pub fn editor_plugin_declaration() -> EditorPluginDeclaration {
    editor_plugin().declaration().clone()
}

fn register_animation_authoring_extensions(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    register_authoring_extensions(
        registry,
        EditorAuthoringExtensions {
            drawer_id: ANIMATION_DRAWER_ID,
            drawer_display_name: "Animation Tools",
            template_id: ANIMATION_TEMPLATE_ID,
            template_document: "plugins://animation/editor/authoring.zui",
            surfaces: &[EditorAuthoringSurface::new(
                ANIMATION_AUTHORING_VIEW_ID,
                "Animation",
                "World",
            )],
        },
    )?;
    register_authoring_contribution_batch(registry, animation_asset_authoring_batch())
}

fn animation_asset_authoring_batch() -> EditorAuthoringContributionBatch {
    EditorAuthoringContributionBatch {
        inspector_customizations: vec![
            InspectorCustomizationDescriptor::new(
                "animation.Asset.BlendSpace1D",
                "plugins://animation/editor/blend_space_1d.zui",
                "animation.editor.blend_space_1d",
            ),
            InspectorCustomizationDescriptor::new(
                "animation.Asset.BlendSpace2D",
                "plugins://animation/editor/blend_space_2d.zui",
                "animation.editor.blend_space_2d",
            ),
            InspectorCustomizationDescriptor::new(
                "animation.Asset.AvatarMask",
                "plugins://animation/editor/avatar_mask_bone_tree.zui",
                "animation.editor.avatar_mask_bone_tree",
            ),
        ],
        ..Default::default()
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    editor_plugin_declaration().descriptor().clone()
}

pub fn editor_plugin() -> AnimationEditorPlugin {
    AnimationEditorPlugin::new()
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
