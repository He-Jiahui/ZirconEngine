---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: ecs-deferred-command-dense-buffer
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/ecs/commands
  - zircon_runtime/src/scene/ecs/bundle.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/world/commands.rs
tests:
  - cargo test -p zircon_runtime --lib ecs_commands --locked --jobs 1 -- --nocapture --test-threads=1
  - 100k command allocation and worker-merge counters
---

# Runtime08：ECS deferred command dense buffer交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：scene ECS commands 7/7逐Rust文件审查，PERF-MVP-487
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有Commands、structural transaction与apply-deferred语义；Runtime11共同拥有worker-local buffer执行。
- 生命周期键：`ecs-deferred-command-dense-buffer`

## 失败现象与复现证据

`CommandQueue`当前是`Vec<Box<dyn ErasedCommand>>`：每个spawn/insert/remove/resource closure单独heap allocate，apply逐项vtable dispatch并drop。PERF-MVP-486已把outer Vec的`mem::take`改为原地drain，避免每帧丢失queue capacity，但Box allocations仍随command count线性增长。flush持唯一`&mut World`串行执行，并通过World内全局deferred-error Vec clear/take往返；parallel executor没有worker-local command buffers或确定性merge。

Bevy参考`dev/bevy/crates/bevy_ecs/src/world/command_queue.rs`使用可复用dense heterogeneous byte buffer避免`Vec<Box<dyn Command>>`；本仓应采用其“dense reusable owner”原则，但必须独立证明alignment、drop、panic与nested enqueue安全，不能复制未审计unsafe。

## 最低共享层根因

命令表示以每item ownership allocation换取type erasure，且deferred authority仍绑定一个World queue；调度器即使并行运行system也无法把structural mutation留在worker-local staging后批量提交。

## 架构修复验收

- 采用可复用dense heterogeneous buffer或typed structural command lanes，使stable command enqueue无per-item heap allocation；closure/custom command有明确large-payload fallback与byte budget。
- 每worker/system拥有local buffer，stage barrier按compiled schedule/system order确定性append；enqueue不获取World mutex。
- spawn/bundle/component/resource命令按affected archetype/storage批处理，复用PERF-MVP-479/481一次transaction，不退回逐component中间迁移。
- error属于对应buffer/apply report，保留operation/entity/order；panic/unwind、unapplied drop、nested enqueue和shutdown完整释放payload。
- commands/frame 0/1/1k/100k、workers 1/8/64记录alloc/free、bytes、vtable calls、merge/apply时间和World lock：stable per-item heap alloc=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止只引入Box pool却保留每command独立allocation/lifetime管理。
- 禁止在worker线程直接获取全World mutex执行structural command。
- 禁止用未证明alignment/drop cursor的unsafe byte buffer或在panic后重放已消费command。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
