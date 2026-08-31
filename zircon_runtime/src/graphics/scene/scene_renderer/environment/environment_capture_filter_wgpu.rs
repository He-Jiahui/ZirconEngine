use crate::core::framework::render::{
    IblBakeArtifactDescriptor, IblBakeArtifactRequest, RenderEnvironmentCaptureRequest,
};

use super::environment_capture_gpu_target::EnvironmentCaptureGpuTarget;
use super::ibl_bake_shader_plan::{
    ibl_bake_irradiance_sh9_kernel_plan, ibl_bake_pmrem_kernel_plan_with_quality,
};
use super::ibl_bake_wgpu_binding::{
    create_ibl_bake_wgpu_bind_group, create_ibl_bake_wgpu_params_buffer,
    IblBakeWgpuOutputBindingResource,
};
use super::ibl_bake_wgpu_command_plan::ibl_bake_wgpu_command_plan_for_runtime_kernel;
use super::ibl_bake_wgpu_dispatch::encode_ibl_bake_wgpu_compute_dispatch;
use super::ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
use super::realtime_ibl_capture_wgpu::RealtimeIblCaptureWgpuPipelines;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics) struct EnvironmentCaptureFilterWgpuRecordReport {
    pub source_mip_dispatch_count: u32,
    pub pmrem_dispatch_count: u32,
    pub sh9_dispatch_count: u32,
    pub source_mip_params_buffer_creations: usize,
    pub source_mip_bind_group_creations: usize,
    pub source_mip_binding_creation_micros: u64,
    pub ibl_params_buffer_creations: usize,
    pub ibl_bind_group_creations: usize,
}

impl EnvironmentCaptureFilterWgpuRecordReport {
    fn emit_profile_counters(&self) {
        crate::profile_counter!(
            "render",
            "environment_capture_source_mip_dispatch_count",
            self.source_mip_dispatch_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_pmrem_dispatch_count",
            self.pmrem_dispatch_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_sh9_dispatch_count",
            self.sh9_dispatch_count
        );
        crate::profile_counter!(
            "render",
            "environment_capture_source_mip_params_buffer_creation_count",
            self.source_mip_params_buffer_creations
        );
        crate::profile_counter!(
            "render",
            "environment_capture_source_mip_bind_group_creation_count",
            self.source_mip_bind_group_creations
        );
        crate::profile_counter!(
            "render",
            "environment_capture_source_mip_binding_creation_micros",
            self.source_mip_binding_creation_micros
        );
        crate::profile_counter!(
            "render",
            "environment_capture_ibl_params_buffer_creation_count",
            self.ibl_params_buffer_creations
        );
        crate::profile_counter!(
            "render",
            "environment_capture_ibl_bind_group_creation_count",
            self.ibl_bind_group_creations
        );
    }
}

pub(in crate::graphics) struct EnvironmentCaptureFilterWgpuRecorder;

impl EnvironmentCaptureFilterWgpuRecorder {
    pub(in crate::graphics) fn record(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        request: &RenderEnvironmentCaptureRequest,
        target: &EnvironmentCaptureGpuTarget,
        pipelines: &RealtimeIblCaptureWgpuPipelines,
        pipeline_cache: &mut IblBakeWgpuPipelineCache,
    ) -> Result<EnvironmentCaptureFilterWgpuRecordReport, String> {
        let source_mip_creation = pipelines.record_source_mip_chain(
            device,
            encoder,
            false,
            target.plan().face_size(),
            target.sampled_mips(),
            target.storage_mips(),
        )?;
        let bake_request = IblBakeArtifactRequest::new(
            request.ibl_bake_key(),
            target.plan().face_size(),
            target.plan().source_mip_count(),
        );
        let descriptor =
            IblBakeArtifactDescriptor::current_for_runtime_cache_request(&bake_request);
        let mut report = EnvironmentCaptureFilterWgpuRecordReport {
            source_mip_dispatch_count: target.plan().source_mip_count().saturating_sub(1),
            source_mip_params_buffer_creations: source_mip_creation.params_buffer_creations,
            source_mip_bind_group_creations: source_mip_creation.bind_group_creations,
            source_mip_binding_creation_micros: source_mip_creation.creation_micros,
            ..Default::default()
        };

        for mip_level in 0..bake_request.pmrem_mip_count() {
            let command = ibl_bake_wgpu_command_plan_for_runtime_kernel(
                &bake_request,
                descriptor,
                ibl_bake_pmrem_kernel_plan_with_quality(
                    &bake_request,
                    mip_level,
                    request.quality(),
                ),
            );
            let output = target.pmrem_storage_mip(mip_level).ok_or_else(|| {
                format!("environment capture PMREM mip {mip_level} is unavailable")
            })?;
            record_ibl_command(
                device,
                encoder,
                target.sampled_cube(),
                IblBakeWgpuOutputBindingResource::StorageTexture2DArray(output),
                pipeline_cache,
                &command,
            )?;
            report.pmrem_dispatch_count += 1;
            report.ibl_params_buffer_creations += 1;
            report.ibl_bind_group_creations += 1;
        }

        let sh9_command = ibl_bake_wgpu_command_plan_for_runtime_kernel(
            &bake_request,
            descriptor,
            ibl_bake_irradiance_sh9_kernel_plan(&bake_request),
        );
        encoder.clear_buffer(target.sh9_buffer(), 0, None);
        record_ibl_command(
            device,
            encoder,
            target.sampled_cube(),
            IblBakeWgpuOutputBindingResource::StorageBuffer(target.sh9_buffer()),
            pipeline_cache,
            &sh9_command,
        )?;
        report.sh9_dispatch_count = 1;
        report.ibl_params_buffer_creations += 1;
        report.ibl_bind_group_creations += 1;

        report.emit_profile_counters();
        Ok(report)
    }
}

fn record_ibl_command(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    source: &wgpu::TextureView,
    output: IblBakeWgpuOutputBindingResource<'_>,
    pipeline_cache: &mut IblBakeWgpuPipelineCache,
    command: &super::ibl_bake_wgpu_command_plan::IblBakeWgpuCommandPlan,
) -> Result<(), String> {
    let pipeline = pipeline_cache.ensure_compute_pipeline(device, command);
    let params = create_ibl_bake_wgpu_params_buffer(device, command);
    let bind_group = create_ibl_bake_wgpu_bind_group(
        device,
        pipeline_cache.bind_group_layouts(),
        command,
        &params,
        source,
        pipeline_cache.source_sampler(),
        output,
    )?;
    encode_ibl_bake_wgpu_compute_dispatch(encoder, command, &pipeline, &bind_group)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("environment_capture_filter_wgpu.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("environment capture filtering must retain a test boundary")
    }

    #[test]
    fn capture_filter_records_source_mips_pmrem_and_sh9_into_one_encoder() {
        let source = production_source();

        assert!(source.contains("fn record("));
        assert!(source.contains("record_source_mip_chain("));
        assert!(source.contains("ibl_bake_pmrem_kernel_plan_with_quality("));
        assert!(source.contains("ibl_bake_irradiance_sh9_kernel_plan("));
        assert!(source.contains("encode_ibl_bake_wgpu_compute_dispatch("));
        assert!(!source.contains("submit_graphics_command_buffers("));
    }

    #[test]
    fn capture_filter_reuses_renderer_owned_pipelines_and_reports_exact_work() {
        let source = production_source();

        assert!(source.contains("pipelines: &RealtimeIblCaptureWgpuPipelines"));
        assert!(source.contains("pipeline_cache: &mut IblBakeWgpuPipelineCache"));
        assert!(source.contains("source_mip_dispatch_count"));
        assert!(source.contains("pmrem_dispatch_count"));
        assert!(source.contains("sh9_dispatch_count"));
        assert!(!source.contains("RealtimeIblCaptureWgpuPipelines::new("));
        assert!(!source.contains("IblBakeWgpuPipelineCache::new("));
    }

    #[test]
    fn capture_filter_publishes_existing_work_report_to_the_profiler() {
        let source = production_source();

        for counter in [
            "environment_capture_source_mip_dispatch_count",
            "environment_capture_pmrem_dispatch_count",
            "environment_capture_sh9_dispatch_count",
            "environment_capture_source_mip_binding_creation_micros",
            "environment_capture_ibl_bind_group_creation_count",
        ] {
            assert!(
                source.contains(counter),
                "missing profile counter {counter}"
            );
        }
        assert!(source.contains("report.emit_profile_counters()"));
    }
}
