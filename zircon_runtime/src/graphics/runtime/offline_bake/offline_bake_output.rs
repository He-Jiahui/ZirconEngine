use crate::core::framework::render::{RenderBakedLightingExtract, RenderReflectionProbeSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct OfflineBakeOutput {
    pub baked_lighting: RenderBakedLightingExtract,
    pub reflection_probes: Vec<RenderReflectionProbeSnapshot>,
}
