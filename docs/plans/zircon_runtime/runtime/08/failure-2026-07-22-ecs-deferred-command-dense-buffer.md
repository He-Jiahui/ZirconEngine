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
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/scene/ecs/commands/command.rs
  - zircon_runtime/src/scene/ecs/commands/command_metrics.rs
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/ecs/commands/commands/entity_commands.rs
  - zircon_runtime/src/scene/ecs/commands/commands/facade.rs
  - zircon_runtime/src/scene/ecs/commands/commands/mod.rs
  - zircon_runtime/src/scene/ecs/commands/commands/param.rs
  - zircon_runtime/src/scene/ecs/commands/mod.rs
  - zircon_runtime/src/scene/ecs/commands/structural.rs
  - zircon_runtime/src/scene/ecs/commands/worker_command_buffer.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_runner/tests/typed_worker_structural.rs
  - zircon_runtime/src/scene/ecs/schedule_runner/tests/worker_callback_order.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/system/local.rs
  - zircon_runtime/src/scene/ecs/system/mod.rs
  - zircon_runtime/src/scene/ecs/system/native/function_scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/into_scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/mod.rs
  - zircon_runtime/src/scene/ecs/system/native/scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/scene/ecs/system/param_set.rs
  - zircon_runtime/src/scene/ecs/system/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_param_access.rs
  - zircon_runtime/src/scene/ecs/system/system_param_error.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/tests/ecs_commands.rs
  - zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs
  - zircon_runtime/src/scene/tests/ecs_systems/commands.rs
  - zircon_runtime/src/scene/tests/ecs_worker_command_buffers.rs
  - zircon_runtime/src/scene/tests/mod.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/commands.rs
  - zircon_runtime/src/scene/world/deferred_structural_segment.rs
  - zircon_runtime/src/scene/world/mod.rs
  - zircon_runtime/src/scene/world/schedule.rs
  - zircon_runtime/src/scene/world/world.rs
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

Open state: `前向修复中`; no Cargo pass is claimed.

2026-08-02 source progress:

- `CommandQueue` no longer owns ordinary commands as `Vec<Box<dyn ErasedCommand>>`. It uses a reusable ordered vector of 64-byte-aligned, 192-byte inline slots with a 4 MiB active slot-storage budget. Commands that exceed the size/alignment/budget contract retain an explicit boxed fallback. `CommandQueueMetrics` separates logical payload bytes from bounded occupied slot bytes, and exposes backing-vector growth, explicit fallback allocation/release, inline release, dispatch calls, and panic-discard counts.
- Inline payload construction, execution, and abandonment are contained in one audited owner. The armed-state transition happens before user `Command::apply` code, so normal application, queue drop, and unwind each release a payload exactly once. The panic path drains and releases all unconsumed commands without reapplying them.
- `WorkerCommandBuffer` gives each external command callback a prewarmable local `CommandQueue` keyed by `(system_order, system_id)`. Worker-safe callbacks run without the World lock; `SceneScheduleRunner` gathers their buffers at the worker-batch boundary, sorts them by that key, rejects duplicate keys before moving any payload, merges them into the World queue once, and applies the batch once. The direct `World::run_native_scene_systems_for_stage` test helper now flushes once at its stage end as well, so it cannot strand a worker callback whose compiled schedule intentionally omits a per-system ApplyDeferred marker. Main-thread-only or constrained callbacks retain their existing immediate `ApplyDeferred` barrier. Ordinary nested queues use `CommandQueue::append` and merge queue metrics; keyed worker buffers use the dedicated producer-aware transfer path. `World::apply_deferred` restores its active change tick and merges commands enqueued by a running command into the next apply window even when the current apply unwinds. Focused regression coverage now includes worker callback batch order and batch count, direct-stage visibility, compiled worker-buffer ordering, duplicate-key fail-closed behavior, small/large/over-aligned storage selection, fixed-slot budget saturation, unapplied payload release, panic cleanup, nested enqueue visibility, and local-buffer merge order.
- `CommandQueue::with_capacity` provides the Runtime08-side prewarm point for reusable local buffers; a prewarmed queue records one intentional backing-storage allocation and does not grow while its reserved command count is consumed.
- This is not a fixed/accepted handoff: the typed `CommandsParam` lane and structural batching are now source-complete, but their source-bound managed Cargo gate and the declared Windows profiling evidence remain outstanding. No terminal Cargo result is claimed.

2026-08-11 packed-arena hard cut and reference reconciliation:

- Unreal 主参考 `dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassCommandBuffer.h` 与对应 `.cpp` 将命令按 owner/thread staging、稳定 operation group flush、`MoveAppend` ownership transfer 和集中 cleanup 分层；Bevy Rust 参考 `dev/bevy/crates/bevy_ecs/src/world/command_queue.rs` 使用可复用 heterogeneous byte storage。Zircon 采用二者共同的“producer-local dense owner + barrier ownership transfer”原则，同时保留自身 compiled `(system_order, system_id)` 确定性顺序。
- 退役的每命令 192-byte 固定 slot 已硬切为 64-byte aligned、64 KiB block 的 packed arena。普通 payload 只占 `size + alignment padding`，每个 queue 的主动 enqueue 上限仍为 4 MiB；大于 192 bytes、对齐大于 64 或超过 producer-local budget 的 payload 才进入显式 boxed fallback。queued logical bytes、packed occupied bytes、block-vector growth、fallback alloc/free 和 dispatch/drop 分开计数。
- inline metadata 只保存 `(block_index, offset, apply_fn, drop_fn)`，不保存会被 `Vec` relocation 破坏的裸 payload pointer。apply 前先把 entry 改为 `Consumed`；正常执行、queue Drop 与 panic discard 各自只释放一次。panic 后剩余 payload 按原顺序 drop，已消费命令不重放；zero-sized boxed fallback 不虚报 heap allocation。
- worker merge 移动完整 block 并 remap block index，不逐 payload allocate/dispatch。大 queue apply 后若只合入一个小 nested queue，会优先复用既有 block capacity，避免把预热容量 swap 到即将销毁的临时 owner。新增回归覆盖 100,000 个 1-byte 命令零 fallback、64-byte aligned payload padding、4 MiB saturation、64 worker/6,400 command merge、panic cleanup 和 large-arena capacity reuse。
- 2026-08-11 static follow-up: the empty-destination branch of `InlineCommandArena::append` still swaps the whole arena when destination block capacity is smaller than the worker's block count. That transfers payload ownership correctly but leaves the worker with the destination's unprepared backing, so the next worker pass reallocates its producer-local block arena. Merely retaining the worker's `Vec` capacity is not a sufficient repair because its 64 KiB payload blocks were transferred to the World. The real fix needs per-worker provenance plus post-barrier return of drained blocks, or an equivalent bounded worker-owned block pool; it must not copy payloads or use a worker-side mutex. Add a focused worker-merge/apply/requeue allocation-counter regression and obtain managed Cargo evidence before calling the prewarm contract complete.
- F5 前向修复：`DeferredCommandError` 不再把 `SceneError` 压成 `String`。公开报告保存原始 typed error 并通过 `error()` 暴露；spawn、insert、bundle、remove 和 despawn producer 全部移交原始错误，missing despawn 也构造 `SceneError::MissingEntity`。命令回归现匹配 error variant/entity，而不依赖显示文本。
- 精确生产文件 `rustfmt +1.94.1 --check` 与 scoped `git diff --check` 已通过。一个直接包含当前 `command_queue.rs` 的最小 `rustc +1.94.1` 诊断程序实际执行 100k、alignment、capacity reuse 与 caught-panic exactly-once drop，进程 exit 0；该诊断不是 Cargo/受管验收，不能据此关闭 handoff。
- 该阶段保留的未完成项后来已由 typed lane hard cut 收束；worker-side World mutex、运行时 spawn panic 和 non-deterministic atomic id 仍是禁止方案。

2026-08-11 typed command 边界复核（pre-cut snapshot）：

- `CommandsParam::get_param` 现在会真正取得 `World::command_state_mut()` 返回的 queue 与 `next_entity`，而非使用占位 facade；`SystemState<CommandsParam>` 与 `ApplyDeferred` 已有行为测试。但这个可变 World 借用是 main-thread-only 路径，不能作为 worker-local staging。外部 worker callback 的 `WorkerCommandBuffer` 已是独立 local queue，两者还没有统一。
- 统一设计必须先在 compiled schedule 开始前指定 `(system order, system id, local spawn ordinal)` 的 deterministic reservation plan，再令 `CommandsParam::State` 独占 local queue 和该 plan。禁止用执行线程的 atomic `next_id` 分配，否则不同 worker 完成顺序会改变可观测实体 ID。
- 只有 schedule barrier 能按 compiled key merge local queues 并通过唯一 `CommandQueue::append` 发布。每个 built-in structural operation 需在 barrier 内按 entity 聚合为 final row，经一次 archetype/storage transaction 发布 lifecycle；不能以把现有 closure 移到 worker 的方式伪造批处理。

### 2026-08-11 typed worker command research and atomic hard-cut design

- Unreal primary reference is concrete rather than aspirational: `dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassCommandBuffer.h` owns one buffer per producer thread, rejects cross-thread pushes through its owner-thread check, and exposes move-only `MoveAppend`. Its `Private/MassEntityManager.cpp` switches the open deferred buffer before flushing so observer-generated commands land in the next window; `Private/MassCommandBuffer.cpp` groups operations before destructive flush and centralizes cleanup. Zircon must preserve the same ownership and next-window semantics, while retaining its stronger compiled schedule order requirement.
- Bevy's `ParallelCommands` is useful only for its producer-local queue shape. `dev/bevy/crates/bevy_ecs/src/system/commands/parallel_scope.rs` explicitly documents that non-commutative operations are non-deterministic across thread counts. Runtime08 rejects that behavior: thread completion order, worker count, and allocator timing must not affect structural command order or externally observable entity IDs.
- Pre-cut source proof: `CommandsParam::get_param` borrowed `World::command_state_mut()` and `Commands::spawn` incremented the World `u64` `next_entity` immediately. That design could not carry typed worker commands without a World-backed ID borrow or worker-completion ordering, so it was replaced rather than extended.
- The next source milestone is therefore one atomic typed-command cut, not a queue-field refactor. The compiled schedule must create a per-run `DeferredSystemKey { stage_order, system_order, system_id }`; each typed system receives an owned local command lane and local spawn ordinal. `Commands::spawn`/`EntityCommands` must hard-cut from an immediately observable raw `EntityId` to an internal `DeferredEntityRef::{Existing(EntityId), Spawn(DeferredSpawnToken)}`. A token is scoped to its system key plus ordinal, and all operations emitted by an `EntityCommands` handle retain that ref.
- At the single schedule barrier, lanes sort by `DeferredSystemKey`; the barrier allocates actual `EntityId`s in that sorted token order, resolves every token, groups all operations for each resolved entity into one final-row transaction, then publishes lifecycle/generation once. Existing-entity operations retain their schedule order. A failed preflight returns per-lane ordered errors without publishing any row; an unapplied lane drops its owned payload exactly once. Nested commands enter the next deferred window, matching the current `World::apply_deferred` and Unreal buffer-swap rule.
- This requires coordinated changes to the typed command facade, `CommandsParam` state, native system execution/schedule metadata, and the Runtime08 structural transaction owner. It cannot retain `EntityCommands::id() -> EntityId` as a compatibility facade: before the barrier no real ID exists. The replacement must expose an explicit deferred handle and resolve result after commit; only that hard cut can make worker-count-independent ordering truthful. Acceptance must compare 1/8/64 worker runs for identical resolved IDs, final rows, lifecycle sequence, error order, and zero worker-side World locks, in addition to the existing allocation counters.

### Performance evidence plan before typed-worker implementation

- Run on Windows through the managed coordinator only, with artifacts rooted under `E:\Git\ZirconEngine\.codex\artifacts\runtime08\ecs_commands\`; no profiler trace, PDB copy, or generated report may be placed under `C:\`.
- Establish the current packed-arena baseline at command counts `0/1/1k/100k` and worker-lane counts `1/8/64`. Record wall-clock median and p95, process CPU time, queue logical/packed/fallback bytes, fallback alloc/free, block/vector growth, merge time, apply time, final World lock acquisitions, and emitted lifecycle count. The same command mix and warmup count must be used before and after the typed cut.
- Capture an ETW CPU/allocation/lock trace for the 100k and 64-lane cases using the repository Windows profiling wrapper, then publish a compact markdown report with symbol path, command fingerprint, machine and thread-count metadata. The trace is diagnostic evidence only; the numerical comparison is the counter-aligned summary, not an unsupported cross-engine millisecond claim.
- Post-change acceptance: ordinary inline commands retain zero per-item fallback allocations; packed arena growth is `O(total_payload_bytes / 64 KiB)`; merge is linear in lane count plus moved block count and has no per-command heap move; typed-worker final-row commits are `O(affected rows + component columns)` and take the World write boundary once per barrier. ID/lifecycle equality across worker counts is a semantic gate, not a performance tradeoff.

2026-08-12 F6 forward repair:

- Worker inline arenas now retain their compiled producer identity after barrier merge. The World queue keeps them separate from its ordinary arena, resets them after normal apply or panic discard, and returns physical 64 KiB blocks only to the matching empty `WorkerCommandBuffer`. Generic nested `CommandQueue::append` coalesces matching producer keys and remaps every worker block index; Queue padding is never charged to a worker command. The transfer moves blocks and metadata only, with no payload copy and no worker-side mutex.
- Worker callback panic now discards local queued work through the same exact-once owner before rethrowing, both in worker dispatch and direct native dispatch. Regressions cover merge/apply/reclaim/requeue allocation growth, pre-barrier reclaim deferral, nested append, matching-key coalescing, worker callback panic retry, and direct callback panic retry.
- This preliminary packed-arena snapshot was superseded by the complete typed-command/structural-batching manifest below. No terminal Cargo outcome is claimed. This failure remains open until the full declared managed evidence closes.

### 2026-08-13 typed lane atomic implementation

- `CommandsParam::State` 现在拥有 `WorkerCommandBuffer`；`SystemState::run` 和 `run_without_world` 均在每次回调开始新的 run generation。worldless-only system param composition 才可进入 worker，多个 `CommandsParam` 由 access contract 拒绝。
- compiled schedule 向 native/external worker system 注入 `DeferredSystemKey { stage_rank, plan_order, system_id }`。`Commands::spawn*` 返回 deferred handle，`EntityCommands::id() -> EntityId` 与 World-backed reservation API 已硬切；token 包含 run generation，旧 handle 不会在后续窗口或同 key 的下一轮 alias。
- `CommandQueue::apply` 仅从 spawn operation 收集 token，先为整个 window 建立确定性 ID plan。连续 structural entry 由 `DeferredStructuralBatch` 先完成完整 preflight、再按 entity 发布一次 final row；opaque command 是严格边界，nested enqueue 留在下一 window。worker panic 会丢弃所有尚未合并的 sibling lanes。
- 回归覆盖 100k packed commands、64 worker arena transfer、过期 token、opaque-next-window、panic/retry 和 1/8/64 worker typed structural parity。源文件级 manifest 包含 38 个 owner 文件，受管 Windows reservation `38269ec1fb6447b0957147539d01837c`（command fingerprint `aa04c427…`，manifest fingerprint `fe1e913a…`）仍为 pending；没有终态 Cargo 或 profiling 通过结论。

### 2026-08-13 compiled-key and structural-segment reconciliation

- The pre-cut worker key based on registration metadata was replaced. `ScheduledSceneStep` now injects `DeferredSystemKey { stage_rank, plan_order, system_id }` for every dispatch; worker completion order, metadata order and allocator timing do not determine observable structural order.
- `CommandsParam` now owns the system-local `WorkerCommandBuffer` lane rather than a World queue/ID borrow. The packed owner carries both opaque `Command(&mut World)` entries and typed structural metadata; opaque `queue`/`queue_fn`, resource operations and custom commands remain non-crossable segment boundaries.
- The barrier reserves whole-window spawn IDs by `(DeferredSystemKey, run_generation, local_spawn_ordinal)`, then preflights each contiguous structural segment before final-row publication. A failed segment has no row/lifecycle/derived-state publication; resolved IDs are not recycled and nested commands remain in the next window.
- The hard cut removed `EntityCommands::id() -> EntityId`, `Commands::entity_or_spawn`, `World::command_state_mut` and `DeferredCommandError::entity() -> EntityId`. The only current API targets are `DeferredEntityRef::{Existing, Spawn}`, explicit `DeferredEntity` handles and post-commit report resolution. This remains source-open only pending the already-reserved managed test and profiling evidence; no fixed return is claimed.

### 2026-08-14 Frameworks01 compile handoff: schedule-runner test module path

- Frameworks01 managed Windows profiling job `25f05854c2114f1ea657d76fea939358` reached `zircon_runtime` compilation, then stopped before the ResourceManagement test body ran. Its source-bound rustc error named the Runtime08 test declaration for `typed_worker_structural.rs`; the adjacent `worker_callback_order.rs` declaration had the same stale split-file shape and was repaired in the same owner slice.
- The Runtime08 test owner now declares nested test modules in `scene/ecs/schedule_runner.rs` with `include!("schedule_runner/tests/typed_worker_structural.rs")` and `include!("schedule_runner/tests/worker_callback_order.rs")`. Both leaf files physically exist under `scene/ecs/schedule_runner/tests/`, so Rust resolves them from the containing root source rather than applying `#[path]` relative to the synthetic `schedule_runner/tests` module directory.
- Local static evidence: `rustfmt +1.94.1 --check` for the root and both leaves passed; scoped `git diff --check` passed (the repository reports only its existing LF-to-CRLF warning). The exact current local SHA-256 values are `schedule_runner.rs=AAFBCA3D3433459053AD7EC40E09BF0ED07C4072C77A8DF939F6151B2AA2FB29`, `typed_worker_structural.rs=5116C2445977B40857ACADA2DC66B5AC35A17A23BB60A9B274F4518DDC81706F`, and `worker_callback_order.rs=B006A495FCA44022453612C33D5ED2B7D95942FBF7C2185E968A8726CAEB9AE4`.
- This is forward source repair only. It is not a coordinator source snapshot fingerprint and not an accepted compile result. A fresh managed `zircon_runtime` compile must run after the UI12 validation lane is released; Frameworks01's blocked test must not be reported as executed or passed from job `25f05854c2114f1ea657d76fea939358`.

### 2026-08-14 Frameworks01 revalidation evidence

- Frameworks01's managed Windows job `ceaaffb6bb374111a92c40aee8cdb722` ran on the D: target from 20:02:37 to 20:14:00 (683.3 seconds; 695 seconds end-to-end), and rustc crossed both repaired `schedule_runner/tests/{typed_worker_structural,worker_callback_order}.rs` includes.
- The `zircon_runtime` lib-test still failed during current-source compilation before any test executed: 361 errors and 1,520 warnings. The visible terminal tail is independently owned Text code at `src/text/cache/rich_cache.rs:477` (`expected Arc<str>, found String`).
- Classification: the schedule-runner split-file failure is no longer the active blocker; this terminal is shared current-source compile evidence, not Runtime08 behavior RED/GREEN evidence and not a Frameworks01 failure. No Runtime08 test is claimed to have executed or passed from this job.

### 2026-08-14 UI12 current-source compile-anchor reconciliation

- `CommandQueue` now initializes and owns `worker_inline_arenas`; the retired
  `queued_inline_storage_bytes` private metric is absent from the current command-queue owner, so
  the former missing-field/private-field call-site diagnostics have no current source target.
- Mutable combinations explicitly construct `[StableEntityLocation; K]` and `[D::Item<'_>; K]`;
  mutable query iteration consumes the cache leaf's sole `pub(crate)`
  `cached_archetype_plans()` accessor. `SceneScheduleStagePlan` clones the owned scheduled id into
  its deferred key, and `Component: 'static + Send + Sync` satisfies cached-query type identity.
- A non-Cargo current-source audit covered these anchors together with the coupled World/UI
  visibility repairs (17 checks passed). `rustfmt +1.94.1 --check` passed for the command/query
  owner files. The managed `ecs_commands` and profiling gates remain pending; this is not a fixed
  return or behavioral acceptance result.
