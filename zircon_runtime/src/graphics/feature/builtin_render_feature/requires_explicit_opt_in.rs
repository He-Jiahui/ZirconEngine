use super::builtin_render_feature::BuiltinRenderFeature;

impl BuiltinRenderFeature {
    pub const fn requires_explicit_opt_in(self) -> bool {
        matches!(
            self,
            Self::MeshLod
                | Self::GlobalIllumination
                | Self::HistoryResolve
                | Self::Particle
                | Self::NeuralCompute
                | Self::SparseTexture
                | Self::Terrain
                | Self::Tree
                | Self::Decal
                | Self::Projector
                | Self::Halo
                | Self::LensFlare
                | Self::Trail
                | Self::Billboard
                | Self::Tilemap
                | Self::TextShaping
                | Self::Skybox
                | Self::Cubemap
                | Self::Texture2dArray
                | Self::NormalMap
                | Self::Mipmap
                | Self::ColorSpace
                | Self::RayTracing
                | Self::VirtualGeometry
        )
    }
}
