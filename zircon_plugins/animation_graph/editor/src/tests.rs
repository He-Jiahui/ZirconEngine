use super::*;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::asset::{
    AnimationStateAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::AnimationParameterValue;

#[test]
fn animation_graph_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("animation graph authoring registration");
    let operation = EditorOperationPath::parse("AnimationGraph.Authoring.Compile")
        .expect("valid animation graph operation path");
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
        vec![zircon_runtime::builtin::RuntimeTargetMode::EditorHost]
    );
    assert_eq!(manifest.capabilities, vec![CAPABILITY.to_string()]);
    assert_eq!(editor_module.capabilities, manifest.capabilities);
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
