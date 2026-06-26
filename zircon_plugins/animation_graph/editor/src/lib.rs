use std::collections::BTreeSet;

mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use extension_ids::{
    ANIMATION_GRAPH_DRAWER_ID, ANIMATION_GRAPH_TEMPLATE_ID, ANIMATION_GRAPH_VIEW_ID,
};
pub use plugin::{
    animation_graph_dist_module_manifest, editor_capabilities, editor_plugin,
    editor_plugin_descriptor, package_manifest, plugin_registration, AnimationGraphEditorPlugin,
    ANIMATION_GRAPH_DIST_CRATE_NAME, ANIMATION_GRAPH_DIST_EDITOR_ENTRY,
};
use zircon_runtime::asset::{
    AnimationConditionOperatorAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationStateMachineAsset,
};

pub fn validate_animation_graph_asset(graph: &AnimationGraphAsset) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut node_ids = BTreeSet::new();
    let mut output_count = 0;

    for node in &graph.nodes {
        match node {
            AnimationGraphNodeAsset::Clip {
                id, playback_speed, ..
            } => {
                validate_node_id(id, "clip", &mut node_ids, &mut diagnostics);
                if *playback_speed <= 0.0 {
                    diagnostics.push(format!(
                        "animation graph clip `{id}` playback speed must be greater than zero"
                    ));
                }
            }
            AnimationGraphNodeAsset::Blend { id, inputs, .. } => {
                validate_node_id(id, "blend", &mut node_ids, &mut diagnostics);
                if inputs.is_empty() {
                    diagnostics.push(format!(
                        "animation graph blend `{id}` must have at least one input"
                    ));
                }
            }
            AnimationGraphNodeAsset::Additive {
                id, base, additive, ..
            } => {
                validate_node_id(id, "additive", &mut node_ids, &mut diagnostics);
                if base.trim().is_empty() || additive.trim().is_empty() {
                    diagnostics.push(format!(
                        "animation graph additive `{id}` must have base and additive inputs"
                    ));
                }
            }
            AnimationGraphNodeAsset::Mask {
                id,
                input,
                target_ids,
            } => {
                validate_node_id(id, "mask", &mut node_ids, &mut diagnostics);
                if input.trim().is_empty() || target_ids.is_empty() {
                    diagnostics.push(format!(
                        "animation graph mask `{id}` must have an input and at least one target"
                    ));
                }
            }
            AnimationGraphNodeAsset::Output { .. } => {
                output_count += 1;
            }
        }
    }

    match output_count {
        0 => diagnostics.push("animation graph has no output node".to_string()),
        1 => {}
        _ => diagnostics.push("animation graph must contain exactly one output node".to_string()),
    }

    for node in &graph.nodes {
        match node {
            AnimationGraphNodeAsset::Blend { id, inputs, .. } => {
                for input in inputs {
                    if !node_ids.contains(input.as_str()) {
                        diagnostics.push(format!(
                            "animation graph blend `{id}` references missing input `{input}`"
                        ));
                    }
                }
            }
            AnimationGraphNodeAsset::Additive {
                id, base, additive, ..
            } => {
                for input in [base, additive] {
                    if !node_ids.contains(input.as_str()) {
                        diagnostics.push(format!(
                            "animation graph additive `{id}` references missing input `{input}`"
                        ));
                    }
                }
            }
            AnimationGraphNodeAsset::Mask { id, input, .. } => {
                if !node_ids.contains(input.as_str()) {
                    diagnostics.push(format!(
                        "animation graph mask `{id}` references missing input `{input}`"
                    ));
                }
            }
            AnimationGraphNodeAsset::Output { source } => {
                if !node_ids.contains(source.as_str()) {
                    diagnostics.push(format!(
                        "animation graph output references missing source `{source}`"
                    ));
                }
            }
            AnimationGraphNodeAsset::Clip { .. } => {}
        }
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn compile_animation_graph(graph: &AnimationGraphAsset) -> Result<String, Vec<String>> {
    let diagnostics = validate_animation_graph_asset(graph);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let output_source = graph
        .nodes
        .iter()
        .find_map(|node| match node {
            AnimationGraphNodeAsset::Output { source } => Some(source.clone()),
            _ => None,
        })
        .expect("validated animation graph has output");
    Ok(output_source)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationStateMachineCompileReport {
    pub entry_state: String,
    pub state_count: usize,
    pub transition_count: usize,
}

pub fn validate_animation_state_machine_asset(machine: &AnimationStateMachineAsset) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut states = BTreeSet::new();

    for state in &machine.states {
        if state.name.trim().is_empty() {
            diagnostics.push("animation state name must not be empty".to_string());
            continue;
        }
        if !states.insert(state.name.as_str()) {
            diagnostics.push(format!(
                "animation state machine has duplicate state `{}`",
                state.name
            ));
        }
    }

    if !states.contains(machine.entry_state.as_str()) {
        diagnostics.push(format!(
            "animation state machine entry state `{}` does not exist",
            machine.entry_state
        ));
    }

    for transition in &machine.transitions {
        if !states.contains(transition.from_state.as_str()) {
            diagnostics.push(format!(
                "animation transition references missing from_state `{}`",
                transition.from_state
            ));
        }
        if !states.contains(transition.to_state.as_str()) {
            diagnostics.push(format!(
                "animation transition references missing to_state `{}`",
                transition.to_state
            ));
        }
        if transition.duration_seconds < 0.0 {
            diagnostics.push(format!(
                "animation transition `{} -> {}` duration must not be negative",
                transition.from_state, transition.to_state
            ));
        }
        for condition in &transition.conditions {
            if condition.parameter.trim().is_empty() {
                diagnostics.push(format!(
                    "animation transition `{} -> {}` condition parameter must not be empty",
                    transition.from_state, transition.to_state
                ));
            }
            if condition.operator == AnimationConditionOperatorAsset::Triggered
                && condition.value.is_some()
            {
                diagnostics.push(format!(
                    "animation transition `{} -> {}` triggered condition must not carry a comparison value",
                    transition.from_state, transition.to_state
                ));
            }
        }
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn compile_animation_state_machine(
    machine: &AnimationStateMachineAsset,
) -> Result<AnimationStateMachineCompileReport, Vec<String>> {
    let diagnostics = validate_animation_state_machine_asset(machine);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(AnimationStateMachineCompileReport {
        entry_state: machine.entry_state.clone(),
        state_count: machine.states.len(),
        transition_count: machine.transitions.len(),
    })
}

fn validate_node_id<'a>(
    id: &'a str,
    kind: &str,
    node_ids: &mut BTreeSet<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if id.trim().is_empty() {
        diagnostics.push(format!("animation graph {kind} node id must not be empty"));
    } else if !node_ids.insert(id) {
        diagnostics.push(format!("animation graph has duplicate node `{id}`"));
    }
}
