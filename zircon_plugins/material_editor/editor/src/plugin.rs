use zircon_editor::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId,
};
use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodeDescriptor, GraphNodePaletteDescriptor, GraphPinDescriptor,
};
use zircon_editor::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::{
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ExportTargetPlatform, plugin::PluginDistributionManifest,
    plugin::PluginModuleManifest, plugin::PluginPackageManifest,
};
use zircon_runtime_interface::resource::ResourceKind;

use crate::capability::{CAPABILITY, PLUGIN_ID};
use crate::extension_ids::{
    MATERIAL_EDITOR_DRAWER_ID, MATERIAL_EDITOR_TEMPLATE_ID, MATERIAL_EDITOR_VIEW_ID,
};

pub const MATERIAL_EDITOR_DIST_CRATE_NAME: &str = "zircon_plugin_material_editor_dist";
pub const MATERIAL_EDITOR_DIST_EDITOR_ENTRY: &str = "zircon_plugin_material_editor_editor_entry_v3";
const MATERIAL_EDITOR_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct MaterialEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl MaterialEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for MaterialEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: MATERIAL_EDITOR_DRAWER_ID,
                drawer_display_name: "Material Editor",
                template_id: MATERIAL_EDITOR_TEMPLATE_ID,
                template_document: "plugins://material_editor/editor/graph.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    MATERIAL_EDITOR_VIEW_ID,
                    "Material Editor",
                    "Assets",
                    "Plugins/Material Editor",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, material_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Material Editor",
        "zircon_plugin_material_editor_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> MaterialEditorPlugin {
    MaterialEditorPlugin::new()
}

fn base_manifest() -> PluginPackageManifest {
    PluginPackageManifest::new(PLUGIN_ID, "Material Editor")
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
        .with_native_module(material_editor_dist_module_manifest())
        .with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: MATERIAL_EDITOR_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: MATERIAL_EDITOR_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            editor_entry: MATERIAL_EDITOR_DIST_EDITOR_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
}

pub fn package_manifest() -> PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_manifest())
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(&editor_plugin(), base_manifest())
}

pub fn material_editor_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native("material_editor.dist", MATERIAL_EDITOR_DIST_CRATE_NAME)
        .with_target_modes([RuntimeTargetMode::EditorHost])
        .with_capabilities([CAPABILITY])
}

fn material_authoring_batch() -> EditorAuthoringContributionBatch {
    let open_graph = operation("material_editor.graph.open");
    let open_material = operation("material_editor.material.open");
    let validate = operation("material_editor.graph.validate");
    let compile = operation("material_editor.graph.compile");
    let preview = operation("material_editor.graph.preview");
    let create = operation("material_editor.graph.create");
    EditorAuthoringContributionBatch {
        commands: vec![
            EditorCommandDescriptor::pending_operation(open_graph.clone(), "Open Material Graph")
                .with_menu_path("Plugins/Material Editor/Open Graph")
                .with_payload_schema_id("material_editor.open_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(open_material.clone(), "Open Material")
                .with_menu_path("Plugins/Material Editor/Open Material")
                .with_payload_schema_id("material_editor.open_material.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(validate.clone(), "Validate Material Graph")
                .with_menu_path("Plugins/Material Editor/Validate Graph")
                .with_payload_schema_id("material_editor.validate_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(compile.clone(), "Compile Material Graph")
                .with_menu_path("Plugins/Material Editor/Compile Graph")
                .with_payload_schema_id("material_editor.compile_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(preview.clone(), "Preview Material Graph")
                .with_menu_path("Plugins/Material Editor/Preview Graph")
                .with_payload_schema_id("material_editor.preview_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorCommandDescriptor::pending_operation(create.clone(), "Create Material Graph")
                .with_menu_path("Plugins/Material Editor/Create Graph")
                .with_payload_schema_id("material_editor.create_graph.v1")
                .with_required_capabilities([CAPABILITY]),
        ],
        menu_items: vec![
            menu_item("Plugins/Material Editor/Open Graph", &open_graph),
            menu_item("Plugins/Material Editor/Open Material", &open_material),
            menu_item("Plugins/Material Editor/Validate Graph", &validate),
            menu_item("Plugins/Material Editor/Compile Graph", &compile),
            menu_item("Plugins/Material Editor/Preview Graph", &preview),
            menu_item("Plugins/Material Editor/Create Graph", &create),
        ],
        asset_type_contributions: vec![
            AssetTypeContribution::augment(AssetTypeId::from_resource_kind(
                ResourceKind::MaterialGraph,
            ))
            .with_toolkit(
                AssetToolkitDescriptor::new(MATERIAL_EDITOR_VIEW_ID, open_graph.clone())
                    .with_required_capabilities([CAPABILITY]),
            )
            .with_creation_template(
                AssetCreationTemplateDescriptor::new(
                    "material_editor.template.graph",
                    "Material Graph",
                    create,
                )
                .with_default_document(
                    "plugins://material_editor/templates/default_material_graph.toml",
                )
                .with_required_capabilities([CAPABILITY]),
            ),
            AssetTypeContribution::augment(AssetTypeId::from_resource_kind(ResourceKind::Material))
                .with_toolkit(
                    AssetToolkitDescriptor::new(MATERIAL_EDITOR_VIEW_ID, open_material)
                        .with_required_capabilities([CAPABILITY]),
                ),
        ],
        graph_editors: vec![GraphEditorDescriptor::new(
            AssetTypeId::from_resource_kind(ResourceKind::MaterialGraph),
            MATERIAL_EDITOR_VIEW_ID,
            "Material Graph",
            open_graph,
            validate,
        )
        .with_compile_operation(compile)
        .with_required_capabilities([CAPABILITY])],
        graph_node_palettes: vec![material_node_palette()],
        ..Default::default()
    }
}

fn material_node_palette() -> GraphNodePaletteDescriptor {
    GraphNodePaletteDescriptor::new(
        "material_editor.palette",
        AssetTypeId::from_resource_kind(ResourceKind::MaterialGraph),
    )
    .with_node(
        GraphNodeDescriptor::new("output", "Output", "Material")
            .with_input(GraphPinDescriptor::new("base_color", "vec4").required(true)),
    )
    .with_node(
        GraphNodeDescriptor::new("texture_sample", "Texture Sample", "Texture")
            .with_output(GraphPinDescriptor::new("color", "vec4")),
    )
    .with_node(
        GraphNodeDescriptor::new("scalar_parameter", "Scalar Parameter", "Parameter")
            .with_output(GraphPinDescriptor::new("value", "float")),
    )
    .with_node(
        GraphNodeDescriptor::new("vector_parameter", "Vector Parameter", "Parameter")
            .with_output(GraphPinDescriptor::new("value", "vec4")),
    )
    .with_node(
        GraphNodeDescriptor::new("add", "Add", "Math")
            .with_input(GraphPinDescriptor::new("a", "float").required(true))
            .with_input(GraphPinDescriptor::new("b", "float").required(true))
            .with_output(GraphPinDescriptor::new("value", "float")),
    )
    .with_node(
        GraphNodeDescriptor::new("multiply", "Multiply", "Math")
            .with_input(GraphPinDescriptor::new("a", "float").required(true))
            .with_input(GraphPinDescriptor::new("b", "float").required(true))
            .with_output(GraphPinDescriptor::new("value", "float")),
    )
    .with_required_capabilities([CAPABILITY])
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid material operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone()).with_required_capabilities([CAPABILITY])
}
