use crate::core::framework::render::{ShadingModelDescriptor, ShadingModelId};
use crate::graphics::material::{ShadingModelIncludeSourceError, ShadingModelIncludeSourceSet};

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn shading_model_descriptor(
        &self,
        id: ShadingModelId,
    ) -> Option<&ShadingModelDescriptor> {
        self.shading_model_registry.get(id)
    }

    pub(crate) fn shading_model_descriptor_for_pipeline_key(
        &self,
        key: &super::super::PipelineKey,
    ) -> Option<&ShadingModelDescriptor> {
        self.shading_model_descriptor(key.shading_model_id)
    }

    pub(crate) fn shading_model_descriptors(&self) -> Vec<ShadingModelDescriptor> {
        self.shading_model_registry.descriptors().cloned().collect()
    }

    pub(crate) fn shading_model_include_source_set(
        &self,
    ) -> Result<ShadingModelIncludeSourceSet, ShadingModelIncludeSourceError> {
        let asset_manager =
            self.asset_manager()
                .map_err(|error| ShadingModelIncludeSourceError::LoadShader {
                    token: "project_asset_manager".to_string(),
                    locator: "runtime://manager/project-assets".to_string(),
                    message: error.to_string(),
                })?;
        ShadingModelIncludeSourceSet::from_project_asset_manager_iter(
            asset_manager.as_ref(),
            self.shading_model_registry.descriptors(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::{
        GBufferChannelMask, ShadingModelDescriptor, ShadingModelId,
    };
    use crate::graphics::material::ShadingModelRegistry;

    #[test]
    fn shading_model_include_source_set_borrows_registry_descriptors() {
        let source = include_str!("resource_streamer_shading_models.rs");
        assert!(source.contains("from_project_asset_manager_iter"));
        assert!(source.contains("self.shading_model_registry.descriptors()"));
        assert!(!source.contains("&self.shading_model_descriptors()"));
    }

    #[test]
    fn shading_model_include_source_set_iterator_preserves_descriptor_access() {
        let source = include_str!("../../../material/shading_models/include_sources.rs");
        assert!(
            source.contains(
                "Self::from_project_asset_manager_iter(asset_manager, descriptors.iter())"
            )
        );
        assert!(
            source.contains("descriptors: impl IntoIterator<Item = &'a ShadingModelDescriptor>")
        );
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_do_borrowed_shading_model_descriptors_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const SNAPSHOTS_PER_SAMPLE: usize = 8_192;
        const DESCRIPTOR_COUNT: usize = 48;

        let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
        for index in 0..DESCRIPTOR_COUNT {
            registry
                .register_plugin_descriptor(ShadingModelDescriptor::new(
                    ShadingModelId::new(16 + index as u8),
                    format!("plugin_{index}"),
                    format!("forward_{index}"),
                    format!("gbuffer_{index}"),
                    format!("deferred_{index}"),
                    GBufferChannelMask::standard_lit(),
                ))
                .expect("unique plugin descriptor");
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_descriptor_snapshots(
                    &registry,
                    SNAPSHOTS_PER_SAMPLE,
                    true,
                ));
                optimized_samples.push(measure_descriptor_snapshots(
                    &registry,
                    SNAPSHOTS_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples.push(measure_descriptor_snapshots(
                    &registry,
                    SNAPSHOTS_PER_SAMPLE,
                    false,
                ));
                legacy_samples.push(measure_descriptor_snapshots(
                    &registry,
                    SNAPSHOTS_PER_SAMPLE,
                    true,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME423_BORROWED_SHADING_MODEL_DESCRIPTORS_BENCH_V1 descriptors={DESCRIPTOR_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "borrowed shading model descriptor p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_descriptor_snapshots(
        registry: &ShadingModelRegistry,
        snapshot_count: usize,
        legacy: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..snapshot_count {
            if legacy {
                let descriptors = registry.descriptors().cloned().collect::<Vec<_>>();
                checksum = checksum.wrapping_add(
                    descriptors
                        .iter()
                        .map(|descriptor| descriptor.token.len() + descriptor.id.value() as usize)
                        .sum::<usize>(),
                );
            } else {
                checksum = checksum.wrapping_add(
                    registry
                        .descriptors()
                        .map(|descriptor| descriptor.token.len() + descriptor.id.value() as usize)
                        .sum::<usize>(),
                );
            }
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
