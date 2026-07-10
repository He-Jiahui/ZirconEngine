use std::borrow::Cow;

const RAW_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(11) var scene_depth_tex: texture_depth_2d;";
const FALLBACK_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(11) var scene_depth_tex: texture_2d<f32>;";
const RAW_DEPTH_SAMPLE_RETURN: &str =
    "return clamp(textureLoad(scene_depth_tex, physical_coord, 0), 0.0, 1.0);";
const FALLBACK_DEPTH_SAMPLE_RETURN: &str = "return clamp(uv.y, 0.0, 1.0);";
const DOF_PREPARE_RAW_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(0) var scene_depth_tex: texture_depth_2d;";
const DOF_PREPARE_FALLBACK_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;";
const DOF_PREPARE_RAW_DEPTH_LOAD_RETURN: &str = "return clamp(
        textureLoad(scene_depth_tex, params.viewport.zw + clamped, 0),
        0.0,
        1.0
    );";
const DOF_PREPARE_FALLBACK_DEPTH_LOAD_RETURN: &str =
    "return clamp((vec2<f32>(clamped) + vec2<f32>(0.5, 0.5)).y / f32(viewport_size.y), 0.0, 1.0);";
const VELOCITY_CAMERA_RAW_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(0) var scene_depth_tex: texture_depth_2d;";
const VELOCITY_CAMERA_FALLBACK_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;";
const VELOCITY_CAMERA_RAW_DEPTH_LOAD_RETURN: &str =
    "return clamp(textureLoad(scene_depth_tex, clamped, 0), 0.0, 1.0);";
const VELOCITY_CAMERA_FALLBACK_DEPTH_LOAD_RETURN: &str =
    "return clamp((vec2<f32>(clamped) + vec2<f32>(0.5, 0.5)).y / f32(viewport_size.y), 0.0, 1.0);";
const TAA_RESOLVE_RAW_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(1) var scene_depth_tex: texture_depth_2d;";
const TAA_RESOLVE_FALLBACK_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(1) var scene_depth_tex: texture_2d<f32>;";
const TAA_RESOLVE_RAW_DEPTH_LOAD_RETURN: &str =
    "return clamp(textureLoad(scene_depth_tex, clamped, 0), 0.0, 1.0);";
const TAA_RESOLVE_FALLBACK_DEPTH_LOAD_RETURN: &str =
    "return clamp((vec2<f32>(clamped) + vec2<f32>(0.5, 0.5)).y / f32(size.y), 0.0, 1.0);";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::graphics::scene::scene_renderer) enum PostProcessDepthSamplingMode {
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

    pub(in crate::graphics::scene::scene_renderer::post_process) fn depth_of_field_prepare_shader_source(
        self,
        raw_shader_source: &'static str,
    ) -> Cow<'static, str> {
        match self {
            Self::RawDepthTexture => Cow::Borrowed(raw_shader_source),
            Self::ViewportDepthFallback => Cow::Owned(
                raw_shader_source
                    .replace(
                        DOF_PREPARE_RAW_DEPTH_BINDING_DECLARATION,
                        DOF_PREPARE_FALLBACK_DEPTH_BINDING_DECLARATION,
                    )
                    .replace(
                        DOF_PREPARE_RAW_DEPTH_LOAD_RETURN,
                        DOF_PREPARE_FALLBACK_DEPTH_LOAD_RETURN,
                    ),
            ),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn velocity_camera_shader_source(
        self,
        raw_shader_source: &'static str,
    ) -> Cow<'static, str> {
        match self {
            Self::RawDepthTexture => Cow::Borrowed(raw_shader_source),
            Self::ViewportDepthFallback => Cow::Owned(
                raw_shader_source
                    .replace(
                        VELOCITY_CAMERA_RAW_DEPTH_BINDING_DECLARATION,
                        VELOCITY_CAMERA_FALLBACK_DEPTH_BINDING_DECLARATION,
                    )
                    .replace(
                        VELOCITY_CAMERA_RAW_DEPTH_LOAD_RETURN,
                        VELOCITY_CAMERA_FALLBACK_DEPTH_LOAD_RETURN,
                    ),
            ),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::post_process) fn taa_resolve_shader_source(
        self,
        raw_shader_source: &'static str,
    ) -> Cow<'static, str> {
        match self {
            Self::RawDepthTexture => Cow::Borrowed(raw_shader_source),
            Self::ViewportDepthFallback => Cow::Owned(
                raw_shader_source
                    .replace(
                        TAA_RESOLVE_RAW_DEPTH_BINDING_DECLARATION,
                        TAA_RESOLVE_FALLBACK_DEPTH_BINDING_DECLARATION,
                    )
                    .replace(
                        TAA_RESOLVE_RAW_DEPTH_LOAD_RETURN,
                        TAA_RESOLVE_FALLBACK_DEPTH_LOAD_RETURN,
                    ),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::shader_sources::POST_PROCESS_SHADER;
    use super::PostProcessDepthSamplingMode;

    const DEPTH_OF_FIELD_PREPARE_SHADER: &str =
        include_str!("../shaders/depth_of_field_prepare.wgsl");
    const VELOCITY_CAMERA_SHADER: &str =
        include_str!("../../temporal/velocity/shaders/velocity_camera.wgsl");
    const TAA_RESOLVE_SHADER: &str = include_str!("../../temporal/taa/shaders/taa_resolve.wgsl");

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
    fn raw_depth_shader_uses_derivative_free_integer_loads() {
        let shader_source = PostProcessDepthSamplingMode::RawDepthTexture
            .post_process_shader_source(POST_PROCESS_SHADER);

        naga::front::wgsl::parse_str(&shader_source)
            .expect("raw-depth post-process shader must parse");
        assert!(shader_source
            .contains("return clamp(textureLoad(scene_depth_tex, physical_coord, 0), 0.0, 1.0);"));
        assert!(!shader_source.contains("textureSample(scene_depth_tex"));
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

    #[test]
    fn viewport_depth_fallback_rewrites_depth_of_field_prepare_shader() {
        let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
            .depth_of_field_prepare_shader_source(DEPTH_OF_FIELD_PREPARE_SHADER);

        naga::front::wgsl::parse_str(&shader_source)
            .expect("fallback DoF prepare shader must parse");
        assert!(!shader_source.contains("texture_depth_2d"));
        assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
        assert!(
            shader_source.contains("@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;")
        );
        assert!(shader_source.contains("return clamp((vec2<f32>(clamped)"));
    }

    #[test]
    fn viewport_depth_fallback_rewrites_velocity_camera_shader() {
        let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
            .velocity_camera_shader_source(VELOCITY_CAMERA_SHADER);

        naga::front::wgsl::parse_str(&shader_source)
            .expect("fallback camera velocity shader must parse");
        assert!(!shader_source.contains("texture_depth_2d"));
        assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
        assert!(
            shader_source.contains("@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;")
        );
        assert!(shader_source.contains("return clamp((vec2<f32>(clamped)"));
    }

    #[test]
    fn viewport_depth_fallback_rewrites_taa_resolve_shader() {
        let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
            .taa_resolve_shader_source(TAA_RESOLVE_SHADER);

        naga::front::wgsl::parse_str(&shader_source).expect("fallback TAA shader must parse");
        assert!(!shader_source.contains("texture_depth_2d"));
        assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
        assert!(
            shader_source.contains("@group(0) @binding(1) var scene_depth_tex: texture_2d<f32>;")
        );
        assert!(shader_source.contains("return clamp((vec2<f32>(clamped)"));
    }
}
