use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::ShaderVariantKey;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantMissReport {
    pub request_count: usize,
    pub memory_hit_count: usize,
    pub disk_hit_count: usize,
    pub compile_miss_count: usize,
    pub disk_write_count: usize,
    pub disk_error_count: usize,
    #[serde(default)]
    pub dimension_summary: ShaderVariantRuntimeDimensionSummary,
}

impl ShaderVariantMissReport {
    pub fn record_request(&mut self, key: &ShaderVariantKey) {
        self.request_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::Request);
    }

    pub fn record_memory_hit(&mut self, key: &ShaderVariantKey) {
        self.request_count += 1;
        self.memory_hit_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::MemoryHit);
    }

    pub fn record_disk_hit(&mut self, key: &ShaderVariantKey) {
        self.disk_hit_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::DiskHit);
    }

    pub fn record_compile_miss(&mut self, key: &ShaderVariantKey) {
        self.compile_miss_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::CompileMiss);
    }

    pub fn record_disk_write(&mut self, key: &ShaderVariantKey) {
        self.disk_write_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::DiskWrite);
    }

    pub fn record_disk_error(&mut self, key: &ShaderVariantKey) {
        self.disk_error_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::DiskError);
    }

    pub fn accumulate(&mut self, other: Self) {
        self.request_count += other.request_count;
        self.memory_hit_count += other.memory_hit_count;
        self.disk_hit_count += other.disk_hit_count;
        self.compile_miss_count += other.compile_miss_count;
        self.disk_write_count += other.disk_write_count;
        self.disk_error_count += other.disk_error_count;
        self.dimension_summary.accumulate(&other.dimension_summary);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantRuntimeDimensionSummary {
    pub pass_types: BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    pub geometry_source_ids: BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    pub shading_model_ids: BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    pub quality_tiers: BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
}

impl ShaderVariantRuntimeDimensionSummary {
    fn record(&mut self, key: &ShaderVariantKey, outcome: ShaderVariantRuntimeOutcome) {
        record_dimension(&mut self.pass_types, key.pass_type.token(), outcome);
        let geometry_source_id = key.geometry_source.value().to_string();
        record_dimension(&mut self.geometry_source_ids, &geometry_source_id, outcome);
        let shading_model_id = key.shading_model.value().to_string();
        record_dimension(&mut self.shading_model_ids, &shading_model_id, outcome);
        record_dimension(&mut self.quality_tiers, key.quality.token(), outcome);
    }

    fn accumulate(&mut self, other: &Self) {
        accumulate_dimensions(&mut self.pass_types, &other.pass_types);
        accumulate_dimensions(&mut self.geometry_source_ids, &other.geometry_source_ids);
        accumulate_dimensions(&mut self.shading_model_ids, &other.shading_model_ids);
        accumulate_dimensions(&mut self.quality_tiers, &other.quality_tiers);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantRuntimeDimensionCount {
    pub request_count: usize,
    pub memory_hit_count: usize,
    pub disk_hit_count: usize,
    pub compile_miss_count: usize,
    pub disk_write_count: usize,
    pub disk_error_count: usize,
}

impl ShaderVariantRuntimeDimensionCount {
    fn record(&mut self, outcome: ShaderVariantRuntimeOutcome) {
        match outcome {
            ShaderVariantRuntimeOutcome::Request => self.request_count += 1,
            ShaderVariantRuntimeOutcome::MemoryHit => {
                self.request_count += 1;
                self.memory_hit_count += 1;
            }
            ShaderVariantRuntimeOutcome::DiskHit => self.disk_hit_count += 1,
            ShaderVariantRuntimeOutcome::CompileMiss => self.compile_miss_count += 1,
            ShaderVariantRuntimeOutcome::DiskWrite => self.disk_write_count += 1,
            ShaderVariantRuntimeOutcome::DiskError => self.disk_error_count += 1,
        }
    }

    fn accumulate(&mut self, other: Self) {
        self.request_count += other.request_count;
        self.memory_hit_count += other.memory_hit_count;
        self.disk_hit_count += other.disk_hit_count;
        self.compile_miss_count += other.compile_miss_count;
        self.disk_write_count += other.disk_write_count;
        self.disk_error_count += other.disk_error_count;
    }
}

#[derive(Clone, Copy, Debug)]
enum ShaderVariantRuntimeOutcome {
    Request,
    MemoryHit,
    DiskHit,
    CompileMiss,
    DiskWrite,
    DiskError,
}

fn record_dimension(
    counts: &mut BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    key: &str,
    outcome: ShaderVariantRuntimeOutcome,
) {
    if let Some(count) = counts.get_mut(key) {
        count.record(outcome);
        return;
    }

    let mut count = ShaderVariantRuntimeDimensionCount::default();
    count.record(outcome);
    counts.insert(key.to_owned(), count);
}

fn accumulate_dimensions(
    counts: &mut BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    other: &BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
) {
    for (key, other_count) in other {
        counts
            .entry(key.clone())
            .or_default()
            .accumulate(*other_count);
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::resource::ResourceId;

    use crate::core::framework::render::{
        GeometrySourceId, SHADING_MODEL_ID_STANDARD_PBR, ShaderFeatureBits, ShaderPassType,
        ShaderQualityTier, ShaderVariantKey,
    };

    use super::ShaderVariantMissReport;

    #[test]
    fn shader_variant_miss_report_groups_runtime_outcomes_by_variant_dimensions() {
        let key = ShaderVariantKey {
            material_shader: ResourceId::from_stable_label("res://materials/runtime-hit.zshader"),
            material_revision: 11,
            material_layout_hash: 0,
            material_option_bits: 0,
            geometry_source: GeometrySourceId::new(3),
            shading_model: SHADING_MODEL_ID_STANDARD_PBR,
            pass_type: ShaderPassType::Velocity,
            features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
            quality: ShaderQualityTier::High,
            platform_token: "wgpu-runtime".to_string(),
        };
        let mut report = ShaderVariantMissReport::default();

        report.record_request(&key);
        report.record_disk_hit(&key);
        report.record_compile_miss(&key);
        report.record_disk_write(&key);
        report.record_disk_error(&key);

        assert_eq!(report.request_count, 1);
        assert_eq!(report.disk_hit_count, 1);
        assert_eq!(report.compile_miss_count, 1);
        assert_eq!(report.disk_write_count, 1);
        assert_eq!(report.disk_error_count, 1);

        let pass = report
            .dimension_summary
            .pass_types
            .get("velocity")
            .expect("velocity runtime dimension");
        assert_eq!(pass.request_count, 1);
        assert_eq!(pass.disk_hit_count, 1);
        assert_eq!(pass.compile_miss_count, 1);
        assert_eq!(pass.disk_write_count, 1);
        assert_eq!(pass.disk_error_count, 1);
        assert_eq!(
            report.dimension_summary.geometry_source_ids["3"].disk_hit_count,
            1
        );
        assert_eq!(
            report.dimension_summary.shading_model_ids["2"].compile_miss_count,
            1
        );
        assert_eq!(
            report.dimension_summary.quality_tiers["high"].disk_write_count,
            1
        );
    }
}
