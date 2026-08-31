use std::collections::{BTreeMap, BTreeSet, VecDeque};

mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{
    ANIMATION_GRAPH_DECLARATION, CAPABILITY, EDITOR_CAPABILITIES, EDITOR_CRATE_NAME,
    NATIVE_EDITOR_ENTRY, NATIVE_EDITOR_REGISTRATION_MANIFEST, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES, PLUGIN_ID,
};
pub use extension_ids::{
    ANIMATION_GRAPH_DRAWER_ID, ANIMATION_GRAPH_TEMPLATE_ID, ANIMATION_GRAPH_VIEW_ID,
};
pub use plugin::{
    ANIMATION_GRAPH_DIST_CRATE_NAME, ANIMATION_GRAPH_DIST_EDITOR_ENTRY, AnimationGraphEditorPlugin,
    animation_graph_dist_module_manifest, editor_capabilities, editor_plugin,
    editor_plugin_descriptor, package_manifest, plugin_registration,
};
use zircon_runtime::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationStateMachineAsset,
};

pub fn validate_animation_graph_asset(graph: &AnimationGraphAsset) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let index = AnimationGraphIndex::build(graph, &mut diagnostics);
    validate_animation_graph_with_index(graph, &index, diagnostics)
}

fn validate_animation_graph_with_index(
    graph: &AnimationGraphAsset,
    index: &AnimationGraphIndex<'_>,
    mut diagnostics: Vec<String>,
) -> Vec<String> {
    for node in &graph.nodes {
        match node {
            AnimationGraphNodeAsset::Clip {
                id, playback_speed, ..
            } => {
                if *playback_speed <= 0.0 {
                    diagnostics.push(format!(
                        "animation graph clip `{id}` playback speed must be greater than zero"
                    ));
                }
            }
            AnimationGraphNodeAsset::Blend { id, inputs, .. } => {
                if inputs.is_empty() {
                    diagnostics.push(format!(
                        "animation graph blend `{id}` must have at least one input"
                    ));
                }
            }
            AnimationGraphNodeAsset::Additive {
                id, base, additive, ..
            } => {
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
                if input.trim().is_empty() || target_ids.is_empty() {
                    diagnostics.push(format!(
                        "animation graph mask `{id}` must have an input and at least one target"
                    ));
                }
            }
            AnimationGraphNodeAsset::Output { .. } => {}
        }
    }

    match index.output_count {
        0 => diagnostics.push("animation graph has no output node".to_string()),
        1 => {}
        _ => diagnostics.push("animation graph must contain exactly one output node".to_string()),
    }

    let mut has_missing_reference = false;
    for node in &graph.nodes {
        match node {
            AnimationGraphNodeAsset::Blend { id, inputs, .. } => {
                for input in inputs {
                    if !index.contains(input) {
                        has_missing_reference = true;
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
                    if !index.contains(input) {
                        has_missing_reference = true;
                        diagnostics.push(format!(
                            "animation graph additive `{id}` references missing input `{input}`"
                        ));
                    }
                }
            }
            AnimationGraphNodeAsset::Mask { id, input, .. } => {
                if !index.contains(input) {
                    has_missing_reference = true;
                    diagnostics.push(format!(
                        "animation graph mask `{id}` references missing input `{input}`"
                    ));
                }
            }
            AnimationGraphNodeAsset::Output { source } => {
                if !index.contains(source) {
                    has_missing_reference = true;
                    diagnostics.push(format!(
                        "animation graph output references missing source `{source}`"
                    ));
                }
            }
            AnimationGraphNodeAsset::Clip { .. } => {}
        }
    }

    if !index.topology_ambiguous && !has_missing_reference && index.contains_cycle(graph) {
        diagnostics.push("animation graph contains a cyclic node dependency".to_string());
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn compile_animation_graph(graph: &AnimationGraphAsset) -> Result<String, Vec<String>> {
    let mut diagnostics = Vec::new();
    let index = AnimationGraphIndex::build(graph, &mut diagnostics);
    let diagnostics = validate_animation_graph_with_index(graph, &index, diagnostics);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(index
        .output_source
        .expect("validated animation graph has output")
        .to_string())
}

struct AnimationGraphIndex<'a> {
    node_indices: BTreeMap<&'a str, usize>,
    output_source: Option<&'a str>,
    output_count: usize,
    topology_ambiguous: bool,
}

impl<'a> AnimationGraphIndex<'a> {
    fn build(graph: &'a AnimationGraphAsset, diagnostics: &mut Vec<String>) -> Self {
        let mut node_indices = BTreeMap::new();
        let mut output_source = None;
        let mut output_count = 0;
        let mut topology_ambiguous = false;

        for node in &graph.nodes {
            let Some((id, kind)) = animation_graph_node_identity(node) else {
                output_count += 1;
                if output_source.is_none() {
                    let AnimationGraphNodeAsset::Output { source } = node else {
                        unreachable!("only output nodes omit an identity")
                    };
                    output_source = Some(source.as_str());
                }
                continue;
            };
            if id.trim().is_empty() {
                topology_ambiguous = true;
                diagnostics.push(format!("animation graph {kind} node id must not be empty"));
                continue;
            }
            let next_index = node_indices.len();
            match node_indices.entry(id) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(next_index);
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    topology_ambiguous = true;
                    diagnostics.push(format!("animation graph has duplicate node `{id}`"));
                }
            }
        }

        Self {
            node_indices,
            output_source,
            output_count,
            topology_ambiguous,
        }
    }

    fn contains(&self, node_id: &str) -> bool {
        self.node_indices.contains_key(node_id)
    }

    fn contains_cycle(&self, graph: &AnimationGraphAsset) -> bool {
        let mut incoming_counts = vec![0_usize; self.node_indices.len()];
        let mut dependents = vec![Vec::new(); self.node_indices.len()];

        for node in &graph.nodes {
            let Some((node_id, _)) = animation_graph_node_identity(node) else {
                continue;
            };
            let Some(&node_index) = self.node_indices.get(node_id) else {
                continue;
            };
            match node {
                AnimationGraphNodeAsset::Blend { inputs, .. } => {
                    for input in inputs {
                        self.add_dependency(
                            input,
                            node_index,
                            &mut incoming_counts,
                            &mut dependents,
                        );
                    }
                }
                AnimationGraphNodeAsset::Additive { base, additive, .. } => {
                    for input in [base, additive] {
                        self.add_dependency(
                            input,
                            node_index,
                            &mut incoming_counts,
                            &mut dependents,
                        );
                    }
                }
                AnimationGraphNodeAsset::Mask { input, .. } => {
                    self.add_dependency(input, node_index, &mut incoming_counts, &mut dependents)
                }
                AnimationGraphNodeAsset::Clip { .. } | AnimationGraphNodeAsset::Output { .. } => {}
            }
        }

        let mut ready = incoming_counts
            .iter()
            .enumerate()
            .filter_map(|(index, incoming)| (*incoming == 0).then_some(index))
            .collect::<VecDeque<_>>();
        let mut visited = 0;
        while let Some(node_index) = ready.pop_front() {
            visited += 1;
            for &dependent in &dependents[node_index] {
                incoming_counts[dependent] -= 1;
                if incoming_counts[dependent] == 0 {
                    ready.push_back(dependent);
                }
            }
        }
        visited != self.node_indices.len()
    }

    fn add_dependency(
        &self,
        dependency_id: &str,
        dependent_index: usize,
        incoming_counts: &mut [usize],
        dependents: &mut [Vec<usize>],
    ) {
        let dependency_index = self.node_indices[dependency_id];
        dependents[dependency_index].push(dependent_index);
        incoming_counts[dependent_index] += 1;
    }
}

fn animation_graph_node_identity(node: &AnimationGraphNodeAsset) -> Option<(&str, &'static str)> {
    match node {
        AnimationGraphNodeAsset::Clip { id, .. } => Some((id, "clip")),
        AnimationGraphNodeAsset::Blend { id, .. } => Some((id, "blend")),
        AnimationGraphNodeAsset::Additive { id, .. } => Some((id, "additive")),
        AnimationGraphNodeAsset::Mask { id, .. } => Some((id, "mask")),
        AnimationGraphNodeAsset::Output { .. } => None,
    }
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
