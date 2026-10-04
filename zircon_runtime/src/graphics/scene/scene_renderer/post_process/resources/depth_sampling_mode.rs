//! 按后端选择深度绑定类型，并以固定片段生成 WGSL 变体；视口回退按纵坐标近似深度。
//! 修改深度声明或加载表达式时须同步维护替换片段，并分别确认各变体的作用域。
use std::borrow::Cow;

const RAW_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(11) var scene_depth_tex: texture_depth_2d;";
const FALLBACK_DEPTH_BINDING_DECLARATION: &str =
    "@group(0) @binding(11) var scene_depth_tex: texture_2d<f32>;";
const RAW_DEPTH_SAMPLE_RETURN: &str =
    "return clamp(textureLoad(scene_depth_tex, physical_coord, 0), 0.0, 1.0);";
const FALLBACK_DEPTH_SAMPLE_RETURN: &str =
    "return clamp((vec2<f32>(clamped) + vec2<f32>(0.5, 0.5)).y / f32(viewport_size.y), 0.0, 1.0);";
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
// BUG: [CR-W13-PPRES-0001] TAA 回退表达式引用 size.y，但深度读取函数未定义 size；GL/ANGLE 完整资源构造会引入未声明标识符。
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
        if contains_ascii_case_insensitive(backend_name, "gl")
            || contains_ascii_case_insensitive(backend_name, "angle")
        {
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

fn contains_ascii_case_insensitive(value: &str, expected: &str) -> bool {
    !expected.is_empty()
        && value
            .as_bytes()
            .windows(expected.len())
            .any(|window| window.eq_ignore_ascii_case(expected.as_bytes()))
}

#[cfg(test)]
#[path = "tests/depth_sampling_mode.rs"]
mod tests;

#[cfg(test)]
#[path = "depth_sampling_mode/tests/backend_name_tests.rs"]
mod backend_name_tests;
