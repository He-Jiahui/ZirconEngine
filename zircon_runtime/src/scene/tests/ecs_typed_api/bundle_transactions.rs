use crate::scene::components::{
    ActiveSelf, AnimationSkeletonComponent, ColliderComponent, Hierarchy, JointComponent,
    LocalTransform, Mesh2dComponent, MeshRenderer, Mobility, Name, PostProcessSettingsComponent,
    PostProcessVolumeComponent, RenderLayerMask, RigidBodyComponent, Sprite2dComponent,
};
use crate::scene::ecs::{Bundle, BundleStaging};
use crate::scene::{SceneError, SceneResult, World};

use super::{Health, Mana};

fn assert_default_node_component_types_remain_unregistered(world: &World) {
    assert_eq!(world.registered_component_id::<Name>(), None);
    assert_eq!(world.registered_component_id::<Hierarchy>(), None);
    assert_eq!(world.registered_component_id::<LocalTransform>(), None);
    assert_eq!(world.registered_component_id::<ActiveSelf>(), None);
    assert_eq!(world.registered_component_id::<RenderLayerMask>(), None);
    assert_eq!(world.registered_component_id::<Mobility>(), None);
    assert_eq!(world.registered_component_id::<MeshRenderer>(), None);
}

struct UnvalidatedHealthBundle;

impl Bundle for UnvalidatedHealthBundle {
    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.stage(Health(7))
    }
}

struct DuplicateHealthBundle;

impl Bundle for DuplicateHealthBundle {
    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.stage(Health(7))?;
        staging.stage(Health(9))
    }
}

struct OverwideBundle;

impl Bundle for OverwideBundle {
    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.stage(Health(7))?;
        staging.stage(Mana(9))?;
        staging.stage(ActiveSelf(true))?;
        staging.stage(RenderLayerMask::default())?;
        staging.stage(Mobility::default())?;
        staging.stage(MeshRenderer::default())?;
        staging.stage(Sprite2dComponent::default())?;
        staging.stage(RigidBodyComponent::default())?;
        staging.stage(ColliderComponent::default())
    }
}

struct RestagedHealthManaBundle;

impl Bundle for RestagedHealthManaBundle {
    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.stage(Health(7))?;
        staging.validate_final_state()?;
        staging.stage(Mana(9))
    }
}

struct ValidatedHealthManaBundle;

impl Bundle for ValidatedHealthManaBundle {
    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.stage(Health(7))?;
        staging.stage(Mana(9))?;
        staging.validate_final_state()
    }
}

#[test]
fn bundle_commit_requires_final_state_validation_before_publishing() {
    let mut world = World::empty();
    let generation_before = world.world_generation();

    let error = world
        .spawn(UnvalidatedHealthBundle)
        .expect_err("custom bundles must validate their final state before commit");

    assert!(matches!(error, SceneError::BundleFinalStateNotValidated));
    assert!(world.node_record(1).is_none());
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_default_node_component_types_remain_unregistered(&world);
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn bundle_stage_after_validation_requires_a_new_final_state_validation() {
    let mut world = World::empty();
    let generation_before = world.world_generation();

    let error = world
        .spawn(RestagedHealthManaBundle)
        .expect_err("a stage after validation must invalidate the commit gate");

    assert!(matches!(error, SceneError::BundleFinalStateNotValidated));
    assert!(world.node_record(1).is_none());
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(world.registered_component_id::<Mana>(), None);
    assert_default_node_component_types_remain_unregistered(&world);
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn duplicate_bundle_stage_after_a_successful_stage_publishes_no_partial_diagnostics() {
    let mut world = World::empty();
    world.reset_ecs_frame_performance_diagnostics();
    let generation_before = world.world_generation();

    let error = world
        .spawn(DuplicateHealthBundle)
        .expect_err("duplicate staged component types must reject before commit");

    assert!(matches!(error, SceneError::DuplicateBundleComponentType));
    assert!(world.node_record(1).is_none());
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_default_node_component_types_remain_unregistered(&world);
    assert_eq!(world.world_generation(), generation_before);
    assert_eq!(
        world
            .ecs_frame_performance_diagnostics()
            .bundle_transactions,
        Default::default(),
        "a rejected staged bundle must not publish partial storage, lifecycle, or transaction metrics"
    );
}

#[test]
fn overwide_bundle_stage_after_eight_successes_publishes_no_partial_diagnostics() {
    let mut world = World::empty();
    world.reset_ecs_frame_performance_diagnostics();
    let generation_before = world.world_generation();

    let error = world
        .spawn(OverwideBundle)
        .expect_err("the ninth staged component must reject before commit");

    assert!(matches!(
        error,
        SceneError::BundleComponentLimitExceeded { limit: 8 }
    ));
    assert!(world.node_record(1).is_none());
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(world.registered_component_id::<Mana>(), None);
    assert_default_node_component_types_remain_unregistered(&world);
    assert_eq!(world.world_generation(), generation_before);
    assert_eq!(
        world
            .ecs_frame_performance_diagnostics()
            .bundle_transactions,
        Default::default(),
        "a capacity-rejected staged bundle must not publish partial storage, lifecycle, or transaction metrics"
    );
}

#[test]
fn custom_bundle_publishes_the_exact_values_it_staged() {
    let mut world = World::empty();

    let entity = world
        .spawn(ValidatedHealthManaBundle)
        .expect("validated custom bundle must publish its staged values");

    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get::<Mana>(entity), Some(&Mana(9)));
}

#[test]
fn runtime_only_post_process_components_survive_generic_storage_clone() {
    let mut world = World::empty();
    let entity = world
        .spawn(())
        .expect("default node should publish before post-process components");
    let settings = PostProcessSettingsComponent::default();
    let volume = PostProcessVolumeComponent::default();

    world
        .insert(entity, settings.clone())
        .expect("post-process settings should use component storage");
    world
        .insert(entity, volume.clone())
        .expect("post-process volume should use component storage");

    let cloned = world.clone();
    assert_eq!(
        cloned.get::<PostProcessSettingsComponent>(entity),
        Some(&settings)
    );
    assert_eq!(
        cloned.get::<PostProcessVolumeComponent>(entity),
        Some(&volume)
    );
}

#[test]
fn persistent_physics_components_use_generic_storage_across_clone_and_serde() {
    let mut world = World::empty();
    let entity = world
        .spawn(())
        .expect("default node should publish before physics components");
    let rigid_body = RigidBodyComponent::default();
    let collider = ColliderComponent::default();
    let joint = JointComponent::default();

    world
        .insert(entity, rigid_body.clone())
        .expect("rigid body should use component storage");
    world
        .insert(entity, collider.clone())
        .expect("collider should use component storage");
    world
        .insert(entity, joint.clone())
        .expect("joint should use component storage");

    let cloned = world.clone();
    let decoded: World = serde_json::from_str(
        &serde_json::to_string(&world).expect("world persistence must serialize physics storage"),
    )
    .expect("world persistence must restore physics storage");

    for restored in [&cloned, &decoded] {
        assert_eq!(
            restored.get::<RigidBodyComponent>(entity),
            Some(&rigid_body)
        );
        assert_eq!(restored.get::<ColliderComponent>(entity), Some(&collider));
        assert_eq!(restored.get::<JointComponent>(entity), Some(&joint));
    }
}

#[test]
fn node_record_restore_publishes_the_final_physics_archetype_signature() {
    let mut world = World::empty();
    let entity = world
        .spawn(())
        .expect("default node should exist before record restore");
    let mut record = world
        .node_record(entity)
        .expect("default node should project to a record");
    world.remove_entity(entity).unwrap();

    record.rigid_body = Some(RigidBodyComponent::default());
    record.collider = Some(ColliderComponent::default());
    record.joint = Some(JointComponent::default());
    world
        .insert_node_record(record)
        .expect("record restore should publish physics storage once");

    type PhysicsData<'query> = (
        crate::scene::EntityId,
        &'query RigidBodyComponent,
        &'query ColliderComponent,
        &'query JointComponent,
    );
    let rows = world
        .query::<PhysicsData<'static>>()
        .iter(&world)
        .map(|(entity, _, _, _)| entity)
        .collect::<Vec<_>>();

    assert_eq!(rows, vec![entity]);
}

#[test]
fn persistent_render_2d_components_use_generic_storage_across_clone_serde_and_records() {
    let mut world = World::empty();
    let entity = world
        .spawn(())
        .expect("default node should publish before 2D render components");
    let sprite = Sprite2dComponent::default();
    let mesh_2d = Mesh2dComponent::default();

    world
        .insert(entity, sprite.clone())
        .expect("sprite should use component storage");
    world
        .insert(entity, mesh_2d.clone())
        .expect("2D mesh should use component storage");

    let cloned = world.clone();
    let decoded: World = serde_json::from_str(
        &serde_json::to_string(&world).expect("world persistence must serialize 2D render storage"),
    )
    .expect("world persistence must restore 2D render storage");

    for restored in [&cloned, &decoded] {
        assert_eq!(restored.get::<Sprite2dComponent>(entity), Some(&sprite));
        assert_eq!(restored.get::<Mesh2dComponent>(entity), Some(&mesh_2d));
    }

    let record = world
        .node_record(entity)
        .expect("2D render components must project to a node record");
    world.remove_entity(entity).unwrap();
    world
        .insert_node_record(record)
        .expect("record restore should stage generic 2D render components before final publish");

    type Render2dData<'query> = (
        crate::scene::EntityId,
        &'query Sprite2dComponent,
        &'query Mesh2dComponent,
    );
    let rows = world
        .query::<Render2dData<'static>>()
        .iter(&world)
        .map(|(entity, _, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(rows, vec![entity]);
}

#[test]
fn persistent_render_2d_components_do_not_retain_world_map_owners() {
    let world_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("world.rs"),
    )
    .expect("read World storage owner source");
    let fixed_components_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("fixed_components.rs"),
    )
    .expect("read fixed component adapter source");

    for retired_owner in ["pub(super) sprite_2d:", "pub(super) mesh_2d:"] {
        assert!(
            !world_source.contains(retired_owner),
            "World must not retain the retired 2D render component map owner: {retired_owner}"
        );
    }
    for retired_adapter in [
        "fixed_component_map!(Sprite2dComponent, sprite_2d)",
        "fixed_component_map!(Mesh2dComponent, mesh_2d)",
    ] {
        assert!(
            !fixed_components_source.contains(retired_adapter),
            "fixed-component adapter must not reintroduce a 2D render map owner: {retired_adapter}"
        );
    }
    assert!(
        world_source.contains("persistent_render_2d_component_snapshot"),
        "World persistence must project 2D render values from generic component storage"
    );
}

#[test]
fn persistent_animation_skeleton_uses_generic_storage_across_clone_serde_and_records() {
    let mut world = World::empty();
    let entity = world
        .spawn(())
        .expect("default node should publish before animation skeleton");
    let skeleton = AnimationSkeletonComponent {
        skeleton: crate::core::resource::ResourceHandle::<
            crate::core::resource::AnimationSkeletonMarker,
        >::new(crate::core::resource::ResourceId::from_stable_label(
            "res://animation/runtime08-skeleton.zranim",
        )),
    };
    world
        .insert(entity, skeleton.clone())
        .expect("animation skeleton should use component storage");

    let cloned = world.clone();
    let decoded: World = serde_json::from_str(
        &serde_json::to_string(&world)
            .expect("world persistence must serialize animation skeleton storage"),
    )
    .expect("world persistence must restore animation skeleton storage");
    for restored in [&cloned, &decoded] {
        assert_eq!(
            restored.get::<AnimationSkeletonComponent>(entity),
            Some(&skeleton)
        );
    }

    let record = world
        .node_record(entity)
        .expect("animation skeleton must project to a node record");
    world.remove_entity(entity).unwrap();
    world
        .insert_node_record(record)
        .expect("record restore should stage animation skeleton before final publish");

    type SkeletonData<'query> = (crate::scene::EntityId, &'query AnimationSkeletonComponent);
    let rows = world
        .query::<SkeletonData<'static>>()
        .iter(&world)
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(rows, vec![entity]);
}

#[test]
fn persistent_animation_runtime_components_do_not_retain_world_map_owners() {
    let world_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("world.rs"),
    )
    .expect("read World storage owner source");
    let fixed_components_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("fixed_components.rs"),
    )
    .expect("read fixed component adapter source");

    for legacy_field in [
        "pub(super) animation_skeletons:",
        "pub(super) animation_players:",
        "pub(super) animation_sequence_players:",
        "pub(super) animation_graph_players:",
        "pub(super) animation_state_machine_players:",
    ] {
        assert!(
            !world_source.contains(legacy_field),
            "World must not retain a live animation runtime map owner: {legacy_field}",
        );
    }
    for legacy_adapter in [
        "fixed_component_map!(AnimationSkeletonComponent, animation_skeletons)",
        "fixed_component_map!(AnimationPlayerComponent, animation_players)",
        "fixed_component_map!(AnimationSequencePlayerComponent, animation_sequence_players)",
        "fixed_component_map!(AnimationGraphPlayerComponent, animation_graph_players)",
        "fixed_component_map!(AnimationStateMachinePlayerComponent, animation_state_machine_players)",
    ] {
        assert!(
            !fixed_components_source.contains(legacy_adapter),
            "fixed component adapters must not retain an animation runtime map owner: {legacy_adapter}",
        );
    }
    assert!(
        world_source.contains("persistent_animation_runtime_component_snapshot"),
        "World persistence must project animation runtime values from generic component storage"
    );
}

#[test]
fn persistent_physics_components_do_not_retain_world_map_owners() {
    let world_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("world.rs"),
    )
    .expect("read World storage owner source");
    let fixed_components_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("fixed_components.rs"),
    )
    .expect("read fixed component adapter source");
    let physics_snapshot = fixed_components_source
        .split("fn persistent_component_snapshot")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(super) fn validate_fixed_component")
                .next()
        })
        .expect("read persistent physics snapshot");

    for retired_owner in [
        "pub(super) rigid_bodies:",
        "pub(super) colliders:",
        "pub(super) joints:",
    ] {
        assert!(
            !world_source.contains(retired_owner),
            "World must not retain the retired physics component map owner: {retired_owner}"
        );
    }
    for retired_adapter in [
        "fixed_component_map!(RigidBodyComponent, rigid_bodies)",
        "fixed_component_map!(ColliderComponent, colliders)",
        "fixed_component_map!(JointComponent, joints)",
    ] {
        assert!(
            !fixed_components_source.contains(retired_adapter),
            "fixed-component adapter must not reintroduce a physics map owner: {retired_adapter}"
        );
    }
    assert!(
        world_source.contains("impl Serialize for World")
            && world_source.contains("persistent_physics_component_snapshot"),
        "World persistence must project physics values from generic component storage"
    );
    assert!(
        fixed_components_source.contains("append_storage_presence!(RigidBodyComponent)")
            && fixed_components_source.contains("append_storage_presence!(ColliderComponent)")
            && fixed_components_source.contains("append_storage_presence!(JointComponent)"),
        "record restore must include generic-only physics components in its final archetype signature"
    );
    assert!(
        physics_snapshot.contains("for_each_table_component")
            && !physics_snapshot.contains("self.entities"),
        "persistent physics snapshots must visit populated storage rows, not scan every world entity"
    );
}

#[test]
fn unit_bundle_spawn_validates_and_publishes_the_default_node_signature() {
    let mut world = World::empty();

    let entity = world
        .spawn(())
        .expect("the unit bundle must validate the default node before publishing it");

    assert_eq!(entity, 1);
    assert!(world.node_record(entity).is_some());
    assert!(world.contains_component::<Name>(entity));
    assert!(world.contains_component::<LocalTransform>(entity));
}

#[test]
fn bundle_commit_derives_one_final_archetype_signature_from_staged_metadata() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("bundle_transaction.rs"),
    )
    .expect("read bundle transaction source");
    let finish = source
        .split("pub(crate) fn finish")
        .nth(1)
        .and_then(|text| text.split("fn validate_commit_invariants").next())
        .expect("read bundle transaction finish body");
    let final_signature = source
        .split("fn final_archetype_signature")
        .nth(1)
        .and_then(|text| text.split("fn validate_final_state").next())
        .expect("read final archetype signature body");

    assert!(
        finish.contains("let final_signature =")
            && finish.contains("self.final_archetype_signature(")
            && finish
                .contains("let final_archetype_transition = current_signature != final_signature")
            && finish
                .contains("transition_entity_archetype_row(self.entity, final_signature, updates)")
            && finish.contains("table_values")
            && !finish.contains("publish_entity_archetype_signature(")
            && !finish.contains("refresh_entity_archetype(self.entity)"),
        "bundle commit must apply its precomputed final signature through one complete row transition instead of rescanning component storage"
    );
    assert!(
        final_signature.contains("default_value.component_id()")
            && final_signature.contains("default_value.storage_type()")
            && final_signature.contains("prepared.preflight.component_id")
            && final_signature.contains("prepared.preflight.storage_type")
            && final_signature.contains("with_component_added"),
        "bundle final signature must merge default and staged component metadata by storage partition"
    );
}

#[test]
fn typed_storage_projection_rebuild_publishes_one_complete_row_per_entity() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("projection_rebuild.rs"),
    )
    .expect("read component projection rebuild source");
    let projection = source
        .split("pub(super) fn rebuild_component_storage_projection_with_owned_components")
        .nth(1)
        .expect("read typed storage projection rebuild");

    assert!(
        projection.contains("self.component_storage = Default::default();")
            && projection.contains("rows.insert(entity, self.begin_empty_component_row());")
            && projection.contains("self.stage_component_row_value_with_id(")
            && projection.contains("stage_values(self, &mut rows, persistent_entity_core.names);")
            && projection.contains("persistent_animation_runtime.state_machine_players")
            && projection.contains("self.reset_archetype_index_for_projection();")
            && projection.contains("self.commit_rebuilt_component_row(entity, row);")
            && !projection.contains("self.rebuild_archetype_index();")
            && !projection.contains("self.commit_component_row(entity, row, false);")
            && !projection.contains("insert_dynamic_component_presence(")
            && !projection.contains("restore_persistent_"),
        "storage projection rebuild must append each aggregate directly to its final archetype exactly once"
    );
}

#[test]
fn fixed_record_restore_uses_the_shared_complete_row_transaction() {
    let row_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("component_row.rs"),
    )
    .expect("read component row source");
    let records_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("records.rs"),
    )
    .expect("read record restore source");
    let restore = records_source
        .split("pub(super) fn insert_prevalidated_node_record")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn register_prevalidated_node_identity_without_components")
                .next()
        })
        .expect("read fixed record row transaction");

    assert!(
        restore.contains("let mut row = self.begin_component_row(record.id);")
            && restore.contains("self.stage_component_row_value(&mut row, Name(record.name));")
            && restore.contains("self.stage_component_row_value(&mut row, record.mobility);")
            && restore.contains("self.commit_component_row(record.id, row, true);")
            && !restore.contains("insert_rebuilt_fixed_component_presence_without_archetype")
            && !restore.contains("rebuild_fixed_component_presence_into_final_archetype"),
        "record restore must aggregate its complete fixed row before one structural commit"
    );
    assert!(
        row_source.contains("validate_transition(")
            && row_source
                .contains("transition_entity_archetype_row(entity, signature, dense_updates)")
            && row_source.contains("for (component_id, sparse) in sparse_values")
            && !row_source.contains("component_ids_for_entity_by_storage"),
        "shared component row commit must preflight the final schema and publish sparse plus dense values without storage scans"
    );
}

#[test]
fn runtime_only_post_process_components_do_not_retain_fixed_map_owners() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let world_source = std::fs::read_to_string(manifest_root.join("src/scene/world/world.rs"))
        .expect("read world owner source");
    let fixed_components_source = std::fs::read_to_string(
        manifest_root.join("src/scene/world/typed_api/fixed_components.rs"),
    )
    .expect("read fixed component source");
    let runtime_only_snapshot = fixed_components_source
        .split("fn runtime_only_component_snapshot")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(super) fn validate_fixed_component")
                .next()
        })
        .expect("read runtime-only post-process snapshot");

    for retired_owner in [
        "post_process_settings: HashMap<EntityId, PostProcessSettingsComponent>",
        "post_process_volumes: HashMap<EntityId, PostProcessVolumeComponent>",
        "fixed_component_map!(PostProcessSettingsComponent, post_process_settings)",
        "fixed_component_map!(PostProcessVolumeComponent, post_process_volumes)",
    ] {
        assert!(
            !world_source.contains(retired_owner)
                && !fixed_components_source.contains(retired_owner),
            "runtime-only post-process component must not retain fixed-map owner `{retired_owner}`"
        );
    }
    assert!(
        !fixed_components_source.contains("fixed_component_map!")
            && !fixed_components_source.contains("TypeId::of::<PostProcessSettingsComponent>()")
            && !fixed_components_source.contains("TypeId::of::<PostProcessVolumeComponent>()"),
        "runtime-only post-process components must stay on the common generic storage path"
    );
    assert!(
        runtime_only_snapshot.contains("for_each_table_component")
            && !runtime_only_snapshot.contains("self.entities"),
        "runtime-only component snapshots must visit populated storage rows, not scan every world entity"
    );
}
