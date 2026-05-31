use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph, SoundSourceId};

use crate::descriptor_validation::source::validate_source_descriptor_for_graph;
use crate::engine::{SoundEngineState, SourceVoice};

pub(crate) fn configured_sources(
    state: &mut SoundEngineState,
    graph: &SoundMixerGraph,
) -> Result<HashMap<SoundSourceId, SourceVoice>, SoundError> {
    let mut sources = HashMap::new();
    let mut next_source_id = state.next_source_id;

    for mut descriptor in graph.sources.iter().cloned() {
        validate_source_descriptor_for_graph(state, graph, &descriptor)?;
        let source_id = descriptor.id.unwrap_or_else(|| {
            next_source_id += 1;
            SoundSourceId::new(next_source_id)
        });
        next_source_id = next_source_id.max(source_id.raw());
        descriptor.id = Some(source_id);
        if sources
            .insert(
                source_id,
                SourceVoice {
                    descriptor,
                    cursor_frame: 0,
                    cursor_position: 0.0,
                    pending_finish: None,
                },
            )
            .is_some()
        {
            return Err(SoundError::InvalidParameter(
                "configured mixer graph contains duplicate source ids".to_string(),
            ));
        }
    }

    state.next_source_id = next_source_id;
    Ok(sources)
}
