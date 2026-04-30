use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometryInstance,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPage,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
};
use crate::core::math::{Transform, UVec2, Vec3};
use crate::graphics::tests::plugin_render_feature_fixtures::virtual_geometry_render_feature_descriptor;
use crate::scene::world::World;
use crate::{
    types::ViewportRenderFrame, BuiltinRenderFeature, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

fn compile_virtual_geometry_pipeline(
    extract: &crate::core::framework::render::RenderFrameExtract,
) -> crate::CompiledRenderPipeline {
    RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([virtual_geometry_render_feature_descriptor()])
        .compile_with_options(
            extract,
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_async_compute(false),
        )
        .unwrap()
}

fn virtual_geometry_scene_renderer(asset_manager: Arc<ProjectAssetManager>) -> SceneRenderer {
    SceneRenderer::new_with_plugin_render_features(
        asset_manager,
        [virtual_geometry_render_feature_descriptor()],
    )
    .unwrap()
}

#[test]
fn seed_backed_node_and_cluster_cull_can_drive_execution_selected_clusters_without_explicit_cluster_selections(
) {
    let (renderer, mesh) = render_seed_backed_execution_frame(1, None);

    assert_eq!(
        renderer.read_last_virtual_geometry_cluster_selection_input_source(),
        RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
        "expected this baseline path to start from NodeAndClusterCull root seeds rather than explicit or prepare-owned ClusterSelection input"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_node_and_cluster_cull_source(),
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        "expected the seed-driven execution baseline path to reuse the existing NodeAndClusterCull cull-input bridge as its upstream source of truth"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_selected_cluster_source(),
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected a non-empty seed-driven baseline execution path to publish execution-owned selected clusters instead of staying clear-only"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_selected_clusters()
            .expect("expected seed-driven execution selected clusters"),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 1,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected the minimal seed consumer to promote the first cluster in the seeded instance range into the shared executed-cluster seam so downstream baseline passes can stop depending on explicit ClusterSelection injection"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_visbuffer64_source(),
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        "expected the seed-driven executed-cluster seam to feed VisBuffer64 so this path no longer collapses into clear-only when explicit ClusterSelection input is absent"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_visbuffer64_words()
            .expect("expected seed-driven VisBuffer64 words")
            .1,
        vec![RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
            Some(0),
            20,
            200,
            1,
            RenderVirtualGeometryExecutionState::Resident,
        )],
        "expected the seed-driven execution selection path to preserve the selected cluster identity all the way into VisBuffer64 packing"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_hardware_rasterization_source(),
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        "expected the seed-driven executed-cluster seam to feed hardware-rasterization startup records instead of leaving that pass clear-only"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected seed-driven hardware-rasterization records"),
        vec![RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 1,
            submission_index: 0,
            submission_page_id: 200,
            submission_lod_level: 1,
            entity_cluster_start_ordinal: 0,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 2,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected the seed-driven execution selection path to preserve the same selected cluster identity and synthesize the minimal submission metadata needed by the hardware-rasterization startup seam"
    );
}

#[test]
fn seed_backed_node_and_cluster_cull_can_drive_multiple_execution_selected_clusters_without_explicit_cluster_selections(
) {
    let (renderer, mesh) = render_seed_backed_execution_frame(4, None);

    assert_eq!(
        renderer.read_last_virtual_geometry_cluster_selection_input_source(),
        RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
        "expected this baseline path to still start from NodeAndClusterCull root seeds rather than explicit or prepare-owned ClusterSelection input"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_node_and_cluster_cull_source(),
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        "expected the multi-cluster baseline path to reuse the existing NodeAndClusterCull cull-input bridge as its upstream source of truth"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_selected_cluster_source(),
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected a multi-cluster seed-driven baseline execution path to publish execution-owned selected clusters instead of staying clear-only"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_selected_clusters()
            .expect("expected multi-cluster seed-driven execution selected clusters"),
        vec![
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 1,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 0,
                state: RenderVirtualGeometryExecutionState::PendingUpload,
            },
        ],
        "expected the seed-driven execution selection path to expand the full seeded cluster range into the shared executed-cluster seam instead of truncating each instance to a single root candidate"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_visbuffer64_source(),
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        "expected the multi-cluster seed-driven executed-cluster seam to feed VisBuffer64 instead of collapsing back to clear-only when explicit ClusterSelection input is absent"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_visbuffer64_words()
            .expect("expected multi-cluster seed-driven VisBuffer64 words")
            .1,
        vec![
            RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(0),
                20,
                200,
                1,
                RenderVirtualGeometryExecutionState::Resident,
            ),
            RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(0),
                30,
                300,
                0,
                RenderVirtualGeometryExecutionState::PendingUpload,
            ),
        ],
        "expected the seed-driven execution selection path to preserve all expanded cluster identities all the way into VisBuffer64 packing"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_hardware_rasterization_source(),
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        "expected the multi-cluster seed-driven executed-cluster seam to feed hardware-rasterization startup records instead of leaving that pass clear-only"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected multi-cluster seed-driven hardware-rasterization records"),
        vec![
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 1,
                submission_index: 0,
                submission_page_id: 200,
                submission_lod_level: 1,
                entity_cluster_start_ordinal: 0,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 2,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 0,
                submission_index: 0,
                submission_page_id: 300,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 2,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::PendingUpload,
            },
        ],
        "expected the seed-driven execution selection path to synthesize one hardware-rasterization startup record per expanded cluster instead of preserving only the first seeded cluster"
    );
}

#[test]
fn seed_backed_node_and_cluster_cull_respects_forced_mip_without_explicit_cluster_selections() {
    let (renderer, mesh) = render_seed_backed_execution_frame(4, Some(0));

    assert_eq!(
        renderer.read_last_virtual_geometry_selected_cluster_source(),
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected the forced-mip seed-backed baseline path to keep publishing execution-owned selected clusters instead of collapsing to clear-only"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_selected_clusters()
            .expect("expected forced-mip seed-driven execution selected clusters"),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::PendingUpload,
        }],
        "expected the seed-driven execution selection path to honor forced_mip when expanding a seeded cluster range so render-path execution selection stays aligned with the Nanite teaching/debug mip override instead of selecting every seeded cluster regardless of mip"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_visbuffer64_words()
            .expect("expected forced-mip seed-driven VisBuffer64 words")
            .1,
        vec![RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
            Some(0),
            30,
            300,
            0,
            RenderVirtualGeometryExecutionState::PendingUpload,
        )],
        "expected forced_mip filtering to survive all the way into VisBuffer64 packing on the seed-driven baseline path"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected forced-mip seed-driven hardware-rasterization records"),
        vec![RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            submission_index: 0,
            submission_page_id: 300,
            submission_lod_level: 0,
            entity_cluster_start_ordinal: 1,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 2,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state: RenderVirtualGeometryExecutionState::PendingUpload,
        }],
        "expected forced_mip filtering to survive all the way into hardware-rasterization startup records on the seed-driven baseline path"
    );
}

#[test]
fn seed_backed_node_and_cluster_cull_preserves_lineage_depth_in_hardware_rasterization_records_without_explicit_cluster_selections(
) {
    let (renderer, mesh) = render_seed_backed_hierarchical_execution_frame(4, None);

    assert_eq!(
        renderer.read_last_virtual_geometry_selected_cluster_source(),
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected the hierarchical seed-backed baseline path to keep publishing execution-owned selected clusters instead of collapsing to clear-only"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected hierarchical seed-driven hardware-rasterization records"),
        vec![
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 2,
                submission_index: 0,
                submission_page_id: 200,
                submission_lod_level: 2,
                entity_cluster_start_ordinal: 0,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::PendingUpload,
            },
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 1,
                submission_index: 0,
                submission_page_id: 300,
                submission_lod_level: 1,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 1,
                frontier_rank: 1,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::PendingUpload,
            },
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 40,
                cluster_ordinal: 2,
                page_id: 400,
                lod_level: 0,
                submission_index: 0,
                submission_page_id: 400,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 2,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 2,
                frontier_rank: 2,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::Missing,
            },
        ],
        "expected the hierarchical seed-driven execution selection path to preserve non-zero lineage depth on child and grandchild clusters so hardware-rasterization startup metadata matches the existing visibility-plan parent-chain semantics"
    );
}

#[test]
fn seed_backed_node_and_cluster_cull_keeps_instance_local_cluster_slice_metadata_for_subset_seed_ranges_without_explicit_cluster_selections(
) {
    let (renderer, mesh) = render_seed_backed_subset_execution_frame(4, None);

    assert_eq!(
        renderer
            .read_last_virtual_geometry_selected_clusters()
            .expect("expected subset-range seed-driven execution selected clusters"),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 1,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected the subset-range seed-backed baseline path to preserve entity-local cluster ordinal instead of reusing the raw extract slice offset as though it were already the stable per-entity ordinal"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected subset-range seed-driven hardware-rasterization records"),
        vec![RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 1,
            submission_index: 0,
            submission_page_id: 200,
            submission_lod_level: 1,
            entity_cluster_start_ordinal: 0,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 1,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected subset-range seed metadata to remap the submission slice to the stable instance-local cluster ordinal instead of reusing the raw extract offset when the current root seed only spans one cluster"
    );
}

#[test]
fn seed_backed_node_and_cluster_cull_falls_back_to_resident_parent_cluster_without_explicit_cluster_selections(
) {
    let (renderer, mesh) = render_seed_backed_parent_fallback_execution_frame(4, None);

    assert_eq!(
        renderer
            .read_last_virtual_geometry_selected_clusters()
            .expect("expected resident-parent fallback selected clusters"),
        vec![
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 2,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 1,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
        ],
        "expected the seed-backed baseline path to replace an undrawable child cluster with its nearest resident ancestor while preserving resident clusters that were already selected earlier in the same seeded instance slice"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected resident-parent fallback hardware-rasterization records"),
        vec![
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 2,
                submission_index: 0,
                submission_page_id: 200,
                submission_lod_level: 2,
                entity_cluster_start_ordinal: 0,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 1,
                submission_index: 0,
                submission_page_id: 400,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 2,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 2,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
        ],
        "expected resident-parent fallback to keep the original child submission metadata even while the baseline raster path consumes the resolved resident ancestor cluster that can actually draw in place of the missing child"
    );
}

#[test]
fn seed_backed_node_and_cluster_cull_keeps_selected_cluster_order_when_later_child_overwrites_fallback_metadata_without_explicit_cluster_selections(
) {
    let (renderer, mesh) = render_seed_backed_duplicate_parent_fallback_execution_frame(4, None);

    assert_eq!(
        renderer
            .read_last_virtual_geometry_selected_clusters()
            .expect("expected duplicate resident-parent fallback selected clusters"),
        vec![
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 0,
                page_id: 300,
                lod_level: 2,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 40,
                cluster_ordinal: 1,
                page_id: 400,
                lod_level: 1,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
        ],
        "expected duplicate resident-parent fallback to keep the resolved selected-cluster order stable even when a later child request overwrites the startup metadata for the earlier resident ancestor"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_visbuffer64_words()
            .expect("expected duplicate resident-parent fallback visbuffer words")
            .1,
        vec![
            RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(0),
                30,
                300,
                2,
                RenderVirtualGeometryExecutionState::Resident,
            ),
            RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(0),
                40,
                400,
                1,
                RenderVirtualGeometryExecutionState::Resident,
            ),
        ],
        "expected VisBuffer64 packing to follow the same stable selected-cluster order as the duplicate-fallback seam instead of reordering by the overwritten child submission metadata"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected duplicate resident-parent fallback hardware-rasterization records"),
        vec![
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 0,
                page_id: 300,
                lod_level: 2,
                submission_index: 0,
                submission_page_id: 500,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 2,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 1,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 40,
                cluster_ordinal: 1,
                page_id: 400,
                lod_level: 1,
                submission_index: 0,
                submission_page_id: 400,
                submission_lod_level: 1,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
        ],
        "expected duplicate resident-parent fallback to keep hardware-rasterization startup order aligned with the selected-cluster seam while still overwriting only the earlier resident ancestor's submission metadata with the later child request"
    );
}

#[test]
fn seed_backed_node_and_cluster_cull_applies_cluster_budget_after_stable_selected_cluster_order_without_explicit_cluster_selections(
) {
    let (renderer, mesh) = render_seed_backed_budget_order_execution_frame(1, None);

    assert_eq!(
        renderer
            .read_last_virtual_geometry_selected_clusters()
            .expect("expected budget-ordered seed-driven execution selected clusters"),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 0,
            page_id: 300,
            lod_level: 1,
            state: RenderVirtualGeometryExecutionState::Missing,
        }],
        "expected cluster_budget clamping to happen after stable selected-cluster ordering on the root-seed baseline path, so an unsorted extract still chooses the ordinal-0 cluster"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_visbuffer64_words()
            .expect("expected budget-ordered seed-driven visbuffer words")
            .1,
        vec![RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
            Some(0),
            30,
            300,
            1,
            RenderVirtualGeometryExecutionState::Missing,
        )],
        "expected VisBuffer64 packing to reflect the post-ordering budget clamp instead of the first raw extract entry on the root-seed baseline path"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected budget-ordered seed-driven hardware-rasterization records"),
        vec![RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 0,
            page_id: 300,
            lod_level: 1,
            submission_index: 0,
            submission_page_id: 300,
            submission_lod_level: 1,
            entity_cluster_start_ordinal: 0,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 2,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state: RenderVirtualGeometryExecutionState::Missing,
        }],
        "expected hardware-rasterization startup records to see the same post-ordering budget clamp as the selected-cluster seam on the root-seed baseline path"
    );
}

#[test]
fn seed_backed_node_and_cluster_cull_derives_frontier_rank_from_unresolved_page_order_without_explicit_cluster_selections(
) {
    let (renderer, mesh) = render_seed_backed_frontier_rank_execution_frame(8, None);

    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected seed-driven frontier-rank hardware-rasterization records"),
        vec![
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 3,
                submission_index: 0,
                submission_page_id: 200,
                submission_lod_level: 3,
                entity_cluster_start_ordinal: 0,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 4,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 2,
                submission_index: 0,
                submission_page_id: 300,
                submission_lod_level: 2,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 4,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::PendingUpload,
            },
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 40,
                cluster_ordinal: 2,
                page_id: 400,
                lod_level: 1,
                submission_index: 0,
                submission_page_id: 400,
                submission_lod_level: 1,
                entity_cluster_start_ordinal: 2,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 4,
                lineage_depth: 0,
                frontier_rank: 1,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::Missing,
            },
            RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(0),
                entity: mesh,
                cluster_id: 50,
                cluster_ordinal: 3,
                page_id: 500,
                lod_level: 0,
                submission_index: 0,
                submission_page_id: 500,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 3,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 4,
                lineage_depth: 0,
                frontier_rank: 2,
                resident_slot: None,
                submission_slot: None,
                state: RenderVirtualGeometryExecutionState::PendingUpload,
            },
        ],
        "expected the seed-backed baseline execution seam to assign stable frontier_rank values from first unresolved page occurrence so downstream raster/debug consumers stop seeing every expanded cluster as rank zero before true traversal-owned frontier ordering exists"
    );
}

fn render_seed_backed_execution_frame(
    cluster_budget: u32,
    forced_mip: Option<u8>,
) -> (SceneRenderer, u64) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget,
        page_budget: 2,
        clusters: vec![
            virtual_geometry_cluster(mesh, 20, 200, 1, None, Vec3::new(0.0, 0.0, 0.0), 0.5),
            virtual_geometry_cluster(mesh, 30, 300, 0, None, Vec3::new(0.5, 0.0, 0.0), 0.25),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(200, true), page(300, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("SeedDrivenExecutionSelectionTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip,
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    (renderer, mesh)
}

fn render_seed_backed_frontier_rank_execution_frame(
    cluster_budget: u32,
    forced_mip: Option<u8>,
) -> (SceneRenderer, u64) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget,
        page_budget: 4,
        clusters: vec![
            virtual_geometry_cluster(mesh, 20, 200, 3, None, Vec3::new(0.0, 0.0, 0.0), 1.0),
            virtual_geometry_cluster(mesh, 30, 300, 2, None, Vec3::new(0.5, 0.0, 0.0), 0.75),
            virtual_geometry_cluster(mesh, 40, 400, 1, None, Vec3::new(1.0, 0.0, 0.0), 0.5),
            virtual_geometry_cluster(mesh, 50, 500, 0, None, Vec3::new(1.5, 0.0, 0.0), 0.25),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(200, true), page(300, false), page(500, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 4,
            page_offset: 0,
            page_count: 4,
            mesh_name: Some("SeedDrivenFrontierRankExecutionSelectionTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip,
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    (renderer, mesh)
}

fn render_seed_backed_parent_fallback_execution_frame(
    cluster_budget: u32,
    forced_mip: Option<u8>,
) -> (SceneRenderer, u64) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget,
        page_budget: 3,
        clusters: vec![
            virtual_geometry_cluster(mesh, 20, 200, 2, None, Vec3::new(0.0, 0.0, 0.0), 0.75),
            virtual_geometry_cluster(mesh, 30, 300, 1, Some(20), Vec3::new(0.5, 0.0, 0.0), 0.5),
            virtual_geometry_cluster(mesh, 40, 400, 0, Some(30), Vec3::new(1.0, 0.0, 0.0), 0.25),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(200, true), page(300, true), page(400, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 3,
            page_offset: 0,
            page_count: 3,
            mesh_name: Some(
                "SeedDrivenResidentParentFallbackExecutionSelectionTestMesh".to_string(),
            ),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip,
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    (renderer, mesh)
}

fn render_seed_backed_duplicate_parent_fallback_execution_frame(
    cluster_budget: u32,
    forced_mip: Option<u8>,
) -> (SceneRenderer, u64) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget,
        page_budget: 3,
        clusters: vec![
            virtual_geometry_cluster(mesh, 30, 300, 2, None, Vec3::new(0.0, 0.0, 0.0), 0.75),
            virtual_geometry_cluster(mesh, 40, 400, 1, None, Vec3::new(0.5, 0.0, 0.0), 0.5),
            virtual_geometry_cluster(mesh, 50, 500, 0, Some(30), Vec3::new(1.0, 0.0, 0.0), 0.25),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(300, true), page(400, true), page(500, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 3,
            page_offset: 0,
            page_count: 3,
            mesh_name: Some(
                "SeedDrivenDuplicateResidentParentFallbackExecutionSelectionTestMesh".to_string(),
            ),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip,
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    (renderer, mesh)
}

fn render_seed_backed_budget_order_execution_frame(
    cluster_budget: u32,
    forced_mip: Option<u8>,
) -> (SceneRenderer, u64) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget,
        page_budget: 2,
        clusters: vec![
            virtual_geometry_cluster(mesh, 40, 400, 0, None, Vec3::new(0.5, 0.0, 0.0), 0.25),
            virtual_geometry_cluster(mesh, 30, 300, 1, None, Vec3::new(0.0, 0.0, 0.0), 0.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("SeedDrivenBudgetOrderExecutionSelectionTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip,
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    (renderer, mesh)
}

fn render_seed_backed_subset_execution_frame(
    cluster_budget: u32,
    forced_mip: Option<u8>,
) -> (SceneRenderer, u64) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget,
        page_budget: 2,
        clusters: vec![
            virtual_geometry_cluster(mesh, 30, 300, 0, None, Vec3::new(0.5, 0.0, 0.0), 0.25),
            virtual_geometry_cluster(mesh, 20, 200, 1, None, Vec3::new(0.0, 0.0, 0.0), 0.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(200, true), page(300, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 1,
            cluster_count: 1,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("SeedDrivenSubsetExecutionSelectionTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip,
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    (renderer, mesh)
}

fn render_seed_backed_hierarchical_execution_frame(
    cluster_budget: u32,
    forced_mip: Option<u8>,
) -> (SceneRenderer, u64) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget,
        page_budget: 3,
        clusters: vec![
            virtual_geometry_cluster(mesh, 20, 200, 2, None, Vec3::new(0.0, 0.0, 0.0), 0.75),
            virtual_geometry_cluster(mesh, 30, 300, 1, Some(20), Vec3::new(0.5, 0.0, 0.0), 0.5),
            virtual_geometry_cluster(mesh, 40, 400, 0, Some(30), Vec3::new(1.0, 0.0, 0.0), 0.25),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(200, false), page(300, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 3,
            page_offset: 0,
            page_count: 3,
            mesh_name: Some("SeedDrivenHierarchicalExecutionSelectionTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip,
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    (renderer, mesh)
}

fn page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: if resident { 2048 } else { 4096 },
    }
}

fn virtual_geometry_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    parent_cluster_id: Option<u32>,
    bounds_center: Vec3,
    screen_space_error: f32,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level,
        parent_cluster_id,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}
