use std::fmt::Write as _;

use zircon_runtime::core::framework::animation::AnimationGraphNodeAsset;

pub(super) fn graph_node_label(node: &AnimationGraphNodeAsset) -> String {
    let mut label = String::with_capacity(graph_node_label_capacity(node));
    match node {
        AnimationGraphNodeAsset::Clip { id, clip, .. } => {
            write!(&mut label, "Clip {id} • {}", clip.locator)
                .expect("writing to a String cannot fail");
        }
        AnimationGraphNodeAsset::Blend { id, inputs, .. } => {
            write!(&mut label, "Blend {id}").expect("writing to a String cannot fail");
            if !inputs.is_empty() {
                label.push_str(" • ");
                append_graph_ids(&mut label, inputs);
            }
        }
        AnimationGraphNodeAsset::Additive {
            id, base, additive, ..
        } => {
            write!(&mut label, "Additive {id} • {base} + {additive}")
                .expect("writing to a String cannot fail");
        }
        AnimationGraphNodeAsset::Mask {
            id,
            input,
            target_ids,
        } => {
            write!(&mut label, "Mask {id} • {input} [").expect("writing to a String cannot fail");
            append_graph_ids(&mut label, target_ids);
            label.push(']');
        }
        AnimationGraphNodeAsset::Output { source } => {
            write!(&mut label, "Output <- {source}").expect("writing to a String cannot fail");
        }
    }
    label
}

fn graph_node_label_capacity(node: &AnimationGraphNodeAsset) -> usize {
    match node {
        AnimationGraphNodeAsset::Clip { id, clip, .. } => "Clip "
            .len()
            .saturating_add(id.len())
            .saturating_add(" • ".len())
            .saturating_add("package://".len())
            .saturating_add(clip.locator.path().len())
            .saturating_add(
                clip.locator
                    .label()
                    .map_or(0, |label| "#".len().saturating_add(label.len())),
            ),
        AnimationGraphNodeAsset::Blend { id, inputs, .. } => "Blend "
            .len()
            .saturating_add(id.len())
            .saturating_add(if inputs.is_empty() {
                0
            } else {
                " • ".len().saturating_add(graph_ids_text_len(inputs))
            }),
        AnimationGraphNodeAsset::Additive {
            id, base, additive, ..
        } => "Additive "
            .len()
            .saturating_add(id.len())
            .saturating_add(" • ".len())
            .saturating_add(base.len())
            .saturating_add(" + ".len())
            .saturating_add(additive.len()),
        AnimationGraphNodeAsset::Mask {
            id,
            input,
            target_ids,
        } => "Mask "
            .len()
            .saturating_add(id.len())
            .saturating_add(" • ".len())
            .saturating_add(input.len())
            .saturating_add(" [".len())
            .saturating_add(graph_ids_text_len(target_ids))
            .saturating_add("]".len()),
        AnimationGraphNodeAsset::Output { source } => {
            "Output <- ".len().saturating_add(source.len())
        }
    }
}

fn graph_ids_text_len(ids: &[String]) -> usize {
    ids.iter().fold(
        ids.len().saturating_sub(1).saturating_mul(", ".len()),
        |length, id| length.saturating_add(id.len()),
    )
}

fn append_graph_ids(label: &mut String, ids: &[String]) {
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            label.push_str(", ");
        }
        label.push_str(id);
    }
}

#[cfg(test)]
#[path = "graph/tests/single_buffer_tests.rs"]
mod single_buffer_tests;
