use super::super::builtin_render_feature::BuiltinRenderFeature;
use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::descriptor_for::descriptor_for;

impl BuiltinRenderFeature {
    /// 将内建特性映射为编译阶段使用的完整 pass、资源和能力描述。
    pub fn descriptor(self) -> RenderFeatureDescriptor {
        descriptor_for(self)
    }
}
