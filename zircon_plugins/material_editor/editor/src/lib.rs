use std::collections::{BTreeMap, BTreeSet};

use zircon_editor::core::editor_authoring_extension::{
    AssetCreationTemplateDescriptor, GraphEditorDescriptor, GraphNodeDescriptor,
    GraphNodePaletteDescriptor, GraphPinDescriptor,
};
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, EditorExtensionRegistry, EditorMenuItemDescriptor,
};
use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::asset::{
    AlphaMode, AssetReference, MaterialAsset, MaterialGraphAsset, MaterialGraphLinkAsset,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset, MaterialGraphParameterAsset,
};

pub const PLUGIN_ID: &str = "material_editor";
pub const CAPABILITY: &str = "editor.extension.material_editor_authoring";
pub const MATERIAL_EDITOR_VIEW_ID: &str = "material_editor.graph";
pub const MATERIAL_EDITOR_DRAWER_ID: &str = "material_editor.drawer";
pub const MATERIAL_EDITOR_TEMPLATE_ID: &str = "material_editor.graph";

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
                template_document: "plugins://material_editor/editor/graph.ui.toml",
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

fn base_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::PluginPackageManifest::new(PLUGIN_ID, "Material Editor")
        .with_category("authoring")
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

fn material_authoring_batch() -> EditorAuthoringContributionBatch {
    let open_graph = operation("MaterialEditor.Graph.Open");
    let open_material = operation("MaterialEditor.Material.Open");
    let validate = operation("MaterialEditor.Graph.Validate");
    let compile = operation("MaterialEditor.Graph.Compile");
    let preview = operation("MaterialEditor.Graph.Preview");
    let create = operation("MaterialEditor.Graph.Create");
    EditorAuthoringContributionBatch {
        operations: vec![
            EditorOperationDescriptor::new(open_graph.clone(), "Open Material Graph")
                .with_menu_path("Plugins/Material Editor/Open Graph")
                .with_payload_schema_id("material_editor.open_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(open_material.clone(), "Open Material")
                .with_menu_path("Plugins/Material Editor/Open Material")
                .with_payload_schema_id("material_editor.open_material.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(validate.clone(), "Validate Material Graph")
                .with_menu_path("Plugins/Material Editor/Validate Graph")
                .with_payload_schema_id("material_editor.validate_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(compile.clone(), "Compile Material Graph")
                .with_menu_path("Plugins/Material Editor/Compile Graph")
                .with_payload_schema_id("material_editor.compile_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(preview.clone(), "Preview Material Graph")
                .with_menu_path("Plugins/Material Editor/Preview Graph")
                .with_payload_schema_id("material_editor.preview_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(create.clone(), "Create Material Graph")
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
        asset_editors: vec![
            AssetEditorDescriptor::new(
                "material.graph",
                MATERIAL_EDITOR_VIEW_ID,
                "Material Graph",
                open_graph.clone(),
            )
            .with_required_capabilities([CAPABILITY]),
            AssetEditorDescriptor::new(
                "material",
                MATERIAL_EDITOR_VIEW_ID,
                "Material",
                open_material,
            )
            .with_required_capabilities([CAPABILITY]),
        ],
        graph_editors: vec![GraphEditorDescriptor::new(
            "material.graph",
            MATERIAL_EDITOR_VIEW_ID,
            "Material Graph",
            open_graph,
            validate,
        )
        .with_compile_operation(compile)
        .with_required_capabilities([CAPABILITY])],
        graph_node_palettes: vec![material_node_palette()],
        asset_creation_templates: vec![AssetCreationTemplateDescriptor::new(
            "material_editor.template.graph",
            "Material Graph",
            "material.graph",
            create,
        )
        .with_default_document("plugins://material_editor/templates/default_material_graph.toml")
        .with_required_capabilities([CAPABILITY])],
        ..Default::default()
    }
}

fn material_node_palette() -> GraphNodePaletteDescriptor {
    GraphNodePaletteDescriptor::new("material_editor.palette", "material.graph")
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

pub fn validate_material_graph(graph: &MaterialGraphAsset) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut node_ids = BTreeSet::new();
    let mut output_nodes = Vec::new();

    for node in &graph.nodes {
        if node.id.trim().is_empty() {
            diagnostics.push("material graph node id must not be empty".to_string());
            continue;
        }
        if !seen_ids.insert(node.id.as_str()) {
            diagnostics.push(format!("material graph has duplicate node `{}`", node.id));
        }
        node_ids.insert(node.id.as_str());
        if matches!(&node.kind, MaterialGraphNodeKindAsset::Output) {
            output_nodes.push(node.id.as_str());
        }
    }

    match output_nodes.len() {
        0 => diagnostics.push(format!(
            "material graph `{}` has no output node",
            graph.name
        )),
        1 => {
            if incoming_link(graph, output_nodes[0], "base_color").is_none() {
                diagnostics.push(format!(
                    "material graph output `{}` is missing required input `base_color`",
                    output_nodes[0]
                ));
            }
        }
        _ => diagnostics.push("material graph must contain exactly one output node".to_string()),
    }

    for link in &graph.links {
        if !node_ids.contains(link.from_node.as_str()) {
            diagnostics.push(format!(
                "material graph link references missing source node `{}`",
                link.from_node
            ));
        }
        if !node_ids.contains(link.to_node.as_str()) {
            diagnostics.push(format!(
                "material graph link references missing target node `{}`",
                link.to_node
            ));
        }
        if link.from_pin.trim().is_empty() || link.to_pin.trim().is_empty() {
            diagnostics.push("material graph link pins must not be empty".to_string());
        }
    }

    let nodes = material_node_map(graph);
    for node in &graph.nodes {
        if matches!(
            &node.kind,
            MaterialGraphNodeKindAsset::Add | MaterialGraphNodeKindAsset::Multiply
        ) {
            for pin in ["a", "b"] {
                if incoming_link(graph, &node.id, pin).is_none() {
                    diagnostics.push(format!(
                        "material graph node `{}` is missing required input `{}`",
                        node.id, pin
                    ));
                }
            }
        }
        if let MaterialGraphNodeKindAsset::Output = &node.kind {
            if incoming_link(graph, &node.id, "base_color")
                .and_then(|link| nodes.get(link.from_node.as_str()))
                .is_none()
            {
                diagnostics.push(format!(
                    "material graph output `{}` base_color input is disconnected",
                    node.id
                ));
            }
        }
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn compile_material_graph(graph: &MaterialGraphAsset) -> Result<MaterialAsset, Vec<String>> {
    let mut diagnostics = validate_material_graph(graph);
    let Some(shader) = graph.shader.clone() else {
        diagnostics.push(format!(
            "material graph `{}` has no shader target for MaterialAsset compile",
            graph.name
        ));
        return Err(diagnostics);
    };
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let output = graph
        .nodes
        .iter()
        .find(|node| matches!(&node.kind, MaterialGraphNodeKindAsset::Output))
        .expect("validated material graph has an output node");
    let base_color_link =
        incoming_link(graph, &output.id, "base_color").expect("validated output has base_color");
    let mut evaluating = BTreeSet::new();
    let base_color_input = evaluate_color_input(graph, &base_color_link.from_node, &mut evaluating)
        .map_err(|error| vec![error])?;

    let (base_color, base_color_texture) = match base_color_input {
        MaterialColorInput::Constant(value) => (value, None),
        MaterialColorInput::Texture(texture) => ([1.0, 1.0, 1.0, 1.0], Some(texture)),
    };

    Ok(MaterialAsset {
        name: Some(graph.name.clone()),
        shader,
        base_color,
        base_color_texture,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaterialGraphCompileReport {
    pub diagnostics: Vec<String>,
    pub material: Option<MaterialAsset>,
}

pub fn compile_material_graph_operation(graph: &MaterialGraphAsset) -> MaterialGraphCompileReport {
    match compile_material_graph(graph) {
        Ok(material) => MaterialGraphCompileReport {
            diagnostics: Vec::new(),
            material: Some(material),
        },
        Err(diagnostics) => MaterialGraphCompileReport {
            diagnostics,
            material: None,
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
enum MaterialColorInput {
    Constant([f32; 4]),
    Texture(AssetReference),
}

fn evaluate_color_input(
    graph: &MaterialGraphAsset,
    node_id: &str,
    evaluating: &mut BTreeSet<String>,
) -> Result<MaterialColorInput, String> {
    if !evaluating.insert(node_id.to_string()) {
        return Err(format!(
            "material graph contains a cycle at node `{node_id}`"
        ));
    }
    let result = match material_node_map(graph).get(node_id).map(|node| &node.kind) {
        Some(MaterialGraphNodeKindAsset::TextureSample { texture }) => {
            Ok(MaterialColorInput::Texture(texture.clone()))
        }
        Some(MaterialGraphNodeKindAsset::ScalarParameter { name, default }) => {
            let value = match graph.parameters.get(name) {
                Some(MaterialGraphParameterAsset::Scalar(value)) => *value,
                _ => *default,
            };
            Ok(MaterialColorInput::Constant([value, value, value, 1.0]))
        }
        Some(MaterialGraphNodeKindAsset::VectorParameter { name, default }) => {
            let value = match graph.parameters.get(name) {
                Some(MaterialGraphParameterAsset::Vector(value)) => *value,
                _ => *default,
            };
            Ok(MaterialColorInput::Constant(value))
        }
        Some(MaterialGraphNodeKindAsset::Add) => {
            let a = evaluate_color_pin(graph, node_id, "a", evaluating)?;
            let b = evaluate_color_pin(graph, node_id, "b", evaluating)?;
            combine_color_inputs("add", a, b, |left, right| left + right)
        }
        Some(MaterialGraphNodeKindAsset::Multiply) => {
            let a = evaluate_color_pin(graph, node_id, "a", evaluating)?;
            let b = evaluate_color_pin(graph, node_id, "b", evaluating)?;
            combine_color_inputs("multiply", a, b, |left, right| left * right)
        }
        Some(MaterialGraphNodeKindAsset::Output) => Err(format!(
            "material graph output node `{node_id}` cannot feed another node"
        )),
        None => Err(format!(
            "material graph references missing node `{node_id}`"
        )),
    };
    evaluating.remove(node_id);
    result
}

fn evaluate_color_pin(
    graph: &MaterialGraphAsset,
    node_id: &str,
    pin: &str,
    evaluating: &mut BTreeSet<String>,
) -> Result<MaterialColorInput, String> {
    let link = incoming_link(graph, node_id, pin)
        .ok_or_else(|| format!("material graph node `{node_id}` missing input `{pin}`"))?;
    evaluate_color_input(graph, &link.from_node, evaluating)
}

fn combine_color_inputs(
    op: &str,
    a: MaterialColorInput,
    b: MaterialColorInput,
    f: impl Fn(f32, f32) -> f32,
) -> Result<MaterialColorInput, String> {
    match (a, b) {
        (MaterialColorInput::Constant(a), MaterialColorInput::Constant(b)) => {
            Ok(MaterialColorInput::Constant([
                f(a[0], b[0]),
                f(a[1], b[1]),
                f(a[2], b[2]),
                f(a[3], b[3]),
            ]))
        }
        _ => Err(format!(
            "material graph `{op}` node cannot combine texture-backed inputs in v1"
        )),
    }
}

fn material_node_map(graph: &MaterialGraphAsset) -> BTreeMap<&str, &MaterialGraphNodeAsset> {
    graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect()
}

fn incoming_link<'a>(
    graph: &'a MaterialGraphAsset,
    node_id: &str,
    pin: &str,
) -> Option<&'a MaterialGraphLinkAsset> {
    graph
        .links
        .iter()
        .find(|link| link.to_node == node_id && link.to_pin == pin)
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid material operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone()).with_required_capabilities([CAPABILITY])
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_editor::EditorPlugin;
    use zircon_runtime::asset::AssetUri;

    #[test]
    fn material_authoring_registration_exposes_menu_items_and_payload_schemas() {
        let mut registry = EditorExtensionRegistry::default();
        editor_plugin()
            .register_editor_extensions(&mut registry)
            .expect("material authoring registration");
        let operation = operation("MaterialEditor.Graph.Compile");
        let descriptor = registry
            .operations()
            .descriptor(&operation)
            .expect("compile operation registered");

        assert_eq!(
            descriptor.menu_path(),
            Some("Plugins/Material Editor/Compile Graph")
        );
        assert_eq!(
            descriptor.payload_schema_id(),
            Some("material_editor.compile_graph.v1")
        );
        assert!(registry.menu_items().iter().any(|item| {
            item.path() == "Plugins/Material Editor/Compile Graph" && item.operation() == &operation
        }));
    }

    #[test]
    fn material_graph_compile_writes_minimal_material_asset_contract() {
        let mut graph = graph_with_shader();
        graph.nodes.push(node(
            "base",
            MaterialGraphNodeKindAsset::VectorParameter {
                name: "base_color".to_string(),
                default: [0.25, 0.5, 0.75, 1.0],
            },
        ));
        graph
            .nodes
            .push(node("output", MaterialGraphNodeKindAsset::Output));
        graph
            .links
            .push(link("base", "value", "output", "base_color"));

        let material = compile_material_graph(&graph).expect("valid material graph compiles");

        assert_eq!(material.name.as_deref(), Some("Test Material"));
        assert_eq!(material.base_color, [0.25, 0.5, 0.75, 1.0]);
        assert!(material.base_color_texture.is_none());
    }

    #[test]
    fn material_graph_compile_uses_parameter_defaults_and_math_nodes() {
        let mut graph = graph_with_shader();
        graph.parameters.insert(
            "tint".to_string(),
            MaterialGraphParameterAsset::Vector([0.2, 0.3, 0.4, 1.0]),
        );
        graph.nodes.push(node(
            "tint",
            MaterialGraphNodeKindAsset::VectorParameter {
                name: "tint".to_string(),
                default: [1.0, 1.0, 1.0, 1.0],
            },
        ));
        graph.nodes.push(node(
            "gain",
            MaterialGraphNodeKindAsset::ScalarParameter {
                name: "gain".to_string(),
                default: 2.0,
            },
        ));
        graph
            .nodes
            .push(node("multiply", MaterialGraphNodeKindAsset::Multiply));
        graph
            .nodes
            .push(node("output", MaterialGraphNodeKindAsset::Output));
        graph.links.push(link("tint", "value", "multiply", "a"));
        graph.links.push(link("gain", "value", "multiply", "b"));
        graph
            .links
            .push(link("multiply", "value", "output", "base_color"));

        let material = compile_material_graph(&graph).expect("valid math graph compiles");

        assert_eq!(material.base_color, [0.4, 0.6, 0.8, 1.0]);
    }

    #[test]
    fn material_graph_compile_maps_texture_sample_to_base_color_texture() {
        let mut graph = graph_with_shader();
        let texture = asset_ref("res://textures/albedo.png");
        graph.nodes.push(node(
            "albedo",
            MaterialGraphNodeKindAsset::TextureSample {
                texture: texture.clone(),
            },
        ));
        graph
            .nodes
            .push(node("output", MaterialGraphNodeKindAsset::Output));
        graph
            .links
            .push(link("albedo", "color", "output", "base_color"));

        let material = compile_material_graph(&graph).expect("valid texture graph compiles");

        assert_eq!(material.base_color_texture, Some(texture));
        assert_eq!(material.base_color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn material_graph_validation_reports_missing_output_duplicate_node_and_required_input() {
        let mut graph = graph_with_shader();
        graph
            .nodes
            .push(node("color", MaterialGraphNodeKindAsset::Add));
        graph
            .nodes
            .push(node("color", MaterialGraphNodeKindAsset::Multiply));

        let diagnostics = validate_material_graph(&graph);

        assert!(diagnostics
            .iter()
            .any(|message| message.contains("duplicate node `color`")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("has no output node")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("missing required input `a`")));
    }

    #[test]
    fn material_graph_compile_requires_shader_target() {
        let mut graph = graph_with_shader();
        graph.shader = None;
        graph.nodes.push(node(
            "base",
            MaterialGraphNodeKindAsset::VectorParameter {
                name: "base_color".to_string(),
                default: [1.0, 1.0, 1.0, 1.0],
            },
        ));
        graph
            .nodes
            .push(node("output", MaterialGraphNodeKindAsset::Output));
        graph
            .links
            .push(link("base", "value", "output", "base_color"));

        let diagnostics = compile_material_graph(&graph).expect_err("shader is required");

        assert!(diagnostics
            .iter()
            .any(|message| message.contains("has no shader target")));
    }

    #[test]
    fn material_graph_compile_operation_reports_diagnostics_without_material() {
        let graph = graph_with_shader();

        let report = compile_material_graph_operation(&graph);

        assert!(report.material.is_none());
        assert!(report
            .diagnostics
            .iter()
            .any(|message| message.contains("has no output node")));
    }

    fn graph_with_shader() -> MaterialGraphAsset {
        MaterialGraphAsset {
            uri: AssetUri::parse("res://materials/test.material_graph.toml").unwrap(),
            name: "Test Material".to_string(),
            shader: Some(asset_ref("res://shaders/pbr.wgsl")),
            nodes: Vec::new(),
            links: Vec::new(),
            parameters: BTreeMap::new(),
        }
    }

    fn node(id: &str, kind: MaterialGraphNodeKindAsset) -> MaterialGraphNodeAsset {
        MaterialGraphNodeAsset {
            id: id.to_string(),
            position: [0.0, 0.0],
            kind,
        }
    }

    fn link(
        from_node: &str,
        from_pin: &str,
        to_node: &str,
        to_pin: &str,
    ) -> MaterialGraphLinkAsset {
        MaterialGraphLinkAsset {
            from_node: from_node.to_string(),
            from_pin: from_pin.to_string(),
            to_node: to_node.to_string(),
            to_pin: to_pin.to_string(),
        }
    }

    fn asset_ref(locator: &str) -> AssetReference {
        AssetReference::from_locator(AssetUri::parse(locator).unwrap())
    }
}
