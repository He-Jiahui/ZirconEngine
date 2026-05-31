use zircon_runtime::core::framework::sound::SoundMixerSnapshot;

use super::SoundEngineState;

impl SoundEngineState {
    pub(crate) fn snapshot(&self) -> SoundMixerSnapshot {
        let mut graph = self.graph.clone();
        graph.sources = self
            .sources
            .values()
            .map(|source| source.descriptor.clone())
            .collect();
        graph.automation_bindings = self.automation_bindings.values().cloned().collect();
        graph.dynamic_events = self.dynamic_events.clone();
        SoundMixerSnapshot {
            graph,
            meters: self.meters.clone(),
            latency_frames: self.latency_frames,
            ray_tracing: self.ray_tracing.clone(),
        }
    }
}
