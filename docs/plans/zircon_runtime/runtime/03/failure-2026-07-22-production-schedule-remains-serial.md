---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: production-schedule-remains-serial
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/module/world_driver.rs
tests:
  - cargo test -p zircon_runtime --lib ecs_schedule --locked --jobs 1 -- --nocapture --test-threads=1
  - F2 product schedule overlap and World-lock counters
---

# Runtime03：生产ECS schedule仍串行执行交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：scene ECS root 16/16逐Rust文件审查，PERF-MVP-478
- 修复责任计划：`docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md`
- 交接原因：Runtime03拥有WorldDriver、stage ordering和executor选择；父计划已completed，本记录按post-completion failure路由，不改写既有历史验收。
- 生命周期键：`production-schedule-remains-serial`

## 失败现象与复现证据

生产`WorldDriver`只调用`SceneScheduleRunner::run_stage`，逐Internal/Native/Runtime/Hook step串行反复获取Level World mutex并同步apply deferred；全生产调用树没有`ScheduleParallelExecutor`。现有parallel executor/conflict graph测试证明generic task批次自身可运行，不证明产品scene systems并行。该executor每run还clone batch ids、分配per-batch Arc/Mutex result并tail wait，task closure没有disjoint World borrow权限。

## 最低共享层根因

compiled `SceneScheduleStagePlan`只有排序后的descriptor/step，没有可执行conflict/dependency bitsets、direct system slots、exclusive/main-thread分类、sound storage/resource views或worker-local deferred buffers；因此产品只能通过全局World互斥串行调用。

## 架构修复验收

- schedule definition generation改变时一次编译dense dependency/conflict/exclusive/main-thread metadata和direct system slots；stable frame graph/batch/id rebuild=0。
- executor从World取得由SystemParamAccess证明互不冲突的storage/resource views或等价sound分区；禁止多个worker共享`&mut World`或用全局World mutex伪并行。
- 每system使用worker-local deferred/event/lifecycle buffer，在显式ApplyDeferred/barrier按stable plan order确定性merge；panic/error/cancel不发布部分authority。
- non-Send、exclusive、hook、render-extract和无法分区的系统进入明确main-thread lane；worker batch通过Runtime11 reusable scheduler state执行，不每batch新建Arc/Mutex/ID Vec。
- F2 systems 1/16/256/10k、width 1/2/8/64记录overlap、World lock、alloc/jobs/waits和p95：产品parallel batches>0、non-conflicting overlap>0、per-step World mutex acquire=0，串并行state/event/deferred order一致。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止直接把generic closure executor接到持有同一World mutex的tasks，这只会增加调度开销且仍串行。
- 禁止用unsafe并行绕过SystemParamAccess/aliasing证明或把所有系统错误标为exclusive。
- 禁止用测试中的independent atomics/Mutex world-state夹具冒充真实ECS product integration。

## 修复结果与回传

2026-08-10 current-source result:

- 旧的“生产 schedule 仍串行”事实已经过时。当前 `SceneScheduleRunner` 根据 compiled native conflict graph / `SystemParamAccess` 选择互不冲突且 worker-safe 的 native systems，把这些 system 暂时从 `World` 中取出后通过 `JobScheduler::join` 执行；worker 不共享 `&mut World`。
- worker-local command queues 按 stable system order 合并回 `World`。non-worker-safe、Internal、Runtime、Hook 与 deferred barrier 仍保持显式 main-thread lane；worker callback 与 command apply 共享统一 unwind 边界，taken systems 在重新抛出 panic 前恢复。
- 当前行为回归覆盖 production overlap、conflict serialization、stable command merge、main-thread lane、worker callback panic 与 command apply panic restoration。`NativeSystemScheduleDiagnostics` 同时暴露 worker batch/conflict、ready delay、worker utilization、callback p95，以及 callback/conservative writer 和临时控制缓冲容量代理；overlap 目前是行为回归证据，不是发布的产品计数。
- 本轮只完成源码、rustfmt、diff/source contract 核对；没有运行受管 Cargo，也没有把独立原子量夹具当作 F2 产品证据。

Open state: `production_parallel_source_repaired_pending_managed_cargo_and_F2_product_overlap_world_lock_allocation_matrix`; no accepted pass is claimed.
