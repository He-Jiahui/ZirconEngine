use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::RenderHybridGiExtract;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderHybridGiProbe;
use zircon_runtime::core::math::Vec3;
use zircon_runtime::graphics::hybrid_gi_extract_sources::{
    enabled_hybrid_gi_extract, hybrid_gi_extract_probe_records_by_id,
    hybrid_gi_extract_uses_scene_representation_budget, HybridGiExtractProbeRecord,
};

pub(super) trait HybridGiProbeSource {
    fn probe_id(&self) -> u32;
    fn position(&self) -> Vec3;
    fn radius(&self) -> f32;
    fn parent_probe_id(&self) -> Option<u32>;
    fn ray_budget(&self) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct HybridGiRuntimeProbeSource {
    probe_id: u32,
    position: Vec3,
    radius: f32,
    parent_probe_id: Option<u32>,
    ray_budget: u32,
}

impl HybridGiRuntimeProbeSource {
    pub(super) fn new(
        probe_id: u32,
        position: Vec3,
        radius: f32,
        parent_probe_id: Option<u32>,
        ray_budget: u32,
    ) -> Self {
        Self {
            probe_id,
            position,
            radius,
            parent_probe_id,
            ray_budget,
        }
    }
}

impl HybridGiProbeSource for HybridGiRuntimeProbeSource {
    fn probe_id(&self) -> u32 {
        self.probe_id
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn radius(&self) -> f32 {
        self.radius
    }

    fn parent_probe_id(&self) -> Option<u32> {
        self.parent_probe_id
    }

    fn ray_budget(&self) -> u32 {
        self.ray_budget
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct HybridGiExtractProbeSource {
    probe_id: u32,
    position: Vec3,
    radius: f32,
    parent_probe_id: Option<u32>,
    ray_budget: u32,
}

pub(super) fn fallback_probe_sources_by_id(
    extract: Option<&RenderHybridGiExtract>,
) -> BTreeMap<u32, HybridGiExtractProbeSource> {
    let Some(extract) = enabled_hybrid_gi_extract(extract) else {
        return BTreeMap::new();
    };
    if hybrid_gi_extract_uses_scene_representation_budget(extract) {
        return BTreeMap::new();
    }

    hybrid_gi_extract_probe_records_by_id(extract)
        .into_iter()
        .map(|(probe_id, probe)| (probe_id, HybridGiExtractProbeSource::from(probe)))
        .collect()
}

impl From<HybridGiExtractProbeRecord> for HybridGiExtractProbeSource {
    fn from(probe: HybridGiExtractProbeRecord) -> Self {
        Self {
            probe_id: probe.probe_id,
            position: probe.position,
            radius: probe.radius,
            parent_probe_id: probe.parent_probe_id,
            ray_budget: probe.ray_budget,
        }
    }
}

impl HybridGiProbeSource for HybridGiExtractProbeSource {
    fn probe_id(&self) -> u32 {
        self.probe_id
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn radius(&self) -> f32 {
        self.radius
    }

    fn parent_probe_id(&self) -> Option<u32> {
        self.parent_probe_id
    }

    fn ray_budget(&self) -> u32 {
        self.ray_budget
    }
}

#[cfg(test)]
impl HybridGiProbeSource for RenderHybridGiProbe {
    fn probe_id(&self) -> u32 {
        self.probe_id
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn radius(&self) -> f32 {
        self.radius
    }

    fn parent_probe_id(&self) -> Option<u32> {
        self.parent_probe_id
    }

    fn ray_budget(&self) -> u32 {
        self.ray_budget
    }
}
