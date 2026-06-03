use std::borrow::Cow;

const RAW_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(11) var scene_depth_tex: texture_depth_2d;";
const FALLBACK_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(11) var scene_depth_tex: texture_2d<f32>;";
const RAW_DEPTH_SAMPLE_RETURN: &str =
    "return clamp(textureSample(scene_depth_tex, scene_depth_sampler, uv), 0.0, 1.0);";
const FALLBACK_DEPTH_SAMPLE_RETURN: &str = "return clamp(uv.y, 0.0, 1.0);";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::post_process) enum PostProcessDepthSamplingMode {
    RawDepthTexture,
    ViewportDepthFallback,
}

impl PostProcessDepthSamplingMode {
    pub(in crate::graphics::scene::scene_renderer::post_process) fn for_backend_name(
        backend_name: &str,
    ) -> Self {
        let normalized_backend_name = backend_name.to_ascii_lowercase();
        if normalized_backend_name.contains("gl") || normalized_backend_name.contains("angle") {
            Self::ViewportDepthFallback
        } else {
            Self::RawDepthTexture
        }
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn scene_depth_sample_type(
        self,
    ) -> wgpu::TextureSampleType {
        match self {
            Self::RawDepthTexture => wgpu::TextureSampleType::Depth,
            Self::ViewportDepthFallback => wgpu::TextureSampleType::Float { filterable: false },
        }
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn post_process_shader_source(
        self,
        raw_shader_source: &'static str,
    ) -> Cow<'static, str> {
        match self {
            Self::RawDepthTexture => Cow::Borrowed(raw_shader_source),
            Self::ViewportDepthFallback => Cow::Owned(
                raw_shader_source
                    .replace(
                        RAW_DEPTH_BINDING_DECLARATION,
                        FALLBACK_DEPTH_BINDING_DECLARATION,
                    )
                    .replace(RAW_DEPTH_SAMPLE_RETURN, FALLBACK_DEPTH_SAMPLE_RETURN),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PostProcessDepthSamplingMode;

    const POST_PROCESS_SHADER: &str = include_str!("../shaders/post_process.wgsl");

    #[test]
    fn gl_backends_use_viewport_depth_fallback() {
        assert_eq!(
            PostProcessDepthSamplingMode::for_backend_name("wgpu(gl)"),
            PostProcessDepthSamplingMode::ViewportDepthFallback
        );
        assert_eq!(
            PostProcessDepthSamplingMode::for_backend_name("wgpu(webgl)"),
            PostProcessDepthSamplingMode::ViewportDepthFallback
        );
        assert_eq!(
            PostProcessDepthSamplingMode::for_backend_name("wgpu(ANGLE)"),
            PostProcessDepthSamplingMode::ViewportDepthFallback
        );
    }

    #[test]
    fn non_gl_backends_keep_raw_depth_texture_sampling() {
        assert_eq!(
            PostProcessDepthSamplingMode::for_backend_name("wgpu(vulkan)"),
            PostProcessDepthSamplingMode::RawDepthTexture
        );
        assert_eq!(
            PostProcessDepthSamplingMode::for_backend_name("wgpu(dx12)"),
            PostProcessDepthSamplingMode::RawDepthTexture
        );
        assert_eq!(
            PostProcessDepthSamplingMode::for_backend_name("wgpu(metal)"),
            PostProcessDepthSamplingMode::RawDepthTexture
        );
    }

    #[test]
    fn viewport_depth_fallback_shader_removes_raw_depth_texture_sampling() {
        let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
            .post_process_shader_source(POST_PROCESS_SHADER);

        naga::front::wgsl::parse_str(&shader_source)
            .expect("fallback post-process shader must parse");
        assert!(!shader_source.contains("texture_depth_2d"));
        assert!(!shader_source.contains("textureSample(scene_depth_tex"));
        assert!(
            shader_source.contains("@group(0) @binding(11) var scene_depth_tex: texture_2d<f32>;")
        );
        assert!(shader_source.contains("return clamp(uv.y, 0.0, 1.0);"));
    }
}
