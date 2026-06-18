use crate::core::CoreHandle;

use super::pending::AnimationSceneScan;

pub(super) const ANIMATION_SCENE_SCANNED_ENTITIES_DIAGNOSTIC: &str =
    "animation.scene.scanned_entities";
pub(super) const ANIMATION_SCENE_SEQUENCE_SAMPLES_DIAGNOSTIC: &str =
    "animation.scene.sequence_samples";
pub(super) const ANIMATION_SCENE_CLIP_POSE_SAMPLES_DIAGNOSTIC: &str =
    "animation.scene.clip_pose_samples";
pub(super) const ANIMATION_SCENE_CLIP_EVENT_SAMPLES_DIAGNOSTIC: &str =
    "animation.scene.clip_event_samples";
pub(super) const ANIMATION_SCENE_GRAPH_POSE_SAMPLES_DIAGNOSTIC: &str =
    "animation.scene.graph_pose_samples";
pub(super) const ANIMATION_SCENE_STATE_MACHINE_POSE_SAMPLES_DIAGNOSTIC: &str =
    "animation.scene.state_machine_pose_samples";
pub(super) const ANIMATION_SCENE_OUTPUT_POSES_DIAGNOSTIC: &str = "animation.scene.output_poses";
pub(super) const ANIMATION_SCENE_APPLIED_TRANSFORMS_DIAGNOSTIC: &str =
    "animation.scene.applied_transforms";
pub(super) const ANIMATION_SCENE_PUBLISHED_EVENTS_DIAGNOSTIC: &str =
    "animation.scene.published_events";
pub(super) const ANIMATION_SCENE_STATE_TRANSITIONS_DIAGNOSTIC: &str =
    "animation.scene.state_transitions";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AnimationSceneFrameDiagnostics {
    pub scanned_entities: usize,
    pub sequence_samples: usize,
    pub clip_pose_samples: usize,
    pub clip_event_samples: usize,
    pub graph_pose_samples: usize,
    pub state_machine_pose_samples: usize,
    pub output_poses: usize,
    pub applied_transforms: usize,
    pub published_events: usize,
    pub state_transitions: usize,
}

impl AnimationSceneFrameDiagnostics {
    pub(super) fn from_scan(scan: &AnimationSceneScan) -> Self {
        Self {
            scanned_entities: scan.scanned_entities,
            sequence_samples: scan.sequences.len(),
            clip_pose_samples: scan.clip_samples.len(),
            clip_event_samples: scan.clip_event_samples.len(),
            graph_pose_samples: scan.graph_samples.len(),
            state_machine_pose_samples: scan.state_machine_samples.len(),
            ..Self::default()
        }
    }

    pub(super) fn record(self, core: &CoreHandle) {
        let frame_index = core.real_time().frame_index();
        for (path, value) in [
            (
                ANIMATION_SCENE_SCANNED_ENTITIES_DIAGNOSTIC,
                self.scanned_entities,
            ),
            (
                ANIMATION_SCENE_SEQUENCE_SAMPLES_DIAGNOSTIC,
                self.sequence_samples,
            ),
            (
                ANIMATION_SCENE_CLIP_POSE_SAMPLES_DIAGNOSTIC,
                self.clip_pose_samples,
            ),
            (
                ANIMATION_SCENE_CLIP_EVENT_SAMPLES_DIAGNOSTIC,
                self.clip_event_samples,
            ),
            (
                ANIMATION_SCENE_GRAPH_POSE_SAMPLES_DIAGNOSTIC,
                self.graph_pose_samples,
            ),
            (
                ANIMATION_SCENE_STATE_MACHINE_POSE_SAMPLES_DIAGNOSTIC,
                self.state_machine_pose_samples,
            ),
            (ANIMATION_SCENE_OUTPUT_POSES_DIAGNOSTIC, self.output_poses),
            (
                ANIMATION_SCENE_APPLIED_TRANSFORMS_DIAGNOSTIC,
                self.applied_transforms,
            ),
            (
                ANIMATION_SCENE_PUBLISHED_EVENTS_DIAGNOSTIC,
                self.published_events,
            ),
            (
                ANIMATION_SCENE_STATE_TRANSITIONS_DIAGNOSTIC,
                self.state_transitions,
            ),
        ] {
            core.record_diagnostic(
                path,
                frame_index,
                value as f64,
                Some("count"),
                ["animation", "scene"],
            );
        }
    }
}
