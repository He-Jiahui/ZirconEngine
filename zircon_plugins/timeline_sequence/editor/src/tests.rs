use std::collections::BTreeMap;

use super::*;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::asset::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::plugin::{ExportPackagingStrategy, PluginModuleKind};

#[test]
fn timeline_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("timeline authoring registration");
    let operation =
        EditorOperationPath::parse("timeline_sequence.keyframe.move").expect("timeline operation");
    let descriptor = registry
        .operations()
        .descriptor(&operation)
        .expect("move keyframe operation registered");

    assert_eq!(
        descriptor.menu_path(),
        Some("Plugins/Timeline Sequence/Move Keyframe")
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("timeline_sequence.move_keyframe.v1")
    );
    assert!(registry.menu_items().iter().any(|item| {
        item.path() == "Plugins/Timeline Sequence/Move Keyframe" && item.operation() == &operation
    }));
}

#[test]
fn timeline_sequence_package_manifest_declares_editor_only_metadata() {
    let manifest = package_manifest();
    let editor_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Editor)
        .expect("timeline sequence editor module");

    assert_eq!(manifest.category, "authoring");
    assert_eq!(
        manifest.supported_targets,
        vec![zircon_runtime::builtin::RuntimeTargetMode::EditorHost]
    );
    assert_eq!(manifest.capabilities, vec![CAPABILITY.to_string()]);
    assert_eq!(editor_module.capabilities, manifest.capabilities);
}

#[test]
fn timeline_sequence_package_manifest_declares_editor_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("timeline_sequence declares standalone distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, TIMELINE_SEQUENCE_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert!(distribution.runtime_entry.is_empty());
    assert_eq!(
        distribution.editor_entry,
        TIMELINE_SEQUENCE_DIST_EDITOR_ENTRY
    );

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "timeline_sequence.dist")
        .expect("timeline_sequence dist module is declared");
    assert_eq!(dist_module.kind, PluginModuleKind::Native);
    assert_eq!(dist_module.crate_name, TIMELINE_SEQUENCE_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(dist_module.capabilities, vec![CAPABILITY.to_string()]);
}

#[test]
fn timeline_sequence_validation_accepts_sorted_keyframes_in_range() {
    let sequence = sequence_with_keys([0.0, 0.5, 1.0]);

    assert!(validate_timeline_sequence(&sequence).is_empty());
}

#[test]
fn timeline_sequence_validation_reports_range_and_sorting_errors() {
    let sequence = sequence_with_keys([0.75, 0.25, 1.5]);

    let diagnostics = validate_timeline_sequence(&sequence);

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("outside timeline range")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("must be sorted by time")));
}

#[test]
fn timeline_track_paths_are_sorted_for_deterministic_authoring() {
    let sequence = AnimationSequenceAsset {
        name: Some("Timeline".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![
            binding("root/z", "Transform.translation"),
            binding("root/a", "Transform.translation"),
        ],
    };

    assert_eq!(
        sorted_timeline_track_paths(&sequence),
        vec![
            "root/a:Transform.translation".to_string(),
            "root/z:Transform.translation".to_string()
        ]
    );
}

#[test]
fn timeline_keyframe_move_updates_time_and_restores_track_sort_order() {
    let mut sequence = sequence_with_keys([0.0, 0.25, 1.0]);

    move_timeline_keyframe(
        &mut sequence,
        &TimelineKeyframeMoveRequest {
            binding_index: 0,
            track_index: 0,
            key_index: 0,
            new_time_seconds: 0.75,
        },
    )
    .expect("keyframe move is valid");

    let times = sequence.bindings[0].tracks[0]
        .channel
        .keys
        .iter()
        .map(|key| key.time_seconds)
        .collect::<Vec<_>>();
    assert_eq!(times, vec![0.25, 0.75, 1.0]);
}

#[test]
fn timeline_keyframe_move_reports_bad_indices_and_time_range() {
    let mut sequence = sequence_with_keys([0.0, 0.25, 1.0]);

    let diagnostics = move_timeline_keyframe(
        &mut sequence,
        &TimelineKeyframeMoveRequest {
            binding_index: 0,
            track_index: 0,
            key_index: 5,
            new_time_seconds: 2.0,
        },
    )
    .expect_err("keyframe index and time are invalid");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("outside timeline range")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("keyframe index 5")));
}

#[test]
fn timeline_event_marker_payload_validation_rejects_empty_event_and_bad_payload_key() {
    let marker = TimelineEventMarker {
        time_seconds: 1.0,
        event: " ".to_string(),
        payload: BTreeMap::new(),
    };
    assert!(validate_event_marker_payload(&marker, 1.0)
        .expect_err("event name is required")
        .contains("must name an event"));

    let mut payload = BTreeMap::new();
    payload.insert(" ".to_string(), "value".to_string());
    let marker = TimelineEventMarker {
        time_seconds: 0.5,
        event: "Footstep".to_string(),
        payload: payload.clone(),
    };
    assert!(validate_event_marker_payload(&marker, 1.0)
        .expect_err("payload keys are checked")
        .contains("payload keys must not be empty"));

    let marker = TimelineEventMarker {
        time_seconds: 2.0,
        event: "Footstep".to_string(),
        payload,
    };
    assert!(validate_event_marker_payload(&marker, 1.0)
        .expect_err("event time range is checked")
        .contains("outside timeline range"));
}

fn sequence_with_keys(times: [f32; 3]) -> AnimationSequenceAsset {
    let mut binding = binding("root/player", "Transform.translation");
    binding.tracks[0].channel.keys = times
        .into_iter()
        .map(|time_seconds| AnimationChannelKeyAsset {
            time_seconds,
            value: AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0]),
            in_tangent: None,
            out_tangent: None,
        })
        .collect();
    AnimationSequenceAsset {
        name: Some("Timeline".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![binding],
    }
}

fn binding(entity: &str, property: &str) -> AnimationSequenceBindingAsset {
    AnimationSequenceBindingAsset {
        entity_path: EntityPath::parse(entity).unwrap(),
        target_id: None,
        tracks: vec![AnimationSequenceTrackAsset {
            property_path: ComponentPropertyPath::parse(property).unwrap(),
            channel: AnimationChannelAsset {
                interpolation: AnimationInterpolationAsset::Hermite,
                keys: Vec::new(),
            },
        }],
    }
}
