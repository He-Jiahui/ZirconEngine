use super::*;

use crate::scene::components::{MeshRendererPrimitiveBinding, Name};
use std::sync::Arc;

#[test]
fn world_render_extract_stamps_the_exact_source_world_generation() {
    let mut world = World::empty();
    world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");

    let extract = world.to_render_frame_extract();

    assert_eq!(extract.world.raw(), 0);
    assert_eq!(extract.world.generation(), world.world_generation());
    assert_ne!(extract.world.generation(), 0);
}

#[test]
#[cfg(feature = "animation")]
fn level_system_render_extract_uses_world_direct_path_and_merges_animation_poses() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), Default::default());
    let (mesh_with_skeleton, mesh_without_skeleton, skeleton_handle) =
        level.with_world_mut(|world| {
            let camera = spawn_camera_on_layer(world, 0b1111);
            world.set_active_camera(camera);
            let mesh_with_skeleton = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
            let mesh_without_skeleton = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
            let skeleton_handle = ResourceHandle::<AnimationSkeletonMarker>::new(
                ResourceId::from_stable_label("res://animation/hero.skeleton.zranim"),
            );
            world
                .set_animation_skeleton(
                    mesh_with_skeleton,
                    Some(AnimationSkeletonComponent {
                        skeleton: skeleton_handle,
                    }),
                )
                .unwrap();
            (mesh_with_skeleton, mesh_without_skeleton, skeleton_handle)
        });
    let missing_entity = 99_999;
    let pose = test_pose("hip");
    let replacement_epoch = level.capture_world_replacement_epoch();
    assert!(level.record_animation_pose_snapshot(
        replacement_epoch,
        Arc::new(BTreeMap::from([
            (mesh_with_skeleton, Arc::new(pose.clone())),
            (
                mesh_without_skeleton,
                Arc::new(test_pose("filtered-no-skeleton")),
            ),
            (missing_entity, Arc::new(test_pose("filtered-missing"))),
        ]))
    ));

    let extract = RenderExtractProducer::build_render_frame_extract(
        &level,
        &RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(705),
            SceneViewportExtractRequest::default(),
        ),
    );

    assert_eq!(extract.world.raw(), 705);
    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == mesh_with_skeleton));
    assert_eq!(extract.animation_poses.len(), 1);
    assert_eq!(extract.animation_poses[0].entity, mesh_with_skeleton);
    assert_eq!(extract.animation_poses[0].skeleton, skeleton_handle.id());
    assert_eq!(extract.animation_poses[0].pose.as_ref(), &pose);
    assert!(level.with_world(|world| !world.has_pending_scene_systems()));
}

#[test]
#[cfg(feature = "animation")]
fn level_frame_snapshot_publishes_a_new_animation_generation_without_retiring_the_old_handle() {
    let level = DefaultLevelManager::default().create_default_level();
    let entity = level.with_world_mut(|world| {
        world
            .spawn_node(NodeKind::Mesh)
            .expect("test scene spawn should succeed")
    });
    let pose = test_pose("frame-snapshot");
    let initial = level.frame_state_snapshot();
    let replacement_epoch = level.capture_world_replacement_epoch();

    assert!(level.record_animation_pose_snapshot(
        replacement_epoch,
        Arc::new(BTreeMap::from([(entity, Arc::new(pose.clone()))])),
    ));
    let published = level.frame_state_snapshot();

    assert_eq!(initial.animation_generation(), 0);
    assert!(initial.animation_poses().is_empty());
    assert_eq!(published.animation_generation(), 1);
    assert_eq!(
        published.animation_poses().get(&entity).map(Arc::as_ref),
        Some(&pose)
    );
    assert!(
        !std::sync::Arc::ptr_eq(initial.animation_poses(), published.animation_poses()),
        "a new animation publication must not mutate an earlier frame handle"
    );

    assert!(level.record_animation_pose_snapshot(
        replacement_epoch,
        Arc::new(BTreeMap::from([(entity, Arc::new(pose.clone()))])),
    ));
    let unchanged = level.frame_state_snapshot();
    assert_eq!(
        unchanged.animation_generation(),
        published.animation_generation()
    );
    assert!(
        std::sync::Arc::ptr_eq(unchanged.animation_poses(), published.animation_poses()),
        "an unchanged pose payload must retain its sealed frame handle"
    );

    assert!(level.record_animation_pose_snapshot(replacement_epoch, Arc::default()));
    let cleared = level.frame_state_snapshot();
    assert_eq!(cleared.animation_generation(), 2);
    assert!(cleared.animation_poses().is_empty());
    assert_eq!(
        published.animation_poses().get(&entity).map(Arc::as_ref),
        Some(&pose)
    );

    let published_world_generation = published.world_generation();
    level.replace_world_and_reset_runtime_state(World::empty());
    let replaced = level.frame_state_snapshot();
    assert_ne!(replaced.world_generation(), published_world_generation);
    assert!(replaced.animation_poses().is_empty());
    assert_eq!(
        published.animation_poses().get(&entity).map(Arc::as_ref),
        Some(&pose)
    );
}

#[test]
#[cfg(feature = "animation")]
fn level_system_render_extract_consumes_the_sealed_animation_frame_handle_without_resorting() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("level_system_render_extract.rs"),
    )
    .unwrap();

    assert!(
        source.contains("let frame_state = self.frame_state_snapshot();")
            && source.contains("frame_state.animation_poses()")
            && source.contains("frame_state.world_generation() != world.world_generation()")
            && source.contains(".iter()")
            && !source.contains("animation_pose_entries"),
        "scene render extraction must consume the sealed Arc-backed BTreeMap handle instead of cloning poses before filtering"
    );
    assert!(
        !source.contains("animation_poses.sort") && !source.contains("sort_by_key"),
        "the animation-pose cache is a BTreeMap, so filtering its ordered iterator must not add an O(n log n) per-frame resort"
    );
}

#[test]
fn render_frame_extract_snapshot_adapters_are_not_scene_production_paths() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "src/scene/render_extract/mod.rs",
        "src/scene/world/render.rs",
        "src/scene/level_system_render_extract.rs",
    ] {
        assert_source_excludes_file(
            &manifest_root.join(relative),
            &["RenderFrameExtract::from_snapshot"],
            "scene production extraction must populate RenderFrameExtract directly; snapshot adapters are allowed only for preview/test/roundtrip/synthetic helpers",
        );
    }

    let submit_root = manifest_root
        .join("src")
        .join("graphics")
        .join("runtime")
        .join("render_framework")
        .join("submit_frame_extract");
    assert_runtime_submit_tree_excludes_snapshot_adapters(&submit_root);
}

#[test]
fn render_view_extract_keeps_selected_scene_camera_descriptor_when_inactive() {
    let render_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("render.rs"),
    )
    .unwrap();
    let view_builder = render_source
        .split("fn build_render_view_extract")
        .nth(1)
        .and_then(|text| text.split("fn render_extract_layers_for_view").next())
        .expect("read render view extract builder");

    assert!(
        view_builder.contains("descriptor.entity == Some(entity) || descriptor.is_active()")
            && !view_builder.contains(".filter(CameraRenderDescriptor::is_active)"),
        "scene render view extraction must keep the selected camera descriptor even when the camera is inactive"
    );
}

#[test]
fn mesh_presence_preserves_node_semantics_and_does_not_mutate_world() {
    let mut world = World::empty();
    let mesh = spawn_mesh_on_layer(&mut world, 0b0001, Mobility::Dynamic);
    let primitive = MeshRendererPrimitiveBinding {
        mesh: mesh_handle("res://meshes/presence.mesh"),
        material: material_handle("res://materials/presence.zmaterial"),
    };
    let mut renderer = MeshRenderer::from_handles(
        model_handle("res://models/presence.obj"),
        primitive.material,
    );
    renderer.morph_weights = vec![0.25, 0.75];
    renderer.primitives = vec![primitive.clone()];
    let mut lod = MeshRendererLodLevel::from_handles(
        10.0,
        model_handle("res://models/presence-lod.obj"),
        primitive.material,
    );
    lod.primitives = vec![primitive];
    renderer.lods = vec![lod];
    world.insert(mesh, renderer.clone()).unwrap();

    let empty_kind = world.spawn_node(NodeKind::Empty).unwrap();
    world.insert(empty_kind, MeshRenderer::default()).unwrap();
    world.set_active_self(empty_kind, false).unwrap();
    let no_mesh = world.spawn_node(NodeKind::Empty).unwrap();
    let no_name = world.spawn_node(NodeKind::Mesh).unwrap();
    assert!(world.remove::<Name>(no_name).unwrap().is_some());
    let missing = u64::MAX;
    assert!(!world.contains_entity(missing));
    assert!(world.get::<MeshRenderer>(mesh).unwrap().mesh.is_none());
    assert_eq!(world.find_node(empty_kind).unwrap().kind, NodeKind::Empty);
    assert_eq!(world.active_self(empty_kind), Some(false));
    assert!(world.get::<MeshRenderer>(no_name).is_some());

    let generation = world.world_generation();
    let change_tick = world.read_change_tick();
    let last_change_tick = world.last_change_tick();
    let mesh_ticks = world.component_change_ticks::<MeshRenderer>(mesh).unwrap();
    let name_ticks = world.component_change_ticks::<Name>(mesh).unwrap();
    let name = world.get::<Name>(mesh).unwrap().clone();
    for _ in 0..3 {
        for (entity, expected) in [
            (mesh, true),
            (empty_kind, true),
            (no_mesh, false),
            (no_name, false),
            (missing, false),
        ] {
            assert_eq!(world.has_mesh_node_for_extract(entity), expected);
            assert_eq!(
                world.has_mesh_node_for_extract(entity),
                world
                    .find_node(entity)
                    .is_some_and(|node| node.mesh.is_some())
            );
        }
    }
    assert_eq!(world.world_generation(), generation);
    assert_eq!(world.read_change_tick(), change_tick);
    assert_eq!(world.last_change_tick(), last_change_tick);
    assert_eq!(
        world.component_change_ticks::<MeshRenderer>(mesh),
        Some(mesh_ticks)
    );
    assert_eq!(world.component_change_ticks::<Name>(mesh), Some(name_ticks));
    assert_eq!(world.get::<MeshRenderer>(mesh), Some(&renderer));
    assert_eq!(world.get::<Name>(mesh), Some(&name));
}

#[test]
#[cfg(feature = "animation")]
fn level_pose_filter_preserves_typed_presence_order_and_pose_owners() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let skeleton = ResourceHandle::<AnimationSkeletonMarker>::new(ResourceId::from_stable_label(
        "res://animation/presence.skeleton.zranim",
    ));
    let (mesh, empty_kind, no_skeleton, skeleton_only, no_name) = level.with_world_mut(|world| {
        let camera = spawn_camera_on_layer(world, 0b1111);
        world.set_active_camera(camera);
        let mesh = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
        let empty_kind = world.spawn_node(NodeKind::Empty).unwrap();
        world.insert(empty_kind, MeshRenderer::default()).unwrap();
        world.set_active_self(empty_kind, false).unwrap();
        let no_skeleton = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
        let skeleton_only = world.spawn_node(NodeKind::Empty).unwrap();
        let no_name = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
        for entity in [mesh, empty_kind, skeleton_only, no_name] {
            world
                .set_animation_skeleton(entity, Some(AnimationSkeletonComponent { skeleton }))
                .unwrap();
        }
        assert!(world.remove::<Name>(no_name).unwrap().is_some());
        assert!(world.get::<MeshRenderer>(mesh).unwrap().mesh.is_none());
        assert!(world
            .get::<MeshRenderer>(empty_kind)
            .unwrap()
            .mesh
            .is_none());
        (mesh, empty_kind, no_skeleton, skeleton_only, no_name)
    });
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(706),
        SceneViewportExtractRequest::default(),
    );
    assert!(
        RenderExtractProducer::build_render_frame_extract(&level, &context)
            .animation_poses
            .is_empty()
    );
    assert!(level.with_world(|world| !world.has_pending_scene_systems()));
    let missing = u64::MAX;
    let poses = Arc::new(BTreeMap::from([
        (missing, Arc::new(test_pose("missing"))),
        (no_name, Arc::new(test_pose("name-removed"))),
        (skeleton_only, Arc::new(test_pose("skeleton-only"))),
        (no_skeleton, Arc::new(test_pose("no-skeleton"))),
        (empty_kind, Arc::new(test_pose("inactive-empty-kind"))),
        (mesh, Arc::new(test_pose("mesh"))),
    ]));
    assert!(level.record_animation_pose_snapshot(
        level.capture_world_replacement_epoch(),
        Arc::clone(&poses)
    ));
    let published = level.frame_state_snapshot();
    let generation = level.world_generation();
    let mut expected_entities = vec![mesh, empty_kind];
    expected_entities.sort_unstable();

    for _ in 0..2 {
        let extract = RenderExtractProducer::build_render_frame_extract(&level, &context);
        assert_eq!(extract.world.raw(), 706);
        assert_eq!(extract.world.generation(), generation);
        assert_eq!(
            extract
                .animation_poses
                .iter()
                .map(|entry| entry.entity)
                .collect::<Vec<_>>(),
            expected_entities
        );
        for entry in extract.animation_poses.iter() {
            assert_eq!(entry.skeleton, skeleton.id());
            assert!(Arc::ptr_eq(&entry.pose, &poses[&entry.entity]));
            assert_eq!(entry.pose.as_ref(), poses[&entry.entity].as_ref());
        }
        assert_eq!(level.world_generation(), generation);
        assert!(Arc::ptr_eq(
            published.animation_poses(),
            level.frame_state_snapshot().animation_poses()
        ));
        assert!(level.with_world(|world| !world.has_pending_scene_systems()));
    }
}

#[test]
#[cfg(feature = "animation")]
fn level_pose_filter_rejects_stale_generation_and_preserves_empty_publication() {
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let entity = level.with_world_mut(|world| {
        let camera = spawn_camera_on_layer(world, 0b1111);
        world.set_active_camera(camera);
        let entity = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::new(ResourceId::from_stable_label(
                        "res://animation/generation.skeleton.zranim",
                    )),
                }),
            )
            .unwrap();
        entity
    });
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(707),
        SceneViewportExtractRequest::default(),
    );
    assert!(
        RenderExtractProducer::build_render_frame_extract(&level, &context)
            .animation_poses
            .is_empty()
    );
    let epoch = level.capture_world_replacement_epoch();
    let poses = Arc::new(BTreeMap::from([(
        entity,
        Arc::new(test_pose("generation")),
    )]));
    assert!(level.record_animation_pose_snapshot(epoch, Arc::clone(&poses)));
    let published = level.frame_state_snapshot();
    let healthy = RenderExtractProducer::build_render_frame_extract(&level, &context);
    assert_eq!(healthy.animation_poses.len(), 1);
    assert!(Arc::ptr_eq(
        &healthy.animation_poses[0].pose,
        &poses[&entity]
    ));

    level.with_world_mut(|world| {
        world
            .insert(entity, Name("changed-name".to_string()))
            .unwrap();
    });
    assert_ne!(level.world_generation(), published.world_generation());
    let stale = RenderExtractProducer::build_render_frame_extract(&level, &context);
    assert!(stale.animation_poses.is_empty());
    assert_eq!(stale.world.generation(), level.world_generation());
    assert!(Arc::ptr_eq(
        published.animation_poses(),
        level.frame_state_snapshot().animation_poses()
    ));
    assert!(level.with_world(|world| !world.has_pending_scene_systems()));

    assert!(level.record_animation_pose_snapshot(epoch, Arc::clone(&poses)));
    let refreshed = RenderExtractProducer::build_render_frame_extract(&level, &context);
    assert_eq!(refreshed.animation_poses.len(), 1);
    assert!(Arc::ptr_eq(
        &refreshed.animation_poses[0].pose,
        &poses[&entity]
    ));
    assert!(level.record_animation_pose_snapshot(epoch, Arc::default()));
    assert!(
        RenderExtractProducer::build_render_frame_extract(&level, &context)
            .animation_poses
            .is_empty()
    );
    level.replace_world_and_reset_runtime_state(World::empty());
    let replaced = RenderExtractProducer::build_render_frame_extract(&level, &context);
    assert!(replaced.animation_poses.is_empty());
    assert_eq!(replaced.world.generation(), level.world_generation());
    assert!(Arc::ptr_eq(
        &published.animation_poses()[&entity],
        &poses[&entity]
    ));
}

#[test]
#[cfg(feature = "animation")]
#[ignore = "R4 paired cost diagnostic; requires an admitted optimized run, not a timing acceptance gate"]
fn level_pose_mesh_presence_paired_cost_diagnostic() {
    use crate::core::framework::render::RenderSkeletalPoseExtract;
    use std::hint::black_box;
    use std::time::Instant;

    const HEAVY_NODES: usize = 64;
    const PRIMITIVES: usize = 32;
    const MORPH_WEIGHTS: usize = 128;
    const WARMUP_PAIRS: usize = 8;
    const SAMPLE_PAIRS: usize = 30;
    const FILTERS_PER_SAMPLE: usize = 8;

    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let skeleton = ResourceHandle::<AnimationSkeletonMarker>::new(ResourceId::from_stable_label(
        "res://animation/r4-cost.skeleton.zranim",
    ));
    let (heavy_entities, candidates, mut expected_entities) = level.with_world_mut(|world| {
        let camera = spawn_camera_on_layer(world, 0b1111);
        world.set_active_camera(camera);
        let primitive = MeshRendererPrimitiveBinding {
            mesh: mesh_handle("res://meshes/r4-cost.mesh"),
            material: material_handle("res://materials/r4-cost.zmaterial"),
        };
        let mut renderer = MeshRenderer::from_handles(
            model_handle("res://models/r4-cost.obj"),
            primitive.material,
        );
        renderer.morph_weights = vec![0.25; MORPH_WEIGHTS];
        renderer.primitives = vec![primitive.clone(); PRIMITIVES];
        renderer.lods = [10.0, 25.0, 50.0, 100.0]
            .into_iter()
            .map(|distance| {
                let mut lod = MeshRendererLodLevel::from_handles(
                    distance,
                    model_handle("res://models/r4-cost-lod.obj"),
                    primitive.material,
                );
                lod.primitives = vec![primitive.clone(); PRIMITIVES];
                lod
            })
            .collect();
        let mut heavy = Vec::with_capacity(HEAVY_NODES);
        for index in 0..HEAVY_NODES {
            let entity = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
            world
                .insert(
                    entity,
                    Name(format!(
                        "r4-cost-{index:03}-{}",
                        "nested-payload-".repeat(16)
                    )),
                )
                .unwrap();
            world.insert(entity, renderer.clone()).unwrap();
            world
                .set_animation_skeleton(entity, Some(AnimationSkeletonComponent { skeleton }))
                .unwrap();
            heavy.push(entity);
        }

        // Preserve outer presence: an inactive Empty node with mesh.mesh == None is eligible.
        let empty_kind = world.spawn_node(NodeKind::Empty).unwrap();
        world.insert(empty_kind, renderer.clone()).unwrap();
        world.set_active_self(empty_kind, false).unwrap();
        let no_skeleton = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
        world.insert(no_skeleton, renderer.clone()).unwrap();
        let no_mesh = world.spawn_node(NodeKind::Empty).unwrap();
        let no_name = spawn_mesh_on_layer(world, 0b0001, Mobility::Dynamic);
        world.insert(no_name, renderer).unwrap();
        for entity in [empty_kind, no_mesh, no_name] {
            world
                .set_animation_skeleton(entity, Some(AnimationSkeletonComponent { skeleton }))
                .unwrap();
        }
        assert!(world.remove::<Name>(no_name).unwrap().is_some());
        assert!(world.get::<MeshRenderer>(heavy[0]).unwrap().mesh.is_none());
        let missing = u64::MAX;
        assert!(!world.contains_entity(missing));
        let mut candidates = heavy.clone();
        candidates.extend([empty_kind, no_skeleton, no_mesh, no_name, missing]);
        let mut expected = heavy.clone();
        expected.push(empty_kind);
        (heavy, candidates, expected)
    });
    expected_entities.sort_unstable();
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(708),
        SceneViewportExtractRequest::default(),
    );
    // Exercise the real production prepare path before sealing the measurement input.
    assert!(
        RenderExtractProducer::build_render_frame_extract(&level, &context)
            .animation_poses
            .is_empty()
    );
    assert!(level.with_world(|world| !world.has_pending_scene_systems()));
    let poses = Arc::new(
        candidates
            .iter()
            .map(|&entity| (entity, Arc::new(test_pose("r4-fixed-bone"))))
            .collect::<BTreeMap<_, _>>(),
    );
    assert!(level.record_animation_pose_snapshot(
        level.capture_world_replacement_epoch(),
        Arc::clone(&poses),
    ));
    let frame = level.frame_state_snapshot();
    let product = RenderExtractProducer::build_render_frame_extract(&level, &context);
    assert_eq!(product.world.generation(), frame.world_generation());
    assert_eq!(
        product
            .animation_poses
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        expected_entities,
    );

    let check_rows = |rows: &[RenderSkeletalPoseExtract]| {
        assert_eq!(rows.len(), product.animation_poses.len());
        for (row, actual) in rows.iter().zip(product.animation_poses.iter()) {
            assert_eq!(row.entity, actual.entity);
            assert_eq!(row.skeleton, actual.skeleton);
            assert_eq!(row.pose.as_ref(), actual.pose.as_ref());
            assert!(Arc::ptr_eq(&row.pose, &actual.pose));
            assert!(Arc::ptr_eq(&row.pose, &poses[&row.entity]));
        }
    };

    level.with_world(|world| {
        let source_state = (world.world_generation(), world.read_change_tick(), world.last_change_tick());
        let ticks = heavy_entities.iter()
            .map(|&entity| world.component_change_ticks::<MeshRenderer>(entity))
            .collect::<Vec<_>>();
        let mut distinct_owned_buffers = 0usize;
        let mut copied_payload_bytes = 0usize;
        // This is a standalone real find_node observation, not a global allocator hook.
        // It witnesses clone storage; optimized timed calls may eliminate unused work.
        for &entity in &heavy_entities {
            let original = world.get::<MeshRenderer>(entity).unwrap();
            let name = world.get::<Name>(entity).unwrap();
            let owned = world.find_node(entity).unwrap();
            let cloned = owned.mesh.as_ref().unwrap();
            assert_eq!(cloned, original);
            assert_eq!(owned.name, name.0);
            assert_ne!(owned.name.as_ptr(), name.0.as_ptr());
            assert_ne!(cloned.morph_weights.as_ptr(), original.morph_weights.as_ptr());
            assert_ne!(cloned.primitives.as_ptr(), original.primitives.as_ptr());
            assert_ne!(cloned.lods.as_ptr(), original.lods.as_ptr());
            distinct_owned_buffers += 4;
            copied_payload_bytes += name.0.len()
                + std::mem::size_of_val(original.morph_weights.as_slice())
                + std::mem::size_of_val(original.primitives.as_slice())
                + std::mem::size_of_val(original.lods.as_slice());
            for (copy, source) in cloned.lods.iter().zip(original.lods.iter()) {
                assert!(!source.primitives.is_empty());
                assert_eq!(copy.primitives, source.primitives);
                assert_ne!(copy.primitives.as_ptr(), source.primitives.as_ptr());
                distinct_owned_buffers += 1;
                copied_payload_bytes += std::mem::size_of_val(source.primitives.as_slice());
            }
        }

        let filter = |world: &World, legacy: bool| -> Vec<RenderSkeletalPoseExtract> {
            if frame.world_generation() != world.world_generation() || frame.animation_poses().is_empty() {
                return Vec::new();
            }
            frame.animation_poses().iter().filter_map(|(&entity, pose)| {
                let present = if legacy {
                    // Exact legacy expression: do not black_box the entire owned node.
                    world.find_node(black_box(entity)).is_some_and(|node| node.mesh.is_some())
                } else {
                    world.has_mesh_node_for_extract(black_box(entity))
                };
                if !present {
                    return None;
                }
                world.animation_skeleton(entity).map(|skeleton| RenderSkeletalPoseExtract {
                    entity,
                    skeleton: skeleton.skeleton.id(),
                    pose: Arc::clone(pose),
                })
            }).collect()
        };
        check_rows(&filter(world, true));
        check_rows(&filter(world, false));
        for pair in 0..WARMUP_PAIRS {
            for legacy in if pair % 2 == 0 { [true, false] } else { [false, true] } {
                check_rows(&black_box(filter(black_box(world), legacy)));
            }
        }
        let measure = |world: &World, legacy: bool| {
            let start = Instant::now();
            let mut rows = Vec::new();
            for _ in 0..FILTERS_PER_SAMPLE {
                rows = filter(black_box(world), legacy);
                black_box(&rows);
            }
            (start.elapsed(), rows)
        };
        eprintln!(
            "R4 fixture: candidates={} eligible={} heavy={} primitives={} lods=4 morph_weights={} standalone_distinct_owned_buffers={} standalone_payload_copy_bytes={}",
            poses.len(), product.animation_poses.len(), HEAVY_NODES, PRIMITIVES, MORPH_WEIGHTS,
            distinct_owned_buffers, copied_payload_bytes,
        );
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let (legacy, borrowed) = if pair % 2 == 0 {
                (measure(world, true), measure(world, false))
            } else {
                let borrowed = measure(world, false);
                (measure(world, true), borrowed)
            };
            check_rows(&legacy.1);
            check_rows(&borrowed.1);
            legacy_samples.push(legacy.0.as_nanos());
            borrowed_samples.push(borrowed.0.as_nanos());
            eprintln!(
                "R4 pair={} first={} filters={} legacy_ns={} borrowed_ns={}",
                pair, if pair % 2 == 0 { "legacy" } else { "borrowed" },
                FILTERS_PER_SAMPLE, legacy.0.as_nanos(), borrowed.0.as_nanos(),
            );
        }
        legacy_samples.sort_unstable();
        borrowed_samples.sort_unstable();
        eprintln!(
            "R4 diagnostic only: median_sample_legacy_ns={} median_sample_borrowed_ns={} pairs={}; no speed threshold or product performance acceptance",
            legacy_samples[SAMPLE_PAIRS / 2], borrowed_samples[SAMPLE_PAIRS / 2], SAMPLE_PAIRS,
        );
        assert_eq!(source_state,
            (world.world_generation(), world.read_change_tick(), world.last_change_tick()));
        assert_eq!(ticks, heavy_entities.iter()
            .map(|&entity| world.component_change_ticks::<MeshRenderer>(entity))
            .collect::<Vec<_>>());
    });
    let after = RenderExtractProducer::build_render_frame_extract(&level, &context);
    check_rows(after.animation_poses.as_ref());
    assert_eq!(after.world.generation(), frame.world_generation());
    assert!(Arc::ptr_eq(
        frame.animation_poses(),
        level.frame_state_snapshot().animation_poses()
    ));
    assert!(level.with_world(|world| !world.has_pending_scene_systems()));
}
