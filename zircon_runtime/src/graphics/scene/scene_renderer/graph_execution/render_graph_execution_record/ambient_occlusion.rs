use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderAmbientOcclusionExecutionFailureFlags,
    RenderAmbientOcclusionExecutionReport, RenderAmbientOcclusionExecutionStatus,
};
use crate::graphics::pipeline::CompiledRenderPipeline;
use crate::render_graph::{
    RenderGraphComputePipelineResolution, RenderGraphComputePipelineResolutionStatus,
    RenderGraphResourceAccessKind,
};

use super::super::RenderPassDeviceEpoch;
use super::RenderGraphExecutionRecord;

const EVALUATE_PASS_NAME: &str = "ssao-evaluate";
const SPATIAL_PASS_NAME: &str = "ssao-spatial-denoise";
const UPSAMPLE_PASS_NAME: &str = "ssao-bilateral-upsample";
const LIGHTING_PASS_NAME: &str = "deferred-lighting";
const COMPUTE_EXECUTOR_ID: &str = "compute.generic";
const LIGHTING_EXECUTOR_ID: &str = "lighting.deferred";
const EVALUATE_PIPELINE_FAMILY: &str = "ambient-occlusion.evaluate";
const SPATIAL_PIPELINE_FAMILY: &str = "ambient-occlusion.spatial-denoise";
const UPSAMPLE_PIPELINE_FAMILY: &str = "ambient-occlusion.bilateral-upsample";

#[derive(Clone, Debug)]
pub(super) struct AmbientOcclusionExecutionContract {
    enabled: bool,
    complete: bool,
    profile_artifact_version: u32,
    profile_compiler_version: u32,
    shader_interface_version: u32,
    resolution_divisor: u8,
    pipeline_generation: u64,
    output_generation: u64,
    output_producer: String,
}

impl AmbientOcclusionExecutionContract {
    pub(super) fn disabled() -> Self {
        Self {
            enabled: false,
            complete: true,
            profile_artifact_version: 0,
            profile_compiler_version: 0,
            shader_interface_version: 0,
            resolution_divisor: 0,
            pipeline_generation: 0,
            output_generation: 0,
            output_producer: String::new(),
        }
    }

    pub(super) fn enabled(
        profile_artifact_version: u32,
        profile_compiler_version: u32,
        shader_interface_version: u32,
        resolution_divisor: u8,
        pipeline_generation: u64,
        output_generation: u64,
        output_producer: impl Into<String>,
    ) -> Self {
        Self {
            enabled: true,
            complete: true,
            profile_artifact_version,
            profile_compiler_version,
            shader_interface_version,
            resolution_divisor,
            pipeline_generation,
            output_generation,
            output_producer: output_producer.into(),
        }
    }

    fn incomplete(profile: Option<&crate::graphics::pipeline::CompiledAoProfile>) -> Self {
        Self {
            enabled: true,
            complete: false,
            profile_artifact_version: profile.map_or(0, |value| value.artifact_version()),
            profile_compiler_version: profile.map_or(0, |value| value.compiler_version()),
            shader_interface_version: profile.map_or(0, |value| value.shader_interface_version()),
            resolution_divisor: profile.map_or(0, |value| value.resolution_divisor()),
            pipeline_generation: profile.map_or(0, |value| value.pipeline_generation()),
            output_generation: 0,
            output_producer: String::new(),
        }
    }

    fn from_pipeline(pipeline: &CompiledRenderPipeline) -> Self {
        match (
            pipeline.ambient_occlusion_profile(),
            pipeline.ambient_occlusion_outputs(),
        ) {
            (None, None) => Self::disabled(),
            (Some(profile), Some(outputs)) => Self::enabled(
                profile.artifact_version(),
                profile.compiler_version(),
                profile.shader_interface_version(),
                profile.resolution_divisor(),
                profile.pipeline_generation(),
                outputs.producer_generation(),
                outputs.indirect_diffuse_ao().producer_pass_name(),
            ),
            (profile, _) => Self::incomplete(profile),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct DispatchSummary {
    count: usize,
    group_count: usize,
    target_write_count: usize,
}

impl RenderGraphExecutionRecord {
    pub(crate) fn finalize_ambient_occlusion_report(
        &mut self,
        frame_generation: u64,
        pipeline: &CompiledRenderPipeline,
    ) {
        self.finalize_ambient_occlusion_report_from_contract(
            frame_generation,
            AmbientOcclusionExecutionContract::from_pipeline(pipeline),
        );
    }

    #[cfg(test)]
    pub(super) fn finalize_ambient_occlusion_report_for_test(
        &mut self,
        frame_generation: u64,
        contract: AmbientOcclusionExecutionContract,
    ) {
        self.finalize_ambient_occlusion_report_from_contract(frame_generation, contract);
    }

    fn finalize_ambient_occlusion_report_from_contract(
        &mut self,
        frame_generation: u64,
        contract: AmbientOcclusionExecutionContract,
    ) {
        let half_resolution = contract.resolution_divisor == 2;
        let evaluate_pass_count = self.executed_pass_count(EVALUATE_PASS_NAME, COMPUTE_EXECUTOR_ID);
        let spatial_pass_count = self.executed_pass_count(SPATIAL_PASS_NAME, COMPUTE_EXECUTOR_ID);
        let upsample_pass_count = self.executed_pass_count(UPSAMPLE_PASS_NAME, COMPUTE_EXECUTOR_ID);
        let lighting_pass_count =
            self.executed_pass_count(LIGHTING_PASS_NAME, LIGHTING_EXECUTOR_ID);
        let evaluate_raw_write_count = self.executed_access_count(
            EVALUATE_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
            RenderGraphResourceAccessKind::Write,
        );
        let spatial_raw_read_count = self.executed_access_count(
            SPATIAL_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
            RenderGraphResourceAccessKind::Read,
        );
        let spatial_final_write_count = self.executed_access_count(
            SPATIAL_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphResourceAccessKind::Write,
        );
        let spatial_intermediate_write_count = self.executed_access_count(
            SPATIAL_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL,
            RenderGraphResourceAccessKind::Write,
        );
        let upsample_spatial_read_count = self.executed_access_count(
            UPSAMPLE_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL,
            RenderGraphResourceAccessKind::Read,
        );
        let upsample_final_write_count = self.executed_access_count(
            UPSAMPLE_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphResourceAccessKind::Write,
        );
        let lighting_final_read_count = self.executed_access_count(
            LIGHTING_PASS_NAME,
            LIGHTING_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphResourceAccessKind::Read,
        );
        let evaluate_dispatch = self.dispatch_summary(
            EVALUATE_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION_RAW,
        );
        let spatial_dispatch = self.dispatch_summary(
            SPATIAL_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            if half_resolution {
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION_SPATIAL
            } else {
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION
            },
        );
        let upsample_dispatch = self.dispatch_summary(
            UPSAMPLE_PASS_NAME,
            COMPUTE_EXECUTOR_ID,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        );
        let evaluate_pipeline_resolution = self
            .compute_pipeline_resolution(EVALUATE_PASS_NAME, COMPUTE_EXECUTOR_ID)
            .cloned();
        let spatial_pipeline_resolution = self
            .compute_pipeline_resolution(SPATIAL_PASS_NAME, COMPUTE_EXECUTOR_ID)
            .cloned();
        let upsample_pipeline_resolution = self
            .compute_pipeline_resolution(UPSAMPLE_PASS_NAME, COMPUTE_EXECUTOR_ID)
            .cloned();
        let recorded_pass_count = evaluate_pass_count
            .saturating_add(spatial_pass_count)
            .saturating_add(upsample_pass_count)
            .saturating_add(lighting_pass_count);

        let mut failure_flags = RenderAmbientOcclusionExecutionFailureFlags::NONE;
        if !contract.enabled {
            let unexpected_access_count = evaluate_raw_write_count
                .saturating_add(spatial_raw_read_count)
                .saturating_add(spatial_final_write_count)
                .saturating_add(spatial_intermediate_write_count)
                .saturating_add(upsample_spatial_read_count)
                .saturating_add(upsample_final_write_count)
                .saturating_add(lighting_final_read_count);
            if evaluate_pass_count != 0
                || spatial_pass_count != 0
                || upsample_pass_count != 0
                || evaluate_dispatch.count != 0
                || spatial_dispatch.count != 0
                || upsample_dispatch.count != 0
                || unexpected_access_count != 0
            {
                failure_flags
                    .insert(RenderAmbientOcclusionExecutionFailureFlags::UNEXPECTED_DISABLED_WORK);
            }
            self.ambient_occlusion_execution_report = RenderAmbientOcclusionExecutionReport {
                status: if failure_flags.is_empty() {
                    RenderAmbientOcclusionExecutionStatus::Disabled
                } else {
                    RenderAmbientOcclusionExecutionStatus::Failed
                },
                failure_flags,
                frame_generation,
                lighting_pass_count,
                recorded_pass_count,
                ..RenderAmbientOcclusionExecutionReport::default()
            };
            return;
        }

        if !contract.complete {
            failure_flags
                .insert(RenderAmbientOcclusionExecutionFailureFlags::MISSING_COMPILED_CONTRACT);
        }
        if contract.pipeline_generation == 0
            || contract.output_generation == 0
            || contract.pipeline_generation != contract.output_generation
        {
            failure_flags.insert(RenderAmbientOcclusionExecutionFailureFlags::GENERATION_MISMATCH);
        }
        if contract.resolution_divisor != 1 && !half_resolution {
            failure_flags
                .insert(RenderAmbientOcclusionExecutionFailureFlags::MISSING_COMPILED_CONTRACT);
        }
        let expected_output_producer = if half_resolution {
            UPSAMPLE_PASS_NAME
        } else {
            SPATIAL_PASS_NAME
        };
        if contract.output_producer != expected_output_producer {
            failure_flags
                .insert(RenderAmbientOcclusionExecutionFailureFlags::OUTPUT_PRODUCER_MISMATCH);
        }
        flag_count_mismatch(
            &mut failure_flags,
            evaluate_pass_count,
            RenderAmbientOcclusionExecutionFailureFlags::EVALUATE_PASS,
        );
        if evaluate_dispatch.count != 1
            || evaluate_dispatch.group_count == 0
            || evaluate_dispatch.target_write_count != 1
        {
            failure_flags.insert(RenderAmbientOcclusionExecutionFailureFlags::EVALUATE_DISPATCH);
        }
        if !pipeline_resolution_matches(
            evaluate_pipeline_resolution.as_ref(),
            EVALUATE_PIPELINE_FAMILY,
            u64::from(contract.shader_interface_version),
        ) {
            failure_flags
                .insert(RenderAmbientOcclusionExecutionFailureFlags::EVALUATE_PIPELINE_RESOLUTION);
        }
        flag_count_mismatch(
            &mut failure_flags,
            evaluate_raw_write_count,
            RenderAmbientOcclusionExecutionFailureFlags::EVALUATE_RAW_WRITE,
        );
        flag_count_mismatch(
            &mut failure_flags,
            spatial_pass_count,
            RenderAmbientOcclusionExecutionFailureFlags::SPATIAL_PASS,
        );
        if spatial_dispatch.count != 1
            || spatial_dispatch.group_count == 0
            || spatial_dispatch.target_write_count != 1
        {
            failure_flags.insert(RenderAmbientOcclusionExecutionFailureFlags::SPATIAL_DISPATCH);
        }
        if !pipeline_resolution_matches(
            spatial_pipeline_resolution.as_ref(),
            SPATIAL_PIPELINE_FAMILY,
            u64::from(contract.shader_interface_version),
        ) {
            failure_flags
                .insert(RenderAmbientOcclusionExecutionFailureFlags::SPATIAL_PIPELINE_RESOLUTION);
        }
        flag_count_mismatch(
            &mut failure_flags,
            spatial_raw_read_count,
            RenderAmbientOcclusionExecutionFailureFlags::SPATIAL_RAW_READ,
        );
        flag_expected_count_mismatch(
            &mut failure_flags,
            spatial_final_write_count,
            usize::from(!half_resolution),
            RenderAmbientOcclusionExecutionFailureFlags::SPATIAL_FINAL_WRITE,
        );
        flag_expected_count_mismatch(
            &mut failure_flags,
            spatial_intermediate_write_count,
            usize::from(half_resolution),
            RenderAmbientOcclusionExecutionFailureFlags::SPATIAL_INTERMEDIATE_WRITE,
        );
        flag_expected_count_mismatch(
            &mut failure_flags,
            upsample_pass_count,
            usize::from(half_resolution),
            RenderAmbientOcclusionExecutionFailureFlags::UPSAMPLE_PASS,
        );
        if half_resolution {
            if upsample_dispatch.count != 1
                || upsample_dispatch.group_count == 0
                || upsample_dispatch.target_write_count != 1
            {
                failure_flags
                    .insert(RenderAmbientOcclusionExecutionFailureFlags::UPSAMPLE_DISPATCH);
            }
            if !pipeline_resolution_matches(
                upsample_pipeline_resolution.as_ref(),
                UPSAMPLE_PIPELINE_FAMILY,
                u64::from(contract.shader_interface_version),
            ) {
                failure_flags.insert(
                    RenderAmbientOcclusionExecutionFailureFlags::UPSAMPLE_PIPELINE_RESOLUTION,
                );
            }
        } else if upsample_dispatch.count != 0 || upsample_dispatch.group_count != 0 {
            failure_flags.insert(RenderAmbientOcclusionExecutionFailureFlags::UPSAMPLE_DISPATCH);
        }
        flag_expected_count_mismatch(
            &mut failure_flags,
            upsample_spatial_read_count,
            usize::from(half_resolution),
            RenderAmbientOcclusionExecutionFailureFlags::UPSAMPLE_SPATIAL_READ,
        );
        flag_expected_count_mismatch(
            &mut failure_flags,
            upsample_final_write_count,
            usize::from(half_resolution),
            RenderAmbientOcclusionExecutionFailureFlags::UPSAMPLE_FINAL_WRITE,
        );
        flag_count_mismatch(
            &mut failure_flags,
            lighting_pass_count,
            RenderAmbientOcclusionExecutionFailureFlags::LIGHTING_PASS,
        );
        flag_count_mismatch(
            &mut failure_flags,
            lighting_final_read_count,
            RenderAmbientOcclusionExecutionFailureFlags::LIGHTING_FINAL_READ,
        );

        let evaluate_device_epoch = pipeline_device_epoch(evaluate_pipeline_resolution.as_ref());
        let spatial_device_epoch = pipeline_device_epoch(spatial_pipeline_resolution.as_ref());
        let upsample_device_epoch = pipeline_device_epoch(upsample_pipeline_resolution.as_ref());
        if evaluate_device_epoch.is_none()
            || evaluate_device_epoch != spatial_device_epoch
            || (half_resolution && evaluate_device_epoch != upsample_device_epoch)
        {
            failure_flags.insert(
                RenderAmbientOcclusionExecutionFailureFlags::PIPELINE_DEVICE_EPOCH_MISMATCH,
            );
        }
        let upsample_relevant_resolution = if half_resolution {
            upsample_pipeline_resolution.as_ref()
        } else {
            None
        };
        let last_good_dispatch_count = [
            evaluate_pipeline_resolution.as_ref(),
            spatial_pipeline_resolution.as_ref(),
            upsample_relevant_resolution,
        ]
        .into_iter()
        .flatten()
        .filter(|resolution| {
            resolution.status == RenderGraphComputePipelineResolutionStatus::UsingLastGood
        })
        .count();
        let (device_id, device_generation) = evaluate_device_epoch.map_or((None, None), |epoch| {
            let (device_id, device_generation) = epoch.raw_parts();
            (Some(device_id), Some(device_generation))
        });

        self.ambient_occlusion_execution_report = RenderAmbientOcclusionExecutionReport {
            status: if failure_flags.is_empty() {
                if last_good_dispatch_count == 0 {
                    RenderAmbientOcclusionExecutionStatus::Ready
                } else {
                    RenderAmbientOcclusionExecutionStatus::UsingLastGood
                }
            } else {
                RenderAmbientOcclusionExecutionStatus::Failed
            },
            failure_flags,
            frame_generation,
            profile_artifact_version: contract.profile_artifact_version,
            profile_compiler_version: contract.profile_compiler_version,
            shader_interface_version: contract.shader_interface_version,
            pipeline_generation: contract.pipeline_generation,
            output_generation: contract.output_generation,
            device_id,
            device_generation,
            evaluate_candidate_artifact_fingerprint: pipeline_candidate_fingerprint(
                evaluate_pipeline_resolution.as_ref(),
            ),
            evaluate_resolved_artifact_fingerprint: pipeline_resolved_fingerprint(
                evaluate_pipeline_resolution.as_ref(),
            ),
            spatial_candidate_artifact_fingerprint: pipeline_candidate_fingerprint(
                spatial_pipeline_resolution.as_ref(),
            ),
            spatial_resolved_artifact_fingerprint: pipeline_resolved_fingerprint(
                spatial_pipeline_resolution.as_ref(),
            ),
            upsample_candidate_artifact_fingerprint: pipeline_candidate_fingerprint(
                upsample_relevant_resolution,
            ),
            upsample_resolved_artifact_fingerprint: pipeline_resolved_fingerprint(
                upsample_relevant_resolution,
            ),
            last_good_dispatch_count,
            expected_pass_count: 3 + usize::from(half_resolution),
            recorded_pass_count,
            evaluate_pass_count,
            spatial_pass_count,
            upsample_pass_count,
            lighting_pass_count,
            evaluate_dispatch_count: evaluate_dispatch.count,
            spatial_dispatch_count: spatial_dispatch.count,
            upsample_dispatch_count: upsample_dispatch.count,
            evaluate_dispatch_group_count: evaluate_dispatch.group_count,
            spatial_dispatch_group_count: spatial_dispatch.group_count,
            upsample_dispatch_group_count: upsample_dispatch.group_count,
            evaluate_raw_write_count,
            spatial_raw_read_count,
            spatial_final_write_count,
            spatial_intermediate_write_count,
            upsample_spatial_read_count,
            upsample_final_write_count,
            lighting_final_read_count,
        };
    }

    fn executed_pass_count(&self, pass_name: &str, executor_id: &str) -> usize {
        self.executed_passes
            .iter()
            .zip(&self.executed_executor_ids)
            .filter(|(pass, executor)| {
                pass.as_str() == pass_name && executor.as_str() == executor_id
            })
            .count()
    }

    fn executed_access_count(
        &self,
        pass_name: &str,
        executor_id: &str,
        resource_name: &str,
        access_kind: RenderGraphResourceAccessKind,
    ) -> usize {
        self.executed_passes
            .iter()
            .zip(&self.executed_executor_ids)
            .zip(&self.executed_pass_resources)
            .filter(|((pass, executor), _)| {
                pass.as_str() == pass_name && executor.as_str() == executor_id
            })
            .flat_map(|(_, resources)| resources)
            .filter(|resource| resource.name == resource_name && resource.access == access_kind)
            .count()
    }

    fn dispatch_summary(
        &self,
        pass_name: &str,
        executor_id: &str,
        target_name: &str,
    ) -> DispatchSummary {
        self.compute_dispatches
            .iter()
            .filter(|dispatch| {
                dispatch.pass_name == pass_name && dispatch.executor_id == executor_id
            })
            .fold(DispatchSummary::default(), |mut summary, dispatch| {
                summary.count = summary.count.saturating_add(1);
                summary.group_count = summary
                    .group_count
                    .saturating_add(dispatch.dispatch_group_volume());
                summary.target_write_count = summary.target_write_count.saturating_add(
                    dispatch
                        .storage_write_resources
                        .iter()
                        .filter(|resource| resource.as_str() == target_name)
                        .count(),
                );
                summary
            })
    }

    fn compute_pipeline_resolution(
        &self,
        pass_name: &str,
        executor_id: &str,
    ) -> Option<&RenderGraphComputePipelineResolution> {
        self.compute_dispatches
            .iter()
            .find(|dispatch| dispatch.pass_name == pass_name && dispatch.executor_id == executor_id)
            .and_then(|dispatch| dispatch.pipeline_resolution.as_ref())
    }

    pub fn ambient_occlusion_execution_report(&self) -> RenderAmbientOcclusionExecutionReport {
        self.ambient_occlusion_execution_report
    }
}

fn pipeline_resolution_matches(
    resolution: Option<&RenderGraphComputePipelineResolution>,
    expected_family: &str,
    expected_interface_generation: u64,
) -> bool {
    let Some(resolution) = resolution else {
        return false;
    };
    let Some(family) = resolution.family.as_ref() else {
        return false;
    };
    if family.name != expected_family
        || family.interface_generation != expected_interface_generation
        || resolution.device_id.is_none()
        || resolution.device_generation.is_none()
    {
        return false;
    }
    match resolution.status {
        RenderGraphComputePipelineResolutionStatus::Ready => {
            resolution.candidate_artifact_fingerprint == resolution.resolved_artifact_fingerprint
                && resolution.candidate_failure.is_none()
        }
        RenderGraphComputePipelineResolutionStatus::UsingLastGood => {
            resolution.candidate_artifact_fingerprint != resolution.resolved_artifact_fingerprint
                && resolution.candidate_failure.is_some()
        }
    }
}

fn pipeline_device_epoch(
    resolution: Option<&RenderGraphComputePipelineResolution>,
) -> Option<RenderPassDeviceEpoch> {
    let resolution = resolution?;
    Some(RenderPassDeviceEpoch::new(
        resolution.device_id?,
        resolution.device_generation?,
    ))
}

fn pipeline_candidate_fingerprint(
    resolution: Option<&RenderGraphComputePipelineResolution>,
) -> u64 {
    resolution.map_or(0, |resolution| resolution.candidate_artifact_fingerprint)
}

fn pipeline_resolved_fingerprint(resolution: Option<&RenderGraphComputePipelineResolution>) -> u64 {
    resolution.map_or(0, |resolution| resolution.resolved_artifact_fingerprint)
}

fn flag_count_mismatch(
    failure_flags: &mut RenderAmbientOcclusionExecutionFailureFlags,
    count: usize,
    flag: RenderAmbientOcclusionExecutionFailureFlags,
) {
    if count != 1 {
        failure_flags.insert(flag);
    }
}

fn flag_expected_count_mismatch(
    failure_flags: &mut RenderAmbientOcclusionExecutionFailureFlags,
    count: usize,
    expected: usize,
    flag: RenderAmbientOcclusionExecutionFailureFlags,
) {
    if count != expected {
        failure_flags.insert(flag);
    }
}
