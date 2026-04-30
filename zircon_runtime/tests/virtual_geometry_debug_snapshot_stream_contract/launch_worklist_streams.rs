use super::prelude::*;

#[test]
fn debug_snapshot_exports_optional_node_and_cluster_cull_launch_worklist_words() {
    let launch_worklist = RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
        global_state: RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
            cull_input: RenderVirtualGeometryCullInputSnapshot {
                cluster_budget: 12,
                page_budget: 5,
                instance_count: 1,
                cluster_count: 4,
                page_count: 3,
                visible_entity_count: 1,
                visible_cluster_count: 4,
                resident_page_count: 2,
                pending_page_request_count: 0,
                available_page_slot_count: 1,
                evictable_page_count: 0,
                cluster_selection_input_source:
                    RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned,
                debug: RenderVirtualGeometryDebugState {
                    forced_mip: Some(2),
                    ..RenderVirtualGeometryDebugState::default()
                },
            },
            viewport_size: [320, 240],
            camera_translation: [1.0, 2.0, 3.0],
            child_split_screen_space_error_threshold: 16.0,
            child_frustum_culling_enabled: true,
            view_proj: [[1.0, 0.0, 0.0, 0.0]; 4],
            previous_camera_translation: [4.0, 5.0, 6.0],
            previous_view_proj: [[0.5, 0.0, 0.0, 0.0]; 4],
        },
        dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
            instance_seed_count: 1,
            cluster_budget: 12,
            page_budget: 5,
            workgroup_size: 64,
            dispatch_group_count: [1, 1, 1],
        },
        instance_seeds: vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
            instance_index: 3,
            entity: 42,
            cluster_offset: 10,
            cluster_count: 4,
            page_offset: 2,
            page_count: 3,
        }],
    };
    let global_state = launch_worklist.global_state.clone();
    let dispatch_setup = launch_worklist.dispatch_setup;
    let global_state_words = global_state.packed_words().to_vec();
    let dispatch_setup_words = dispatch_setup.packed_words().to_vec();
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        node_and_cluster_cull_global_state: Some(global_state.clone()),
        node_and_cluster_cull_dispatch_setup: Some(dispatch_setup),
        node_and_cluster_cull_launch_worklist: Some(launch_worklist.clone()),
        ..RenderVirtualGeometryDebugSnapshot::default()
    };

    assert_eq!(
        snapshot.node_and_cluster_cull_global_state_words(),
        Some(global_state_words.clone())
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_dispatch_setup_words(),
        Some(dispatch_setup_words.clone())
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_global_state_words(
            &global_state_words
        ),
        Some(global_state)
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_dispatch_setup_words(
            &dispatch_setup_words
        ),
        Some(dispatch_setup)
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_launch_worklist_words(),
        Some(launch_worklist.packed_words())
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_launch_worklist_words(
            &launch_worklist.packed_words()
        ),
        Some(launch_worklist)
    );
    let mut words_with_trailing_word = snapshot
        .node_and_cluster_cull_launch_worklist_words()
        .expect("launch worklist words should be exported");
    words_with_trailing_word.push(99);
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_launch_worklist_words(
            &words_with_trailing_word
        ),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_launch_worklist_words(&[
            1, 2, 3
        ]),
        None
    );
    let mut global_state_words_with_trailing_word = global_state_words;
    global_state_words_with_trailing_word.push(99);
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_global_state_words(
            &global_state_words_with_trailing_word
        ),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_dispatch_setup_words(&[
            1, 2, 3
        ]),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::default().node_and_cluster_cull_global_state_words(),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::default().node_and_cluster_cull_dispatch_setup_words(),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::default().node_and_cluster_cull_launch_worklist_words(),
        None
    );
}
