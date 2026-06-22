use zircon_runtime::asset::assets::{
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
};

use super::parameters::parse_parameter_value;
use super::AnimationEditorSession;

impl AnimationEditorSession {
    pub fn add_graph_node(&mut self, node_id: &str, node_kind: &str) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        let node = match node_kind.to_ascii_lowercase().as_str() {
            "output" => {
                if graph_has_output_node(asset) {
                    return Ok(false);
                }
                AnimationGraphNodeAsset::Output {
                    source: String::new(),
                }
            }
            "blend" => {
                if asset
                    .nodes
                    .iter()
                    .any(|node| graph_node_id(node) == Some(node_id))
                {
                    return Ok(false);
                }
                AnimationGraphNodeAsset::Blend {
                    id: node_id.to_string(),
                    inputs: Vec::new(),
                    weight_parameter: None,
                }
            }
            _ => return Ok(false),
        };
        asset.nodes.push(node);
        self.dirty = true;
        Ok(true)
    }

    pub fn remove_graph_node(&mut self, node_id: &str) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        let before = asset.nodes.len();
        asset.nodes.retain(|node| {
            graph_node_id(node) != Some(node_id)
                && !(node_id == "output" && matches!(node, AnimationGraphNodeAsset::Output { .. }))
        });
        for node in &mut asset.nodes {
            match node {
                AnimationGraphNodeAsset::Blend { inputs, .. } => {
                    inputs.retain(|input| input != node_id);
                }
                AnimationGraphNodeAsset::Additive { base, additive, .. } => {
                    if base == node_id {
                        base.clear();
                    }
                    if additive == node_id {
                        additive.clear();
                    }
                }
                AnimationGraphNodeAsset::Mask { input, .. } if input == node_id => {
                    input.clear();
                }
                AnimationGraphNodeAsset::Output { source } if source == node_id => {
                    source.clear();
                }
                _ => {}
            }
        }
        let changed = before != asset.nodes.len();
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn connect_graph_nodes(
        &mut self,
        from_node_id: &str,
        to_node_id: &str,
    ) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        if from_node_id == to_node_id {
            return Ok(false);
        }
        if !graph_has_named_node(asset, from_node_id) {
            return Ok(false);
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
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn disconnect_graph_nodes(
        &mut self,
        from_node_id: &str,
        to_node_id: &str,
    ) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
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
        self.dirty |= changed;
        Ok(changed)
    }

    pub fn set_graph_parameter(
        &mut self,
        parameter_name: &str,
        value_literal: &str,
    ) -> Result<bool, String> {
        let asset = self.graph_asset_mut()?;
        if let Some(parameter) = asset
            .parameters
            .iter_mut()
            .find(|parameter| parameter.name == parameter_name)
        {
            let Some(next) = parse_parameter_value(Some(&parameter.default_value), value_literal)
            else {
                return Ok(false);
            };
            let changed = parameter.default_value != next;
            parameter.default_value = next;
            self.dirty |= changed;
            return Ok(changed);
        }
        let Some(default_value) = parse_parameter_value(None, value_literal) else {
            return Ok(false);
        };
        asset.parameters.push(AnimationGraphParameterAsset {
            name: parameter_name.to_string(),
            default_value,
        });
        self.dirty = true;
        Ok(true)
    }
}

pub(super) fn graph_node_id(node: &AnimationGraphNodeAsset) -> Option<&str> {
    match node {
        AnimationGraphNodeAsset::Clip { id, .. } => Some(id),
        AnimationGraphNodeAsset::Blend { id, .. } => Some(id),
        AnimationGraphNodeAsset::Additive { id, .. } => Some(id),
        AnimationGraphNodeAsset::Mask { id, .. } => Some(id),
        AnimationGraphNodeAsset::Output { .. } => None,
    }
}

fn graph_has_output_node(asset: &AnimationGraphAsset) -> bool {
    asset
        .nodes
        .iter()
        .any(|node| matches!(node, AnimationGraphNodeAsset::Output { .. }))
}

fn graph_has_named_node(asset: &AnimationGraphAsset, node_id: &str) -> bool {
    asset
        .nodes
        .iter()
        .any(|node| graph_node_id(node) == Some(node_id))
}

pub(super) fn graph_node_label(node: &AnimationGraphNodeAsset) -> String {
    match node {
        AnimationGraphNodeAsset::Clip { id, clip, .. } => {
            format!("Clip {id} • {}", clip.locator)
        }
        AnimationGraphNodeAsset::Blend { id, inputs, .. } => {
            if inputs.is_empty() {
                format!("Blend {id}")
            } else {
                format!("Blend {id} • {}", inputs.join(", "))
            }
        }
        AnimationGraphNodeAsset::Additive {
            id, base, additive, ..
        } => {
            format!("Additive {id} • {base} + {additive}")
        }
        AnimationGraphNodeAsset::Mask {
            id,
            input,
            target_ids,
        } => {
            format!("Mask {id} • {input} [{}]", target_ids.join(", "))
        }
        AnimationGraphNodeAsset::Output { source } => format!("Output <- {source}"),
    }
}
