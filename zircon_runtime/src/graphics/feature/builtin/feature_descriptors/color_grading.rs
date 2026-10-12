use super::super::render_feature_descriptor::RenderFeatureDescriptor;

// Color grading 只声明 view/post_process 提取依赖，实际颜色处理由对应后处理执行器接管。
pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "color_grading",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        Vec::new(),
    )
}
