use super::*;

#[test]
fn optimization_batch_20260830cc_avatar_mask_rejects_include_miss_before_scanning_exclusions() {
    let mask = AnimationAvatarMask {
        id: "upper_body".to_string(),
        included_target_ids: vec!["Rig/Spine/Chest".to_string()],
        excluded_target_ids: vec!["Rig/Face/Jaw".to_string()],
        weight: 1.0,
    };

    assert!(!mask.allows_target("Rig/Hands/Left"));

    let source = include_str!("../avatar_mask.rs");
    let allows_target = source
        .split("pub fn allows_target")
        .nth(1)
        .and_then(|source| source.split("pub fn normalized_weight").next())
        .expect("read avatar-mask target filtering");
    let include_guard = allows_target
        .find("if !self.included_target_ids.is_empty()")
        .expect("an include miss must have an explicit early-return guard");
    let exclusion_scan = allows_target
        .find("self.excluded_target_ids")
        .expect("avatar-mask exclusions must remain enforced");

    assert!(
        include_guard < exclusion_scan,
        "include rejection must happen before the exclusion list is scanned"
    );
    assert!(
        source.contains("PreparedAnimationTargetId"),
        "target path normalization must be prepared once and shared by all list probes"
    );
}

#[test]
fn optimization_batch_20260830cc_avatar_mask_leaf_matching_semantics_remain_symmetric() {
    assert!(animation_target_id_matches("Rig/Spine/Chest", "Chest"));
    assert!(animation_target_id_matches("Chest", "Rig/Spine/Chest"));
    assert!(!animation_target_id_matches(
        "Rig/Spine/Chest",
        "Rig/Face/Chest"
    ));
}
