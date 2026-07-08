use super::sources::*;

#[test]
fn runtime_05_dynamic_scene_patch_preview_behavior_anchors_stay_visible() {
    assert!(
        BEHAVIOR_SOURCE.contains("scene_patch_preview_reports_remaps_without_mutating_target_world")
            && BEHAVIOR_SOURCE.contains("scene_patch_applies_reflected_resources")
            && BEHAVIOR_SOURCE.contains(".preview_apply(&target)")
            && BEHAVIOR_SOURCE.contains("preview.resources[0].type_path")
            && BEHAVIOR_SOURCE.contains("preview.resources_requiring_creation()")
            && BEHAVIOR_SOURCE.contains("register_frame_counter_resource_with_ensure")
            && BEHAVIOR_SOURCE.contains("frame_counter_adapter_with_ensure")
            && BEHAVIOR_SOURCE.contains("frame_counter_ensure")
            && BEHAVIOR_SOURCE.contains("preview_with_ensure.resources[0].can_create_on_apply")
            && BEHAVIOR_SOURCE.contains("target_with_ensure.get_resource::<FrameCounter>().is_none()")
            && BEHAVIOR_SOURCE.contains("preview.entity_remaps[0].source_entity")
            && BEHAVIOR_SOURCE.contains("preview.entity_remaps[0].target_entity")
            && BEHAVIOR_SOURCE.contains("preview.entity_remaps[1].source_entity")
            && BEHAVIOR_SOURCE.contains("preview.entity_remaps[1].target_entity")
            && BEHAVIOR_SOURCE.contains("preview.component_instance_count")
            && BEHAVIOR_SOURCE.contains("preview.has_entity_remaps()")
            && BEHAVIOR_SOURCE.contains("preview.new_component_type_count")
            && BEHAVIOR_SOURCE.contains("preview.has_new_component_types()")
            && BEHAVIOR_SOURCE.contains("component_types[0].type_id")
            && BEHAVIOR_SOURCE.contains("new_component_types()")
            && BEHAVIOR_SOURCE.contains("target_before")
            && BEHAVIOR_SOURCE.contains("assert!(!target.contains_entity(child));"),
        "focused behavior anchor should keep remap diagnostics and target-world immutability checks"
    );
}
