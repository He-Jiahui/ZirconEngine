use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 07 asset worker frame sampler",
        &[
            "AssetWorkerPoolFrameSampler",
            "asset.worker.frame_completed",
            "asset_worker_anchor_count = 13",
            "worker_diagnostic_count = 7",
        ],
    ),
    (
        "Runtime 07 asset worker manager sampler entry",
        &[
            "spawn_worker_pool_with_frame_sampler",
            "AssetWorkerPoolFrameSampler::from_pool(&pool)",
            "asset_worker_anchor_count = 13",
            "expected_source_file_count = 26",
        ],
    ),
    (
        "Runtime 07 artifact cache payload owner split",
        &[
            "cache_payload/{json_value,mesh,toml_value}.rs",
            "runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
            "expected_source_file_count = 22",
            "large_file_hotspot_count = 41",
        ],
    ),
    (
        "Runtime 07 render product diagnostics owner split",
        &[
            "render_stats_store/product/{camera,mesh_queue,gpu_scene}.rs",
            "runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
            "expected_source_file_count = 38",
            "large_file_hotspot_count = 39",
        ],
    ),
    (
        "Runtime 07 virtual geometry debug snapshot owner split",
        &[
            "virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,snapshot}.rs",
            "runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed",
            "Runtime 07 owner-budget 36-hotspot navigation split sync",
            "extract/ecs_query/performance profiling/FPS Cargo gates",
        ],
    ),
];
