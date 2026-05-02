use zircon_editor::core::editor_authoring_extension::{
    AssetCreationTemplateDescriptor, ViewportToolModeDescriptor,
};
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor,
};
use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};

pub const PLUGIN_ID: &str = zircon_plugin_terrain_runtime::PLUGIN_ID;
pub const CAPABILITY: &str = "editor.extension.terrain_authoring";
pub const TERRAIN_AUTHORING_VIEW_ID: &str = "terrain.authoring";
pub const TERRAIN_DRAWER_ID: &str = "terrain.drawer";
pub const TERRAIN_TEMPLATE_ID: &str = "terrain.authoring";

#[derive(Clone, Debug)]
pub struct TerrainEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl TerrainEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for TerrainEditorPlugin {
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
                drawer_id: TERRAIN_DRAWER_ID,
                drawer_display_name: "Terrain Tools",
                template_id: TERRAIN_TEMPLATE_ID,
                template_document: "plugins://terrain/editor/authoring.ui.toml",
                surfaces: &[EditorAuthoringSurface::new(
                    TERRAIN_AUTHORING_VIEW_ID,
                    "Terrain",
                    "World",
                    "Plugins/Terrain",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, terrain_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(PLUGIN_ID, "Terrain", "zircon_plugin_terrain_editor")
        .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> TerrainEditorPlugin {
    TerrainEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_terrain_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_terrain_runtime::package_manifest(),
    )
}

fn terrain_authoring_batch() -> EditorAuthoringContributionBatch {
    let import_heightfield = operation("Terrain.Authoring.ImportHeightfield");
    let import_weightmap = operation("Terrain.Authoring.ImportWeightmap");
    let create = operation("Terrain.Authoring.CreateHeightfield");
    let open = operation("Terrain.Authoring.Open");
    let sculpt = operation("Terrain.Authoring.Sculpt");
    EditorAuthoringContributionBatch {
        operations: vec![
            EditorOperationDescriptor::new(
                import_heightfield.clone(),
                "Import Terrain Heightfield",
            )
            .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(import_weightmap.clone(), "Import Terrain Weightmap")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(create.clone(), "Create Terrain Heightfield")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(open.clone(), "Open Terrain")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(sculpt.clone(), "Activate Terrain Sculpt Tool")
                .with_required_capabilities([CAPABILITY]),
        ],
        asset_importers: vec![
            AssetImporterDescriptor::new(
                "terrain.heightfield.importer",
                "Terrain Heightfield",
                import_heightfield,
            )
            .with_source_extensions(["raw", "r16", "png"])
            .with_output_kind("terrain.heightfield")
            .with_required_capabilities([CAPABILITY]),
            AssetImporterDescriptor::new(
                "terrain.weightmap.importer",
                "Terrain Weightmap",
                import_weightmap,
            )
            .with_source_extensions(["raw", "r16", "png"])
            .with_output_kind("terrain.layer_stack")
            .with_required_capabilities([CAPABILITY]),
        ],
        asset_editors: vec![AssetEditorDescriptor::new(
            "terrain.heightfield",
            TERRAIN_AUTHORING_VIEW_ID,
            "Terrain",
            open,
        )
        .with_required_capabilities([CAPABILITY])],
        component_drawers: vec![ComponentDrawerDescriptor::new(
            zircon_plugin_terrain_runtime::TERRAIN_COMPONENT_TYPE,
            "plugins://terrain/editor/terrain_component.ui.toml",
            "terrain.editor.component",
        )],
        asset_creation_templates: vec![AssetCreationTemplateDescriptor::new(
            "terrain.template.heightfield",
            "Terrain Heightfield",
            "terrain.heightfield",
            create,
        )
        .with_default_document("plugins://terrain/templates/default_heightfield.toml")
        .with_required_capabilities([CAPABILITY])],
        viewport_tool_modes: vec![ViewportToolModeDescriptor::new(
            "terrain.tool.sculpt",
            "Sculpt Terrain",
            TERRAIN_AUTHORING_VIEW_ID,
            sculpt,
        )
        .with_required_capabilities([CAPABILITY])],
        ..Default::default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainHeightfieldImportRequest {
    pub width: u32,
    pub height: u32,
    pub sample_count: Option<usize>,
    pub source_extension: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainImportKind {
    Heightfield,
    LayerStack,
}

impl TerrainImportKind {
    pub fn output_kind(self) -> &'static str {
        match self {
            Self::Heightfield => "terrain.heightfield",
            Self::LayerStack => "terrain.layer_stack",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainImportPlan {
    pub import_kind: TerrainImportKind,
    pub normalized_extension: String,
    pub output_kind: &'static str,
    pub expected_sample_count: usize,
}

pub fn validate_heightfield_import(request: &TerrainHeightfieldImportRequest) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if request.width == 0 || request.height == 0 {
        diagnostics.push("terrain heightfield dimensions must be greater than zero".to_string());
    }
    let extension = normalized_terrain_import_extension(&request.source_extension);
    if !terrain_import_extension_supported(&extension) {
        diagnostics.push(format!(
            "terrain heightfield extension `{}` is not supported",
            request.source_extension
        ));
    }
    if let Some(sample_count) = request.sample_count {
        let expected = request.width as usize * request.height as usize;
        if expected != sample_count {
            diagnostics.push(format!(
                "terrain heightfield import expected {} samples for {}x{} source but found {}",
                expected, request.width, request.height, sample_count
            ));
        }
    }
    diagnostics
}

pub fn plan_terrain_import(
    import_kind: TerrainImportKind,
    request: &TerrainHeightfieldImportRequest,
) -> Result<TerrainImportPlan, Vec<String>> {
    let diagnostics = validate_heightfield_import(request);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let normalized_extension = normalized_terrain_import_extension(&request.source_extension);
    Ok(TerrainImportPlan {
        import_kind,
        normalized_extension,
        output_kind: import_kind.output_kind(),
        expected_sample_count: request.width as usize * request.height as usize,
    })
}

pub fn terrain_import_output_kind(source_extension: &str) -> Option<&'static str> {
    match normalized_terrain_import_extension(source_extension).as_str() {
        "raw" | "r16" | "png" => Some("terrain.heightfield"),
        _ => None,
    }
}

fn normalized_terrain_import_extension(source_extension: &str) -> String {
    source_extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

fn terrain_import_extension_supported(extension: &str) -> bool {
    matches!(extension, "raw" | "r16" | "png")
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid terrain operation path")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_heightfield_import_accepts_supported_extensions_and_matching_samples() {
        let diagnostics = validate_heightfield_import(&TerrainHeightfieldImportRequest {
            width: 4,
            height: 4,
            sample_count: Some(16),
            source_extension: ".r16".to_string(),
        });

        assert!(diagnostics.is_empty());
        assert_eq!(
            terrain_import_output_kind("png"),
            Some("terrain.heightfield")
        );
    }

    #[test]
    fn terrain_import_plan_selects_heightfield_or_layer_stack_output() {
        let request = TerrainHeightfieldImportRequest {
            width: 8,
            height: 4,
            sample_count: Some(32),
            source_extension: ".PNG".to_string(),
        };

        let heightfield = plan_terrain_import(TerrainImportKind::Heightfield, &request)
            .expect("heightfield import request is valid");
        let layer_stack = plan_terrain_import(TerrainImportKind::LayerStack, &request)
            .expect("layer stack import request is valid");

        assert_eq!(heightfield.normalized_extension, "png");
        assert_eq!(heightfield.output_kind, "terrain.heightfield");
        assert_eq!(layer_stack.output_kind, "terrain.layer_stack");
        assert_eq!(heightfield.expected_sample_count, 32);
    }

    #[test]
    fn terrain_heightfield_import_reports_invalid_dimensions_extension_and_sample_count() {
        let mut diagnostics = validate_heightfield_import(&TerrainHeightfieldImportRequest {
            width: 0,
            height: 4,
            sample_count: None,
            source_extension: "exr".to_string(),
        });
        diagnostics.extend(validate_heightfield_import(
            &TerrainHeightfieldImportRequest {
                width: 2,
                height: 4,
                sample_count: Some(5),
                source_extension: "raw".to_string(),
            },
        ));

        assert!(diagnostics
            .iter()
            .any(|message| message.contains("dimensions must be greater")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("extension `exr` is not supported")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("expected 8 samples")));
    }
}
