#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use super::super::root_render_passes::VirtualGeometryIndirectStatsStoreParts;
#[cfg(test)]
use super::read_indirect_authority_records::{
    read_virtual_geometry_indirect_authority_records,
    read_virtual_geometry_indirect_execution_authority_records,
};

#[cfg(test)]
pub(crate) fn read_virtual_geometry_indirect_execution_draw_ref_indices(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<u32> {
    let execution_authority_records =
        read_virtual_geometry_indirect_execution_authority_records(parts);
    if !execution_authority_records.is_empty() {
        return execution_authority_records
            .into_iter()
            .map(|record| record.draw_ref_index())
            .collect();
    }
    if let Some(draw_ref_indices) = draw_ref_indices_from_execution_args_and_authority(parts) {
        return draw_ref_indices;
    }
    if let Some(draw_ref_indices) =
        draw_ref_indices_from_execution_args_and_shared_submission_tokens(parts)
    {
        return draw_ref_indices;
    }
    Vec::new()
}

#[cfg(test)]
fn draw_ref_indices_from_execution_args_and_authority(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Option<Vec<u32>> {
    let execution_tokens = read_execution_submission_tokens(parts);
    if execution_tokens.is_empty() {
        return None;
    }

    let authority_by_submission_token = read_virtual_geometry_indirect_authority_records(parts)
        .into_iter()
        .map(|record| (record.submission_token(), record.draw_ref_index()))
        .collect::<HashMap<_, _>>();
    if authority_by_submission_token.is_empty() {
        return None;
    }

    let draw_ref_indices = execution_tokens
        .into_iter()
        .filter_map(|submission_token| {
            authority_by_submission_token
                .get(&submission_token)
                .copied()
        })
        .collect::<Vec<_>>();
    if draw_ref_indices.is_empty() {
        return None;
    }

    Some(draw_ref_indices)
}

#[cfg(test)]
fn draw_ref_indices_from_execution_args_and_shared_submission_tokens(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Option<Vec<u32>> {
    let execution_tokens = read_execution_submission_tokens(parts);
    if execution_tokens.is_empty() {
        return None;
    }

    let mut shared_draw_ref_index_by_token = read_indirect_submission_tokens(parts)
        .into_iter()
        .enumerate()
        .map(|(draw_ref_index, submission_token)| (submission_token, draw_ref_index as u32))
        .collect::<HashMap<_, _>>();
    if shared_draw_ref_index_by_token.is_empty() {
        shared_draw_ref_index_by_token = read_indirect_args_with_instances(parts)
            .into_iter()
            .enumerate()
            .map(
                |(draw_ref_index, (_first_index, _index_count, submission_token))| {
                    (submission_token, draw_ref_index as u32)
                },
            )
            .collect::<HashMap<_, _>>();
    }
    if shared_draw_ref_index_by_token.is_empty() {
        return None;
    }

    let draw_ref_indices = execution_tokens
        .into_iter()
        .filter_map(|submission_token| {
            shared_draw_ref_index_by_token
                .get(&submission_token)
                .copied()
        })
        .collect::<Vec<_>>();
    if draw_ref_indices.is_empty() {
        return None;
    }

    Some(draw_ref_indices)
}

#[cfg(test)]
fn read_execution_submission_tokens(parts: &VirtualGeometryIndirectStatsStoreParts) -> Vec<u32> {
    parts
        .execution_segments
        .iter()
        .map(|segment| submission_token(segment.submission_index, segment.draw_ref_rank))
        .collect()
}

#[cfg(test)]
fn read_indirect_submission_tokens(parts: &VirtualGeometryIndirectStatsStoreParts) -> Vec<u32> {
    parts
        .draw_submission_token_records
        .iter()
        .map(
            |(_entity, _page_id, submission_index, draw_ref_rank, _original_index)| {
                submission_token(Some(*submission_index), Some(*draw_ref_rank))
            },
        )
        .collect()
}

#[cfg(test)]
fn read_indirect_args_with_instances(
    parts: &VirtualGeometryIndirectStatsStoreParts,
) -> Vec<(u32, u32, u32)> {
    parts
        .execution_segments
        .iter()
        .map(|segment| {
            (
                segment.cluster_start_ordinal,
                segment.cluster_span_count,
                submission_token(segment.submission_index, segment.draw_ref_rank),
            )
        })
        .collect()
}

#[cfg(test)]
fn submission_token(submission_index: Option<u32>, draw_ref_rank: Option<u32>) -> u32 {
    (submission_index.unwrap_or(u32::MAX).min(0xffff) << 16)
        | draw_ref_rank.unwrap_or(u32::MAX).min(0xffff)
}
