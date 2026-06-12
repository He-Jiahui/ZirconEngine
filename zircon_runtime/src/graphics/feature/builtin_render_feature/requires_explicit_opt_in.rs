use super::advanced_slots::is_descriptor_only_advanced_slot;
use super::builtin_render_feature::BuiltinRenderFeature;

impl BuiltinRenderFeature {
    pub fn requires_explicit_opt_in(self) -> bool {
        is_descriptor_only_advanced_slot(self)
            || matches!(
                self,
                Self::GlobalIllumination
                    | Self::HistoryResolve
                    | Self::NeuralCompute
                    | Self::RayTracing
                    | Self::VirtualGeometry
            )
    }
}
