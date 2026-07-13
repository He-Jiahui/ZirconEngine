use super::offline_bake_settings::OfflineBakeSettings;

impl Default for OfflineBakeSettings {
    fn default() -> Self {
        Self {
            reflection_probe_scale: 0.75,
            max_reflection_probes: 4,
        }
    }
}
