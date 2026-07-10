use crate::core::framework::render::{ReflectionProbeData, RenderBakedLightingExtract};

#[derive(Clone, Debug, PartialEq)]
pub struct OfflineBakeOutput {
    pub baked_lighting: RenderBakedLightingExtract,
    pub reflection_probes: Vec<ReflectionProbeData>,
}
