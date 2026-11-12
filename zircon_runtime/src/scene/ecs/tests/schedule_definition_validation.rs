use super::*;
use crate::scene::ecs::{FunctionRuntimeSceneSystem, SceneScheduleStagePlan, Schedule};

fn descriptor(id: &str, order: i32) -> SceneSystemDescriptor {
    SceneSystemDescriptor::new(id, SystemStage::Update, InternalSceneSystem::WorldTransform)
        .with_order(order)
}

#[test]
fn persisted_registry_rejects_duplicate_and_invalid_system_ids() {
    let duplicate = serde_json::to_value([descriptor("same", 0), descriptor("same", 1)]).unwrap();
    let error = serde_json::from_value::<SceneSystemRegistry>(duplicate).unwrap_err();
    assert!(error.to_string().contains("already registered"));
    for id in ["", " ", " surrounded "] {
        let invalid = serde_json::to_value([descriptor(id, 0)]).unwrap();
        assert!(serde_json::from_value::<SceneSystemRegistry>(invalid).is_err());
    }
}

#[test]
fn persisted_registry_uses_registration_order_and_round_trips() {
    let document = serde_json::to_value([
        descriptor("later", 7),
        descriptor("b", 2),
        descriptor("a", 2),
    ])
    .unwrap();
    let registry: SceneSystemRegistry = serde_json::from_value(document).unwrap();
    let ids = registry
        .systems()
        .iter()
        .map(|row| row.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["a", "b", "later"]);
    let decoded: SceneSystemRegistry =
        serde_json::from_value(serde_json::to_value(&registry).unwrap()).unwrap();
    assert_eq!(decoded.systems(), registry.systems());
}

#[test]
fn persisted_schedule_rejects_repeated_stages_before_compiling_buckets() {
    let document = serde_json::json!({ "stages": ["Update", "Update"], "systems": [] });
    let error = serde_json::from_value::<Schedule>(document).unwrap_err();
    assert!(error.to_string().contains("appears more than once"));
    assert_eq!(
        SceneScheduleStagePlan::from_registry(
            &[SystemStage::Update, SystemStage::Update],
            &SceneSystemRegistry::new(),
        )
        .unwrap_err(),
        ScheduleError::DuplicateStage(SystemStage::Update),
    );
}

// R03-01 准入回归（官方评审 R03-01 第 3.2 节，源码注释审计 CR-SCENE-ECS-0002）。Unreal 对应物：
// - Engine/Source/Developer/MassEntityTestSuite/Private/MassDependencyTest.cpp：
//   FBadInput_Empty 与 FBadInput_Duplicates 在求解边界处理坏输入，不把它交给执行阶段。
// - Engine/Source/Runtime/Engine/Private/LevelTick.cpp：UWorld::Tick 以固定顺序调用
//   UWorld::RunTickGroup，后者用 check(TickGroup == Group) 断言顺序没有被打乱。
// - Engine/Source/Runtime/MassEntity/Private/MassProcessingPhaseManager.cpp：
//   FMassProcessingPhaseManager::RegisterDynamicProcessor 不接收阶段超出驱动范围的 processor；
//   MassDynamicProcessorsTest.cpp 的 FDynamicProcessorsAddTrivial 验证合法注册会进入执行。

fn schedule_document(stages: &[SystemStage]) -> serde_json::Value {
    serde_json::json!({ "stages": stages, "systems": [] })
}

fn runtime_system(id: &str, stage: SystemStage) -> BoxedRuntimeSceneSystem {
    Box::new(FunctionRuntimeSceneSystem::new(
        SceneSystemMetadata::new(id, stage, 0),
        |_| Ok(()),
    ))
}

fn assert_stage_list_rejected(stages: &[SystemStage], expected: ScheduleError, message: &str) {
    assert_eq!(
        SceneScheduleStagePlan::from_registry(stages, &SceneSystemRegistry::new()).unwrap_err(),
        expected,
    );
    let error = serde_json::from_value::<Schedule>(schedule_document(stages)).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
}

fn assert_stage_list_admitted(stages: &[SystemStage]) {
    let plan = SceneScheduleStagePlan::from_registry(stages, &SceneSystemRegistry::new()).unwrap();
    assert_eq!(plan.stages(), stages);
    let schedule: Schedule = serde_json::from_value(schedule_document(stages)).unwrap();
    assert_eq!(schedule.stages(), stages);
    assert_eq!(schedule.build_receipt(), plan.build_receipt());
}

#[test]
fn schedule_admission_rejects_stage_lists_against_the_documented_order() {
    let message = "against the documented stage order";
    assert_stage_list_rejected(
        &[SystemStage::Update, SystemStage::PreUpdate],
        ScheduleError::StageOutOfOrder {
            previous: SystemStage::Update,
            stage: SystemStage::PreUpdate,
        },
        message,
    );
    assert_stage_list_rejected(
        &[
            SystemStage::FixedPostUpdate,
            SystemStage::FixedUpdate,
            SystemStage::FixedFirst,
        ],
        ScheduleError::StageOutOfOrder {
            previous: SystemStage::FixedPostUpdate,
            stage: SystemStage::FixedUpdate,
        },
        message,
    );
    // 固定步组被普通阶段穿插：WorldDriver 会把整组提前到 FixedFirst 处执行。
    assert_stage_list_rejected(
        &[
            SystemStage::FixedFirst,
            SystemStage::Update,
            SystemStage::FixedUpdate,
            SystemStage::FixedPostUpdate,
        ],
        ScheduleError::StageOutOfOrder {
            previous: SystemStage::Update,
            stage: SystemStage::FixedUpdate,
        },
        message,
    );
}

#[test]
fn schedule_admission_rejects_partial_fixed_stage_groups() {
    let message = "list the whole fixed group or none of it";
    assert_stage_list_rejected(
        &[SystemStage::FixedUpdate],
        ScheduleError::IncompleteFixedStageGroup {
            present: SystemStage::FixedUpdate,
            missing: SystemStage::FixedFirst,
        },
        message,
    );
    assert_stage_list_rejected(
        &[SystemStage::FixedPostUpdate],
        ScheduleError::IncompleteFixedStageGroup {
            present: SystemStage::FixedPostUpdate,
            missing: SystemStage::FixedFirst,
        },
        message,
    );
    // 组内任何一个阶段缺席都算残缺，包括只缺组尾。
    assert_stage_list_rejected(
        &[SystemStage::FixedFirst, SystemStage::FixedUpdate],
        ScheduleError::IncompleteFixedStageGroup {
            present: SystemStage::FixedFirst,
            missing: SystemStage::FixedPostUpdate,
        },
        message,
    );
}

#[test]
fn schedule_admission_rejects_systems_whose_stage_is_not_listed() {
    let stages = [SystemStage::PreUpdate, SystemStage::Update];
    assert_eq!(
        SceneScheduleStagePlan::from_registry(
            &stages,
            &SceneSystemRegistry::with_builtin_systems(),
        )
        .unwrap_err(),
        ScheduleError::SystemStageNotScheduled {
            system_id: "zircon.scene.events_update_all".to_string(),
            stage: SystemStage::First,
        },
    );
    // 文档省略 systems 时使用内建系统，它们的阶段同样必须在列表内。
    let document = serde_json::json!({ "stages": stages });
    let error = serde_json::from_value::<Schedule>(document).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("which the schedule does not list"),
        "{error}"
    );

    let mut registry = SceneSystemRegistry::new();
    registry
        .register_boxed_runtime_system(runtime_system(
            "r03.runtime.post-update",
            SystemStage::PostUpdate,
        ))
        .unwrap();
    assert_eq!(
        SceneScheduleStagePlan::from_registry(&stages, &registry).unwrap_err(),
        ScheduleError::SystemStageNotScheduled {
            system_id: "r03.runtime.post-update".to_string(),
            stage: SystemStage::PostUpdate,
        },
    );
}

#[test]
fn schedule_admission_accepts_ordered_stage_subsets() {
    assert_stage_list_admitted(&[]);
    assert_stage_list_admitted(&[SystemStage::PreUpdate, SystemStage::Update]);
    assert_stage_list_admitted(&SystemStage::FIXED_LOOP);
    assert_stage_list_admitted(&[
        SystemStage::First,
        SystemStage::FixedFirst,
        SystemStage::FixedUpdate,
        SystemStage::FixedPostUpdate,
        SystemStage::Last,
    ]);
    assert_stage_list_admitted(&SystemStage::ORDER);
}

#[test]
fn default_schedule_still_passes_admission_and_round_trips() {
    let schedule = Schedule::default();
    assert_eq!(schedule.stages(), SystemStage::ORDER.as_slice());
    let decoded: Schedule =
        serde_json::from_value(serde_json::to_value(&schedule).unwrap()).unwrap();
    assert_eq!(decoded, schedule);
    assert_eq!(decoded.build_receipt(), schedule.build_receipt());
}

#[test]
fn rejected_registration_outside_the_stage_list_keeps_the_installed_plan() {
    let stages = [SystemStage::PreUpdate, SystemStage::Update];
    let mut schedule: Schedule = serde_json::from_value(schedule_document(&stages)).unwrap();
    let receipt = schedule.build_receipt();

    let error = schedule
        .register_system(SceneSystemDescriptor::new(
            "r03.internal.late",
            SystemStage::PostUpdate,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap_err();
    assert_eq!(
        error,
        ScheduleError::SystemStageNotScheduled {
            system_id: "r03.internal.late".to_string(),
            stage: SystemStage::PostUpdate,
        },
    );
    let error = schedule
        .register_boxed_runtime_system(runtime_system("r03.runtime.late", SystemStage::Last))
        .unwrap_err();
    assert_eq!(
        error,
        ScheduleError::SystemStageNotScheduled {
            system_id: "r03.runtime.late".to_string(),
            stage: SystemStage::Last,
        },
    );
    assert!(schedule.systems().is_empty());
    assert_eq!(schedule.stage_plan().stages(), stages.as_slice());
    assert_eq!(schedule.build_receipt(), receipt);

    // 被拒的注册不留痕迹：同一 ID 改到列表内的阶段后可以注册并进入执行图。
    schedule
        .register_system(SceneSystemDescriptor::new(
            "r03.internal.late",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap();
    schedule
        .register_boxed_runtime_system(runtime_system("r03.runtime.late", SystemStage::Update))
        .unwrap();
    assert_eq!(
        schedule.build_receipt().execution_step_count(),
        receipt.execution_step_count() + 2,
    );
}
