use crate::core::framework::render::RenderPluginRendererOutputs;

pub(in crate::graphics::scene::scene_renderer::core) fn merge_plugin_renderer_outputs(
    base: &mut RenderPluginRendererOutputs,
    incoming: RenderPluginRendererOutputs,
) {
    if !incoming.virtual_geometry.is_empty() {
        base.virtual_geometry = incoming.virtual_geometry;
    }
    if !incoming.hybrid_gi.is_empty() {
        base.hybrid_gi = incoming.hybrid_gi;
    }
    if !incoming.particles.is_empty() {
        base.particles = incoming.particles;
    }
}
