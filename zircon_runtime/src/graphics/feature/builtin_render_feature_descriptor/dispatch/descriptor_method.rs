use super::super::builtin_render_feature::BuiltinRenderFeature;
use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::descriptor_for::descriptor_for;

impl BuiltinRenderFeature {
    pub fn descriptor(self) -> RenderFeatureDescriptor {
        descriptor_for(self)
    }
}
