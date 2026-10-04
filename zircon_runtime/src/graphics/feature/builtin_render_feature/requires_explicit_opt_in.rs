use super::advanced_slots::is_descriptor_only_advanced_slot;
use super::builtin_render_feature::BuiltinRenderFeature;

impl BuiltinRenderFeature {
    /// 编译选项在收录内建特性前调用此判定，只有声明需要显式能力或高级槽位的特性才等待用户开关。
    pub fn requires_explicit_opt_in(self) -> bool {
        is_descriptor_only_advanced_slot(self)
            || matches!(
                self,
                Self::GlobalIllumination
                    | Self::NeuralCompute
                    | Self::RayTracing
                    | Self::VirtualGeometry
            )
    }
}
