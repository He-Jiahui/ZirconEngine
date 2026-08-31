use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationParameterValue,
};

use super::super::AnimationGraphNodeKind;

pub(super) fn add_node(
    asset: &mut AnimationGraphAsset,
    node_id: &str,
    node_kind: AnimationGraphNodeKind,
) -> Result<bool, super::super::AnimationDocumentMutationError> {
    let node = match node_kind {
        AnimationGraphNodeKind::Output => {
            if has_output_node(asset) {
                return Ok(false);
            }
            AnimationGraphNodeAsset::Output {
                source: String::new(),
            }
        }
        AnimationGraphNodeKind::Blend => {
            if asset
                .nodes
                .iter()
                .any(|node| node_id_for(node) == Some(node_id))
            {
                return Ok(false);
            }
            AnimationGraphNodeAsset::Blend {
                id: node_id.to_string(),
                inputs: Vec::new(),
                weight_parameter: None,
            }
        }
    };
    asset.nodes.push(node);
    Ok(true)
}

pub(super) fn remove_node(asset: &mut AnimationGraphAsset, node_id: &str) -> bool {
    let before = asset.nodes.len();
    asset.nodes.retain(|node| {
        node_id_for(node) != Some(node_id)
            && !(node_id == "output" && matches!(node, AnimationGraphNodeAsset::Output { .. }))
    });
    let mut changed = before != asset.nodes.len();
    for node in &mut asset.nodes {
        match node {
            AnimationGraphNodeAsset::Blend { inputs, .. } => {
                let before = inputs.len();
                inputs.retain(|input| input != node_id);
                changed |= before != inputs.len();
            }
            AnimationGraphNodeAsset::Additive { base, additive, .. } => {
                if base == node_id {
                    base.clear();
                    changed = true;
                }
                if additive == node_id {
                    additive.clear();
                    changed = true;
                }
            }
            AnimationGraphNodeAsset::Mask { input, .. } if input == node_id => {
                input.clear();
                changed = true;
            }
            AnimationGraphNodeAsset::Output { source } if source == node_id => {
                source.clear();
                changed = true;
            }
            _ => {}
        }
    }
    changed
}

pub(super) fn connect_nodes(
    asset: &mut AnimationGraphAsset,
    from_node_id: &str,
    to_node_id: &str,
) -> bool {
    if from_node_id == to_node_id || !has_named_node(asset, from_node_id) {
        return false;
    }
    let mut changed = false;
    for node in &mut asset.nodes {
        match node {
            AnimationGraphNodeAsset::Blend { id, inputs, .. } if id == to_node_id => {
                if !inputs.iter().any(|input| input == from_node_id) {
                    inputs.push(from_node_id.to_string());
                    changed = true;
                }
            }
            AnimationGraphNodeAsset::Additive { id, base, .. } if id == to_node_id => {
                if base != from_node_id {
                    *base = from_node_id.to_string();
                    changed = true;
                }
            }
            AnimationGraphNodeAsset::Mask { id, input, .. } if id == to_node_id => {
                if input != from_node_id {
                    *input = from_node_id.to_string();
                    changed = true;
                }
            }
            AnimationGraphNodeAsset::Output { source } if to_node_id == "output" => {
                if source != from_node_id {
                    *source = from_node_id.to_string();
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

pub(super) fn disconnect_nodes(
    asset: &mut AnimationGraphAsset,
    from_node_id: &str,
    to_node_id: &str,
) -> bool {
    let mut changed = false;
    for node in &mut asset.nodes {
        match node {
            AnimationGraphNodeAsset::Blend { id, inputs, .. } if id == to_node_id => {
                let before = inputs.len();
                inputs.retain(|input| input != from_node_id);
                changed |= before != inputs.len();
            }
            AnimationGraphNodeAsset::Additive {
                id, base, additive, ..
            } if id == to_node_id => {
                if base == from_node_id {
                    base.clear();
                    changed = true;
                }
                if additive == from_node_id {
                    additive.clear();
                    changed = true;
                }
            }
            AnimationGraphNodeAsset::Mask { id, input, .. } if id == to_node_id => {
                if input == from_node_id {
                    input.clear();
                    changed = true;
                }
            }
            AnimationGraphNodeAsset::Output { source } if to_node_id == "output" => {
                if source == from_node_id {
                    source.clear();
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

pub(super) fn set_parameter(
    asset: &mut AnimationGraphAsset,
    parameter_name: &str,
    value_literal: &str,
) -> bool {
    if let Some(parameter) = asset
        .parameters
        .iter_mut()
        .find(|parameter| parameter.name == parameter_name)
    {
        let Some(next) = parse_parameter_value(Some(&parameter.default_value), value_literal)
        else {
            return false;
        };
        let changed = parameter.default_value != next;
        parameter.default_value = next;
        return changed;
    }
    let Some(default_value) = parse_parameter_value(None, value_literal) else {
        return false;
    };
    asset.parameters.push(AnimationGraphParameterAsset {
        name: parameter_name.to_string(),
        default_value,
    });
    true
}

fn node_id_for(node: &AnimationGraphNodeAsset) -> Option<&str> {
    match node {
        AnimationGraphNodeAsset::Clip { id, .. }
        | AnimationGraphNodeAsset::Blend { id, .. }
        | AnimationGraphNodeAsset::Additive { id, .. }
        | AnimationGraphNodeAsset::Mask { id, .. } => Some(id),
        AnimationGraphNodeAsset::Output { .. } => None,
    }
}

fn has_output_node(asset: &AnimationGraphAsset) -> bool {
    asset
        .nodes
        .iter()
        .any(|node| matches!(node, AnimationGraphNodeAsset::Output { .. }))
}

fn has_named_node(asset: &AnimationGraphAsset, node_id: &str) -> bool {
    asset
        .nodes
        .iter()
        .any(|node| node_id_for(node) == Some(node_id))
}

fn parse_parameter_value(
    existing: Option<&AnimationParameterValue>,
    value_literal: &str,
) -> Option<AnimationParameterValue> {
    match existing {
        Some(AnimationParameterValue::Trigger) => parse_trigger_literal(value_literal),
        Some(AnimationParameterValue::Bool(_)) => {
            parse_bool_literal(value_literal).map(AnimationParameterValue::Bool)
        }
        Some(AnimationParameterValue::Integer(_)) => value_literal
            .parse::<i32>()
            .ok()
            .map(AnimationParameterValue::Integer),
        Some(AnimationParameterValue::Scalar(_)) => {
            parse_finite_scalar_literal(value_literal).map(AnimationParameterValue::Scalar)
        }
        Some(AnimationParameterValue::Vec2(_)) => {
            parse_vector_literal::<2>(value_literal).map(AnimationParameterValue::Vec2)
        }
        Some(AnimationParameterValue::Vec3(_)) => {
            parse_vector_literal::<3>(value_literal).map(AnimationParameterValue::Vec3)
        }
        Some(AnimationParameterValue::Vec4(_)) => {
            parse_vector_literal::<4>(value_literal).map(AnimationParameterValue::Vec4)
        }
        None => parse_trigger_literal(value_literal)
            .or_else(|| parse_bool_literal(value_literal).map(AnimationParameterValue::Bool))
            .or_else(|| {
                value_literal
                    .parse::<i32>()
                    .ok()
                    .map(AnimationParameterValue::Integer)
            })
            .or_else(|| {
                parse_finite_scalar_literal(value_literal).map(AnimationParameterValue::Scalar)
            })
            .or_else(|| parse_vector_literal::<2>(value_literal).map(AnimationParameterValue::Vec2))
            .or_else(|| parse_vector_literal::<3>(value_literal).map(AnimationParameterValue::Vec3))
            .or_else(|| {
                parse_vector_literal::<4>(value_literal).map(AnimationParameterValue::Vec4)
            }),
    }
}

fn parse_finite_scalar_literal(value_literal: &str) -> Option<f32> {
    let value = value_literal.parse::<f32>().ok()?;
    value.is_finite().then_some(value)
}

fn parse_trigger_literal(value_literal: &str) -> Option<AnimationParameterValue> {
    value_literal
        .eq_ignore_ascii_case("trigger")
        .then_some(AnimationParameterValue::Trigger)
}

fn parse_bool_literal(value_literal: &str) -> Option<bool> {
    if value_literal.eq_ignore_ascii_case("true") {
        Some(true)
    } else if value_literal.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

fn parse_vector_literal<const N: usize>(value_literal: &str) -> Option<[f32; N]> {
    let parts: Vec<_> = value_literal.split(',').map(str::trim).collect();
    if parts.len() != N {
        return None;
    }
    let mut values = [0.0; N];
    for (index, part) in parts.into_iter().enumerate() {
        values[index] = parse_finite_scalar_literal(part)?;
    }
    Some(values)
}
