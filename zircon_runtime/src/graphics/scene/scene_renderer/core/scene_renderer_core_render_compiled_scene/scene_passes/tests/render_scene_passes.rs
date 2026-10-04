use crate::graphics::pipeline::RenderPassStage;

#[test]
fn deferred_scene_executes_ambient_occlusion_between_gbuffer_and_lighting() {
    let source = include_str!("../render_scene_passes.rs");
    let deferred = source
        .find("RenderPassStage::Deferred,")
        .expect("deferred GBuffer stage");
    let ambient_occlusion = source[deferred..]
        .find("RenderPassStage::AmbientOcclusion,")
        .map(|offset| deferred + offset)
        .expect("deferred ambient-occlusion stage");
    let lighting = source[ambient_occlusion..]
        .find("RenderPassStage::Lighting,")
        .map(|offset| ambient_occlusion + offset)
        .expect("deferred lighting stage");

    assert!(deferred < ambient_occlusion);
    assert!(ambient_occlusion < lighting);

    let alpha_mask = source
        .find("RenderPassStage::AlphaMask3d,")
        .expect("deferred alpha-mask stage");
    assert!(deferred < alpha_mask);
    assert!(alpha_mask < ambient_occlusion);
}
