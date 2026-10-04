use super::*;
use crate::graphics::feature::render_feature_pass_descriptor::{
    RenderFeatureResourceAccess, RenderFeatureResourceKind,
};

#[test]
fn motion_vector_tile_max_uses_its_fullscreen_input_once() {
    let descriptor = descriptor();
    let pass = descriptor
        .stage_passes
        .iter()
        .find(|pass| pass.pass_name == "motion-vector-tile-max")
        .expect("motion-vector tile-max pass");

    let velocity_reads = pass
        .resources
        .iter()
        .filter(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY
                && resource.kind == RenderFeatureResourceKind::Texture
                && resource.access == RenderFeatureResourceAccess::Read
        })
        .count();

    assert_eq!(velocity_reads, 1);
}
