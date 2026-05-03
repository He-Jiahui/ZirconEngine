use std::collections::BTreeSet;

use zircon_editor::core::editor_authoring_extension::{
    GraphEditorDescriptor, GraphNodeDescriptor, GraphNodePaletteDescriptor, GraphPinDescriptor,
};
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, ComponentDrawerDescriptor, EditorMenuItemDescriptor,
};
use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::asset::{
    AnimationConditionOperatorAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationStateMachineAsset,
};

pub const PLUGIN_ID: &str = "animation_graph";
pub const CAPABILITY: &str = "editor.extension.animation_graph_authoring";
pub const ANIMATION_GRAPH_VIEW_ID: &str = "animation_graph.authoring";
pub const ANIMATION_GRAPH_DRAWER_ID: &str = "animation_graph.drawer";
pub const ANIMATION_GRAPH_TEMPLATE_ID: &str = "animation_graph.authoring";

#[derive(Clone, Debug)]
pub struct AnimationGraphEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl AnimationGraphEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for AnimationGraphEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: ANIMATION_GRAPH_DRAWER_ID,
                drawer_display_name: "Animation Graph",
                template_id: ANIMATION_GRAPH_TEMPLATE_ID,
                template_document: "plugins://animation_graph/editor/authoring.ui.toml",
                surfaces: &[EditorAuthoringSurface::new(
                    ANIMATION_GRAPH_VIEW_ID,
                    "Animation Graph",
                    "Animation",
                    "Plugins/Animation Graph",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, animation_graph_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Animation Graph",
        "zircon_plugin_animation_graph_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> AnimationGraphEditorPlugin {
    AnimationGraphEditorPlugin::new()
}

fn base_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::PluginPackageManifest::new(PLUGIN_ID, "Animation Graph")
        .with_category("authoring")
        .with_dependency(
            zircon_runtime::plugin::PluginDependencyManifest::new("animation", true)
                .with_capability("runtime.plugin.animation"),
        )
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

fn animation_graph_authoring_batch() -> EditorAuthoringContributionBatch {
    let open_graph = operation("AnimationGraph.Authoring.OpenGraph");
    let open_state_machine = operation("AnimationGraph.Authoring.OpenStateMachine");
    let validate = operation("AnimationGraph.Authoring.Validate");
    let compile = operation("AnimationGraph.Authoring.Compile");
    EditorAuthoringContributionBatch {
        operations: vec![
            EditorOperationDescriptor::new(open_graph.clone(), "Open Animation Graph")
                .with_menu_path("Plugins/Animation Graph/Open Graph")
                .with_payload_schema_id("animation_graph.open_graph.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(
                open_state_machine.clone(),
                "Open Animation State Machine",
            )
            .with_menu_path("Plugins/Animation Graph/Open State Machine")
            .with_payload_schema_id("animation_graph.open_state_machine.v1")
            .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(validate.clone(), "Validate Animation Graph")
                .with_menu_path("Plugins/Animation Graph/Validate")
                .with_payload_schema_id("animation_graph.validate.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(compile.clone(), "Compile Animation Graph")
                .with_menu_path("Plugins/Animation Graph/Compile")
                .with_payload_schema_id("animation_graph.compile.v1")
                .with_required_capabilities([CAPABILITY]),
        ],
        menu_items: vec![
            menu_item("Plugins/Animation Graph/Open Graph", &open_graph),
            menu_item(
                "Plugins/Animation Graph/Open State Machine",
                &open_state_machine,
            ),
            menu_item("Plugins/Animation Graph/Validate", &validate),
            menu_item("Plugins/Animation Graph/Compile", &compile),
        ],
        asset_editors: vec![
            AssetEditorDescriptor::new(
                "animation.graph",
                ANIMATION_GRAPH_VIEW_ID,
                "Animation Graph",
                open_graph.clone(),
            )
            .with_required_capabilities([CAPABILITY]),
            AssetEditorDescriptor::new(
                "animation.state_machine",
                ANIMATION_GRAPH_VIEW_ID,
                "Animation State Machine",
                open_state_machine,
            )
            .with_required_capabilities([CAPABILITY]),
        ],
        graph_editors: vec![
            GraphEditorDescriptor::new(
                "animation.graph",
                ANIMATION_GRAPH_VIEW_ID,
                "Animation Graph",
                open_graph,
                validate.clone(),
            )
            .with_compile_operation(compile.clone())
            .with_required_capabilities([CAPABILITY]),
            GraphEditorDescriptor::new(
                "animation.state_machine",
                ANIMATION_GRAPH_VIEW_ID,
                "Animation State Machine",
                operation("AnimationGraph.Authoring.OpenStateMachine"),
                validate,
            )
            .with_compile_operation(compile)
            .with_required_capabilities([CAPABILITY]),
        ],
        graph_node_palettes: vec![animation_graph_palette(), animation_state_machine_palette()],
        component_drawers: vec![
            ComponentDrawerDescriptor::new(
                "animation.Component.GraphPlayer",
                "plugins://animation_graph/editor/graph_player.ui.toml",
                "animation_graph.editor.graph_player",
            ),
            ComponentDrawerDescriptor::new(
                "animation.Component.StateMachinePlayer",
                "plugins://animation_graph/editor/state_machine_player.ui.toml",
                "animation_graph.editor.state_machine_player",
            ),
        ],
        ..Default::default()
    }
}

fn animation_graph_palette() -> GraphNodePaletteDescriptor {
    GraphNodePaletteDescriptor::new("animation_graph.palette.graph", "animation.graph")
        .with_node(
            GraphNodeDescriptor::new("clip", "Clip", "Playback")
                .with_output(GraphPinDescriptor::new("pose", "pose")),
        )
        .with_node(
            GraphNodeDescriptor::new("blend", "Blend", "Blend")
                .with_input(GraphPinDescriptor::new("a", "pose").required(true))
                .with_input(GraphPinDescriptor::new("b", "pose").required(true))
                .with_output(GraphPinDescriptor::new("pose", "pose")),
        )
        .with_node(
            GraphNodeDescriptor::new("output", "Output", "Output")
                .with_input(GraphPinDescriptor::new("pose", "pose").required(true)),
        )
        .with_required_capabilities([CAPABILITY])
}

fn animation_state_machine_palette() -> GraphNodePaletteDescriptor {
    GraphNodePaletteDescriptor::new(
        "animation_graph.palette.state_machine",
        "animation.state_machine",
    )
    .with_node(GraphNodeDescriptor::new("state", "State", "State"))
    .with_node(GraphNodeDescriptor::new(
        "transition",
        "Transition",
        "Transition",
    ))
    .with_node(GraphNodeDescriptor::new(
        "condition",
        "Condition",
        "Transition",
    ))
    .with_required_capabilities([CAPABILITY])
}

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

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid animation graph operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone()).with_required_capabilities([CAPABILITY])
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_editor::EditorPlugin;
    use zircon_runtime::asset::{
        AnimationStateAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
    };
    use zircon_runtime::asset::{AssetReference, AssetUri};
    use zircon_runtime::core::framework::animation::AnimationParameterValue;

    #[test]
    fn animation_graph_authoring_registration_exposes_menu_items_and_payload_schemas() {
        let mut registry =
            zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
        editor_plugin()
            .register_editor_extensions(&mut registry)
            .expect("animation graph authoring registration");
        let operation = operation("AnimationGraph.Authoring.Compile");
        let descriptor = registry
            .operations()
            .descriptor(&operation)
            .expect("compile operation registered");

        assert_eq!(
            descriptor.menu_path(),
            Some("Plugins/Animation Graph/Compile")
        );
        assert_eq!(
            descriptor.payload_schema_id(),
            Some("animation_graph.compile.v1")
        );
        assert!(registry.menu_items().iter().any(|item| {
            item.path() == "Plugins/Animation Graph/Compile" && item.operation() == &operation
        }));
    }

    #[test]
    fn animation_graph_compile_returns_output_source_for_valid_graph() {
        let graph = AnimationGraphAsset {
            name: Some("Locomotion".to_string()),
            parameters: Vec::new(),
            nodes: vec![
                AnimationGraphNodeAsset::Clip {
                    id: "idle".to_string(),
                    clip: asset_ref("res://animation/idle.anim_clip"),
                    playback_speed: 1.0,
                    looping: true,
                },
                AnimationGraphNodeAsset::Output {
                    source: "idle".to_string(),
                },
            ],
        };

        assert_eq!(compile_animation_graph(&graph), Ok("idle".to_string()));
    }

    #[test]
    fn animation_graph_validation_reports_duplicate_missing_output_and_missing_source() {
        let graph = AnimationGraphAsset {
            name: None,
            parameters: Vec::new(),
            nodes: vec![
                AnimationGraphNodeAsset::Clip {
                    id: "clip".to_string(),
                    clip: asset_ref("res://animation/a.anim_clip"),
                    playback_speed: 1.0,
                    looping: true,
                },
                AnimationGraphNodeAsset::Blend {
                    id: "clip".to_string(),
                    inputs: vec!["missing".to_string()],
                    weight_parameter: None,
                },
            ],
        };

        let diagnostics = validate_animation_graph_asset(&graph);

        assert!(diagnostics
            .iter()
            .any(|message| message.contains("duplicate node `clip`")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("has no output node")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("references missing input `missing`")));
    }

    #[test]
    fn animation_graph_validation_reports_missing_output_source() {
        let graph = AnimationGraphAsset {
            name: None,
            parameters: Vec::new(),
            nodes: vec![AnimationGraphNodeAsset::Output {
                source: "missing".to_string(),
            }],
        };

        assert!(validate_animation_graph_asset(&graph)
            .iter()
            .any(|message| message.contains("missing source `missing`")));
    }

    #[test]
    fn animation_state_machine_validation_reports_illegal_transition_and_condition() {
        let machine = AnimationStateMachineAsset {
            name: Some("Locomotion".to_string()),
            entry_state: "Idle".to_string(),
            states: vec![AnimationStateAsset {
                name: "Idle".to_string(),
                graph: asset_ref("res://animation/idle.anim_graph"),
            }],
            transitions: vec![AnimationStateTransitionAsset {
                from_state: "Idle".to_string(),
                to_state: "Run".to_string(),
                duration_seconds: -0.1,
                conditions: vec![AnimationTransitionConditionAsset {
                    parameter: " ".to_string(),
                    operator: AnimationConditionOperatorAsset::Triggered,
                    value: Some(AnimationParameterValue::Bool(true)),
                }],
            }],
        };

        let diagnostics = validate_animation_state_machine_asset(&machine);

        assert!(diagnostics
            .iter()
            .any(|message| message.contains("missing to_state `Run`")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("duration must not be negative")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("condition parameter must not be empty")));
        assert!(diagnostics
            .iter()
            .any(|message| message.contains("triggered condition must not carry")));
    }

    #[test]
    fn animation_state_machine_compile_reports_entry_state_and_counts() {
        let machine = AnimationStateMachineAsset {
            name: Some("Locomotion".to_string()),
            entry_state: "Idle".to_string(),
            states: vec![
                AnimationStateAsset {
                    name: "Idle".to_string(),
                    graph: asset_ref("res://animation/idle.anim_graph"),
                },
                AnimationStateAsset {
                    name: "Run".to_string(),
                    graph: asset_ref("res://animation/run.anim_graph"),
                },
            ],
            transitions: vec![AnimationStateTransitionAsset {
                from_state: "Idle".to_string(),
                to_state: "Run".to_string(),
                duration_seconds: 0.2,
                conditions: vec![AnimationTransitionConditionAsset {
                    parameter: "speed".to_string(),
                    operator: AnimationConditionOperatorAsset::Greater,
                    value: Some(AnimationParameterValue::Scalar(0.1)),
                }],
            }],
        };

        assert_eq!(
            compile_animation_state_machine(&machine),
            Ok(AnimationStateMachineCompileReport {
                entry_state: "Idle".to_string(),
                state_count: 2,
                transition_count: 1,
            })
        );
    }

    fn asset_ref(locator: &str) -> AssetReference {
        AssetReference::from_locator(AssetUri::parse(locator).unwrap())
    }
}
