//! Regressions for observable virtual-geometry submission behavior through WGPU.
//!
//! Orphaned tests that depended on the removed `SceneRenderer` prepare injection or
//! private buffer-drop/readback hooks are deliberately not represented as product tests.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::assets::{AlphaMode, MaterialAsset};
use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderFramework, RenderLayerSet, RenderMeshSnapshot, RenderOverlayExtract,
    RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderSubmissionConfig,
    RenderViewportDescriptor, RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage, RenderWorldSnapshotHandle,
    RendererCommon, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use super::virtual_geometry_render_framework_stats as render_fixture;
use crate::test_support::render_feature_fixtures::{
    pluginized_wgpu_render_framework_with_asset_manager,
    pluginized_wgpu_render_framework_with_uploader_collector, IndirectReadbackWords,
    PluginizedWgpuRenderFrameworkFixture,
};
use crate::virtual_geometry::VirtualGeometryUploaderTestCollector;

const VIEWPORT_SIZE: UVec2 = UVec2::new(160, 120);

struct ProductFrame {
    rgba: Vec<u8>,
    debug: zircon_runtime::core::framework::render::RenderVirtualGeometryDebugSnapshot,
    stats: zircon_runtime::core::framework::render::RenderStats,
    indirect: IndirectReadbackWords,
    terminal_authority: Vec<[u32; 14]>,
    terminal_records: Vec<(u32, u64, u32, u32, u32)>,
}

fn render(extract: RenderFrameExtract, assets: Arc<ProjectAssetManager>) -> ProductFrame {
    let framework = pluginized_wgpu_render_framework_with_uploader_collector(
        assets,
        Arc::new(VirtualGeometryUploaderTestCollector::default()),
    );
    render_with_fixture(extract, framework)
}

fn render_with_fixture(
    extract: RenderFrameExtract,
    mut framework: PluginizedWgpuRenderFrameworkFixture,
) -> ProductFrame {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .unwrap();
    framework
        .set_quality_profile(viewport, virtual_geometry_only_quality_profile())
        .unwrap();
    framework.submit_frame_extract(viewport, extract).unwrap();
    let stats = framework.query_stats().unwrap();
    let debug = framework
        .query_virtual_geometry_debug_snapshot()
        .unwrap()
        .expect("public WGPU virtual-geometry snapshot");
    let rgba = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("public WGPU frame capture")
        .rgba;
    let indirect = framework
        .read_indirect_words_after_submit()
        .expect("actual indirect args and authority GPU readback");
    let terminal_authority = framework
        .read_last_virtual_geometry_terminal_authority_rows()
        .expect("actual same-frame terminal authority");
    let terminal_records = framework
        .read_last_virtual_geometry_indirect_execution_records()
        .expect("actual same-scope physical execution records");
    framework.destroy_viewport(viewport).unwrap();
    ProductFrame {
        rgba,
        debug,
        stats,
        indirect,
        terminal_authority,
        terminal_records,
    }
}

fn virtual_geometry_only_quality_profile() -> RenderQualityProfile {
    render_fixture::virtual_geometry_only_quality_profile()
}

fn basic_assets() -> Arc<ProjectAssetManager> {
    Arc::new(ProjectAssetManager::default())
}

fn dual_extract() -> RenderFrameExtract {
    let assets = ProjectAssetManager::default();
    let model = resource_handle::<ModelMarker>(&assets, "builtin://cube");
    let material = resource_handle::<MaterialMarker>(&assets, "builtin://material/default");
    render_fixture::build_same_page_dual_entity_extract(VIEWPORT_SIZE, model, material)
}

#[test]
fn public_wgpu_submission_records_follow_visible_entity_order() {
    let result = render(dual_extract(), basic_assets());
    let entities = result
        .debug
        .submission_order
        .iter()
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();
    assert_eq!(entities, vec![2, 3]);
    assert_eq!(result.debug.submission_records.len(), entities.len());
    assert!(!result.rgba.is_empty());
}

#[test]
fn public_wgpu_selected_replay_args_preserve_first_instance_indices() {
    let assets = basic_assets();
    let extract = dual_extract();
    let framework = pluginized_wgpu_render_framework_with_asset_manager(Arc::clone(&assets));
    framework
        .set_submission_config(
            RenderSubmissionConfig::synchronous().with_hzb_diagnostics_readback(),
        )
        .unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .unwrap();
    framework
        .set_quality_profile(viewport, virtual_geometry_only_quality_profile())
        .unwrap();
    framework
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    framework.submit_frame_extract(viewport, extract).unwrap();

    let stats = framework.query_stats().unwrap();
    assert!(stats.last_hzb_occlusion_indirect_args_readback_available);
    assert!(!stats.last_hzb_occlusion_first_instances_truncated);
    let first_instance_count = usize::from(stats.last_hzb_occlusion_first_instance_count);
    let first_instances = &stats.last_hzb_occlusion_first_instances[..first_instance_count];
    assert!(first_instances.len() >= 2, "{first_instances:?}");
    assert_eq!(first_instances.len() % 2, 0, "{first_instances:?}");
    for phase_args in first_instances.chunks_exact(2) {
        assert_eq!(phase_args, [0, 1]);
    }

    framework.destroy_viewport(viewport).unwrap();
}

#[test]
fn public_wgpu_execution_segments_preserve_submission_tokens() {
    let result = render(dual_extract(), basic_assets());
    let tokens = result
        .debug
        .submission_records
        .iter()
        .map(|record| record.submission_index)
        .collect::<Vec<_>>();
    assert_eq!(tokens, vec![0, 1]);
    for (segment, record) in result
        .debug
        .execution_segments
        .iter()
        .zip(&result.debug.submission_records)
    {
        assert_eq!(segment.submission_index, Some(record.submission_index));
        assert_eq!(segment.entity, record.entity);
    }
    assert_eq!(
        result.debug.execution_segment_count as usize,
        result.stats.last_virtual_geometry_execution_segment_count
    );
}

#[test]
fn public_wgpu_hierarchy_reports_child_lineage() {
    let assets = ProjectAssetManager::default();
    let model = resource_handle::<ModelMarker>(&assets, "builtin://cube");
    let material = resource_handle::<MaterialMarker>(&assets, "builtin://material/default");
    let extract =
        render_fixture::build_hierarchical_instance_extract(VIEWPORT_SIZE, model, material);
    let result = render(extract, Arc::new(assets));
    assert!(result
        .debug
        .execution_segments
        .iter()
        .any(|segment| segment.lineage_depth > 0));
    assert!(!result.rgba.is_empty());
}

#[test]
fn virtual_geometry_unified_indirect_keeps_lineage_depth_in_gpu_submission_and_indirect_args() {
    let assets = Arc::new(ProjectAssetManager::default());
    let fixture = CookedHierarchyFixture::new(&assets);
    let result = render(fixture.extract(&assets), Arc::clone(&assets));

    let authority_records = result
        .indirect
        .authority
        .chunks_exact(14)
        .take(result.debug.execution_segments.len())
        .collect::<Vec<_>>();
    assert!(!authority_records.is_empty());
    let mut observed_lineage = false;
    for (slot, authority) in authority_records.iter().enumerate() {
        let segment = result
            .debug
            .execution_segments
            .iter()
            .find(|segment| {
                segment.original_index == authority[7]
                    && segment.entity as u32 == authority[1]
                    && result.debug.selected_clusters.iter().any(|selected| {
                        let end = segment
                            .cluster_start_ordinal
                            .checked_add(segment.cluster_span_count)
                            .expect("execution segment cluster span must not overflow");
                        selected.instance_index == segment.instance_index
                            && selected.entity == segment.entity
                            && selected.page_id == segment.page_id
                            && selected.cluster_id == authority[4]
                            && (segment.cluster_start_ordinal..end)
                                .contains(&selected.cluster_ordinal)
                    })
            })
            .expect("GPU authority must retain its source execution segment");
        assert_eq!(authority[12], segment.lineage_depth);
        let args = &result.indirect.args[1 + slot * 4..1 + slot * 4 + 4];
        assert_eq!(args[0] > 0, args[1] == 1);
        assert_eq!(
            args[3],
            (authority[8].min(0xffff) << 16) | authority[9].min(0xffff)
        );
        observed_lineage |= segment.lineage_depth > 0;
    }
    assert!(
        observed_lineage,
        "expected a hierarchy-derived GPU lineage value"
    );

    drop(fixture);
}

#[test]
fn public_wgpu_pending_page_exposes_request_frontier() {
    let assets = ProjectAssetManager::default();
    let mut extract = render_fixture::build_single_entity_extract(
        VIEWPORT_SIZE,
        resource_handle::<ModelMarker>(&assets, "builtin://cube"),
        resource_handle::<MaterialMarker>(&assets, "builtin://material/default"),
    );
    let virtual_geometry = extract.geometry.virtual_geometry.as_mut().unwrap();
    virtual_geometry.pages[0].resident = false;
    let expected_page = virtual_geometry.pages[0].page_id;
    let result = render(extract, Arc::new(assets));
    assert!(result
        .debug
        .pending_page_request_inspections
        .iter()
        .any(|request| request.page_id == expected_page));
}

#[test]
fn public_wgpu_transparent_order_controls_captured_center_pixel() {
    let (root, assets, model, red, blue) = transparent_project("transparent_order");
    let forward = render(
        transparent_extract(model.clone(), red.clone(), blue.clone(), false),
        Arc::clone(&assets),
    );
    let reverse = render(
        transparent_extract(model, red, blue, true),
        Arc::clone(&assets),
    );
    let forward_center = center_pixel(&forward.rgba);
    let reverse_center = center_pixel(&reverse.rgba);
    assert!(forward_center[2] > forward_center[0], "{forward_center:?}");
    assert!(reverse_center[0] > reverse_center[2], "{reverse_center:?}");
    assert_eq!(
        forward
            .debug
            .submission_order
            .iter()
            .map(|entry| entry.entity)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert_eq!(
        reverse
            .debug
            .submission_order
            .iter()
            .map(|entry| entry.entity)
            .collect::<Vec<_>>(),
        vec![3, 2]
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn public_wgpu_multi_primitive_draws_have_distinct_submission_tokens() {
    let (root, assets, model, red, blue) = transparent_project("multi_primitive_tokens");
    let result = render(
        transparent_extract(model, red, blue, false),
        Arc::clone(&assets),
    );
    assert_eq!(result.debug.submission_records.len(), 4);
    assert_eq!(result.terminal_records.len(), 4);
    assert_eq!(result.terminal_authority.len(), 4);
    for (slot, record) in result.debug.submission_records.iter().enumerate() {
        let gpu = result.terminal_records[slot];
        assert_eq!(
            (
                record.draw_ref_index.unwrap(),
                record.entity,
                record.page_id,
                record.submission_index,
                record.draw_ref_rank
            ),
            gpu
        );
        let authority = &result.terminal_authority[slot];
        let segment = &result.debug.execution_segments[slot];
        assert_eq!(authority[0], record.instance_index.unwrap());
        assert_eq!(
            u64::from(authority[1]) | (u64::from(authority[2]) << 32),
            record.entity
        );
        assert_eq!(authority[3], record.page_id);
        assert_eq!(authority[6], record.draw_ref_index.unwrap());
        assert_eq!(authority[7], record.original_index);
        assert_eq!(authority[8], record.submission_index);
        assert_eq!(authority[9], record.draw_ref_rank);
        assert_eq!(authority[10], segment.submission_slot.unwrap());
        assert_eq!(authority[12], segment.lineage_depth);
        assert_eq!(authority[13], segment.frontier_rank);
    }
    assert!(result.stats.last_virtual_geometry_indirect_draw_count > 0);
    let mut records_by_submission = result.debug.submission_records.iter().collect::<Vec<_>>();
    records_by_submission.sort_by_key(|record| record.submission_index);
    assert_eq!(
        records_by_submission
            .iter()
            .map(|record| (record.submission_index, record.draw_ref_index))
            .collect::<Vec<_>>(),
        {
            let mut rows = result
                .terminal_records
                .iter()
                .map(|record| (record.3, Some(record.0)))
                .collect::<Vec<_>>();
            rows.sort_by_key(|row| row.0);
            rows
        },
        "multi-primitive draws must retain distinct compacted draw-ref indices in submission order"
    );
    let mut segments_by_submission = result.debug.execution_segments.iter().collect::<Vec<_>>();
    segments_by_submission.sort_by_key(|segment| segment.submission_index);
    assert_eq!(
        segments_by_submission
            .iter()
            .map(|segment| (segment.submission_index, segment.draw_ref_index))
            .collect::<Vec<_>>(),
        records_by_submission
            .iter()
            .map(|record| (
                Some(record.submission_index),
                record.draw_ref_index.unwrap()
            ))
            .collect::<Vec<_>>(),
        "execution segments must carry the same GPU-authoritative compacted draw-ref indices as submission records"
    );
    for entity in [2, 3] {
        let mut records = result
            .debug
            .submission_records
            .iter()
            .filter(|record| record.entity == entity)
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.draw_ref_rank);
        assert_eq!(
            records
                .iter()
                .map(|record| record.draw_ref_rank)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }
    let mut tokens = result
        .debug
        .submission_records
        .iter()
        .map(|record| {
            (record.submission_index.min(0xffff) << 16) | record.draw_ref_rank.min(0xffff)
        })
        .collect::<Vec<_>>();
    tokens.sort_unstable();
    tokens.dedup();
    assert_eq!(
        tokens,
        vec![0, 1, 65536, 65537],
        "each primitive draw needs its own token"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_framework_stats_follow_actual_virtual_geometry_gpu_submission_for_multi_primitive_model()
{
    let (root, assets, model, material, _) = transparent_project("unified_indirect_stats");
    let extract = render_fixture::build_single_entity_extract(VIEWPORT_SIZE, model, material);
    let result = render(extract, Arc::clone(&assets));

    assert_eq!(result.stats.last_virtual_geometry_indirect_draw_count, 2);
    assert_eq!(result.stats.last_virtual_geometry_indirect_segment_count, 1);
    assert_eq!(result.stats.last_virtual_geometry_indirect_args_count, 2);
    assert_eq!(result.stats.last_virtual_geometry_indirect_buffer_count, 1);
    assert_eq!(result.indirect.args.first(), Some(&2));
    assert!(result.indirect.args.len() >= 1 + 2 * 4);
    assert!(result.indirect.args[1..1 + 2 * 4]
        .chunks_exact(4)
        .all(|args| args[0] > 0 && args[1] == 1));
    assert!(result.indirect.authority.len() >= 2 * 14);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_prepare_keeps_one_visibility_owned_segment_but_distinct_gpu_args_for_multi_primitive_model(
) {
    let (root, assets, model, material, _) = transparent_project("unified_indirect_two_args");
    let extract = render_fixture::build_single_entity_extract(VIEWPORT_SIZE, model, material);
    let result = render(extract, Arc::clone(&assets));

    assert_eq!(result.debug.execution_segment_count, 1);
    assert_eq!(result.indirect.args.first(), Some(&2));
    let args = &result.indirect.args[1..1 + 2 * 4];
    assert_eq!(args.len(), 8);
    assert!(args
        .chunks_exact(4)
        .all(|draw| draw[0] == 3 && draw[1] == 1));
    let first_instances = args.chunks_exact(4).map(|draw| draw[3]).collect::<Vec<_>>();
    assert_eq!(first_instances, vec![0, 1]);

    let authority = &result.indirect.authority[..2 * 14];
    assert_eq!(authority[9], 0);
    assert_eq!(authority[14 + 9], 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_unified_indirect_propagates_multi_primitive_draw_ref_compaction_into_gpu_args()
{
    let (root, assets, model, material, _) = transparent_project("unified_indirect_compaction");
    let extract = render_fixture::build_single_entity_extract(VIEWPORT_SIZE, model, material);
    let result = render(extract, Arc::clone(&assets));

    assert_eq!(result.debug.execution_segment_count, 1);
    assert_eq!(result.indirect.args.first(), Some(&2));
    let args = &result.indirect.args[1..1 + 2 * 4];
    assert_eq!(
        args.chunks_exact(4)
            .map(|draw| (draw[2], draw[0]))
            .collect::<Vec<_>>(),
        vec![(0, 3), (3, 3)],
        "GPU draw args must retain the compacted per-primitive vertex ranges"
    );
    let authority = result.indirect.authority[..2 * 14]
        .chunks_exact(14)
        .collect::<Vec<_>>();
    assert_eq!(
        authority.iter().map(|record| record[6]).collect::<Vec<_>>(),
        vec![3, 6],
        "GPU authority must preserve the compacted primitive draw-ref indices"
    );
    assert_eq!(
        authority.iter().map(|record| record[7]).collect::<Vec<_>>(),
        vec![0, 0],
        "both draw refs must remain attached to the same execution segment"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn virtual_geometry_renderer_submission_records_preserve_draw_level_tokens_for_repeated_primitives()
{
    let (root, assets, model, material, _) = transparent_project("unified_indirect_draw_tokens");
    let extract = render_fixture::build_single_entity_extract(VIEWPORT_SIZE, model, material);
    let result = render(extract, Arc::clone(&assets));

    assert_eq!(result.debug.submission_records.len(), 2);
    assert_eq!(result.debug.execution_segments.len(), 2);
    let tokens = result
        .debug
        .submission_records
        .iter()
        .map(|record| {
            (record.submission_index.min(0xffff) << 16) | record.draw_ref_rank.min(0xffff)
        })
        .collect::<Vec<_>>();
    assert_eq!(tokens, vec![0, 1]);
    assert_eq!(
        result
            .debug
            .submission_records
            .iter()
            .map(|record| {
                (
                    record.submission_index,
                    record.draw_ref_index,
                    record.draw_ref_rank,
                )
            })
            .collect::<Vec<_>>(),
        vec![(0, Some(3), 0), (0, Some(6), 1)]
    );
    for (slot, record) in result.debug.submission_records.iter().enumerate() {
        let authority = &result.terminal_authority[slot];
        let args = &result.indirect.args[1 + slot * 4..1 + slot * 4 + 4];
        let gpu = result.terminal_records[slot];
        assert_eq!(
            (
                record.draw_ref_index.unwrap(),
                record.entity,
                record.page_id,
                record.submission_index,
                record.draw_ref_rank
            ),
            gpu
        );
        let segment = &result.debug.execution_segments[slot];
        assert_eq!(record.instance_index, segment.instance_index);
        assert_eq!(record.entity, segment.entity);
        assert_eq!(record.page_id, segment.page_id);
        assert_eq!(record.original_index, segment.original_index);
        assert_eq!(record.draw_ref_index, Some(segment.draw_ref_index));
        assert_eq!(record.draw_ref_rank, segment.draw_ref_rank.unwrap());
        assert_eq!(authority[0], record.instance_index.unwrap());
        assert_eq!(
            u64::from(authority[1]) | (u64::from(authority[2]) << 32),
            record.entity
        );
        assert_eq!(authority[3], record.page_id);
        assert_eq!(authority[7], record.original_index);
        assert_eq!(authority[9], record.draw_ref_rank);
        assert_eq!(authority[10], segment.submission_slot.unwrap());
        assert_eq!(authority[12], segment.lineage_depth);
        assert_eq!(authority[13], segment.frontier_rank);
        assert_eq!(authority[8], record.submission_index);
        assert_eq!(authority[6], record.draw_ref_index.unwrap());
        assert_eq!(
            args[3],
            (authority[8].min(0xffff) << 16) | authority[9].min(0xffff)
        );
    }

    let _ = fs::remove_dir_all(root);
}

// A single real cooked primitive owns both hierarchy leaves. The production
// provider projects its imported IDs/pages; no authored row substitutes geometry.
struct CookedHierarchyFixture {
    root: PathBuf,
}

impl CookedHierarchyFixture {
    fn new(assets: &ProjectAssetManager) -> Self {
        let cube = resource_handle::<ModelMarker>(assets, "builtin://cube");
        let source = assets.load_model_asset(cube.id()).unwrap();
        assert_eq!(source.primitives.len(), 1);
        let mut primitive = source.primitives.into_iter().next().unwrap();
        assert_eq!(primitive.indices.len() % 3, 0);
        let triangle_count = primitive.indices.len() / 3;
        assert!(triangle_count >= 2 && triangle_count % 2 == 0);
        // Preserve every original vertex/index; split the real source in half.
        let cooked = zircon_runtime::asset::cook_virtual_geometry_from_mesh(
            &primitive.vertices,
            &primitive.indices,
            zircon_runtime::asset::VirtualGeometryCookConfig {
                cluster_triangle_count: triangle_count / 2,
                page_cluster_count: 1,
                mesh_name: Some("product-lineage-cube".to_owned()),
                source_hint: Some("res://models/lineage.model.toml".to_owned()),
            },
        )
        .expect("original cube must cook a genuine parent and leaf partition");
        assert!(cooked
            .cluster_headers
            .iter()
            .any(|row| row.parent_cluster_id.is_some()));
        let leaves = cooked
            .cluster_headers
            .iter()
            .filter(|row| {
                !cooked
                    .cluster_headers
                    .iter()
                    .any(|child| child.parent_cluster_id == Some(row.cluster_id))
            })
            .count();
        assert_eq!(
            leaves, 2,
            "source halves must become two actual cooked leaves"
        );
        let expected_cook = cooked.clone();
        primitive.mesh = None;
        primitive.mesh_sdf = None;
        primitive.virtual_geometry = Some(cooked);
        let root = unique_temp_project_root("vg_real_cooked_lineage");
        fs::create_dir(&root).expect("lineage project must be exclusively task-owned");
        let fixture = Self { root };
        let paths = ProjectPaths::from_root(&fixture.root).unwrap();
        paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
        ProjectManifest::new(
            "VirtualGeometryRealCookedLineage",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        let uri = AssetUri::parse("res://models/lineage.model.toml").unwrap();
        let model = zircon_runtime::asset::ModelAsset {
            uri,
            primitives: vec![primitive],
        };
        let document = model
            .to_project_toml_string(|_| panic!("lineage model has no external asset references"))
            .unwrap();
        let path = paths
            .asset_root(&RelPath::project_assets())
            .join("models/lineage.model.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, document).unwrap();
        assets
            .open_project(fixture.root.to_string_lossy().as_ref())
            .unwrap();
        ProjectManager::open(&fixture.root)
            .unwrap()
            .scan_and_import()
            .unwrap();
        let imported = assets
            .load_model_asset(
                resource_handle::<ModelMarker>(assets, "res://models/lineage.model.toml").id(),
            )
            .unwrap();
        let imported_cook = imported.primitives[0].virtual_geometry.as_ref().unwrap();
        assert_eq!(
            imported_cook, &expected_cook,
            "import must retain the complete cooked partition"
        );
        assert_eq!(imported.primitives.len(), 1);
        assert!(imported_cook
            .cluster_headers
            .iter()
            .any(|row| row.parent_cluster_id.is_some()));
        fixture
    }

    fn extract(&self, assets: &ProjectAssetManager) -> RenderFrameExtract {
        let mut extract = render_fixture::build_single_entity_extract(
            VIEWPORT_SIZE,
            resource_handle::<ModelMarker>(assets, "res://models/lineage.model.toml"),
            resource_handle::<MaterialMarker>(assets, "builtin://material/default"),
        );
        // Automatic production extraction retains the actual imported hierarchy,
        // payloads and exact primitive admissions, including non-leaf ranges.
        extract.geometry.virtual_geometry = None;
        extract
    }
}

impl Drop for CookedHierarchyFixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.root) {
            eprintln!(
                "lineage fixture remains at {}: {error}",
                self.root.display()
            );
            if !std::thread::panicking() {
                panic!(
                    "lineage fixture cleanup failed at {}: {error}",
                    self.root.display()
                );
            }
        }
    }
}

fn transparent_project(
    label: &str,
) -> (
    PathBuf,
    Arc<ProjectAssetManager>,
    ResourceHandle<ModelMarker>,
    ResourceHandle<MaterialMarker>,
    ResourceHandle<MaterialMarker>,
) {
    let root = unique_temp_project_root(label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "VirtualGeometryPublicTransparentOrder",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let assets_root = paths.asset_root(&RelPath::project_assets());
    write_flat_color_wgsl(assets_root.join("shaders/flat_color.wgsl"));
    write_solid_png(assets_root.join("textures/white.png"));
    write_two_primitive_gltf(assets_root.join("models/two_primitives.gltf"));
    write_material(
        assets_root.join("materials/red.zmaterial"),
        "PublicOrderRed",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [1.0, 0.0, 0.0, 0.65],
    );
    write_material(
        assets_root.join("materials/blue.zmaterial"),
        "PublicOrderBlue",
        "res://shaders/flat_color.wgsl",
        "res://textures/white.png",
        [0.0, 0.0, 1.0, 0.65],
    );
    let assets = Arc::new(ProjectAssetManager::default());
    assets
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    ProjectManager::open(&root)
        .unwrap()
        .scan_and_import()
        .unwrap();
    let model = resource_handle::<ModelMarker>(&assets, "res://models/two_primitives.gltf");
    let red = resource_handle::<MaterialMarker>(&assets, "res://materials/red.zmaterial");
    let blue = resource_handle::<MaterialMarker>(&assets, "res://materials/blue.zmaterial");
    (root, assets, model, red, blue)
}

fn transparent_extract(
    model: ResourceHandle<ModelMarker>,
    red: ResourceHandle<MaterialMarker>,
    blue: ResourceHandle<MaterialMarker>,
    reverse: bool,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Orthographic,
        ortho_size: 1.2,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(VIEWPORT_SIZE);
    let mesh = |entity, material| RenderMeshSnapshot {
        node_id: entity,
        stable_instance_key: entity << 16,
        transform_revision: 0,
        transform: Transform {
            scale: Vec3::new(0.8, 0.8, 1.0),
            ..Transform::default()
        },
        model: model.clone(),
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            is_static: false,
            ..RendererCommon::default()
        },
    };
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![mesh(2, red), mesh(3, blue)],
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: Default::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(91), snapshot);
    let clusters = if reverse {
        vec![cluster(3, 30, 301), cluster(2, 20, 300)]
    } else {
        vec![cluster(2, 20, 300), cluster(3, 30, 301)]
    };
    let (red_offset, blue_offset) = if reverse { (1, 0) } else { (0, 1) };
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters,
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(300), page(301)],
        page_dependencies: Vec::new(),
        instances: vec![instance(2, red_offset, 300), instance(3, blue_offset, 301)],
        debug: Default::default(),
    });
    extract
}

fn cluster(entity: u64, cluster_id: u32, page_id: u32) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level: 0,
        parent_cluster_id: None,
        bounds_center: Vec3::ZERO,
        bounds_radius: 1.0,
        screen_space_error: 1.0,
    }
}

fn page(page_id: u32) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident: true,
        size_bytes: 4096,
    }
}

fn instance(entity: u64, cluster_offset: u32, page_id: u32) -> RenderVirtualGeometryInstance {
    RenderVirtualGeometryInstance {
        entity,
        stable_instance_key: 0,
        source_model: None,
        transform: Transform {
            scale: Vec3::new(0.8, 0.8, 1.0),
            ..Transform::default()
        },
        cluster_offset,
        cluster_count: 1,
        page_offset: page_id - 300,
        page_count: 1,
        mesh_name: Some(format!("product-order-{entity}")),
        source_hint: Some("public-wgpu-product-order".to_string()),
    }
}

fn center_pixel(rgba: &[u8]) -> [u8; 4] {
    let index = ((VIEWPORT_SIZE.y / 2 * VIEWPORT_SIZE.x + VIEWPORT_SIZE.x / 2) * 4) as usize;
    [
        rgba[index],
        rgba[index + 1],
        rgba[index + 2],
        rgba[index + 3],
    ]
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_{label}_{unique}"))
}

fn write_flat_color_wgsl(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        r#"
struct SceneUniform { view_proj: mat4x4<f32>, light_dir: vec4<f32>, light_color: vec4<f32>, ambient_color: vec4<f32>, };
struct ModelUniform { model: mat4x4<f32>, tint: vec4<f32>, };
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
struct VertexInput { @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>, };
struct VertexOutput { @builtin(position) clip_position: vec4<f32>, @location(0) uv: vec2<f32>, };
@vertex fn vs_main(input: VertexInput) -> VertexOutput { var output: VertexOutput; output.clip_position = scene.view_proj * model_data.model * vec4<f32>(input.position, 1.0); output.uv = input.uv; return output; }
@fragment fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> { return textureSample(albedo_tex, albedo_sampler, input.uv) * model_data.tint; }
"#,
    )
    .unwrap();
}

fn write_two_primitive_gltf(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    enable_fixture_virtual_geometry_cook(&path);
    let mut binary = Vec::new();
    for value in [
        -1.0f32, -1.0, 0.0, 0.0, 1.0, 0.0, -1.0, 1.0, 0.0, // left triangle
    ] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u32, 1, 2] {
        binary.extend_from_slice(&index.to_le_bytes());
    }
    for value in [
        0.0f32, -1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 0.0, // right triangle
    ] {
        binary.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u32, 1, 2] {
        binary.extend_from_slice(&index.to_le_bytes());
    }
    let binary_path = path.with_file_name("two_primitives.bin");
    fs::create_dir_all(binary_path.parent().unwrap()).unwrap();
    fs::write(&binary_path, &binary).unwrap();
    let gltf = format!(
        r#"{{
  "asset": {{"version": "2.0"}},
  "buffers": [{{"uri": "two_primitives.bin", "byteLength": {}}}],
  "bufferViews": [
    {{"buffer":0,"byteOffset":0,"byteLength":36,"target":34962}},
    {{"buffer":0,"byteOffset":36,"byteLength":12,"target":34963}},
    {{"buffer":0,"byteOffset":48,"byteLength":36,"target":34962}},
    {{"buffer":0,"byteOffset":84,"byteLength":12,"target":34963}}
  ],
  "accessors": [
    {{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-1.0,-1.0,0.0],"max":[0.0,1.0,0.0]}},
    {{"bufferView":1,"componentType":5125,"count":3,"type":"SCALAR","min":[0],"max":[2]}},
    {{"bufferView":2,"componentType":5126,"count":3,"type":"VEC3","min":[0.0,-1.0,0.0],"max":[1.0,1.0,0.0]}},
    {{"bufferView":3,"componentType":5125,"count":3,"type":"SCALAR","min":[0],"max":[2]}}
  ],
  "meshes": [{{"primitives": [
    {{"attributes":{{"POSITION":0}},"indices":1}},
    {{"attributes":{{"POSITION":2}},"indices":3}}
  ]}}],
  "nodes": [{{"mesh":0}}],
  "scenes": [{{"nodes":[0]}}],
  "scene": 0
}}"#,
        binary.len()
    );
    fs::write(path, gltf).unwrap();
}

fn write_solid_png(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_pixel(2, 2, Rgba([255, 255, 255, 255]))
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
}

fn write_material(path: PathBuf, name: &str, shader_uri: &str, texture_uri: &str, color: [f32; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some(name.to_string()),
        shader: asset_reference(shader_uri),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: color,
        base_color_texture: Some(asset_reference(texture_uri)),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0; 3],
        emissive_texture: None,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    let project_root = fixture_project_root(&path);
    let document = material
        .to_project_toml_string(|reference| {
            Ok(persisted_reference_for_fixture(
                &project_root,
                &path,
                reference,
            ))
        })
        .unwrap();
    fs::write(path, document).unwrap();
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn resource_handle<T>(manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing resource {uri}")),
    )
}

fn fixture_project_root(source_path: &Path) -> PathBuf {
    source_path
        .ancestors()
        .find(|candidate| candidate.join("zircon-project.toml").is_file())
        .expect("asset fixture must live below a project manifest")
        .to_path_buf()
}

// 夹具只把 res:// 路径映射到唯一资源根的持久化 hint；多根歧义必须显式失败。
fn persisted_reference_for_fixture(
    project_root: &Path,
    source_path: &Path,
    reference: &AssetReference,
) -> PersistedAssetReference {
    if reference.locator.scheme() == ResourceScheme::Builtin {
        return PersistedAssetReference::builtin(reference.locator.clone());
    }
    assert_eq!(reference.locator.scheme(), ResourceScheme::Res);

    let manifest = ProjectManifest::load(project_root.join("zircon-project.toml")).unwrap();
    let roots = manifest
        .asset_roots
        .iter()
        .filter(|root| {
            root.join_to(project_root)
                .join(reference.locator.path())
                .is_file()
        })
        .collect::<Vec<_>>();
    let root = match roots.as_slice() {
        [root] => *root,
        [] => manifest
            .asset_roots
            .iter()
            .find(|root| source_path.starts_with(root.join_to(project_root)))
            .expect("asset fixture source must live below a manifest asset root"),
        _ => panic!(
            "asset fixture reference {} resolves through multiple project asset roots",
            reference.locator
        ),
    };
    let path_hint =
        RelPath::parse(format!("{}/{}", root.as_str(), reference.locator.path())).unwrap();
    PersistedAssetReference::project(
        AssetRef::try_new(
            reference.uuid,
            path_hint,
            reference.locator.label().map(str::to_owned),
        )
        .unwrap(),
    )
}

fn enable_fixture_virtual_geometry_cook(path: &std::path::Path) {
    let file_name = path.file_name().unwrap();
    let uri = format!("res://models/{}", file_name.to_str().unwrap());
    let mut metadata = zircon_runtime::asset::project::AssetMetaDocument::new(
        zircon_runtime::asset::AssetUuid::new(),
        AssetUri::parse(&uri).unwrap(),
        zircon_runtime::asset::AssetKind::Model,
    );
    // Each authored entity has one raw cluster per real primitive. The original
    // triangle/quad bytes fit a single cooked leaf without copying geometry.
    metadata.import_settings =
        "[virtual_geometry]\nenabled = true\ncluster_triangle_count = 8\npage_cluster_count = 1\n"
            .parse()
            .unwrap();
    let mut sidecar_name = file_name.to_os_string();
    sidecar_name.push(".zmeta");
    metadata.save(path.with_file_name(sidecar_name)).unwrap();
}
