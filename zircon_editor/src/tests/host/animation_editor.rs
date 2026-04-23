use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use zircon_runtime::asset::assets::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationConditionOperatorAsset, AnimationInterpolationAsset, AnimationSequenceAsset,
    AnimationSequenceBindingAsset, AnimationSequenceTrackAsset, AnimationStateAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{AnimationParameterValue, AnimationTrackPath};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};

use crate::core::editor_event::EditorAnimationEvent;
use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::ViewDescriptorId;

fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

fn env_lock() -> &'static Mutex<()> {
    crate::tests::support::env_lock()
}

fn editor_runtime_with_config_path(path: &Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    runtime
}

fn scalar_channel(value: f32) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Scalar(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn write_sequence_asset(path: &Path) {
    let asset = AnimationSequenceAsset {
        name: Some("Hero Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                channel: scalar_channel(1.0),
            }],
        }],
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn write_state_machine_asset(path: &Path) {
    let graph_reference = zircon_runtime::asset::AssetReference::from_locator(
        AssetUri::parse("res://animation/hero.graph.zranim").unwrap(),
    );
    let asset = AnimationStateMachineAsset {
        name: Some("Hero State Machine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: graph_reference.clone(),
            },
            AnimationStateAsset {
                name: "Run".to_string(),
                graph: graph_reference,
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.25,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::GreaterEqual,
                value: Some(AnimationParameterValue::Scalar(1.0)),
            }],
        }],
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

#[test]
fn editor_manager_restores_animation_sequence_editor_session_from_payload_path() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_animation_sequence_host");
    let asset_path =
        unique_temp_dir("zircon_editor_animation_sequence_host_asset").join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset(&asset_path);

    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.animation_sequence"), None)
        .unwrap();
    manager
        .update_view_instance_metadata(
            &instance_id,
            Some("Hero Sequence".to_string()),
            Some(false),
            Some(json!({ "path": asset_path.to_string_lossy().to_string() })),
        )
        .unwrap();

    let pane = manager
        .animation_editor_pane_presentation(&instance_id)
        .expect("sequence editor session should restore from payload path");

    assert_eq!(pane.mode, "sequence");
    assert_eq!(pane.asset_path, asset_path.to_string_lossy());
    assert_eq!(pane.track_items, vec!["Root/Hero:AnimationPlayer.weight"]);
    assert_eq!(pane.timeline_end_frame, 60);
}

#[test]
fn editor_manager_restores_state_machine_graph_editor_session_from_payload_path() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_animation_state_machine_host");
    let asset_path = unique_temp_dir("zircon_editor_animation_state_machine_host_asset")
        .join("hero.state_machine.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_state_machine_asset(&asset_path);

    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.animation_graph"), None)
        .unwrap();
    manager
        .update_view_instance_metadata(
            &instance_id,
            Some("Hero Graph".to_string()),
            Some(false),
            Some(json!({ "path": asset_path.to_string_lossy().to_string() })),
        )
        .unwrap();

    let pane = manager
        .animation_editor_pane_presentation(&instance_id)
        .expect("graph editor should restore state machine session from payload path");

    assert_eq!(pane.mode, "state_machine");
    assert_eq!(pane.asset_path, asset_path.to_string_lossy());
    assert_eq!(pane.state_items, vec!["Idle", "Run"]);
    assert_eq!(pane.transition_items, vec!["Idle -> Run"]);
}

#[test]
fn editor_manager_saves_animation_sequence_editor_session_and_clears_dirty_metadata() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_animation_sequence_save_host");
    let asset_path = unique_temp_dir("zircon_editor_animation_sequence_save_host_asset")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset(&asset_path);

    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.animation_sequence"), None)
        .unwrap();
    manager
        .update_view_instance_metadata(
            &instance_id,
            Some("Hero Sequence".to_string()),
            Some(false),
            Some(json!({ "path": asset_path.to_string_lossy().to_string() })),
        )
        .unwrap();
    manager
        .animation_editor_pane_presentation(&instance_id)
        .expect("sequence editor session should restore before save");

    let changed = manager
        .apply_animation_event(&EditorAnimationEvent::CreateTrack {
            track_path: AnimationTrackPath::parse("Root/Hero:AnimationPlayer.speed").unwrap(),
        })
        .unwrap();
    assert!(
        changed,
        "expected animation authoring edit to dirty the session before save"
    );

    manager.save_animation_editor(&instance_id).unwrap();

    let saved = AnimationSequenceAsset::from_bytes(&fs::read(&asset_path).unwrap()).unwrap();
    let saved_track_paths = saved
        .track_paths()
        .into_iter()
        .map(|path| path.to_string())
        .collect::<Vec<_>>();
    assert!(
        saved_track_paths.contains(&"Root/Hero:AnimationPlayer.speed".to_string()),
        "saving through the editor manager should persist animation track edits to disk"
    );

    let instance = manager
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.instance_id == instance_id)
        .expect("expected saved animation editor instance to remain open");
    assert!(
        !instance.dirty,
        "saving the animation editor should clear the workbench dirty metadata"
    );
    assert_eq!(
        instance.serializable_payload["path"],
        json!(asset_path.to_string_lossy().to_string()),
        "saving the animation editor should preserve the serialized payload path"
    );
}
