//! R03-01 adaptations of existing Unreal MassEntityTestSuite tests:
//! - Engine/Source/Developer/MassEntityTestSuite/Private/MassProcessingPhasesTest.cpp:
//!   FTestSetupTest verifies configured processors are ticked by the phase manager.
//! - Engine/Source/Developer/MassEntityTestSuite/Private/MassCompositeProcessorTest.cpp:
//!   FCompositeProcessorTest_MultipleSubProcessors verifies every configured child runs.
//! - Engine/Source/Developer/MassEntityTestSuite/Private/MassDependencyTest.cpp:
//!   FSimpleDependency verifies the resolved execution order.
//!
//! These mirror execution completeness and plan order using ZE's existing WorldDriver.
//! Unreal does not expose ZE's serialized stage vectors. Custom-stage policy remains
//! open: admission may reject or canonicalize a candidate, but an admitted plan must
//! describe the callbacks that run. No particular rejection policy is required here.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::{CoreRuntime, TimePolicyTransaction};
use crate::scene::ecs::{FunctionRuntimeSceneSystem, SceneSystemMetadata, Schedule, SystemStage};
use crate::scene::{create_default_level, module_descriptor};

#[test]
fn default_schedule_executes_all_stages_in_declared_order() {
    assert_driver_follows_admitted_plan(Schedule::default());
}

#[test]
fn canonical_persisted_stage_list_executes_every_stage_once() {
    let schedule = persisted_schedule(&SystemStage::ORDER)
        .expect("the canonical stage list must remain admissible");
    assert_driver_follows_admitted_plan(schedule);
}

#[test]
fn reversed_nonfixed_stages_agree_with_the_admitted_plan() {
    assert_custom_admission_agrees_with_driver(&[SystemStage::Update, SystemStage::PreUpdate]);
}

#[test]
fn fixed_update_without_fixed_first_agrees_with_the_admitted_plan() {
    assert_custom_admission_agrees_with_driver(&[SystemStage::FixedUpdate]);
}

#[test]
fn fixed_post_update_without_fixed_first_agrees_with_the_admitted_plan() {
    assert_custom_admission_agrees_with_driver(&[SystemStage::FixedPostUpdate]);
}

#[test]
fn reordered_fixed_group_agrees_with_the_admitted_plan() {
    assert_custom_admission_agrees_with_driver(&[
        SystemStage::FixedPostUpdate,
        SystemStage::FixedUpdate,
        SystemStage::FixedFirst,
    ]);
}

#[test]
fn interleaved_fixed_group_agrees_with_the_admitted_plan() {
    assert_custom_admission_agrees_with_driver(&[
        SystemStage::FixedFirst,
        SystemStage::Update,
        SystemStage::FixedUpdate,
        SystemStage::FixedPostUpdate,
    ]);
}

fn persisted_schedule(stages: &[SystemStage]) -> Result<Schedule, serde_json::Error> {
    serde_json::from_value(serde_json::json!({ "stages": stages, "systems": [] }))
}

fn assert_custom_admission_agrees_with_driver(stages: &[SystemStage]) {
    // Rejection is a valid way to keep unsupported custom plans out of the driver.
    // The required canonical controls above prevent a broken serializer from
    // satisfying the entire batch by rejecting every candidate.
    if let Ok(schedule) = persisted_schedule(stages) {
        assert_driver_follows_admitted_plan(schedule);
    }
}

fn assert_driver_follows_admitted_plan(schedule: Schedule) {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    level
        .apply_time_policy(TimePolicyTransaction::new(
            level
                .time_policy()
                .with_fixed_timestep(Duration::from_millis(10)),
        ))
        .unwrap();

    let admitted_stages = schedule.stages().to_vec();
    assert_eq!(schedule.stage_plan().stages(), admitted_stages.as_slice());
    let original_step_count = schedule.build_receipt().execution_step_count();
    let observed = Arc::new(Mutex::new(Vec::new()));
    level.with_world_mut(|world| {
        *world.schedule_mut() = schedule;
        for stage in admitted_stages.iter().copied() {
            let observed = Arc::clone(&observed);
            world
                .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                    SceneSystemMetadata::new(format!("r03.stage-contract.{stage:?}"), stage, 0),
                    move |context| {
                        assert_eq!(context.tick().stage(), stage);
                        observed.lock().unwrap().push(stage);
                        Ok(())
                    },
                )))
                .expect("a recorder for an admitted stage must register");
        }
        assert_eq!(
            world.schedule().stage_plan().stages(),
            admitted_stages.as_slice(),
        );
        assert_eq!(
            world.schedule().build_receipt().execution_step_count(),
            original_step_count + admitted_stages.len() as u64,
            "every recorder must be present in the compiled execution graph",
        );
    });

    // Exactly one due fixed step makes the expected trace one callback per stage,
    // so dropping a fixed stage or expanding the group in a different order is visible.
    let advance = runtime.advance_time_by(Duration::from_millis(10), 1);
    level
        .tick(&runtime.handle(), advance)
        .expect("an admitted plan with infallible recorders must tick successfully");

    assert_eq!(
        observed.lock().unwrap().as_slice(),
        admitted_stages.as_slice(),
        "WorldDriver must execute every admitted stage callback in compiled plan order",
    );
}
