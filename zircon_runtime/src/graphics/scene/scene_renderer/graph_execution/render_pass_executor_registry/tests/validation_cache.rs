use super::*;
use crate::graphics::{RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage};

#[test]
fn render01_compiled_pipeline_executor_validation_cache_skips_stable_10_100_500_pass_rescans() {
    for pass_count in [10_usize, 100, 500] {
        let (pipeline, executor_id) = compiled_pipeline_with_plugin_passes(pass_count);
        let mut registry = RenderPassExecutorRegistry::with_builtin_noop_executors();
        registry.register(executor_id.clone(), benchmark_executor);
        let executable_pass_count = pipeline
            .graph()
            .passes()
            .iter()
            .filter(|pass| !pass.culled)
            .count() as u64;

        registry.validate_compiled_pipeline(&pipeline).unwrap();
        assert_eq!(registry.full_validation_scan_count(), 1);
        assert_eq!(
            registry.full_validation_scanned_pass_count(),
            executable_pass_count
        );

        for _ in 0..128 {
            registry.validate_compiled_pipeline(&pipeline).unwrap();
        }
        assert_eq!(registry.full_validation_scan_count(), 1);
        assert_eq!(
            registry.full_validation_scanned_pass_count(),
            executable_pass_count
        );

        let generation_before_reload = registry.generation();
        registry.register(executor_id, benchmark_executor);
        assert!(registry.generation() > generation_before_reload);
        registry.validate_compiled_pipeline(&pipeline).unwrap();
        assert_eq!(registry.full_validation_scan_count(), 2);
        assert_eq!(
            registry.full_validation_scanned_pass_count(),
            executable_pass_count * 2
        );
    }
}

#[test]
fn render01_compiled_pipeline_executor_revoke_invalidates_cache_before_submission() {
    let (pipeline, executor_id) = compiled_pipeline_with_plugin_passes(10);
    let mut registry = RenderPassExecutorRegistry::with_builtin_noop_executors();
    registry.register(executor_id.clone(), benchmark_executor);
    registry.validate_compiled_pipeline(&pipeline).unwrap();

    let generation_before_revoke = registry.generation();
    assert!(registry.unregister_executor(&executor_id).is_some());
    assert!(registry.generation() > generation_before_revoke);
    let error = registry.validate_compiled_pipeline(&pipeline).unwrap_err();

    assert!(error.contains("references unregistered executor `runtime-metadata.benchmark`"));
    assert_eq!(registry.full_validation_scan_count(), 2);
}

fn compiled_pipeline_with_plugin_passes(
    pass_count: usize,
) -> (CompiledRenderPipeline, RenderPassExecutorId) {
    let executor_id = RenderPassExecutorId::new("runtime-metadata.benchmark");
    let passes = (0..pass_count)
        .map(|index| {
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                format!("runtime-metadata-benchmark-{pass_count}-{index}"),
                QueueLane::Graphics,
            )
            .with_executor_id(executor_id.as_str())
            .write_external(format!("runtime-metadata-output-{pass_count}-{index}"))
        })
        .collect::<Vec<_>>();
    let descriptor = RenderFeatureDescriptor::new(
        format!("runtime-metadata-benchmark-{pass_count}"),
        Vec::new(),
        Vec::new(),
        passes,
    );
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([descriptor])
        .compile(&test_extract())
        .unwrap();
    (pipeline, executor_id)
}

fn benchmark_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}
