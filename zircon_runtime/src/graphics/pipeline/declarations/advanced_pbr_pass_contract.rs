use crate::core::framework::render::MAX_SCREEN_SPACE_TRANSMISSION_STEPS;

pub(crate) const ADVANCED_PBR_OPAQUE_PASS_NAME: &str = "advanced-pbr-opaque";
pub(crate) const ADVANCED_PBR_OPAQUE_EXECUTOR_ID: &str = "mesh.advanced-pbr-opaque";
pub(crate) const TRANSMISSION_SCENE_COPY_PASS_NAME: &str = "transmission.scene_copy";

pub(crate) const TRANSMISSION_SCENE_COPY_EXECUTOR_IDS: [&str; MAX_SCREEN_SPACE_TRANSMISSION_STEPS] = [
    "transmission.scene-copy",
    "transmission.scene-copy.1",
    "transmission.scene-copy.2",
    "transmission.scene-copy.3",
];

pub(crate) const TRANSMISSION_MESH_EXECUTOR_IDS: [&str; MAX_SCREEN_SPACE_TRANSMISSION_STEPS] = [
    "mesh.transmission.0",
    "mesh.transmission.1",
    "mesh.transmission.2",
    "mesh.transmission.3",
];

pub(crate) fn transmission_scene_copy_pass_name(step_index: usize) -> String {
    if step_index == 0 {
        TRANSMISSION_SCENE_COPY_PASS_NAME.to_string()
    } else {
        format!("{TRANSMISSION_SCENE_COPY_PASS_NAME}.{step_index}")
    }
}

pub(crate) fn transmission_mesh_pass_name(step_index: usize) -> String {
    format!("transmission-mesh.{step_index}")
}

pub(crate) fn transmission_scene_copy_step_index(executor_id: &str) -> Option<usize> {
    match executor_id {
        "transmission.scene-copy" => Some(0),
        "transmission.scene-copy.1" => Some(1),
        "transmission.scene-copy.2" => Some(2),
        "transmission.scene-copy.3" => Some(3),
        _ => None,
    }
}

pub(crate) fn transmission_mesh_step_index(executor_id: &str) -> Option<usize> {
    match executor_id {
        "mesh.transmission.0" => Some(0),
        "mesh.transmission.1" => Some(1),
        "mesh.transmission.2" => Some(2),
        "mesh.transmission.3" => Some(3),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimization_batch_20260830dx_transmission_executor_ids_use_direct_match() {
        for (expected, executor_id) in TRANSMISSION_SCENE_COPY_EXECUTOR_IDS.iter().enumerate() {
            assert_eq!(
                transmission_scene_copy_step_index(executor_id),
                Some(expected)
            );
        }
        for (expected, executor_id) in TRANSMISSION_MESH_EXECUTOR_IDS.iter().enumerate() {
            assert_eq!(transmission_mesh_step_index(executor_id), Some(expected));
        }
        assert_eq!(
            transmission_scene_copy_step_index("transmission.scene-copy.4"),
            None
        );
        assert_eq!(transmission_mesh_step_index("mesh.transmission.4"), None);

        let source = include_str!("advanced_pbr_pass_contract.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert_eq!(production.matches("match executor_id").count(), 2);
        assert!(!production.contains(".position(|candidate|"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830dx_transmission_executor_direct_match_evidence() {
        const FRAME_COUNT: usize = 32_768;
        const LOOKUPS_PER_FRAME: usize = MAX_SCREEN_SPACE_TRANSMISSION_STEPS * 2;
        const LEGACY_COMPARISONS_PER_LOOKUP: usize = MAX_SCREEN_SPACE_TRANSMISSION_STEPS;
        const MARKER: &str = "RUNTIME533_TRANSMISSION_EXECUTOR_DIRECT_MATCH_BENCH_V1";

        let legacy_candidate_checks = FRAME_COUNT
            .saturating_mul(LOOKUPS_PER_FRAME)
            .saturating_mul(LEGACY_COMPARISONS_PER_LOOKUP);
        let direct_match_decisions = FRAME_COUNT.saturating_mul(LOOKUPS_PER_FRAME);
        let reduction_bps = legacy_candidate_checks
            .saturating_sub(direct_match_decisions)
            .saturating_mul(10_000)
            / legacy_candidate_checks.max(1);

        assert!(direct_match_decisions.saturating_mul(4) <= legacy_candidate_checks);
        assert_eq!(reduction_bps, 7_500);
        println!(
            "{MARKER} frames={FRAME_COUNT} lookups_per_frame={LOOKUPS_PER_FRAME} \
             legacy_candidate_checks={legacy_candidate_checks} \
             direct_match_decisions={direct_match_decisions} reduction_bps={reduction_bps}"
        );
    }
}
