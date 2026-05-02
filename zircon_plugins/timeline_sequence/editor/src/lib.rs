use std::collections::BTreeMap;

use zircon_editor::core::editor_authoring_extension::{
    TimelineEditorDescriptor, TimelineTrackDescriptor,
};
use zircon_editor::core::editor_extension::AssetEditorDescriptor;
use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::asset::AnimationSequenceAsset;

pub const PLUGIN_ID: &str = "timeline_sequence";
pub const CAPABILITY: &str = "editor.extension.timeline_sequence_authoring";
pub const TIMELINE_SEQUENCE_VIEW_ID: &str = "timeline_sequence.authoring";
pub const TIMELINE_SEQUENCE_DRAWER_ID: &str = "timeline_sequence.drawer";
pub const TIMELINE_SEQUENCE_TEMPLATE_ID: &str = "timeline_sequence.authoring";

#[derive(Clone, Debug)]
pub struct TimelineSequenceEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl TimelineSequenceEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for TimelineSequenceEditorPlugin {
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
                drawer_id: TIMELINE_SEQUENCE_DRAWER_ID,
                drawer_display_name: "Timeline Sequence",
                template_id: TIMELINE_SEQUENCE_TEMPLATE_ID,
                template_document: "plugins://timeline_sequence/editor/authoring.ui.toml",
                surfaces: &[EditorAuthoringSurface::new(
                    TIMELINE_SEQUENCE_VIEW_ID,
                    "Timeline Sequence",
                    "Animation",
                    "Plugins/Timeline Sequence",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, timeline_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Timeline Sequence",
        "zircon_plugin_timeline_sequence_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> TimelineSequenceEditorPlugin {
    TimelineSequenceEditorPlugin::new()
}

fn base_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::PluginPackageManifest::new(PLUGIN_ID, "Timeline Sequence")
        .with_category("authoring")
        .with_dependency(
            zircon_runtime::plugin::PluginDependencyManifest::new("animation", true)
                .with_capability("runtime.feature.animation.timeline_event_track"),
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

fn timeline_authoring_batch() -> EditorAuthoringContributionBatch {
    let open = operation("TimelineSequence.Authoring.Open");
    let create_track = operation("TimelineSequence.Track.Create");
    let delete_track = operation("TimelineSequence.Track.Delete");
    let move_key = operation("TimelineSequence.Keyframe.Move");
    let validate = operation("TimelineSequence.Authoring.Validate");
    EditorAuthoringContributionBatch {
        operations: vec![
            EditorOperationDescriptor::new(open.clone(), "Open Timeline Sequence")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(create_track, "Create Timeline Track")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(delete_track, "Delete Timeline Track")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(move_key, "Move Timeline Keyframe")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(validate, "Validate Timeline Sequence")
                .with_required_capabilities([CAPABILITY]),
        ],
        asset_editors: vec![AssetEditorDescriptor::new(
            "animation.sequence",
            TIMELINE_SEQUENCE_VIEW_ID,
            "Timeline Sequence",
            open.clone(),
        )
        .with_required_capabilities([CAPABILITY])],
        timeline_track_types: vec![
            TimelineTrackDescriptor::new(
                "timeline_sequence.track.transform",
                "Transform",
                "transform",
            )
            .with_required_capabilities([CAPABILITY]),
            TimelineTrackDescriptor::new(
                "timeline_sequence.track.component_property",
                "Component Property",
                "component_property",
            )
            .with_required_capabilities([CAPABILITY]),
            TimelineTrackDescriptor::new(
                "timeline_sequence.track.event_marker",
                "Event Marker",
                "event_marker",
            )
            .with_required_capabilities([CAPABILITY]),
        ],
        timeline_editors: vec![TimelineEditorDescriptor::new(
            "animation.sequence",
            TIMELINE_SEQUENCE_VIEW_ID,
            "Timeline Sequence",
            open,
        )
        .with_track_type("timeline_sequence.track.transform")
        .with_track_type("timeline_sequence.track.component_property")
        .with_track_type("timeline_sequence.track.event_marker")
        .with_required_capabilities([CAPABILITY])],
        ..Default::default()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineEventMarker {
    pub time_seconds: f32,
    pub event: String,
    pub payload: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineKeyframeMoveRequest {
    pub binding_index: usize,
    pub track_index: usize,
    pub key_index: usize,
    pub new_time_seconds: f32,
}

pub fn validate_timeline_sequence(sequence: &AnimationSequenceAsset) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if sequence.duration_seconds <= 0.0 {
        diagnostics.push("timeline duration must be greater than zero".to_string());
    }
    if sequence.frames_per_second <= 0.0 {
        diagnostics.push("timeline frame rate must be greater than zero".to_string());
    }

    for binding in &sequence.bindings {
        for track in &binding.tracks {
            let mut previous_time = None;
            for key in &track.channel.keys {
                if key.time_seconds < 0.0 || key.time_seconds > sequence.duration_seconds {
                    diagnostics.push(format!(
                        "keyframe `{}` on `{}` is outside timeline range 0..{}",
                        key.time_seconds, track.property_path, sequence.duration_seconds
                    ));
                }
                if let Some(previous_time) = previous_time {
                    if key.time_seconds < previous_time {
                        diagnostics.push(format!(
                            "keyframes on `{}` must be sorted by time",
                            track.property_path
                        ));
                    }
                }
                previous_time = Some(key.time_seconds);
            }
        }
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn move_timeline_keyframe(
    sequence: &mut AnimationSequenceAsset,
    request: &TimelineKeyframeMoveRequest,
) -> Result<(), Vec<String>> {
    let mut diagnostics = Vec::new();
    if request.new_time_seconds < 0.0 || request.new_time_seconds > sequence.duration_seconds {
        diagnostics.push(format!(
            "timeline keyframe move target `{}` is outside timeline range 0..{}",
            request.new_time_seconds, sequence.duration_seconds
        ));
    }
    let Some(binding) = sequence.bindings.get_mut(request.binding_index) else {
        diagnostics.push(format!(
            "timeline binding index {} is outside {} bindings",
            request.binding_index,
            sequence.bindings.len()
        ));
        return Err(diagnostics);
    };
    let Some(track) = binding.tracks.get_mut(request.track_index) else {
        diagnostics.push(format!(
            "timeline track index {} is outside {} tracks",
            request.track_index,
            binding.tracks.len()
        ));
        return Err(diagnostics);
    };
    let Some(key) = track.channel.keys.get_mut(request.key_index) else {
        diagnostics.push(format!(
            "timeline keyframe index {} is outside {} keys",
            request.key_index,
            track.channel.keys.len()
        ));
        return Err(diagnostics);
    };
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    key.time_seconds = request.new_time_seconds;
    track
        .channel
        .keys
        .sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
    let diagnostics = validate_timeline_sequence(sequence);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub fn sorted_timeline_track_paths(sequence: &AnimationSequenceAsset) -> Vec<String> {
    let mut paths = sequence
        .bindings
        .iter()
        .flat_map(|binding| {
            binding
                .tracks
                .iter()
                .map(|track| format!("{}:{}", binding.entity_path, track.property_path))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

pub fn validate_event_marker_payload(
    marker: &TimelineEventMarker,
    duration_seconds: f32,
) -> Result<(), String> {
    if marker.event.trim().is_empty() {
        return Err("timeline event marker must name an event".to_string());
    }
    if marker.time_seconds < 0.0 || marker.time_seconds > duration_seconds {
        return Err(format!(
            "timeline event marker `{}` is outside timeline range 0..{}",
            marker.event, duration_seconds
        ));
    }
    if marker.payload.keys().any(|key| key.trim().is_empty()) {
        return Err("timeline event marker payload keys must not be empty".to_string());
    }
    Ok(())
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid timeline operation path")
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::asset::{
        AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
        AnimationInterpolationAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
    };
    use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

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
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse(property).unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Hermite,
                    keys: Vec::new(),
                },
            }],
        }
    }
}
