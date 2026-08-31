use std::sync::Arc;

use zircon_runtime::core::framework::sound::SoundMixerGraph;

use super::SoundEngineState;

#[derive(Clone)]
pub(crate) struct SoundGraphSnapshot {
    pub(crate) revision: u64,
    pub(crate) kira_active: bool,
    pub(crate) graph: Arc<SoundMixerGraph>,
}

impl SoundEngineState {
    pub(crate) fn graph_snapshot(&self) -> SoundGraphSnapshot {
        SoundGraphSnapshot {
            revision: self.graph_revision,
            kira_active: self.kira.is_active(),
            graph: Arc::clone(&self.graph),
        }
    }

    pub(crate) fn replace_graph(&mut self, graph: SoundMixerGraph) {
        self.graph = Arc::new(graph);
        self.graph_revision = self.graph_revision.wrapping_add(1);
    }

    pub(crate) fn commit_validated_graph_mutation(
        &mut self,
        mutate: impl FnOnce(&mut SoundMixerGraph),
    ) {
        mutate(Arc::make_mut(&mut self.graph));
        self.graph_revision = self.graph_revision.wrapping_add(1);
    }

    pub(crate) fn update_graph_format(
        &mut self,
        sample_rate_hz: u32,
        channel_count: u16,
        channel_layout: zircon_runtime::core::framework::audio::AudioChannelLayout,
    ) {
        self.commit_validated_graph_mutation(|graph| {
            graph.sample_rate_hz = sample_rate_hz;
            graph.channel_count = channel_count;
            graph.channel_layout = channel_layout;
        });
    }
}
