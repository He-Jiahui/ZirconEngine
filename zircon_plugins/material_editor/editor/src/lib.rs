use std::collections::{BTreeMap, BTreeSet};

mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{
    MATERIAL_EDITOR_DRAWER_ID, MATERIAL_EDITOR_TEMPLATE_ID, MATERIAL_EDITOR_VIEW_ID,
};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    plugin_registration, MaterialEditorPlugin,
};
use zircon_runtime::asset::{
    AlphaMode, AssetReference, MaterialAsset, MaterialGraphAsset, MaterialGraphLinkAsset,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset, MaterialGraphParameterAsset,
};

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
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
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
