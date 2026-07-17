use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundError, SoundMixerGraph, SoundSourceDescriptor, SoundSourceId,
};

use crate::descriptor_validation::source::validate_source_descriptor_for_graph;
use crate::engine::{SoundEngineState, SourceVoice};

pub(crate) fn configured_sources(
    state: &SoundEngineState,
    graph: &SoundMixerGraph,
) -> Result<
    (
        HashMap<SoundSourceId, SourceVoice>,
        u64,
        Vec<SoundSourceDescriptor>,
    ),
    SoundError,
> {
    let mut sources = HashMap::new();
    let mut descriptors = Vec::with_capacity(graph.sources.len());
    let mut next_source_id = state.next_source_id;

    for mut descriptor in graph.sources.iter().cloned() {
        validate_source_descriptor_for_graph(state, graph, &descriptor)?;
        let source_id = descriptor.id.unwrap_or_else(|| {
            next_source_id += 1;
            SoundSourceId::new(next_source_id)
        });
        next_source_id = next_source_id.max(source_id.raw());
        descriptor.id = Some(source_id);
        descriptors.push(descriptor.clone());
        if sources
            .insert(source_id, SourceVoice::new(descriptor))
            .is_some()
        {
            return Err(SoundError::InvalidParameter(
                "configured mixer graph contains duplicate source ids".to_string(),
            ));
        }
    }

    Ok((sources, next_source_id, descriptors))
}
