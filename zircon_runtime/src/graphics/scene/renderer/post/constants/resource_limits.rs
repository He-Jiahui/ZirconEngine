// 固定后处理上传缓冲区的预算；编码器必须先截断有效前缀，再把计数传给着色器。
// 这里的反射探针上限属于屏幕空间后处理布局，与 environment 的场景探针池独立。
pub(in crate::graphics::scene::scene_renderer::post_process) const MAX_DIRECTIONAL_LIGHTS: usize =
    8;
pub(in crate::graphics::scene::scene_renderer::post_process) const MAX_REFLECTION_PROBES: usize = 8;
pub(in crate::graphics::scene::scene_renderer::post_process) const MAX_HYBRID_GI_PROBES: usize = 16;
pub(in crate::graphics::scene::scene_renderer::post_process) const MAX_HYBRID_GI_TRACE_REGIONS:
    usize = 16;
