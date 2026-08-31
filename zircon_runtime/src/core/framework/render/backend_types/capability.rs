#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderQueueCapability {
    Graphics,
    Compute,
    Copy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderCapabilityKind {
    VirtualGeometry,
    HybridGlobalIllumination,
    AccelerationStructures,
    InlineRayQuery,
    RayTracingPipeline,
    BufferBindingArray,
    TextureBindingArray,
    NonUniformResourceIndexing,
    PartiallyBoundBindingArray,
    ScreenSpaceAntiAlias,
    StorageBuffers,
    IndirectDraw,
    BufferReadback,
    AsyncCompute,
    AsyncCopy,
    NeuralCompute,
    SparseTexture,
    SubgroupOps,
    PipelineStatisticsQuery,
}

impl RenderCapabilityKind {
    pub const ALL: [Self; 19] = [
        Self::VirtualGeometry,
        Self::HybridGlobalIllumination,
        Self::AccelerationStructures,
        Self::InlineRayQuery,
        Self::RayTracingPipeline,
        Self::BufferBindingArray,
        Self::TextureBindingArray,
        Self::NonUniformResourceIndexing,
        Self::PartiallyBoundBindingArray,
        Self::ScreenSpaceAntiAlias,
        Self::StorageBuffers,
        Self::IndirectDraw,
        Self::BufferReadback,
        Self::AsyncCompute,
        Self::AsyncCopy,
        Self::NeuralCompute,
        Self::SparseTexture,
        Self::SubgroupOps,
        Self::PipelineStatisticsQuery,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::VirtualGeometry => "virtual_geometry",
            Self::HybridGlobalIllumination => "hybrid_global_illumination",
            Self::AccelerationStructures => "acceleration_structures",
            Self::InlineRayQuery => "inline_ray_query",
            Self::RayTracingPipeline => "ray_tracing_pipeline",
            Self::BufferBindingArray => "buffer_binding_array",
            Self::TextureBindingArray => "texture_binding_array",
            Self::NonUniformResourceIndexing => "non_uniform_resource_indexing",
            Self::PartiallyBoundBindingArray => "partially_bound_binding_array",
            Self::ScreenSpaceAntiAlias => "screen_space_anti_alias",
            Self::StorageBuffers => "storage_buffers",
            Self::IndirectDraw => "indirect_draw",
            Self::BufferReadback => "buffer_readback",
            Self::AsyncCompute => "async_compute",
            Self::AsyncCopy => "async_copy",
            Self::NeuralCompute => "neural_compute",
            Self::SparseTexture => "sparse_texture",
            Self::SubgroupOps => "subgroup_ops",
            Self::PipelineStatisticsQuery => "pipeline_statistics_query",
        }
    }

    pub const fn capability_class(self) -> RenderCapabilityClass {
        match self {
            Self::ScreenSpaceAntiAlias => RenderCapabilityClass::Default,
            Self::VirtualGeometry
            | Self::HybridGlobalIllumination
            | Self::StorageBuffers
            | Self::IndirectDraw
            | Self::BufferReadback
            | Self::AsyncCompute
            | Self::AsyncCopy => RenderCapabilityClass::Advanced,
            Self::AccelerationStructures
            | Self::InlineRayQuery
            | Self::RayTracingPipeline
            | Self::BufferBindingArray
            | Self::TextureBindingArray
            | Self::NonUniformResourceIndexing
            | Self::PartiallyBoundBindingArray
            | Self::NeuralCompute
            | Self::SparseTexture
            | Self::SubgroupOps
            | Self::PipelineStatisticsQuery => RenderCapabilityClass::Experimental,
        }
    }

    pub fn is_satisfied_by(self, capabilities: &RenderCapabilitySummary) -> bool {
        match self {
            Self::VirtualGeometry => capabilities.virtual_geometry_supported,
            Self::HybridGlobalIllumination => capabilities.hybrid_global_illumination_supported,
            Self::AccelerationStructures => capabilities.acceleration_structures_supported,
            Self::InlineRayQuery => capabilities.inline_ray_query,
            Self::RayTracingPipeline => capabilities.ray_tracing_pipeline,
            Self::BufferBindingArray => capabilities.supports_buffer_binding_array,
            Self::TextureBindingArray => capabilities.supports_texture_binding_array,
            Self::NonUniformResourceIndexing => capabilities.supports_non_uniform_resource_indexing,
            Self::PartiallyBoundBindingArray => capabilities.supports_partially_bound_binding_array,
            Self::ScreenSpaceAntiAlias => capabilities.supports_fxaa || capabilities.supports_smaa,
            Self::StorageBuffers => capabilities.supports_storage_buffers,
            Self::IndirectDraw => capabilities.supports_indirect_draw,
            Self::BufferReadback => capabilities.supports_buffer_readback,
            Self::AsyncCompute => capabilities.supports_async_compute,
            Self::AsyncCopy => capabilities.supports_async_copy,
            Self::NeuralCompute => capabilities.supports_neural_compute,
            Self::SparseTexture => capabilities.supports_sparse_texture,
            Self::SubgroupOps => capabilities.supports_subgroup,
            Self::PipelineStatisticsQuery => capabilities.supports_pipeline_statistics_query,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderCapabilityClass {
    Default,
    Advanced,
    Experimental,
}

impl RenderCapabilityClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Advanced => "advanced",
            Self::Experimental => "experimental",
        }
    }

    const fn capability_count(self) -> usize {
        match self {
            Self::Default => 1,
            Self::Advanced => 7,
            Self::Experimental => 11,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderCapabilityMismatchDetail {
    pub capability: RenderCapabilityKind,
}

impl RenderCapabilityMismatchDetail {
    // Keep mismatch payloads backend-neutral so framework consumers never need graphics enums.
    pub const fn new(capability: RenderCapabilityKind) -> Self {
        Self { capability }
    }

    pub const fn label(self) -> &'static str {
        self.capability.label()
    }

    pub const fn capability_class(self) -> RenderCapabilityClass {
        self.capability.capability_class()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderCapabilitySummary {
    pub backend_name: String,
    pub queue_classes: Vec<RenderQueueCapability>,
    pub supports_surface: bool,
    pub supports_offscreen: bool,
    pub supports_async_compute: bool,
    pub supports_async_copy: bool,
    pub supports_pipeline_cache: bool,
    pub supports_gpu_timestamp: bool,
    pub supports_subgroup: bool,
    pub supports_pipeline_statistics_query: bool,
    pub supports_storage_buffers: bool,
    pub supports_fragment_writable_storage: bool,
    pub max_storage_buffers_per_shader_stage: u32,
    pub max_storage_buffer_binding_size: u64,
    pub max_binding_array_elements_per_shader_stage: u32,
    pub max_binding_array_sampler_elements_per_shader_stage: u32,
    pub supports_indirect_draw: bool,
    pub supports_multi_draw_indirect: bool,
    pub supports_multi_draw_indirect_count: bool,
    pub supports_indirect_first_instance: bool,
    pub supports_buffer_readback: bool,
    pub acceleration_structures_supported: bool,
    pub inline_ray_query: bool,
    pub ray_tracing_pipeline: bool,
    pub supports_buffer_binding_array: bool,
    pub supports_texture_binding_array: bool,
    pub supports_non_uniform_resource_indexing: bool,
    pub supports_partially_bound_binding_array: bool,
    pub supports_fxaa: bool,
    pub supports_smaa: bool,
    pub supports_taa: bool,
    pub supports_cas: bool,
    pub supports_dlss: bool,
    pub supports_neural_compute: bool,
    pub supports_sparse_texture: bool,
    pub max_supported_msaa_samples: u32,
    pub virtual_geometry_supported: bool,
    pub hybrid_global_illumination_supported: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderCapabilityClassReport {
    pub class: RenderCapabilityClass,
    pub satisfied: Vec<RenderCapabilityKind>,
    pub missing: Vec<RenderCapabilityMismatchDetail>,
}

impl RenderCapabilitySummary {
    pub const fn indirect_draw_submission_supported(&self) -> bool {
        self.supports_indirect_draw && self.supports_indirect_first_instance
    }

    pub const fn gpu_driven_submission_supported(&self) -> bool {
        self.indirect_draw_submission_supported() && self.supports_multi_draw_indirect
    }

    pub const fn storage_buffer_binding_capacity_supported(&self, required: u32) -> bool {
        self.max_storage_buffers_per_shader_stage >= required
    }

    pub const fn gpu_driven_indirect_count_supported(&self) -> bool {
        self.gpu_driven_submission_supported() && self.supports_multi_draw_indirect_count
    }

    pub const fn bindless_material_supported(&self) -> bool {
        self.supports_texture_binding_array
            && self.supports_partially_bound_binding_array
            && self.supports_non_uniform_resource_indexing
            && self.bindless_material_slot_capacity() >= 2
    }

    pub const fn bindless_material_slot_capacity(&self) -> u32 {
        if self.max_binding_array_elements_per_shader_stage
            < self.max_binding_array_sampler_elements_per_shader_stage
        {
            self.max_binding_array_elements_per_shader_stage
        } else {
            self.max_binding_array_sampler_elements_per_shader_stage
        }
    }

    pub const fn oit_supported(&self, required_storage_buffers_per_shader_stage: u32) -> bool {
        self.supports_storage_buffers
            && self.supports_fragment_writable_storage
            && self.storage_buffer_binding_capacity_supported(
                required_storage_buffers_per_shader_stage,
            )
    }

    pub const fn hzb_occlusion_culling_supported(
        &self,
        required_storage_buffers_per_shader_stage: u32,
    ) -> bool {
        self.supports_storage_buffers
            && self.storage_buffer_binding_capacity_supported(
                required_storage_buffers_per_shader_stage,
            )
            && self.gpu_driven_submission_supported()
    }

    pub fn capability_class_report(
        &self,
        class: RenderCapabilityClass,
    ) -> RenderCapabilityClassReport {
        let capacity = class.capability_count();
        let mut satisfied = Vec::with_capacity(capacity);
        let mut missing = Vec::with_capacity(capacity);

        for capability in RenderCapabilityKind::ALL {
            if capability.capability_class() != class {
                continue;
            }
            if capability.is_satisfied_by(self) {
                satisfied.push(capability);
            } else {
                missing.push(RenderCapabilityMismatchDetail::new(capability));
            }
        }

        RenderCapabilityClassReport {
            class,
            satisfied,
            missing,
        }
    }
}

#[cfg(test)]
mod optimization_batch_20260830bp_runtime_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        RenderCapabilityClass, RenderCapabilityKind, RenderCapabilityMismatchDetail,
        RenderCapabilitySummary,
    };

    const PERF_MARKER: &str = "RUNTIME368_CAPABILITY_REPORT_CAPACITY_BENCH_V1";

    #[test]
    fn capability_class_report_preserves_membership_and_order() {
        let capabilities = RenderCapabilitySummary {
            virtual_geometry_supported: true,
            supports_storage_buffers: true,
            supports_indirect_draw: true,
            supports_buffer_readback: true,
            ..RenderCapabilitySummary::default()
        };

        let report = capabilities.capability_class_report(RenderCapabilityClass::Advanced);
        assert_eq!(
            report.satisfied,
            vec![
                RenderCapabilityKind::VirtualGeometry,
                RenderCapabilityKind::StorageBuffers,
                RenderCapabilityKind::IndirectDraw,
                RenderCapabilityKind::BufferReadback,
            ]
        );
        assert_eq!(
            report.missing,
            vec![
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::HybridGlobalIllumination),
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AsyncCompute),
                RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AsyncCopy),
            ]
        );
        assert_eq!(
            report.satisfied.len() + report.missing.len(),
            RenderCapabilityClass::Advanced.capability_count(),
        );
        assert_eq!(
            report.satisfied.capacity(),
            RenderCapabilityClass::Advanced.capability_count(),
        );
        assert_eq!(
            report.missing.capacity(),
            RenderCapabilityClass::Advanced.capability_count(),
        );
    }

    #[test]
    fn capability_class_report_uses_class_capacity_hint() {
        let source = include_str!("capability.rs");
        let production = source.split("#[cfg(test)]").next().expect("implementation");
        assert!(production.contains("let capacity = class.capability_count();"));
        assert!(production.contains("Vec::with_capacity(capacity)"));
        assert!(!production.contains("let mut satisfied = Vec::new();"));
        assert!(!production.contains("let mut missing = Vec::new();"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn capability_class_report_capacity_p95() {
        const MATCHES: usize = 250_000;
        const SAMPLES: usize = 17;
        let capabilities = black_box(RenderCapabilitySummary {
            virtual_geometry_supported: true,
            hybrid_global_illumination_supported: true,
            acceleration_structures_supported: true,
            inline_ray_query: true,
            ray_tracing_pipeline: true,
            supports_buffer_binding_array: true,
            supports_texture_binding_array: true,
            supports_fxaa: true,
            supports_storage_buffers: true,
            supports_indirect_draw: true,
            supports_buffer_readback: true,
            supports_async_compute: true,
            supports_async_copy: true,
            ..RenderCapabilitySummary::default()
        });
        let mut baseline = Vec::with_capacity(SAMPLES);
        let mut candidate = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
            for pass in order {
                let started = Instant::now();
                let mut checksum = 0usize;
                for _ in 0..MATCHES {
                    let report = if pass == 0 {
                        let mut satisfied = Vec::new();
                        let mut missing = Vec::new();
                        for capability in RenderCapabilityKind::ALL {
                            if capability.capability_class() != RenderCapabilityClass::Experimental
                            {
                                continue;
                            }
                            if capability.is_satisfied_by(&capabilities) {
                                satisfied.push(capability);
                            } else {
                                missing.push(RenderCapabilityMismatchDetail::new(capability));
                            }
                        }
                        (satisfied, missing)
                    } else {
                        let report = capabilities
                            .capability_class_report(RenderCapabilityClass::Experimental);
                        (report.satisfied, report.missing)
                    };
                    checksum = checksum
                        .wrapping_add(report.0.len())
                        .wrapping_add(report.1.len());
                }
                black_box(checksum);
                let elapsed = started.elapsed().as_nanos();
                if pass == 0 {
                    baseline.push(elapsed);
                } else {
                    candidate.push(elapsed);
                }
            }
        }
        baseline.sort_unstable();
        candidate.sort_unstable();
        let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
        let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
        let reduction =
            100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
        println!(
            "{PERF_MARKER} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
        );
        assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
    }
}
