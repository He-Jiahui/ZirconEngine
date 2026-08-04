use crate::scene::components::{LocalTransform, Name};
use crate::scene::ecs::{Bundle, BundleStaging};
use crate::scene::{SceneError, SceneResult, World};

use super::{Health, Mana};

struct UnvalidatedHealthBundle;

impl Bundle for UnvalidatedHealthBundle {
    fn insert_into(self, world: &mut World, entity: u64) -> SceneResult<()> {
        let mut transaction = world.begin_bundle_insertion(entity)?;
        self.stage_into(&mut transaction)?;
        transaction.finish()
    }

    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.stage(&Health(7))?;
        staging.commit(Health(7))
    }
}

struct IncompleteHealthManaBundle;

impl Bundle for IncompleteHealthManaBundle {
    fn insert_into(self, world: &mut World, entity: u64) -> SceneResult<()> {
        let mut transaction = world.begin_bundle_insertion(entity)?;
        self.stage_into(&mut transaction)?;
        transaction.finish()
    }

    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.stage(&Health(7))?;
        staging.stage(&Mana(9))?;
        staging.validate_final_state()?;
        staging.commit(Health(7))
    }
}

struct RestagedHealthManaBundle;

impl Bundle for RestagedHealthManaBundle {
    fn insert_into(self, world: &mut World, entity: u64) -> SceneResult<()> {
        let mut transaction = world.begin_bundle_insertion(entity)?;
        self.stage_into(&mut transaction)?;
        transaction.finish()
    }

    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.stage(&Health(7))?;
        staging.validate_final_state()?;
        staging.stage(&Mana(9))?;
        staging.commit(Health(7))
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
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn incomplete_custom_bundle_commit_leaves_the_world_unpublished() {
    let mut world = World::empty();
    let generation_before = world.world_generation();

    let error = world
        .spawn(IncompleteHealthManaBundle)
        .expect_err("a custom bundle cannot publish only a preflighted prefix");

    assert!(matches!(
        error,
        SceneError::BundleCommitIncomplete {
            staged: 2,
            committed: 1,
        }
    ));
    assert!(world.node_record(1).is_none());
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(world.registered_component_id::<Mana>(), None);
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
    assert_eq!(world.world_generation(), generation_before);
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
