use super::prelude::*;

#[test]
fn debug_snapshot_exports_visbuffer64_readback_value_stream() {
    let selected_cluster = RenderVirtualGeometrySelectedCluster {
        instance_index: Some(4),
        entity: 0x0000_0002_0000_0033,
        cluster_id: 19,
        cluster_ordinal: 7,
        page_id: 13,
        lod_level: 3,
        state: RenderVirtualGeometryExecutionState::Resident,
    };
    let entry_from_selection =
        RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(9, &selected_cluster);
    let explicit_entry = RenderVirtualGeometryVisBuffer64Entry {
        entry_index: 11,
        packed_value: RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
            None,
            31,
            17,
            2,
            RenderVirtualGeometryExecutionState::Missing,
        ),
        instance_index: None,
        entity: 0,
        cluster_id: 31,
        page_id: 17,
        lod_level: 2,
        state: RenderVirtualGeometryExecutionState::Missing,
    };
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        visbuffer64_source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        visbuffer64_clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        visbuffer64_entries: vec![entry_from_selection.clone(), explicit_entry.clone()],
        ..RenderVirtualGeometryDebugSnapshot::default()
    };

    assert_eq!(
        snapshot.visbuffer64_readback_stream(),
        RenderVirtualGeometryVisBuffer64ReadbackStream {
            source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
            clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
            entry_indices: vec![9, 11],
            packed_values: vec![
                entry_from_selection.packed_value,
                explicit_entry.packed_value
            ],
        }
    );
    assert_eq!(
        snapshot.visbuffer64_decoded_stream(),
        Some(RenderVirtualGeometryVisBuffer64DecodedStream {
            source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
            clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
            entries: vec![
                RenderVirtualGeometryVisBuffer64Entry {
                    entry_index: 9,
                    packed_value: entry_from_selection.packed_value,
                    instance_index: Some(4),
                    entity: 0,
                    cluster_id: 19,
                    page_id: 13,
                    lod_level: 3,
                    state: RenderVirtualGeometryExecutionState::Resident,
                },
                explicit_entry,
            ],
        })
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_visbuffer64_readback_stream(
            &RenderVirtualGeometryVisBuffer64ReadbackStream {
                source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
                clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
                entry_indices: vec![9],
                packed_values: Vec::new(),
            }
        ),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::try_decode_visbuffer64_readback_stream(
            &RenderVirtualGeometryVisBuffer64ReadbackStream {
                source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
                clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
                entry_indices: vec![9],
                packed_values: Vec::new(),
            }
        ),
        Err(
            RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError::MismatchedEntryAndValueCount {
                entry_index_count: 1,
                packed_value_count: 0,
            }
        )
    );
    let invalid_state_value = 3_u64 << 62;
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::try_decode_visbuffer64_readback_stream(
            &RenderVirtualGeometryVisBuffer64ReadbackStream {
                source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
                clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
                entry_indices: vec![12],
                packed_values: vec![invalid_state_value],
            }
        ),
        Err(
            RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError::InvalidPackedState {
                entry_index: 12,
                packed_value: invalid_state_value,
            }
        )
    );
}
