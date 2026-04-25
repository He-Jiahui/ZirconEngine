use super::prelude::*;
use super::support::{execute_pass, viewport_frame};

#[test]
fn node_and_cluster_cull_pass_publishes_cull_input_record_when_present() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let frame = viewport_frame(UVec2::new(96, 64));
    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 4,
        page_budget: 2,
        instance_count: 3,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 2,
        visible_cluster_count: 7,
        resident_page_count: 1,
        pending_page_request_count: 2,
        available_page_slot_count: 1,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(6),
            freeze_cull: true,
            visualize_bvh: false,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };
    let output = execute_pass(&backend, true, &frame, Some(&cull_input));
    assert_eq!(
        output.source,
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput
    );
    assert_eq!(output.record_count, 1);
    assert!(
        output.buffer.is_some(),
        "expected the first NodeAndClusterCull bridge to materialize a real GPU buffer from the stable cull-input layout"
    );
    assert_eq!(
        output.instance_seeds,
        vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
            instance_index: 0,
            entity: 7,
            cluster_offset: 10,
            cluster_count: 2,
            page_offset: 20,
            page_count: 1,
        }],
        "expected the next NodeAndClusterCull bridge step to publish one per-instance root seed row from the effective VG extract instead of leaving the typed global-state record without an instance worklist seam"
    );
    assert_eq!(output.instance_seed_count, 1);
    assert!(output.instance_seed_buffer.is_some());
}

#[test]
fn node_and_cluster_cull_pass_reports_clear_only_without_cull_input() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let output = execute_pass(&backend, true, &viewport_frame(UVec2::new(96, 64)), None);

    assert_eq!(
        output.source,
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathClearOnly
    );
    assert_eq!(output.record_count, 0);
    assert!(output.buffer.is_none());
}

#[test]
fn node_and_cluster_cull_pass_derives_dispatch_setup_from_global_state_and_seed_count() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let output = execute_pass(
        &backend,
        true,
        &viewport_frame(UVec2::new(96, 64)),
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert_eq!(
        output.dispatch_setup,
        Some(RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
            instance_seed_count: 1,
            cluster_budget: 1,
            page_budget: 2,
            workgroup_size: 64,
            dispatch_group_count: [1, 1, 1],
        }),
        "expected the first explicit NodeAndClusterCull dispatch/setup seam to consume the typed global-state budget inputs together with the derived root-seed count instead of leaving dispatch dimensions as implicit host logic"
    );
    assert!(
        output.dispatch_setup_buffer.is_some(),
        "expected the dispatch/setup seam to materialize a dedicated GPU buffer beside the global-state and seed buffers so a later compute NodeAndClusterCull pass can bind explicit startup parameters"
    );
}

#[test]
fn node_and_cluster_cull_pass_publishes_launch_worklist_from_global_state_dispatch_and_seeds() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let frame = viewport_frame(UVec2::new(96, 64));
    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert_eq!(
        output.launch_worklist,
        Some(RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
            global_state: output
                .global_state
                .clone()
                .expect("expected cull global state"),
            dispatch_setup: output
                .dispatch_setup
                .expect("expected cull dispatch setup"),
            instance_seeds: output.instance_seeds.clone(),
        }),
        "expected NodeAndClusterCull to publish one explicit launch/worklist contract that combines the global-state buffer, dispatch/setup record, and root seed rows before the compat execution path or a future compute traversal consumes them"
    );
}

#[test]
fn node_and_cluster_cull_pass_materializes_launch_worklist_buffers_for_seed_consumer() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let frame = viewport_frame(UVec2::new(96, 64));
    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 4,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );
    assert!(
        output.launch_worklist_buffer.is_some(),
        "expected NodeAndClusterCull to materialize a renderer-owned launch-worklist GPU buffer so the seed-backed compat consumer can bind one authoritative startup package instead of reconstructing the root seeds ad hoc"
    );
    assert_eq!(output.instance_seed_count, 1);
    assert!(output.instance_seed_buffer.is_some());
}

#[test]
fn node_and_cluster_cull_pass_publishes_instance_work_items_from_launch_worklist_contract() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let frame = viewport_frame(UVec2::new(96, 64));
    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 4,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert_eq!(output.instance_work_item_count, 1);
    assert_eq!(
        output.instance_work_items,
        vec![RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
            instance_index: 0,
            entity: 7,
            cluster_offset: 10,
            cluster_count: 2,
            page_offset: 20,
            page_count: 1,
            cluster_budget: 4,
            page_budget: 2,
            forced_mip: Some(6),
        }],
        "expected NodeAndClusterCull to publish one typed per-instance work-item seam from the launch-worklist contract so compat execution and later GPU traversal can consume the same first compute-stub output"
    );
    assert!(
        output.instance_work_item_buffer.is_some(),
        "expected the first compute-stub output seam to materialize a renderer-owned GPU buffer instead of keeping instance work items as CPU-only pass-local data"
    );
}

#[test]
fn node_and_cluster_cull_pass_publishes_cluster_work_items_from_instance_work_item_contract() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let frame = viewport_frame(UVec2::new(96, 64));
    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 4,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert_eq!(
        output.cluster_work_items,
        vec![
            VirtualGeometryNodeAndClusterCullClusterWorkItem {
                instance_index: 0,
                entity: 7,
                cluster_array_index: 10,
                hierarchy_node_id: None,
                cluster_budget: 4,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullClusterWorkItem {
                instance_index: 0,
                entity: 7,
                cluster_array_index: 11,
                hierarchy_node_id: None,
                cluster_budget: 4,
                page_budget: 2,
                forced_mip: Some(6),
            },
        ],
        "expected NodeAndClusterCull to expand instance work items into one typed per-cluster candidate seam so later VisitNode or compat cluster selection stops scanning broad instance ranges directly"
    );
}

#[test]
fn node_and_cluster_cull_pass_limits_cluster_work_items_to_cull_input_cluster_count() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let frame = viewport_frame(UVec2::new(96, 64));
    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 4,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 1,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert_eq!(
        output.cluster_work_items,
        vec![VirtualGeometryNodeAndClusterCullClusterWorkItem {
            instance_index: 0,
            entity: 7,
            cluster_array_index: 10,
            hierarchy_node_id: None,
            cluster_budget: 4,
            page_budget: 2,
            forced_mip: Some(6),
        }],
        "expected NodeAndClusterCull startup to honor cull_input.cluster_count as the root cluster-candidate cap before traversal expands child work"
    );
    assert_eq!(output.cluster_work_item_count, 1);
}

#[test]
fn node_and_cluster_cull_global_state_uses_viewport_and_camera_from_frame() {
    let frame = viewport_frame(UVec2::new(320, 180));
    let snapshot = build_node_and_cluster_cull_global_state(
        &frame,
        &RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 4,
            page_budget: 2,
            instance_count: 3,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 2,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        },
        None,
    );

    assert_eq!(snapshot.viewport_size, [320, 180]);
    assert_eq!(snapshot.camera_translation, [3.0, 4.0, 5.0]);
    assert_eq!(
        snapshot.previous_camera_translation,
        snapshot.camera_translation
    );
    assert!(
        snapshot.view_proj.iter().flatten().any(|value| *value != 0.0),
        "expected the upgraded NodeAndClusterCull startup record to carry a non-empty view-projection matrix derived from the frame camera"
    );
    assert_eq!(snapshot.previous_view_proj, snapshot.view_proj);
}

#[test]
fn node_and_cluster_cull_global_state_uses_previous_global_state_when_available() {
    let frame = viewport_frame(UVec2::new(320, 180));
    let mut previous_frame = viewport_frame(UVec2::new(320, 180));
    previous_frame.scene.scene.camera.transform =
        Transform::from_translation(Vec3::new(-8.0, 2.0, 9.0));
    previous_frame.extract.view.camera.transform =
        Transform::from_translation(Vec3::new(-8.0, 2.0, 9.0));
    previous_frame.scene.scene.camera.fov_y_radians = 45.0_f32.to_radians();
    previous_frame.extract.view.camera.fov_y_radians = 45.0_f32.to_radians();

    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 4,
        page_budget: 2,
        instance_count: 3,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 2,
        visible_cluster_count: 7,
        resident_page_count: 1,
        pending_page_request_count: 2,
        available_page_slot_count: 1,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(6),
            freeze_cull: true,
            visualize_bvh: false,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };
    let previous_snapshot =
        build_node_and_cluster_cull_global_state(&previous_frame, &cull_input, None);
    let snapshot =
        build_node_and_cluster_cull_global_state(&frame, &cull_input, Some(&previous_snapshot));

    assert_eq!(
        snapshot.previous_camera_translation,
        previous_snapshot.camera_translation
    );
    assert_eq!(snapshot.previous_view_proj, previous_snapshot.view_proj);
    assert_ne!(
        snapshot.previous_camera_translation,
        snapshot.camera_translation
    );
    assert_ne!(snapshot.previous_view_proj, snapshot.view_proj);
}

#[test]
fn node_and_cluster_cull_global_state_tightens_sse_policy_for_narrower_perspective_projection() {
    let wide_frame = viewport_frame(UVec2::new(96, 64));
    let mut narrow_frame = viewport_frame(UVec2::new(96, 64));
    narrow_frame.scene.scene.camera.fov_y_radians = 30.0_f32.to_radians();
    narrow_frame.extract.view.camera.fov_y_radians = 30.0_f32.to_radians();

    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 4,
        page_budget: 2,
        instance_count: 1,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 1,
        visible_cluster_count: 7,
        resident_page_count: 1,
        pending_page_request_count: 0,
        available_page_slot_count: 1,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState::default(),
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };

    let wide_snapshot = build_node_and_cluster_cull_global_state(&wide_frame, &cull_input, None);
    let narrow_snapshot =
        build_node_and_cluster_cull_global_state(&narrow_frame, &cull_input, None);

    assert!(
        narrow_snapshot.child_split_screen_space_error_threshold
            < wide_snapshot.child_split_screen_space_error_threshold,
        "expected a narrower perspective projection to lower the child split SSE threshold so magnified clusters split sooner; wide={}, narrow={}",
        wide_snapshot.child_split_screen_space_error_threshold,
        narrow_snapshot.child_split_screen_space_error_threshold
    );
}

#[test]
fn node_and_cluster_cull_global_state_tightens_sse_policy_for_smaller_orthographic_window() {
    let mut large_window_frame = viewport_frame(UVec2::new(96, 64));
    large_window_frame.scene.scene.camera.projection_mode = ProjectionMode::Orthographic;
    large_window_frame.extract.view.camera.projection_mode = ProjectionMode::Orthographic;
    large_window_frame.scene.scene.camera.ortho_size = 8.0;
    large_window_frame.extract.view.camera.ortho_size = 8.0;

    let mut small_window_frame = large_window_frame.clone();
    small_window_frame.scene.scene.camera.ortho_size = 2.0;
    small_window_frame.extract.view.camera.ortho_size = 2.0;

    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 4,
        page_budget: 2,
        instance_count: 1,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 1,
        visible_cluster_count: 7,
        resident_page_count: 1,
        pending_page_request_count: 0,
        available_page_slot_count: 1,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState::default(),
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };

    let large_window_snapshot =
        build_node_and_cluster_cull_global_state(&large_window_frame, &cull_input, None);
    let small_window_snapshot =
        build_node_and_cluster_cull_global_state(&small_window_frame, &cull_input, None);

    assert!(
        small_window_snapshot.child_split_screen_space_error_threshold
            < large_window_snapshot.child_split_screen_space_error_threshold,
        "expected a smaller orthographic window to lower the child split SSE threshold so magnified clusters split sooner; large={}, small={}",
        large_window_snapshot.child_split_screen_space_error_threshold,
        small_window_snapshot.child_split_screen_space_error_threshold
    );
}

#[test]
fn node_and_cluster_cull_global_state_disables_child_frustum_culling_for_invalid_clip_range() {
    let mut frame = viewport_frame(UVec2::new(96, 64));
    frame.scene.scene.camera.z_near = 1.0;
    frame.scene.scene.camera.z_far = 0.5;
    frame.extract.view.camera.z_near = 1.0;
    frame.extract.view.camera.z_far = 0.5;

    let snapshot = build_node_and_cluster_cull_global_state(
        &frame,
        &RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 4,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 0,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState::default(),
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        },
        None,
    );

    assert!(
        !snapshot.child_frustum_culling_enabled,
        "expected NodeAndClusterCull startup policy to disable child frustum culling when the camera clip range cannot form a valid frustum"
    );
}
