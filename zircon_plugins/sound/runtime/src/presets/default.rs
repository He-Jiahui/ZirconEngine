use zircon_runtime::core::framework::sound::SoundMixerGraph;

use crate::SoundConfig;

pub(crate) fn default_graph(config: &SoundConfig) -> SoundMixerGraph {
    let mut graph = SoundMixerGraph::default_stereo(config.sample_rate_hz);
    graph.channel_count = config.channel_count.max(1);
    graph
}
