#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OfflineBakeSettings {
    pub ambient_scale: f32,
    pub reflection_probe_scale: f32,
    pub max_reflection_probes: usize,
}
