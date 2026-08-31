use zircon_runtime::core::framework::animation::AnimationGraphNodeAsset;

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
