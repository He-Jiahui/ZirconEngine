use super::{
    diagnostics, outputs, AlphaMode, MaterialAsset, MaterialColorInput, MaterialGraphAsset,
    MaterialGraphDiagnostic, MaterialGraphEvaluator, MaterialGraphIndex,
    MaterialGraphNodeKindAsset,
};

pub fn compile_material_graph(graph: &MaterialGraphAsset) -> Result<MaterialAsset, Vec<String>> {
    compile_material_graph_with_diagnostics(graph).map_err(diagnostics::messages)
}

/// Compiles the supplied graph with locations produced at the actual validation
/// or evaluation error site. The host must retain the input revision/request
/// alongside this result; a target alone does not authorize a document jump.
pub fn compile_material_graph_with_diagnostics(
    graph: &MaterialGraphAsset,
) -> Result<MaterialAsset, Vec<MaterialGraphDiagnostic>> {
    let index = MaterialGraphIndex::new(graph);
    let mut diagnostics = diagnostics::validate_with_index(graph, &index);
    let Some(shader) = graph.shader.clone() else {
        diagnostics.push(MaterialGraphDiagnostic::graph(format!(
            "material graph `{}` has no shader target for MaterialAsset compile",
            graph.name
        )));
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
    let base_color_link = index
        .incoming_link(&output.id, "base_color")
        .expect("validated output has base_color");
    let mut evaluator = MaterialGraphEvaluator::new(graph, &index);
    let base_color_input = evaluator
        .color(&base_color_link.from_node)
        .map_err(|error| vec![error])?;

    let (base_color, base_color_texture) = match base_color_input {
        MaterialColorInput::Constant(value) => (value, None),
        MaterialColorInput::Texture(texture) => ([1.0, 1.0, 1.0, 1.0], Some(texture)),
    };

    let mut material = MaterialAsset {
        name: Some(graph.name.clone()),
        shader,
        parent: None,
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
        options: Default::default(),
        queue: None,
        validation_diagnostics: Vec::new(),
    };
    outputs::apply(&index, &output.id, &mut material, &mut evaluator)
        .map_err(|error| vec![error])?;
    Ok(material)
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
