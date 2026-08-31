use std::sync::{Arc, Mutex};

use zircon_editor::core::asset::{
    AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId, AssetTypePresentation,
    ThumbnailProviderDescriptor,
};
use zircon_editor::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};
use zircon_editor::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodeDescriptor, GraphNodePaletteDescriptor,
};
use zircon_editor::core::editor_extension::{
    AssetImporterDescriptor, EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_ai_runtime::behavior_tree::{standard_node_catalog, BehaviorNodeCategory};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_plugin_sdk::{authoring_plugin, EditorPluginDeclaration};

use crate::capability::{AI_AUTHORING_CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
use crate::extension_ids::{
    AI_BEHAVIOR_TREE_ASSET_TYPE, AI_BEHAVIOR_TREE_COMPILE_OPERATION,
    AI_BEHAVIOR_TREE_IMPORT_OPERATION, AI_BEHAVIOR_TREE_OPEN_OPERATION,
    AI_BEHAVIOR_TREE_PALETTE_ID, AI_BEHAVIOR_TREE_TEMPLATE_ID, AI_BEHAVIOR_TREE_VALIDATE_OPERATION,
    AI_BEHAVIOR_TREE_VIEW_ID,
};
use crate::overlay::register_ai_perception_overlay;
use crate::runtime_mirror::{
    ai_runtime_event_consumers, AiBtNodeResultMirror, AiPieMirror, AI_BEHAVIOR_DEBUG_CONSUMER_ID,
    AI_BT_NODE_RESULT_CONSUMER_ID,
};

authoring_plugin! {
    pub struct AiEditorPlugin {
        package_id: PLUGIN_ID,
        display_name: "AI",
        crate_name: "zircon_plugin_ai_editor",
        category: "runtime",
        description: "AI behavior-tree authoring extensions.",
        maturity: zircon_runtime::plugin::PluginMaturity::Experimental,
        mirrors_runtime_manifest: zircon_plugin_ai_runtime::package_manifest(),
        capabilities: EDITOR_CAPABILITIES,
        runtime_event_consumers: ai_runtime_event_consumers(),
        register_extensions: register_ai_authoring_extensions,
    }
}

pub fn editor_plugin_declaration() -> EditorPluginDeclaration {
    editor_plugin().declaration().clone()
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    editor_plugin_declaration().descriptor().clone()
}

pub fn editor_plugin() -> AiEditorPlugin {
    AiEditorPlugin::new()
}

impl AiEditorPlugin {
    pub fn pie_mirror(&self) -> Arc<Mutex<AiPieMirror>> {
        self.declaration()
            .runtime_event_consumers()
            .registration(AI_BEHAVIOR_DEBUG_CONSUMER_ID)
            .and_then(|registration| registration.state::<AiPieMirror>())
            .expect("AI behavior debug PIE mirror declaration is registered")
    }

    pub fn node_result_mirror(&self) -> Arc<Mutex<AiBtNodeResultMirror>> {
        self.declaration()
            .runtime_event_consumers()
            .registration(AI_BT_NODE_RESULT_CONSUMER_ID)
            .and_then(|registration| registration.state::<AiBtNodeResultMirror>())
            .expect("AI behavior-tree node result mirror declaration is registered")
    }
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

fn register_ai_authoring_extensions(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    register_authoring_extensions(
        registry,
        EditorAuthoringExtensions {
            drawer_id: "ai.drawer",
            drawer_display_name: "AI Tools",
            template_id: AI_BEHAVIOR_TREE_TEMPLATE_ID,
            template_document: "plugins://ai/editor/behavior_tree.zui",
            surfaces: &[EditorAuthoringSurface::new(
                AI_BEHAVIOR_TREE_VIEW_ID,
                "Behavior Tree",
                "AI",
            )],
        },
    )?;
    register_authoring_contribution_batch(registry, ai_behavior_tree_authoring_batch()?)?;
    register_ai_perception_overlay(registry)
}

fn ai_behavior_tree_authoring_batch(
) -> Result<EditorAuthoringContributionBatch, EditorExtensionRegistryError> {
    let asset_type = AssetTypeId::parse(AI_BEHAVIOR_TREE_ASSET_TYPE)?;
    let import = operation(AI_BEHAVIOR_TREE_IMPORT_OPERATION)?;
    let open = operation(AI_BEHAVIOR_TREE_OPEN_OPERATION)?;
    let validate = operation(AI_BEHAVIOR_TREE_VALIDATE_OPERATION)?;
    let compile = operation(AI_BEHAVIOR_TREE_COMPILE_OPERATION)?;
    let capability = [AI_AUTHORING_CAPABILITY];

    Ok(EditorAuthoringContributionBatch {
        commands: vec![
            EditorCommandDescriptor::operation(import.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(&import, "plugins", &["ai"]))
                .with_payload_schema_id("ai.behavior_tree.import.v1")
                .with_required_capabilities(capability),
            EditorCommandDescriptor::operation(open.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(&open, "plugins", &["ai"]))
                .with_payload_schema_id("ai.behavior_tree.open.v1")
                .with_required_capabilities(capability),
            EditorCommandDescriptor::operation(validate.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &validate,
                    "plugins",
                    &["ai"],
                ))
                .with_payload_schema_id("ai.behavior_tree.validate.v1")
                .with_required_capabilities(capability),
            EditorCommandDescriptor::operation(compile.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(&compile, "plugins", &["ai"]))
                .with_payload_schema_id("ai.behavior_tree.compile.v1")
                .with_required_capabilities(capability),
        ],
        menu_items: Vec::new(),
        asset_importers: vec![AssetImporterDescriptor::new(
            "ai.behavior_tree.importer",
            "Behavior Tree",
            import,
        )
        .with_source_extension("btree.toml")
        .with_output_type(asset_type.clone())
        .with_required_capabilities(capability)],
        asset_type_contributions: vec![AssetTypeContribution::define(
            asset_type.clone(),
            AssetTypePresentation::new(
                "Behavior Tree",
                "BT",
                "asset-ai-behavior-tree",
                "asset.ai.behavior_tree",
            ),
            ThumbnailProviderDescriptor::Icon("asset-ai-behavior-tree".to_owned()),
        )
        .with_toolkit(
            AssetToolkitDescriptor::new(AI_BEHAVIOR_TREE_VIEW_ID, open.clone())
                .with_required_capabilities(capability),
        )],
        graph_editors: vec![GraphEditorDescriptor::new(
            asset_type.clone(),
            AI_BEHAVIOR_TREE_VIEW_ID,
            "Behavior Tree",
            open,
            validate,
        )
        .with_compile_operation(compile)
        .with_required_capabilities(capability)],
        graph_node_palettes: vec![behavior_tree_palette(asset_type)?],
        ..Default::default()
    })
}

fn behavior_tree_palette(
    asset_type: AssetTypeId,
) -> Result<GraphNodePaletteDescriptor, EditorExtensionRegistryError> {
    let catalog = standard_node_catalog().map_err(|error| {
        EditorExtensionRegistryError::View(format!(
            "AI behavior-tree standard node catalog is unavailable: {error}"
        ))
    })?;
    let mut palette = GraphNodePaletteDescriptor::new(AI_BEHAVIOR_TREE_PALETTE_ID, asset_type)
        .with_required_capabilities([AI_AUTHORING_CAPABILITY]);
    for descriptor in catalog.descriptors() {
        palette = palette.with_node(GraphNodeDescriptor::new(
            descriptor.id(),
            descriptor.display_name(),
            category_name(descriptor.category()),
        ));
    }
    Ok(palette)
}

fn category_name(category: BehaviorNodeCategory) -> &'static str {
    match category {
        BehaviorNodeCategory::Composite => "Composite",
        BehaviorNodeCategory::Decorator => "Decorator",
        BehaviorNodeCategory::Service => "Service",
        BehaviorNodeCategory::Task => "Task",
    }
}

fn operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::OperationPath)
}
