use crate::core::framework::render::ReflectionProbeData;

#[derive(Clone, Debug, PartialEq)]
pub struct OfflineBakeOutput {
    pub reflection_probes: Vec<ReflectionProbeData>,
}
