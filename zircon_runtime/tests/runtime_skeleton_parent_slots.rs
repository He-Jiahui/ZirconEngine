#![cfg(all(feature = "native-host", feature = "graphics"))]

//! Check that evaluation order does not change the joint slots read by a drawn mesh.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::sync::Arc;

use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::AssetManager;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialAsset, MeshAsset, MeshAttributeValues, MeshIndices,
    MeshSkinAsset, ProjectAssetManager, ZMaterialDocument, MESH_ATTRIBUTE_JOINT_INDEX,
    MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
    MESH_ATTRIBUTE_UV0,
};
use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::framework::render::{
    CameraRenderDescriptor, CastShadowsMode, FallbackSkyboxKind, PreviewEnvironmentExtract,
    ProjectionMode, RenderCameraClear, RenderCameraTarget, RenderFrameExtract, RenderFramework,
    RenderLayerSet, RenderMaterialLightingModel, RenderMeshSnapshot, RenderMeshTopology,
    RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderSkeletalPoseExtract, RenderViewportDescriptor, RenderWorldSnapshotHandle, RendererCommon,
    ViewportCameraSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Mat4, Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{ResourceHandle, ResourceId, ResourceKind, ResourceRecord};
use zircon_runtime::graphics::WgpuRenderFramework;

#[path = "support/project_asset_runtime.rs"]
mod support;

const SIZE: UVec2 = UVec2::new(192, 192);
const ENTITY: u64 = 164;
// Orthographic X in [8, 20], 16 px per unit. Order: Root/Child bind, Root/Child posed.
const PROBE_COLUMNS: [usize; 4] = [32, 48, 96, 144];

#[test]
fn later_parent_slot_preserves_direct_mesh_skinning() {
    let assets = Arc::new(ProjectAssetManager::default());
    open_fixture_project(&assets);
    let material = register_unlit_material(&assets);
    let (ordered_mesh, ordered_skeleton) = register_skin(&assets, false);
    let (late_mesh, late_skeleton) = register_skin(&assets, true);
    let pose = Arc::new(AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        // Same named pose in both submissions, deliberately not permuted with the skin.
        bones: [("Root", 14.0), ("Child", 3.0)]
            .into_iter()
            .map(|(name, x)| AnimationPoseBone {
                name: name.into(),
                local_transform: Transform::from_translation(Vec3::new(x, 0.0, 0.0)),
            })
            .collect(),
    });

    let asset_runtime = support::ProjectAssetTestRuntime::new(assets);
    let framework = WgpuRenderFramework::new(asset_runtime.access(), asset_runtime.worker_pool())
        .expect("headless WGPU adapter/device is a test precondition");
    let framework = support::TestWgpuRenderFramework::new(asset_runtime, framework);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(SIZE))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("skeleton-164")
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false)
                .with_clustered_lighting(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_virtual_geometry(false),
        )
        .unwrap();
    let backend = framework
        .query_graphics_debugger_status()
        .unwrap()
        .backend_name;
    let sample = |label: &str, world, mesh, skeleton: Option<ResourceId>| {
        let pose = skeleton.map(|skeleton| RenderSkeletalPoseExtract {
            entity: ENTITY,
            skeleton,
            pose: Arc::clone(&pose),
        });
        framework
            .submit_frame_extract(viewport, extract(world, material, mesh, pose))
            .expect("direct-mesh frame submission must succeed");
        let frame = framework
            .capture_frame(viewport)
            .expect("readback must succeed before testing skinning")
            .expect("a completed direct-mesh frame is required");
        assert!(
            framework.first_device_fault().is_none(),
            "{label}: device fault"
        );
        let stats = framework.query_stats().unwrap();
        let draws = [
            stats.last_mesh_draw_count,
            stats.last_mesh_skinned_draw_count,
            stats.last_mesh_prepared_geometry_draw_count,
        ];
        assert_eq!((frame.width, frame.height), (SIZE.x, SIZE.y));
        assert_eq!(frame.rgba.len(), (SIZE.x * SIZE.y * 4) as usize);
        let mut pixels = [0_usize; 5];
        for (index, rgba) in frame.rgba.chunks_exact(4).enumerate() {
            if rgba[..3].iter().any(|&channel| channel >= 32) {
                let column = index % SIZE.x as usize;
                let bin = PROBE_COLUMNS
                    .iter()
                    .position(|&probe| column.abs_diff(probe) <= 7)
                    .unwrap_or(4);
                pixels[bin] += 1;
            }
        }
        let generation = frame.generation;
        println!(
            "{label}: {backend} {:?}; generation={generation} draws(total,skinned,prepared)={draws:?} \
             skin_sources={} palette_uploads={} gpu_skinning={} pixels(bind Root/Child, posed Root/Child, other)={pixels:?}",
            stats.device_diagnostics.as_ref().map(|d| &d.adapter_name),
            stats.last_mesh_skinned_gpu_source_candidate_count,
            stats.last_mesh_skinned_palette_upload_count,
            stats.last_mesh_skinned_gpu_skinning_draw_count,
        );
        (generation, draws, pixels)
    };

    // Same late-order mesh without a pose, then the positive parent-first control, then the witness.
    let (rigid_gen, rigid_draws, rigid_pixels) = sample("no-pose", 1, late_mesh, None);
    let (ordered_gen, ordered_draws, ordered_pixels) =
        sample("parent-first", 2, ordered_mesh, Some(ordered_skeleton));
    assert_eq!(rigid_draws, [1, 0, 1], "rigid fallback control");
    assert_coverage(
        rigid_pixels,
        [true, true, false, false, false],
        "rigid control",
    );
    assert_eq!(
        ordered_draws,
        [1, 1, 0],
        "parent-first control never skinned"
    );
    assert_coverage(
        ordered_pixels,
        [false, false, true, true, false],
        "parent-first control",
    );
    let (late_gen, late_draws, late_pixels) =
        sample("child-first", 3, late_mesh, Some(late_skeleton));
    assert!(
        rigid_gen < ordered_gen && ordered_gen < late_gen,
        "readback must be fresh"
    );
    assert!(
        late_draws == [1, 1, 0] && has_coverage(late_pixels, [false, false, true, true, false]),
        "legal [Child,Root] must skin to the parent-first positions, not silently draw bind geometry; \
         draws={late_draws:?}, pixels={late_pixels:?}, no-pose pixels={rigid_pixels:?}",
    );
}

fn assert_coverage(pixels: [usize; 5], expected: [bool; 5], label: &str) {
    assert!(has_coverage(pixels, expected), "{label}: pixels={pixels:?}");
}

fn has_coverage(pixels: [usize; 5], expected: [bool; 5]) -> bool {
    pixels.into_iter().zip(expected).all(
        |(count, present)| {
            if present {
                count >= 24
            } else {
                count <= 4
            }
        },
    )
}

fn open_fixture_project(assets: &ProjectAssetManager) {
    // The native launcher binds TEMP/TMPDIR to the physically admitted volume.
    let temp_var = if cfg!(windows) { "TEMP" } else { "TMPDIR" };
    let temp = std::env::var_os(temp_var)
        .filter(|path| !path.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    assert!(temp.is_absolute(), "fixture temp root must be absolute");
    let parent = ProjectPaths::resolve_existing(temp).expect("resolve fixture temp root");
    let scene = AssetUri::parse("res://scenes/unused.zscene").unwrap();
    // This test submits frame DTOs directly; it never loads the default scene.
    let manifest = ProjectManifest::new("Skeleton164DrawFixture", scene, 1);
    let guid = manifest.project_guid;
    let path = parent.operation_path().join(format!("s164-{guid}"));
    std::fs::create_dir(&path).expect("reserve an exclusive skeleton164 project directory");
    let root = ProjectPaths::resolve_existing(&path).expect("resolve new project root");
    assert_eq!(
        root.operation_path(),
        path.as_path(),
        "fixture root was redirected"
    );
    println!(
        "skeleton164 fixture root={} project_guid={guid}",
        root.display_path().display()
    );
    // Keep this uniquely owned project, its marker, and its caches for failure inspection.
    let mut marker = std::fs::File::create_new(root.operation_path().join(".skeleton164-owner"))
        .expect("reserve the skeleton164 fixture marker");
    writeln!(marker, "runtime_skeleton_parent_slots\nproject_guid={guid}")
        .expect("write the skeleton164 fixture marker");
    drop(marker);

    let paths = ProjectPaths::from_resolved_root(&root);
    let mut document = std::fs::File::create_new(paths.manifest_path())
        .expect("reserve the skeleton164 project manifest");
    document
        .write_all(
            &manifest
                .to_current_format_bytes()
                .expect("encode project manifest"),
        )
        .expect("write project manifest");
    drop(document);

    let project = ProjectManager::open_resolved(&root).expect("open the fixture project");
    assets
        .open_prepared_project(project)
        .expect("activate the fixture project before inserting render assets");
    let active = assets
        .current_project_manager()
        .expect("fixture project is active");
    assert_eq!(active.paths().root(), root.operation_path());
    assert_eq!(active.manifest().project_guid, guid);
}

fn register_unlit_material(assets: &ProjectAssetManager) -> ResourceId {
    let uri = AssetUri::parse("res://materials/skeleton164.zmaterial").unwrap();
    let id = ResourceId::from_locator(&uri);
    let material = MaterialAsset::from_zmaterial_document(ZMaterialDocument {
        version: 2,
        name: Some("Skeleton164Unlit".into()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        overrides: BTreeMap::from([
            (
                "lighting_model".into(),
                toml::Value::String(RenderMaterialLightingModel::Unlit.to_string()),
            ),
            ("double_sided".into(), toml::Value::Boolean(true)),
        ]),
        textures: Default::default(),
        queue: None,
        editor: Default::default(),
        validation_diagnostics: Vec::new(),
    });
    assets
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri),
            material,
        )
        .unwrap();
    id
}

fn register_skin(assets: &ProjectAssetManager, child_first: bool) -> (ResourceId, ResourceId) {
    let (tag, root_slot, child_slot) = if child_first {
        ("late", 1_u16, 0_u16)
    } else {
        ("ordered", 0, 1)
    };
    let bone = |name: &str, parent_index, x| AnimationSkeletonBoneAsset {
        name: name.into(),
        parent_index,
        local_translation: [x, 0.0, 0.0],
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0, 1.0, 1.0],
    };
    let root = bone("Root", None, 10.0);
    let child = bone("Child", Some(u32::from(root_slot)), 1.0);
    let skeleton = AnimationSkeletonAsset {
        name: Some("Skeleton164".into()),
        bones: if child_first {
            vec![child, root]
        } else {
            vec![root, child]
        },
    };
    let bytes = skeleton
        .to_bytes()
        .expect("both parent slot orders are valid assets");
    let decoded = AnimationSkeletonAsset::from_bytes(&bytes).unwrap();
    assert_eq!(
        decoded, skeleton,
        "asset round-trip must retain parent slots"
    );
    let skeleton_uri =
        AssetUri::parse(&format!("res://animation/skeleton164-{tag}.zranim")).unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    assets
        .assets::<AnimationSkeletonAsset>()
        .insert(
            ResourceRecord::new(skeleton_id, ResourceKind::AnimationSkeleton, skeleton_uri),
            decoded,
        )
        .unwrap();
    assert_eq!(
        assets.load_animation_skeleton_asset(skeleton_id).unwrap(),
        skeleton
    );

    let uri = AssetUri::parse(&format!("res://meshes/skeleton164-{tag}.zmesh")).unwrap();
    let mesh_id = ResourceId::from_locator(&uri);
    let positions = [10.0, 11.0]
        .into_iter()
        .flat_map(|x| {
            [
                [x - 0.375, -0.5, 0.0],
                [x + 0.375, -0.5, 0.0],
                [x, 0.5, 0.0],
            ]
        })
        .collect();
    let joints = [root_slot, child_slot]
        .into_iter()
        .flat_map(|slot| [[slot, 0, 0, 0]; 3])
        .collect();
    let mut mesh = MeshAsset::new(
        uri.clone(),
        RenderMeshTopology::TriangleList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.into(),
                MeshAttributeValues::Float32x3(positions),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.into(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 6]),
            ),
            (
                MESH_ATTRIBUTE_UV0.into(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]].repeat(2)),
            ),
            (
                MESH_ATTRIBUTE_JOINT_INDEX.into(),
                MeshAttributeValues::Uint16x4(joints),
            ),
            (
                MESH_ATTRIBUTE_JOINT_WEIGHT.into(),
                MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 0.0]; 6]),
            ),
        ]),
        Some(MeshIndices::U32(vec![0, 1, 2, 3, 4, 5])),
    )
    .unwrap();
    let mut inverse_bind_matrices = vec![Mat4::IDENTITY.to_cols_array_2d(); 2];
    inverse_bind_matrices[usize::from(root_slot)] =
        Mat4::from_translation(Vec3::new(-10.0, 0.0, 0.0)).to_cols_array_2d();
    inverse_bind_matrices[usize::from(child_slot)] =
        Mat4::from_translation(Vec3::new(-11.0, 0.0, 0.0)).to_cols_array_2d();
    mesh.skin = Some(MeshSkinAsset {
        inverse_bind_matrices,
    });
    assets
        .assets::<MeshAsset>()
        .insert(ResourceRecord::new(mesh_id, ResourceKind::Mesh, uri), mesh)
        .unwrap();
    (mesh_id, skeleton_id)
}

fn extract(
    world: u64,
    material: ResourceId,
    mesh: ResourceId,
    pose: Option<RenderSkeletalPoseExtract>,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::from_translation(Vec3::new(14.0, 0.0, 4.0)),
        projection_mode: ProjectionMode::Orthographic,
        ortho_size: 6.0,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(SIZE);
    let mut descriptor = CameraRenderDescriptor::from_camera_payload(None, camera.clone());
    descriptor.target = RenderCameraTarget::Headless { size: SIZE };
    descriptor.clear = RenderCameraClear::Color(Vec4::ZERO);
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: vec![RenderMeshSnapshot {
                    node_id: ENTITY,
                    stable_instance_key: ENTITY << 16,
                    transform_revision: 1,
                    transform: Transform::default(),
                    model: ResourceHandle::new(ResourceId::from_stable_label("builtin://cube")),
                    mesh: Some(ResourceHandle::new(mesh)),
                    material: ResourceHandle::new(material),
                    mesh_lod: None,
                    morph_weights: Vec::new(),
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    static_state: Default::default(),
                    common: RendererCommon {
                        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                        cast_shadows: CastShadowsMode::Off,
                        ..Default::default()
                    },
                }],
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    extract.select_camera_descriptor(descriptor);
    *extract.animation_poses = pose.into_iter().collect();
    extract
}
