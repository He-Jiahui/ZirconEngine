use super::*;
use zircon_runtime::asset::{
    AssetUri, ModelPrimitiveAsset, VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset,
    VirtualGeometryClusterPageHeaderAsset, VirtualGeometryDebugMetadataAsset,
    VirtualGeometryHierarchyNodeAsset, VirtualGeometryPageDependencyAsset,
    VirtualGeometryRootClusterRangeAsset,
};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, RenderLayerSet,
    RenderMeshStaticState, RendererCommon,
};
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::graphics::VirtualGeometryGpuCompletion;
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

#[test]
fn provider_builds_neutral_extract_output_from_cooked_model_meshes() {
    let provider = PluginVirtualGeometryRuntimeProvider;
    let model_id = ResourceId::from_stable_label("res://models/provider-vg.model.toml");
    let material_id = ResourceId::from_stable_label("builtin://material/default");
    let model = cooked_model_asset();
    let node_id = 44;
    let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let mesh = RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(model_id),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::from_transform_static(false),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            is_static: false,
            ..RendererCommon::default()
        },
    };
    let mut load_model = |requested_id| (requested_id == model_id).then(|| model.clone());

    let output = provider
        .build_extract_from_meshes(
            &[mesh],
            Some(RenderVirtualGeometryDebugState {
                forced_mip: Some(10),
                visualize_bvh: true,
                ..RenderVirtualGeometryDebugState::default()
            }),
            &mut load_model,
        )
        .expect("provider should build automatic VG output from cooked model data");

    assert_eq!(output.extract().instances.len(), 1);
    assert_eq!(output.extract().instances[0].source_model, Some(model_id));
    assert_eq!(output.extract().debug.forced_mip, Some(10));
    assert_eq!(output.extract().pages.len(), 1);
    assert_eq!(output.extract().page_dependencies.len(), 1);
    assert!(!output.cpu_reference_instances().is_empty());
    assert!(!output.bvh_visualization_instances().is_empty());
    assert!(output.resident_page_payloads().is_empty());
}

#[test]
fn provider_confirms_only_pending_gpu_page_assignments() {
    let completion = VirtualGeometryGpuCompletion::new(
        vec![(30, 3), (20, 2)],
        vec![(30, 3), (99, 9)],
        Vec::new(),
    );

    let confirmed = confirmed_virtual_geometry_completion(&completion, [], [30]);

    assert_eq!(confirmed.page_table_entries(), &[(20, 2), (30, 3)]);
    assert_eq!(confirmed.completed_page_assignments(), &[(30, 3)]);
    assert!(confirmed.completed_page_replacements().is_empty());
}

#[test]
fn provider_reports_the_page_replaced_by_an_implicitly_reused_slot() {
    let completion =
        VirtualGeometryGpuCompletion::new(vec![(30, 3), (20, 2)], vec![(30, 99)], Vec::new());

    let confirmed = confirmed_virtual_geometry_completion(&completion, [(3, 10)], [30]);

    assert_eq!(confirmed.page_table_entries(), &[(20, 2), (30, 3)]);
    assert_eq!(confirmed.completed_page_assignments(), &[(30, 3)]);
    assert_eq!(confirmed.completed_page_replacements(), &[(30, 10)]);
}

#[test]
fn provider_normalizes_stale_and_conflicting_gpu_page_table_entries() {
    let completion = VirtualGeometryGpuCompletion::new(
        vec![(10, 0), (20, 1), (30, 0), (40, 2)],
        vec![(30, 0)],
        Vec::new(),
    );

    let confirmed = confirmed_virtual_geometry_completion(&completion, [], [30]);

    assert_eq!(confirmed.page_table_entries(), &[(30, 0), (20, 1), (40, 2)]);
    assert_eq!(confirmed.completed_page_assignments(), &[(30, 0)]);
}

#[test]
fn provider_fallback_retains_hot_frontier_until_exact_cooldown_expiry() {
    assert_provider_hot_frontier_lifetime(false);
}

#[test]
fn provider_gpu_completion_retains_hot_frontier_until_exact_cooldown_expiry() {
    assert_provider_hot_frontier_lifetime(true);
}

fn assert_provider_hot_frontier_lifetime(with_gpu_completion: bool) {
    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    };
    use zircon_runtime::graphics::VisibilityVirtualGeometryFeedback;

    // Probe a fresh provider after each quiet-frame count so eviction itself cannot
    // shorten a later observation. Page 10 wins the cold tie, but loses while hot.
    for quiet_frames in 0..=3_u64 {
        let mut provider = PluginVirtualGeometryRuntimeState::default();
        provider
            .state
            .register_extract(Some(&RenderVirtualGeometryExtract {
                page_budget: 2,
                pages: vec![
                    RenderVirtualGeometryPage {
                        page_id: 10,
                        resident: true,
                        size_bytes: 16,
                    },
                    RenderVirtualGeometryPage {
                        page_id: 20,
                        resident: true,
                        size_bytes: 16,
                    },
                    RenderVirtualGeometryPage {
                        page_id: 30,
                        resident: false,
                        size_bytes: 16,
                    },
                ],
                ..Default::default()
            }));
        advance_provider_hot_frontier(&mut provider, vec![10], 1, with_gpu_completion);
        for frame in 0..quiet_frames {
            advance_provider_hot_frontier(
                &mut provider,
                Vec::new(),
                frame + 2,
                with_gpu_completion,
            );
        }
        assert_eq!(provider.state.resident_slot_owners().len(), 2);
        assert!(provider.state.pending_page_ids().is_empty());

        // Stage a real pending request, then let the provider's fallback completion
        // choose the victim on the next tick. No test-only eviction choice is supplied.
        provider.state.ingest_page_requests(quiet_frames + 2, [30]);
        let feedback = VirtualGeometryRuntimeFeedback::new(
            None,
            Vec::new(),
            Some(VisibilityVirtualGeometryFeedback {
                requested_pages: vec![30],
                evictable_pages: vec![10, 20],
                ..Default::default()
            }),
            quiet_frames + 2,
        );
        provider.update_after_render(feedback);
        let mut resident_pages = provider
            .state
            .resident_slot_owners()
            .into_iter()
            .map(|(_, page_id)| page_id)
            .collect::<Vec<_>>();
        resident_pages.sort_unstable();
        let expected_pages = if quiet_frames < 2 {
            vec![10, 30]
        } else {
            vec![20, 30]
        };
        assert_eq!(
            resident_pages, expected_pages,
            "hot protection must last exactly two cooling ticks: gpu_completion={with_gpu_completion}, quiet_frames={quiet_frames}"
        );
        assert!(provider.state.pending_page_ids().is_empty());
    }
}

fn advance_provider_hot_frontier(
    provider: &mut PluginVirtualGeometryRuntimeState,
    hot_resident_pages: Vec<u32>,
    generation: u64,
    with_gpu_completion: bool,
) {
    use zircon_runtime::graphics::VisibilityVirtualGeometryFeedback;

    // A completion carries the actual current slot table, preserving resident pages
    // while exercising the provider's confirmed-completion path without GPU claims.
    let completion = with_gpu_completion.then(|| {
        VirtualGeometryGpuCompletion::new(
            provider
                .state
                .resident_slot_owners()
                .into_iter()
                .map(|(slot, page_id)| (page_id, slot))
                .collect(),
            Vec::new(),
            Vec::new(),
        )
    });
    provider.update_after_render(VirtualGeometryRuntimeFeedback::new(
        completion,
        Vec::new(),
        Some(VisibilityVirtualGeometryFeedback {
            hot_resident_pages,
            ..Default::default()
        }),
        generation,
    ));
}

fn cooked_model_asset() -> ModelAsset {
    ModelAsset {
        uri: AssetUri::parse("res://models/provider-vg.model.toml")
            .expect("model uri should parse"),
        primitives: vec![ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: Some(VirtualGeometryAsset {
                hierarchy_buffer: vec![VirtualGeometryHierarchyNodeAsset {
                    node_id: 0,
                    parent_node_id: None,
                    child_node_ids: Vec::new(),
                    cluster_start: 0,
                    cluster_count: 1,
                    page_id: 10,
                    mip_level: 10,
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 1.0,
                    screen_space_error: 0.25,
                }],
                cluster_headers: vec![VirtualGeometryClusterHeaderAsset {
                    cluster_id: 7,
                    page_id: 10,
                    hierarchy_node_id: 0,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 1.0,
                    screen_space_error: 0.25,
                }],
                cluster_page_headers: vec![VirtualGeometryClusterPageHeaderAsset {
                    page_id: 10,
                    start_offset: 0,
                    payload_size_bytes: 64,
                }],
                cluster_page_data: vec![vec![0; 64]],
                root_page_table: vec![10],
                page_dependencies: vec![VirtualGeometryPageDependencyAsset {
                    page_id: 10,
                    parent_page_id: None,
                    child_page_ids: Vec::new(),
                }],
                root_cluster_ranges: vec![VirtualGeometryRootClusterRangeAsset {
                    node_id: 0,
                    cluster_start: 0,
                    cluster_count: 1,
                }],
                debug: VirtualGeometryDebugMetadataAsset {
                    mesh_name: Some("ProviderCookedMesh".to_string()),
                    source_hint: Some("provider-test".to_string()),
                    notes: Vec::new(),
                },
            }),
        }],
    }
}
