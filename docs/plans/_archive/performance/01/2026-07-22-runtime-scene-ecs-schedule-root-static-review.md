---
related_code:
  - zircon_runtime/src/scene/ecs
  - zircon_runtime/src/scene/module/world_driver.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/single_threaded.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_schedule
  - zircon_runtime/src/scene/tests/ecs_schedule_parallel_executor_structure.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop/runtime_anchors.rs
  - current-source Windows zircon_runtime ECS schedule tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene ECS root与schedule逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/ecs/*.rs`当前 **16/16** 个Rust文件、**2,638** 行已逐文件阅读；覆盖bundle、lifecycle/removal、frame diagnostics、system registry/set、schedule/stage plan/runner/conflict graph/parallel executor。外部`scene/tests/ecs_schedule/**`当前9文件、58 tests和结构守卫已纳入动态验收；`ecs/**`其余110文件另行按storage/query/system/events拆分。

## 已直接修复

`SceneScheduleRunner::run_stage`原为每个stage、每帧执行`format!("runtime_frame_schedule_stage.{stage:?}")`并进入dynamic profiling宏；Tracy构建即使capture inactive也先创建String。现改为9个穷尽match的`&'static str`和`profile_scope!`，保留原trace名称。两个源码守卫先RED后GREEN并锁定全部stage label及禁止动态format，rustfmt/diff通过，归PERF-MVP-477。

## 产品调度仍为串行World临界区

`WorldDriver`生产链只调用`SceneScheduleRunner::run_stage`。runner按sorted step串行执行Internal/Native/Runtime/Hook，每步一次或多次`LevelSystem::with_world_mut`，并在多数step后同步`apply_deferred`；全stage没有调用`ScheduleParallelExecutor`。现有conflict graph和parallel executor只有测试consumer，generic executor每run还为batch clone system-id Vec、分配Arc<Mutex<Option<Result>>>、调度batch job并tail wait，且task closure没有sound的disjoint World/storage borrow合同，不能直接接入产品。

PERF-MVP-478以completed-plan failure回路交接Runtime03，Runtime08/11共同验收：compile schedule时冻结dense dependency/conflict/exclusive/main-thread metadata与direct system slots；frame执行从World取得sound disjoint storage/resource views或等价分区，worker各自拥有deferred buffer，barrier处确定性merge；stable frame不重建graph/batch/ID/Arc/Mutex，exclusive/hook/render extract仍显式回主线程lane。

## Bundle逐组件迁移

tuple `Bundle`的1..8元实现只展开多次`world.insert(entity, component)?`；`World::spawn_bundle_at`先spawn empty再调用该展开。每次typed insert都可能更新ComponentStorage、archetype和lifecycle，因此N组件bundle可产生N个中间signature/archetype迁移，并且中途错误留下部分bundle。

PERF-MVP-479交接Runtime08：Bundle先提供component id/signature和owned value staging，预验证后直接reserve最终archetype row并一次commit全部storage/lifecycle/change tick；失败authority零变化，禁止保留逐组件公共insert循环作为bundle实现。

## 参考引擎对照

Bevy multi-threaded executor在schedule init阶段预分配bitsets、依赖计数与per-system conflict metadata，执行时从同一World建立受access约束的view，区分Send、exclusive和local-thread系统，并在完成事件上推进ready set。Zircon应采用“编译metadata + sound World分区 + reusable executor state”原则；不复制其unsafe实现，也不把当前generic closure executor冒充ECS product executor。

## 动态验收

1. current-source Cargo覆盖ecs_schedule 58 tests、parallel structure、native/runtime/hooks/deferred ordering、fixed stages、failure/poison和profiling source guards。
2. systems 1/16/256/10k、parallel width 1/2/8/64、components/resources 1/1k/100k记录main/worker wall、World lock wait/hold/acquires、batch/ID/result allocations、jobs/waits、parallelism、deferred bytes与p95。
3. 478要求产品trace实际parallel batches>0、non-conflicting overlap>0、stable graph/batch rebuild=0、per-step World mutex acquire=0、serial/parallel state+event order一致；479要求N-component bundle final archetype transitions=1、intermediate signatures=0、failure partial writes=0。

受管Cargo lane当前由其他Session预约，F2产品trace与规模counter未完成，因此该范围继续保留在`pending.md`，不进入`review.md`。
