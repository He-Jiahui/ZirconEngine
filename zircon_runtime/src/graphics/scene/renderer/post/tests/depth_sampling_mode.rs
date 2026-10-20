use super::super::shader_sources::POST_PROCESS_SHADER;
use super::PostProcessDepthSamplingMode;

const DEPTH_OF_FIELD_PREPARE_SHADER: &str =
    include_str!("../../../post/shaders/depth_of_field_prepare.wgsl");
const VELOCITY_CAMERA_SHADER: &str =
    include_str!("../../../temporal/velocity/shaders/velocity_camera.wgsl");
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

    naga::front::wgsl::parse_str(&shader_source).expect("raw-depth post-process shader must parse");
    assert!(shader_source
        .contains("return clamp(textureLoad(scene_depth_tex, physical_coord, 0), 0.0, 1.0);"));
    assert!(!shader_source.contains("textureSample(scene_depth_tex"));
}

#[test]
fn viewport_depth_fallback_shader_removes_raw_depth_texture_sampling() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .post_process_shader_source(POST_PROCESS_SHADER);

    naga::front::wgsl::parse_str(&shader_source).expect("fallback post-process shader must parse");
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureSample(scene_depth_tex"));
    assert!(shader_source.contains("@group(0) @binding(11) var scene_depth_tex: texture_2d<f32>;"));
    assert!(shader_source.contains(
        "return clamp((vec2<f32>(clamped) + vec2<f32>(0.5, 0.5)).y / f32(viewport_size.y), 0.0, 1.0);"
    ));
}

#[test]
fn viewport_depth_fallback_rewrites_depth_of_field_prepare_shader() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .depth_of_field_prepare_shader_source(DEPTH_OF_FIELD_PREPARE_SHADER);

    naga::front::wgsl::parse_str(&shader_source).expect("fallback DoF prepare shader must parse");
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
    assert!(shader_source.contains("@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;"));
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
    assert!(shader_source.contains("@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;"));
    assert!(shader_source.contains("return clamp((vec2<f32>(clamped)"));
}

#[test]
fn viewport_depth_fallback_rewrites_taa_resolve_shader() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .taa_resolve_shader_source(TAA_RESOLVE_SHADER);

    naga::front::wgsl::parse_str(&shader_source).expect("fallback TAA shader must parse");
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
    assert!(shader_source.contains("@group(0) @binding(1) var scene_depth_tex: texture_2d<f32>;"));
    assert!(shader_source.contains("return clamp((vec2<f32>(clamped)"));
}

fn regression_device() -> (wgpu::Device, wgpu::Queue) {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("this regression requires a native GPU adapter");
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("render-contract-regression"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .expect("this regression requires a native GPU device")
}

#[test]
fn viewport_depth_fallback_taa_shader_validates_and_creates_native_pipeline() {
    let source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .taa_resolve_shader_source(TAA_RESOLVE_SHADER);
    let module = naga::front::wgsl::parse_str(&source)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .expect("the complete generated TAA fallback must validate");
    let (device, _queue) = regression_device();
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("complete-production-taa-viewport-depth-fallback"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let target = Some(wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba16Float,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    });
    let _pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("taa-viewport-depth-fallback-regression"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_taa_resolve"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[target.clone(), target],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
}
