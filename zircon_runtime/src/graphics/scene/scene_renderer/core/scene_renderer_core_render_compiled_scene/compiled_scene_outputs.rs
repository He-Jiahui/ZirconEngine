use super::super::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;
use super::render::VirtualGeometryIndirectStats;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererCompiledSceneOutputs {
    advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
    virtual_geometry_indirect_stats: VirtualGeometryIndirectStats,
}

impl SceneRendererCompiledSceneOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
        virtual_geometry_indirect_stats: VirtualGeometryIndirectStats,
    ) -> Self {
        Self {
            advanced_plugin_readbacks,
            virtual_geometry_indirect_stats,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn into_parts(
        self,
    ) -> (
        SceneRendererAdvancedPluginReadbacks,
        VirtualGeometryIndirectStats,
    ) {
        (
            self.advanced_plugin_readbacks,
            self.virtual_geometry_indirect_stats,
        )
    }
}
