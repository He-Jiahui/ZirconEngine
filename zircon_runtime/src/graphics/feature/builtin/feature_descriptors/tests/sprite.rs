use super::*;

#[test]
fn sprite_passes_rely_on_scene_resource_consumers_instead_of_culling_roots() {
    assert!(descriptor()
        .stage_passes
        .iter()
        .all(|pass| !pass.flags.has_side_effects));
}
