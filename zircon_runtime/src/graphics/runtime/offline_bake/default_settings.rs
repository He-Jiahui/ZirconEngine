use super::offline_bake_settings::OfflineBakeSettings;

impl Default for OfflineBakeSettings {
    fn default() -> Self {
        Self {
            ambient_scale: 0.2,
            reflection_probe_scale: 0.75,
            max_reflection_probes: 4,
        }
    }
}
