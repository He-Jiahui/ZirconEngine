//! 注册时先用代表性视图预编译并验证 executor，再替换资产及缓存；真实相机仍按当前提取重编。
use crate::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderFrameExtract, RenderFrameworkError,
    RenderPipelineHandle, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::graphics::{CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn register_pipeline_asset(
    framework: &WgpuRenderFramework,
    mut pipeline: RenderPipelineAsset,
) -> Result<RenderPipelineHandle, RenderFrameworkError> {
    let handle = pipeline.handle;
    if pipeline.revision == 0 {
        pipeline.bump_revision();
    }
    let compiled = compile_pipeline_for_validation(&pipeline)?;

    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
    state
        .renderer
        .validate_compiled_pipeline_executors(&compiled)
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: handle.raw(),
            message,
        })?;
    state.compiled_graph_cache.invalidate_pipeline(handle);
    state.pipelines.insert(handle, pipeline);
    Ok(handle)
}

pub(in crate::graphics::runtime::render_framework) fn compile_pipeline_for_validation(
    pipeline: &RenderPipelineAsset,
) -> Result<CompiledRenderPipeline, RenderFrameworkError> {
    pipeline
        .compile_with_options(
            &validation_extract_for_core_pipeline(pipeline.core_pipeline),
            &validation_compile_options(pipeline),
        )
        .map_err(|message| RenderFrameworkError::GraphCompileFailure {
            pipeline: pipeline.handle.raw(),
            message,
        })
}

fn validation_compile_options(pipeline: &RenderPipelineAsset) -> RenderPipelineCompileOptions {
    pipeline
        .renderer
        .features
        .iter()
        .filter(|feature| feature.enabled)
        .fold(
            RenderPipelineCompileOptions::default(),
            |mut options, feature| {
                if let Some(builtin) = feature.builtin_feature() {
                    options = options.with_feature_enabled(builtin);
                }
                if let Some(gate) = feature.quality_gate {
                    options = options.with_feature_enabled(gate);
                }
                for requirement in feature
                    .capability_requirements
                    .iter()
                    .chain(feature.descriptor().capability_requirements.iter())
                {
                    options = options.with_capability_enabled(*requirement);
                }
                options
            },
        )
}

fn validation_extract_for_core_pipeline(core_pipeline: CorePipelineKind) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(0),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    projection_mode: match core_pipeline {
                        CorePipelineKind::Core2d => ProjectionMode::Orthographic,
                        CorePipelineKind::Core3d => ProjectionMode::Perspective,
                    },
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: crate::core::framework::render::PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: crate::core::framework::render::FallbackSkyboxKind::None,
                clear_color: crate::core::math::Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}

#[cfg(test)]
#[path = "tests/register_pipeline_asset.rs"]
mod tests;
