use zircon_editor::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodeDescriptor, GraphNodePaletteDescriptor, GraphPinDescriptor,
};
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, ComponentDrawerDescriptor, EditorMenuItemDescriptor,
};
use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform,
    plugin::PluginDistributionManifest, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest,
};

use crate::capability::{CAPABILITY, PLUGIN_ID};
use crate::extension_ids::{
    ANIMATION_GRAPH_DRAWER_ID, ANIMATION_GRAPH_TEMPLATE_ID, ANIMATION_GRAPH_VIEW_ID,
};

pub const ANIMATION_GRAPH_DIST_CRATE_NAME: &str = "zircon_plugin_animation_graph_dist";
pub const ANIMATION_GRAPH_DIST_EDITOR_ENTRY: &str = "zircon_plugin_animation_graph_editor_entry_v3";
const ANIMATION_GRAPH_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct AnimationGraphEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl AnimationGraphEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for AnimationGraphEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: ANIMATION_GRAPH_DRAWER_ID,
                drawer_display_name: "Animation Graph",
                template_id: ANIMATION_GRAPH_TEMPLATE_ID,
                template_document: "plugins://animation_graph/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    ANIMATION_GRAPH_VIEW_ID,
                    "Animation Graph",
                    "Animation",
                    "Plugins/Animation Graph",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, animation_graph_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Animation Graph",
        "zircon_plugin_animation_graph_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> AnimationGraphEditorPlugin {
    AnimationGraphEditorPlugin::new()
}

fn base_manifest() -> PluginPackageManifest {
    PluginPackageManifest::new(PLUGIN_ID, "Animation Graph")
        .with_category("authoring")
        .with_supported_targets([RuntimeTargetMode::EditorHost])
        .with_supported_platforms([
            ExportTargetPlatform::Windows,
            ExportTargetPlatform::Linux,
            ExportTargetPlatform::Macos,
        ])
        .with_capabilities([CAPABILITY])
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ])
        .with_dependency(
            zircon_runtime::plugin::PluginDependencyManifest::new("animation", true)
                .with_capability("runtime.plugin.animation"),
        )
        .with_native_module(animation_graph_dist_module_manifest())
        .with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: ANIMATION_GRAPH_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: ANIMATION_GRAPH_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            editor_entry: ANIMATION_GRAPH_DIST_EDITOR_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_manifest())
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(&editor_plugin(), base_manifest())
}

pub fn animation_graph_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native("animation_graph.dist", ANIMATION_GRAPH_DIST_CRATE_NAME)
        .with_target_modes([RuntimeTargetMode::EditorHost])
        .with_capabilities([CAPABILITY])
}

fn animation_graph_authoring_batch() -> EditorAuthoringContributionBatch {
    let open_graph = operation("animation_graph.authoring.open_graph");
    let open_state_machine = operation("animation_graph.authoring.open_state_machine");
    let validate = operation("animation_graph.authoring.validate");
    let compile = operation("animation_graph.authoring.compile");
    EditorAuthoringContributionBatch {
        operations: vec![
            EditorOperationDescriptor::new(open_graph.clone(), "Open Animation Graph")
                .with_menu_path("Plugins/Animation Graph/Open Graph")
                .with_payload_schema_id("animation_graph.open_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(
                open_state_machine.clone(),
                "Open Animation State Machine",
            )
            .with_menu_path("Plugins/Animation Graph/Open State Machine")
            .with_payload_schema_id("animation_graph.open_state_machine.v1")
            .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(validate.clone(), "Validate Animation Graph")
                .with_menu_path("Plugins/Animation Graph/Validate")
                .with_payload_schema_id("animation_graph.validate.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(compile.clone(), "Compile Animation Graph")
                .with_menu_path("Plugins/Animation Graph/Compile")
                .with_payload_schema_id("animation_graph.compile.v1")
                .with_required_capabilities([CAPABILITY]),
        ],
        menu_items: vec![
            menu_item("Plugins/Animation Graph/Open Graph", &open_graph),
            menu_item(
                "Plugins/Animation Graph/Open State Machine",
                &open_state_machine,
            ),
            menu_item("Plugins/Animation Graph/Validate", &validate),
            menu_item("Plugins/Animation Graph/Compile", &compile),
        ],
        asset_editors: vec![
            AssetEditorDescriptor::new(
                "animation.graph",
                ANIMATION_GRAPH_VIEW_ID,
                "Animation Graph",
                open_graph.clone(),
            )
            .with_required_capabilities([CAPABILITY]),
            AssetEditorDescriptor::new(
                "animation.state_machine",
                ANIMATION_GRAPH_VIEW_ID,
                "Animation State Machine",
                open_state_machine,
            )
            .with_required_capabilities([CAPABILITY]),
        ],
        graph_editors: vec![
            GraphEditorDescriptor::new(
                "animation.graph",
                ANIMATION_GRAPH_VIEW_ID,
                "Animation Graph",
                open_graph,
                validate.clone(),
            )
            .with_compile_operation(compile.clone())
            .with_required_capabilities([CAPABILITY]),
            GraphEditorDescriptor::new(
                "animation.state_machine",
                ANIMATION_GRAPH_VIEW_ID,
                "Animation State Machine",
                operation("animation_graph.authoring.open_state_machine"),
                validate,
            )
            .with_compile_operation(compile)
            .with_required_capabilities([CAPABILITY]),
        ],
        graph_node_palettes: vec![animation_graph_palette(), animation_state_machine_palette()],
        component_drawers: vec![
            ComponentDrawerDescriptor::new(
                "animation.Component.GraphPlayer",
                "plugins://animation_graph/editor/graph_player.zui",
                "animation_graph.editor.graph_player",
            ),
            ComponentDrawerDescriptor::new(
                "animation.Component.StateMachinePlayer",
                "plugins://animation_graph/editor/state_machine_player.zui",
                "animation_graph.editor.state_machine_player",
            ),
        ],
        ..Default::default()
    }
}

fn animation_graph_palette() -> GraphNodePaletteDescriptor {
    GraphNodePaletteDescriptor::new("animation_graph.palette.graph", "animation.graph")
        .with_node(
            GraphNodeDescriptor::new("clip", "Clip", "Playback")
                .with_output(GraphPinDescriptor::new("pose", "pose")),
        )
        .with_node(
            GraphNodeDescriptor::new("blend", "Blend", "Blend")
                .with_input(GraphPinDescriptor::new("a", "pose").required(true))
                .with_input(GraphPinDescriptor::new("b", "pose").required(true))
                .with_output(GraphPinDescriptor::new("pose", "pose")),
        )
        .with_node(
            GraphNodeDescriptor::new("output", "Output", "Output")
                .with_input(GraphPinDescriptor::new("pose", "pose").required(true)),
        )
        .with_required_capabilities([CAPABILITY])
}

fn animation_state_machine_palette() -> GraphNodePaletteDescriptor {
    GraphNodePaletteDescriptor::new(
        "animation_graph.palette.state_machine",
        "animation.state_machine",
    )
    .with_node(GraphNodeDescriptor::new("state", "State", "State"))
    .with_node(GraphNodeDescriptor::new(
        "transition",
        "Transition",
        "Transition",
    ))
    .with_node(GraphNodeDescriptor::new(
        "condition",
        "Condition",
        "Transition",
    ))
    .with_required_capabilities([CAPABILITY])
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid animation graph operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone()).with_required_capabilities([CAPABILITY])
}
