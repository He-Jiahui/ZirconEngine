use crate::asset::ProjectAssetManager;
use crate::core::framework::render::ShadingModelDescriptor;
use crate::graphics::material::ShadingModelIncludeSourceSet;
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::types::GraphicsError;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use super::shader_source::{
    DeferredLightingShaderSourceRequest, assemble_deferred_lighting_shader_source,
};

// The DX12 HLSL writer derives vertex outputs from a same-module fragment input.
// These fragment stubs keep the fullscreen vertex stage self-contained without
// requiring the full deferred lighting module to be compiled for that stage.
const DEFERRED_LIGHTING_FULLSCREEN_VERTEX_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct DeferredLightingMrtStubOutput {
    @location(0) lighting: vec4<f32>,
    @location(1) diffuse: vec4<f32>,
    @location(2) retained: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(@builtin(position) _position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0);
}

@fragment
fn fs_main_sss(@builtin(position) _position: vec4<f32>) -> DeferredLightingMrtStubOutput {
    return DeferredLightingMrtStubOutput(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0));
}
"#;

pub(in crate::graphics::scene::scene_renderer::deferred) struct DeferredLightingPipelineCache {
    lighting_fragment_shader_source: String,
    scene_layout: wgpu::BindGroupLayout,
    gpu_scene_layout: wgpu::BindGroupLayout,
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
    target_format: wgpu::TextureFormat,
    foundation: OnceLock<LightingPipelineFoundation>,
    lighting_pipeline: OnceLock<wgpu::RenderPipeline>,
    lighting_subsurface_pipeline: OnceLock<wgpu::RenderPipeline>,
}

struct LightingPipelineFoundation {
    vertex_shader: wgpu::ShaderModule,
    fragment_shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::deferred) struct DeferredLightingPipelineStartupReport
{
    shader_source_assembly: Duration,
    pipeline_foundation: Duration,
    standard_pipeline: Duration,
}

impl DeferredLightingPipelineStartupReport {
    pub(in crate::graphics::scene::scene_renderer::deferred) const fn shader_source_assembly(
        self,
    ) -> Duration {
        self.shader_source_assembly
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) const fn pipeline_foundation(
        self,
    ) -> Duration {
        self.pipeline_foundation
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) const fn standard_pipeline(
        self,
    ) -> Duration {
        self.standard_pipeline
    }
}

impl DeferredLightingPipelineCache {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer::deferred) fn new(
        device: &wgpu::Device,
        asset_manager: &ProjectAssetManager,
        scene_layout: &wgpu::BindGroupLayout,
        lighting_bind_group_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
        plugin_shading_models: &[ShadingModelDescriptor],
        volumetric_enabled: bool,
        deferred_lighting_profile: SceneRendererDeferredLightingProfile,
    ) -> Result<(Self, DeferredLightingPipelineStartupReport), GraphicsError> {
        let shader_source_started = Instant::now();
        let lighting_fragment_shader_source = deferred_lighting_shader_source(
            asset_manager,
            plugin_shading_models,
            volumetric_enabled,
            deferred_lighting_profile,
        )?;
        let shader_source_assembly = shader_source_started.elapsed();
        let cache = Self {
            lighting_fragment_shader_source,
            scene_layout: scene_layout.clone(),
            gpu_scene_layout: gpu_scene_layout.clone(),
            deferred_lighting_profile,
            target_format,
            foundation: OnceLock::new(),
            lighting_pipeline: OnceLock::new(),
            lighting_subsurface_pipeline: OnceLock::new(),
        };

        let (pipeline_foundation, standard_pipeline) = if deferred_lighting_profile
            == SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
        {
            // The viewer records only its BaseScenePass; defer this unused
            // compiled-graph pipeline until a caller actually requests it.
            (Duration::ZERO, Duration::ZERO)
        } else {
            let foundation_started = Instant::now();
            let foundation = cache.foundation(device, lighting_bind_group_layout);
            let pipeline_foundation = foundation_started.elapsed();
            let standard_pipeline_started = Instant::now();
            let _ = cache.pipeline_from_foundation(device, foundation, false);
            let standard_pipeline = standard_pipeline_started.elapsed();
            (pipeline_foundation, standard_pipeline)
        };

        Ok((
            cache,
            DeferredLightingPipelineStartupReport {
                shader_source_assembly,
                pipeline_foundation,
                standard_pipeline,
            },
        ))
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn pipeline(
        &self,
        device: &wgpu::Device,
        lighting_bind_group_layout: &wgpu::BindGroupLayout,
        subsurface_mrt: bool,
    ) -> &wgpu::RenderPipeline {
        let foundation = self.foundation(device, lighting_bind_group_layout);
        self.pipeline_from_foundation(
            device,
            foundation,
            subsurface_mrt
                && self
                    .deferred_lighting_profile
                    .uses_full_lighting_bind_group(),
        )
    }

    fn foundation(
        &self,
        device: &wgpu::Device,
        lighting_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> &LightingPipelineFoundation {
        self.foundation.get_or_init(|| {
            create_lighting_pipeline_foundation_from_source(
                device,
                &self.lighting_fragment_shader_source,
                &self.scene_layout,
                lighting_bind_group_layout,
                self.deferred_lighting_profile
                    .uses_gpu_scene()
                    .then_some(&self.gpu_scene_layout),
            )
        })
    }

    fn pipeline_from_foundation(
        &self,
        device: &wgpu::Device,
        foundation: &LightingPipelineFoundation,
        subsurface_mrt: bool,
    ) -> &wgpu::RenderPipeline {
        let pipeline = if subsurface_mrt {
            &self.lighting_subsurface_pipeline
        } else {
            &self.lighting_pipeline
        };
        pipeline.get_or_init(|| {
            create_lighting_pipeline_from_foundation(
                device,
                &foundation.vertex_shader,
                &foundation.fragment_shader,
                &foundation.layout,
                self.target_format,
                subsurface_mrt,
            )
        })
    }
}

fn create_lighting_pipeline_foundation_from_source(
    device: &wgpu::Device,
    lighting_fragment_shader_source: &str,
    scene_layout: &wgpu::BindGroupLayout,
    lighting_bind_group_layout: &wgpu::BindGroupLayout,
    gpu_scene_layout: Option<&wgpu::BindGroupLayout>,
) -> LightingPipelineFoundation {
    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-deferred-lighting-fullscreen-vertex-shader"),
        source: wgpu::ShaderSource::Wgsl(DEFERRED_LIGHTING_FULLSCREEN_VERTEX_SHADER.into()),
    });
    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-deferred-lighting-fragment-shader"),
        source: wgpu::ShaderSource::Wgsl(lighting_fragment_shader_source.into()),
    });
    let with_gpu_scene = [
        Some(scene_layout),
        Some(lighting_bind_group_layout),
        None,
        gpu_scene_layout,
    ];
    let environment_only = [Some(scene_layout), Some(lighting_bind_group_layout)];
    let bind_group_layouts: &[Option<&wgpu::BindGroupLayout>] = if gpu_scene_layout.is_some() {
        &with_gpu_scene
    } else {
        &environment_only
    };
    let lighting_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-deferred-lighting-layout"),
        bind_group_layouts,
        immediate_size: 0,
    });
    LightingPipelineFoundation {
        vertex_shader,
        fragment_shader,
        layout: lighting_layout,
    }
}

fn create_lighting_pipeline_from_foundation(
    device: &wgpu::Device,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    lighting_layout: &wgpu::PipelineLayout,
    target_format: wgpu::TextureFormat,
    subsurface_mrt: bool,
) -> wgpu::RenderPipeline {
    let mut targets = [
        Some(wgpu::ColorTargetState {
            format: target_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        }),
        None,
        None,
    ];
    let target_count = if subsurface_mrt {
        let subsurface_target = || {
            Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        targets[1] = subsurface_target();
        targets[2] = subsurface_target();
        3
    } else {
        1
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(if subsurface_mrt {
            "zircon-deferred-lighting-subsurface-mrt-pipeline"
        } else {
            "zircon-deferred-lighting-pipeline"
        }),
        layout: Some(lighting_layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: Some(if subsurface_mrt {
                "fs_main_sss"
            } else {
                "fs_main"
            }),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &targets[..target_count],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn deferred_lighting_shader_source(
    asset_manager: &ProjectAssetManager,
    plugin_shading_models: &[ShadingModelDescriptor],
    volumetric_enabled: bool,
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
) -> Result<String, GraphicsError> {
    let mut request = DeferredLightingShaderSourceRequest::new()
        .with_volumetric_enabled(volumetric_enabled)
        .with_deferred_lighting_profile(deferred_lighting_profile);
    for descriptor in plugin_shading_models.iter().cloned() {
        request = request.with_shading_model_descriptor(descriptor);
    }
    if deferred_lighting_profile != SceneRendererDeferredLightingProfile::FullScene {
        return assemble_deferred_lighting_shader_source(request).map_err(|error| {
            GraphicsError::Asset(format!(
                "deferred lighting shading model shader source assembly failed: {error:?}"
            ))
        });
    }
    let source_set = ShadingModelIncludeSourceSet::from_project_asset_manager(
        asset_manager,
        plugin_shading_models,
    )
    .map_err(|error| {
        GraphicsError::Asset(format!(
            "deferred lighting shading model include source export failed: {error}"
        ))
    })?;
    request = request.with_shading_model_deferred_include_sources(&source_set);
    assemble_deferred_lighting_shader_source(request).map_err(|error| {
        GraphicsError::Asset(format!(
            "deferred lighting shading model shader source assembly failed: {error:?}"
        ))
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn deferred_lighting_pipeline_targets_use_fixed_stack_storage() {
        let source = include_str!("create.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("deferred lighting pipeline implementation");

        assert!(!implementation.contains("let mut targets = vec!["));
        assert!(implementation.contains("let mut targets = ["));
    }

    #[test]
    fn deferred_lighting_pipeline_compiles_the_fullscreen_vertex_stage_separately() {
        let source = include_str!("create.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("deferred lighting pipeline implementation");

        assert!(implementation.contains("DEFERRED_LIGHTING_FULLSCREEN_VERTEX_SHADER"));
        assert!(implementation.contains("fn fs_main(@builtin(position) _position"));
        assert!(implementation.contains("fn fs_main_sss(@builtin(position) _position"));
        assert!(implementation.contains("module: vertex_shader,"));
        assert!(implementation.contains("module: fragment_shader,"));
    }

    #[test]
    fn deferred_lighting_pipeline_cache_prewarms_only_full_scene_standard_pipeline() {
        let source = include_str!("create.rs");
        let cache = source
            .split("impl DeferredLightingPipelineCache")
            .nth(1)
            .and_then(|source| {
                source
                    .split("fn create_lighting_pipeline_from_foundation")
                    .next()
            })
            .expect("deferred lighting cache implementation");

        assert!(cache.contains("OnceLock<wgpu::RenderPipeline>"));
        assert!(cache.contains("pipeline.get_or_init"));
        assert!(cache.contains("foundation.get_or_init"));
        assert!(cache.contains("cache.pipeline_from_foundation(device, foundation, false)"));
        assert!(!cache.contains("cache.pipeline_from_foundation(device, foundation, true)"));
        assert!(cache.contains(
            "deferred_lighting_profile\n            == SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview"
        ));
        assert!(cache.contains("(Duration::ZERO, Duration::ZERO)"));
    }

    #[test]
    fn deferred_lighting_pipeline_reports_source_foundation_and_standard_pso_separately() {
        let implementation = include_str!("create.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("deferred lighting pipeline implementation");

        for expected in [
            "DeferredLightingPipelineStartupReport",
            "shader_source_assembly",
            "pipeline_foundation",
            "standard_pipeline",
            "let shader_source_started = Instant::now();",
            "let foundation_started = Instant::now();",
            "let standard_pipeline_started = Instant::now();",
        ] {
            assert!(
                implementation.contains(expected),
                "deferred lighting startup profiling must retain `{expected}`"
            );
        }
    }
}
