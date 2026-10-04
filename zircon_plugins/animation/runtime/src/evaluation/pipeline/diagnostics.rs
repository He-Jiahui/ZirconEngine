use zircon_runtime::core::CoreHandle;

use super::requests::AnimationSceneScan;

const ANIMATION_SCENE_SCANNED_ENTITIES_DIAGNOSTIC: &str = "animation.scene.scanned_entities";
const ANIMATION_SCENE_SEQUENCE_SAMPLES_DIAGNOSTIC: &str = "animation.scene.sequence_samples";
const ANIMATION_SCENE_CLIP_POSE_SAMPLES_DIAGNOSTIC: &str = "animation.scene.clip_pose_samples";
const ANIMATION_SCENE_CLIP_EVENT_SAMPLES_DIAGNOSTIC: &str = "animation.scene.clip_event_samples";
const ANIMATION_SCENE_GRAPH_POSE_SAMPLES_DIAGNOSTIC: &str = "animation.scene.graph_pose_samples";
const ANIMATION_SCENE_STATE_MACHINE_POSE_SAMPLES_DIAGNOSTIC: &str =
    "animation.scene.state_machine_pose_samples";
const ANIMATION_SCENE_OUTPUT_POSES_DIAGNOSTIC: &str = "animation.scene.output_poses";
const ANIMATION_SCENE_APPLIED_TRANSFORMS_DIAGNOSTIC: &str = "animation.scene.applied_transforms";
const ANIMATION_SCENE_PUBLISHED_EVENTS_DIAGNOSTIC: &str = "animation.scene.published_events";
const ANIMATION_SCENE_STATE_TRANSITIONS_DIAGNOSTIC: &str = "animation.scene.state_transitions";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AnimationSceneFrameDiagnostics {
    pub(super) scanned_entities: usize,
    pub(super) sequence_samples: usize,
    pub(super) clip_pose_samples: usize,
    pub(super) clip_event_samples: usize,
    pub(super) graph_pose_samples: usize,
    pub(super) state_machine_pose_samples: usize,
    pub(super) output_poses: usize,
    pub(super) applied_transforms: usize,
    pub(super) published_events: usize,
    pub(super) state_transitions: usize,
}

impl AnimationSceneFrameDiagnostics {
    pub(super) fn from_scan(scan: &AnimationSceneScan) -> Self {
        // The typed projection deliberately avoids a whole-world node scan. This counter tracks
        // the skeleton-indexed candidates it actually evaluates instead of reintroducing one.
        Self {
            scanned_entities: scan.skeletons_by_entity.len(),
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
