use super::{
    MaterialAsset, MaterialColorInput, MaterialGraphDiagnostic, MaterialGraphEvaluator,
    MaterialGraphIndex, MaterialGraphNodeKindAsset,
};

pub(super) fn apply<'a>(
    index: &MaterialGraphIndex<'a>,
    output: &str,
    material: &mut MaterialAsset,
    evaluator: &mut MaterialGraphEvaluator<'a, '_>,
) -> Result<(), MaterialGraphDiagnostic> {
    material.metallic = scalar_output(index, output, "metallic", 0.0, evaluator)?;
    material.roughness = scalar_output(index, output, "roughness", 1.0, evaluator)?;
    if let Some(link) = index.incoming_link(output, "normal") {
        let Some(MaterialGraphNodeKindAsset::TextureSample { texture }) =
            index.node(&link.from_node).map(|node| &node.kind)
        else {
            return Err(MaterialGraphDiagnostic::input(
                output,
                "normal",
                "material graph `normal` requires a texture sample in v1",
            ));
        };
        material.normal_texture = Some(texture.clone());
    }
    if let Some(link) = index.incoming_link(output, "emissive") {
        match evaluator.color(&link.from_node)? {
            MaterialColorInput::Constant(value) => {
                material.emissive = [value[0], value[1], value[2]];
            }
            MaterialColorInput::Texture(texture) => {
                material.emissive = [1.0; 3];
                material.emissive_texture = Some(texture);
            }
        }
    }
    Ok(())
}

fn scalar_output<'a>(
    index: &MaterialGraphIndex<'a>,
    output: &str,
    pin: &str,
    default: f32,
    evaluator: &mut MaterialGraphEvaluator<'a, '_>,
) -> Result<f32, MaterialGraphDiagnostic> {
    let Some(link) = index.incoming_link(output, pin) else {
        return Ok(default);
    };
    evaluator.scalar(&link.from_node).map_err(|mut error| {
        error.message = format!("material graph output `{output}.{pin}`: {}", error.message);
        error
    })
}
