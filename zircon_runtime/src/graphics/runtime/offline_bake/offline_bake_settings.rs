#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OfflineBakeSettings {
    pub reflection_probe_scale: f32,
    pub max_reflection_probes: usize,
}
