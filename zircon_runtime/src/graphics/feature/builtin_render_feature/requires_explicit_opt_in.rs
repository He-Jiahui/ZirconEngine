use super::builtin_render_feature::BuiltinRenderFeature;

impl BuiltinRenderFeature {
    pub const fn requires_explicit_opt_in(self) -> bool {
        matches!(
            self,
            Self::GlobalIllumination
                | Self::HistoryResolve
                | Self::RayTracing
                | Self::VirtualGeometry
        )
    }
}
