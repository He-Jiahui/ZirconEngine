use super::*;

#[test]
fn public_frame_history_slots_map_to_scene_history_domains() {
    assert_eq!(
        scene_history_domain_for_frame_slot(FrameHistorySlot::AmbientOcclusion),
        SceneHistoryDomain::AmbientOcclusion
    );
    assert_eq!(
        scene_history_domain_for_frame_slot(FrameHistorySlot::GlobalIllumination),
        SceneHistoryDomain::HybridGlobalIllumination
    );
    assert_eq!(
        scene_history_domain_for_frame_slot(FrameHistorySlot::HzbFurthest),
        SceneHistoryDomain::HzbFurthest
    );
    assert_eq!(
        scene_history_domain_for_frame_slot(FrameHistorySlot::TaaSceneColor),
        SceneHistoryDomain::TaaSceneColor
    );
}
