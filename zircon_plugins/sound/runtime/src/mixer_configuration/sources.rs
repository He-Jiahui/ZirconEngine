//! 图导入时把缺失的声源 ID 补齐并拒绝重复，提交后推进运行时游标以避免后续分配碰撞。
use std::collections::{HashMap, HashSet};

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
    let mut reserved_source_ids = HashSet::new();

    for descriptor in &graph.sources {
        if let Some(source_id) = descriptor.id {
            if !reserved_source_ids.insert(source_id) {
                return Err(SoundError::InvalidParameter(
                    "configured mixer graph contains duplicate source ids".to_string(),
                ));
            }
        }
    }

    for mut descriptor in graph.sources.iter().cloned() {
        validate_source_descriptor_for_graph(state, graph, &descriptor)?;
        let source_id = if let Some(source_id) = descriptor.id {
            source_id
        } else {
            let mut next = next_source_id.checked_add(1).ok_or_else(|| {
                SoundError::InvalidParameter("sound source id space is exhausted".to_string())
            })?;
            while reserved_source_ids.contains(&SoundSourceId::new(next))
                || sources.contains_key(&SoundSourceId::new(next))
            {
                next = next.checked_add(1).ok_or_else(|| {
                    SoundError::InvalidParameter("sound source id space is exhausted".to_string())
                })?;
            }
            next_source_id = next;
            SoundSourceId::new(next)
        };
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
