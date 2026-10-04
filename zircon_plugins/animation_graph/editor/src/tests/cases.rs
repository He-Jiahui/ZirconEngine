use super::*;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::framework::animation::{
    AnimationStateAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::PluginModuleKind;

#[test]
fn animation_graph_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("animation graph authoring registration");
    let operation = EditorOperationPath::parse("animation_graph.authoring.compile")
        .expect("valid animation graph operation path");
    let descriptor = registry
        .commands()
        .command(&operation)
        .expect("compile operation registered");

    assert_eq!(
        descriptor
            .menu_path()
            .expect("compile command menu path")
            .stable_path(),
        "plugins/animation_graph/animation_graph.authoring.compile"
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("animation_graph.compile.v1")
    );
    assert!(registry.menu_items().is_empty());
}

#[test]
fn blend_space_graph_nodes_are_registered() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("animation graph authoring registration");

    let graph_node_palettes = registry.graph_node_palettes();
    let graph_palette = graph_node_palettes
        .iter()
        .find(|palette| palette.asset_type().as_str() == "animation.graph")
        .expect("animation graph palette");
    for node_id in ["blend_space_1d", "blend_space_2d"] {
        assert!(graph_palette
            .nodes()
            .iter()
            .any(|node| node.id() == node_id));
    }
}

#[test]
fn animation_graph_package_manifest_declares_editor_only_metadata() {
    let manifest = package_manifest();
    let editor_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Editor)
        .expect("animation graph editor module");

    assert_eq!(manifest.category, "authoring");
    assert_eq!(
        manifest.supported_targets,
        vec![zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost]
    );
    assert_eq!(manifest.capabilities, vec![CAPABILITY.to_string()]);
    assert_eq!(editor_module.capabilities, manifest.capabilities);
}

#[test]
fn animation_graph_package_manifest_declares_editor_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("animation_graph declares standalone distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, ANIMATION_GRAPH_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert!(distribution.runtime_entry.is_empty());
    assert_eq!(distribution.editor_entry, ANIMATION_GRAPH_DIST_EDITOR_ENTRY);

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "animation_graph.dist")
        .expect("animation_graph dist module is declared");
    assert_eq!(dist_module.kind, PluginModuleKind::Native);
    assert_eq!(dist_module.crate_name, ANIMATION_GRAPH_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(dist_module.capabilities, vec![CAPABILITY.to_string()]);
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
fn animation_graph_validation_rejects_cyclic_node_dependencies() {
    let graph = AnimationGraphAsset {
        name: Some("Cyclic".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Blend {
                id: "a".to_string(),
                inputs: vec!["b".to_string()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Blend {
                id: "b".to_string(),
                inputs: vec!["a".to_string()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Output {
                source: "a".to_string(),
            },
        ],
    };

    let diagnostics = validate_animation_graph_asset(&graph);

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("cyclic node dependency")));
    assert!(compile_animation_graph(&graph).is_err());
}

#[test]
fn animation_graph_indexed_validation_accepts_an_acyclic_dependency_chain() {
    let graph = AnimationGraphAsset {
        name: Some("Acyclic".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "clip".to_string(),
                clip: asset_ref("res://animation/idle.anim_clip"),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Mask {
                id: "masked".to_string(),
                input: "clip".to_string(),
                target_ids: vec!["spine".to_string()],
            },
            AnimationGraphNodeAsset::Additive {
                id: "final".to_string(),
                base: "masked".to_string(),
                additive: "clip".to_string(),
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Output {
                source: "final".to_string(),
            },
        ],
    };

    assert!(validate_animation_graph_asset(&graph).is_empty());
    assert_eq!(compile_animation_graph(&graph), Ok("final".to_string()));
}

#[test]
fn animation_state_machine_validation_reports_illegal_transition_and_condition() {
    let machine = AnimationStateMachineAsset {
        name: Some("Locomotion".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![AnimationStateAsset::graph_ref(
            "Idle",
            asset_ref("res://animation/idle.anim_graph"),
        )],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: -0.1,
            exit_time: None,
            interruption: Default::default(),
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: " ".to_string(),
                operator: AnimationConditionOperatorAsset::Triggered,
                value: Some(AnimationParameterValue::Bool(true)),
            }],
        }],
        layers: Vec::new(),
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
            AnimationStateAsset::graph_ref("Idle", asset_ref("res://animation/idle.anim_graph")),
            AnimationStateAsset::graph_ref("Run", asset_ref("res://animation/run.anim_graph")),
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.2,
            exit_time: None,
            interruption: Default::default(),
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::Greater,
                value: Some(AnimationParameterValue::Scalar(0.1)),
            }],
        }],
        layers: Vec::new(),
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
