---
title: Runtime Clock、Time Policy、World Fixed Step、Timer 与 Cadence 当前源码工程化差距
category: zircon_runtime
report_id: Runtime155
review_date: 2026-08-29
baseline_head: 8aabbee3e99dc919f6da4611e3a44e8463a7fe7f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_incomplete
source_recheck_required: true
related_code:
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/runtime/clock_source.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/scene/world_time
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/22/2026-08-24-fixed-step-transaction-architecture-and-performance-plan.md
  - docs/plans/optimize/zircon_runtime/22/2026-08-18-virtual-delta-validation-pending.md
  - docs/plans/optimize/zircon_runtime/102-runtime-random-authority-stream-checkpoint-replay-consumer-performance-current-source-review.md
reference_engines:
  - dev/bevy/crates/bevy_time/src/lib.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/bevy/crates/bevy_time/src/timer.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TickTaskManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/TimerManager.h
  - dev/godot/main/main_timer_sync.cpp
  - dev/godot/scene/main/timer.cpp
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Utilities/Playables/VisualEffectControl/VisualEffectControlTrackMixerBehaviour.cs
---

# Runtime Clock、Time Policy、World Fixed Step、Timer 与 Cadence 当前源码工程化差距

## 1. 结论

当前时间系统已经发生了实质性架构迁移，不能再沿用 Runtime22 最初的“Core 同时预推进 real/virtual/fixed”描述。当前 Core 只拥有 monotonic outer-frame clock 和后续 Level 的默认 `TimePolicy`；每个 Level 独立拥有 Virtual/Fixed、pause、scale、debt、policy generation 与 committed interpolation。`WorldFixedStep` 是不可复制 capability，`SimulationTickId` 已包含 world generation、fixed epoch 和 tick index，fixed clock/debt 也只在三个 fixed stage 全部成功后逐步 commit。普通 Update 默认消费 Virtual，暂停时跳过；显式 MonotonicReal system 仍可运行。旧 `TIME-P0-001` 的生产根因已在当前源码关闭，旧 `TIME-P0-002` 的 clock/debt 部分已关闭。

但当前状态仍不是可宣称“工程级时间产品”的终态：

- 当前 untracked 测试仍调用 hard-cut 后已不存在的 `FrameTimeSnapshot::fixed_step_plan()`，测试 target 会在运行前发生方法解析失败。
- outer clock sample 与 Runtime frame commit 分属两把锁；并发 `tick_time` 可按采样 A/B、提交 B/A 的顺序发布 frame。Level 又不拒绝重复、乱序或跨 source generation 的 snapshot。
- fixed transaction 只回滚 clock/debt。直接 World mutation、已 apply 的 deferred command、physics state/event、script external effect 和 Runtime154 的 RNG lease progress 不在同一 commit boundary。
- `fixed_step_budget` 被复制给每个 World；N 个 World 的最坏 fixed work 是 `N * budget * stages`，没有 Runtime 全局预算、fairness、deadline 或 degrade policy。
- dynamic session 正确选择 Client/Headless/Editor/Test policy，但底层公开 API 仍接受 raw `u32` budget 和任意 manual delta；`ProductTimePolicyDigest` 在 production 只有定义与 re-export，没有进入 BuildSet、replay、session receipt 或 diagnostics。
- lifecycle/window discontinuity 只有一个 pending slot，下一事件覆盖前一事件；public custom `ClockSource` 若倒退，则被 `saturating_duration_since` 静默折成零。
- World replacement 只重置 interpolation endpoints，不声明保留/清空 Virtual elapsed、Fixed elapsed/debt、pause、rate 的策略；旧 World 债务可继续在新 World 执行。
- 引擎没有 World gameplay Timer 产品。现有 `TaskTimer` 是进程 monotonic deadline primitive，UI timers 是输入/UI 私有状态，都不能替代绑定 Virtual/Fixed domain、可序列化、可取消、可配 catch-up 的游戏计时器。

本报告新增 **1 项 P0、9 项 P1、3 项 P2**，30 项资格门为 **19 Fail / 6 Partial / 5 Pass**。Runtime22 已有 TIME findings、Runtime154 random transaction、Runtime99zm Physics dual-clock 与 Runtime153 source/Cargo findings继续由原报告唯一计数；本报告只登记当前源码新增的具体断裂和 authority 缺口。

## 2. 冻结范围、currentness 与方法

### 2.1 物理范围

fingerprint 口径为 lower-case repo-relative path、文件 SHA-256，按路径排序后以 `path<TAB>hash` 和 LF 拼接，再计算 SHA-256。

| 范围 | files / lines / nonempty / bytes / tests | tracked / modified / untracked / dirty | fingerprint |
|---|---:|---:|---|
| Zircon clock/time/fixed/timer/cadence + product/adapter/tests | **53 / 12,438 / 11,205 / 438,523 / 102** | **33 / 30 / 20 / 50** | `7320fa4ad3d8622036bdb9155151da7b8eed5d27e9d7031e7562c59eead3a35e` |
| Unreal/Bevy/Godot/Fyrox/Unity Graphics reference selection | **19 / 17,688 / 15,405 / 647,189 / 28** | n/a | `08ffddccf7822e023402c09520dc540dc6e85f53d3a9a23d6ae7bdcce5d71cdf` |

选择集覆盖 framework time 全目录、Core outer clock/policy/handle、World time controller/driver/schedule context、相关单元与 scene tests、dynamic session profile/lifecycle、App cadence、Runtime Interface ABI 负扫描、task/UI timer，以及 Animation/Physics/Script adapter。物理文件集之外，`zircon_runtime/src/core/framework/time/real.rs` 处于 tracked deletion；它是 `MonotonicReal` hard-cut 的迁移证据。

Zircon 精确选择规则：

- 递归目录：`zircon_runtime/src/core/framework/time/**/*.rs`、`zircon_runtime/src/scene/world_time/**/*.rs`。
- Core：`clock_source.rs`、`frame_clock.rs`、`time.rs`、`time/product_policy.rs`、`handle/time.rs`、`runtime.rs`、`modules/time.rs`、`tasks/timer.rs`。
- Scene：`level_system.rs`、`level_manager_lifecycle.rs`、`world_driver.rs`、`tick_context.rs`、`schedule_runner.rs`、`tick_policy.rs`、`runtime_scene_system.rs`、`scene_system_registry.rs`。
- Product/tests：runtime `tests/time.rs`、三个 ECS time/fixed/driver tests、dynamic session construction/profile/state/events、App cadence/frame-loop/lifecycle、Runtime Interface `runtime_api.rs`。
- Adapter/negative control：Physics runtime system/service/clock、Animation runtime system、Script scene system、UI input timers。

19 个 reference 文件精确为 Bevy `lib.rs/time.rs/virt.rs/fixed.rs/timer.rs/stopwatch.rs/main_schedule.rs`；Unreal `App.h/App.cpp/TickTaskManager.cpp/TimerManager.h/TimerManager.cpp`；Godot `main_timer_sync.h/.cpp/main.cpp/timer.h/.cpp`；Fyrox `executor.rs`；Unity Graphics VFX control-track mixer。Unity Graphics只作为feature级preview/scrub参考，不被误写为完整Unity engine源码。

当前 53 个 Zircon 文件中 50 个 dirty，20 个尚未跟踪。HEAD 为 `8aabbee3e99dc919f6da4611e3a44e8463a7fe7f`；因此本报告必须 `source_recheck_required: true`，不得把当前在途源码当作 clean-checkout acceptance。Tooling 按用户要求排除；本轮未查询、轮询、等待或实时跟踪协调器。

### 2.2 动态证据边界

本轮是 review-only，没有运行 Cargo、Miri、loom、fuzz、跨平台 replay、fault/soak、WPR/Tracy、功耗或动态 benchmark。P0 编译断裂来自静态方法定义/调用闭包证明：当前 `FrameTimeSnapshot` 无 `fixed_step_plan` 方法，而 untracked test 直接调用它。其余结论来自逐文件状态机、锁序、调用链和测试覆盖审查；不把历史 child-plan 的 managed validation 声明成当前 dirty workspace 证据。

## 3. 当前可保留的工程基础

| 能力 | 当前证据 | 保留条件 |
|---|---|---|
| Core/World 分层 | Core 只持 MonotonicReal 与 future-Level defaults；Virtual/Fixed 为 Level-local | 不恢复全局 virtual/fixed compatibility clock |
| domain identity | versioned `ClockDomainRegistry`、stable enum、unit、epoch、source generation | 补齐真实 owner、parent/correlation 和 availability，不让空 enum 伪装能力 |
| injected source | `ClockSource`、`ManualClockSource`、typed non-monotonic/out-of-range error | sample 必须带 source identity/sequence/result，进入单一 frame admission lane |
| activation/lifecycle | activation rebase 与 typed discontinuity receipt | 多事件不得 last-write-wins；处理 policy 和完整 history 必须可诊断 |
| virtual policy | pause、relative speed、max delta，且先 clamp/checked conversion | 产品 profile 必须有上限、receipt、BuildSet/replay binding |
| fixed proposal | `plan_steps` 不改变 committed clock/debt | proposal 只能由已接纳 outer frame 创建，不可重复消费 |
| fixed capability | non-cloneable `WorldFixedStep` + world/fixed/tick identity | World/RNG/event/effect/physics 必须共享 step transaction |
| committed interpolation | previous/current 只来自 commit，fraction 使用实际 debt modulo timestep | endpoint payload也必须绑定 committed World snapshot，不只绑定时间 ID |
| tick metadata | stage/domain/outer/simulation tick/delta/elapsed/world generation | adapter 不应立即降级为裸 `f32` 并丢失 identity |
| product presets | Client 8、Headless 16、Editor 4、Test 1；64 Hz、250 ms；stable digest | policy 必须成为不可绕过的 session/BuildSet authority，而非辅助值 |

## 4. 当前 owner 与断路图

```text
ClockSource
  `- FrameClock mutex
       |- tick() -> delta + at-most-one pending rebase
       `- unlock
            `- RuntimeTimeAuthority mutex
                 `- MonotonicReal advance -> FrameTimeSnapshot(raw budget)

FrameTimeSnapshot (Copy)
  |- Level A -> WorldTimeController.advance -> Virtual + Fixed proposal
  |- Level B -> WorldTimeController.advance -> same full budget again
  `- same Level may accept the same/stale snapshot again

WorldDriver
  `- begin WorldFixedStep
       |- FixedFirst   (World/deferred/external effects may publish)
       |- FixedUpdate  (Physics/Script/RNG may publish)
       |- FixedPostUpdate
       `- commit only clock/debt, or abort only clock/debt

TaskTimer ---------------- process monotonic deadlines, not World simulation time
UiInputTimerState -------- UI/input-specific elapsed values, not gameplay Timer
World gameplay Timer ----- absent
```

目标不是把所有 clock 和 timer 合成一把全局锁。目标是：outer frame 有单一 admission/sequence owner；每个 World 验证且只接纳一次 frame；Runtime 分配跨 World 总预算；每个 simulation step 原子提交 clock、World commands、RNG、events/effects 与 subsystem outputs；Timer 绑定明确 domain 和 step transaction。

## 5. Runtime22 既有 finding 当前状态

| Existing ID | 当前状态 | current-source 复核 |
|---|---|---|
| TIME-P0-001 | Source root closed / validation pending | Update 默认 Virtual；pause 跳过 virtual systems，scale/clamp 进入 system tick；MonotonicReal 需显式 policy |
| TIME-P0-002 | Partial | batch pre-advance 已改为逐步 begin/commit/abort；World/effect/RNG/physics 原子性和 typed failure receipt仍未完成 |
| TIME-P1-001 | Partial | Registry v1、ID/unit/epoch/source generation存在；七个非核心 domain 仍只有 descriptor |
| TIME-P1-002 | Partial | `Real` 已 hard-cut 为 `MonotonicReal`；`WallUtc` 仍无 concrete service |
| TIME-P1-003 | Closed | Core snapshot缩为 outer evidence；World snapshot提供 virtual/pause/scale/fixed proposal/discontinuity |
| TIME-P1-004 | Partial | outer simulation source 可注入；大量 profiling/task/UI deadline仍各用 Instant，尚无统一审计/声明产物 |
| TIME-P1-005 | Partial | Core 有 manual/external source；dynamic Runtime Interface/App ABI 无 strategy/capability/version/seek |
| TIME-P1-006 | Closed | session activation完成后 rebase，first-tick policy 和 receipt存在 |
| TIME-P1-007 | Partial | suspend/resume/occlusion/surface recreation映射到 typed cause；pending slot会覆盖较早 cause |
| TIME-P1-008 | Closed | virtual scale 先检查 finite/clamp，再 `try_from_secs_f64`，极大有限 speed 不再 panic |
| TIME-P1-009 | Closed at public boundary | `TimePolicyTransaction` validate 后才 mutation；private invariant setters仍 assert |
| TIME-P1-010 | Partial | versioned profiles/digest存在；digest未消费，raw budget API仍绕过 profile |
| TIME-P1-011 | Partial | plan公开 debt duration/whole steps/ratio；无 catch-up/drop/degrade/fatal/debt-age policy |
| TIME-P1-012 | Closed | total debt 与 modulo interpolation fraction已分离 |
| TIME-P1-013 | Source root closed | dynamic profiles已分化为 8/16/4/1；缺 SLO/benchmark/energy acceptance不重开原命题 |
| TIME-P1-014 | Closed | `SystemTickContext` 已含 stage/domain/outer/sim tick/delta/elapsed/world generation |
| TIME-P1-015 | Closed | `SceneSystemTickPolicy::is_valid_for_stage` 和 registry admission拒绝非法 Fixed/Real 组合 |
| TIME-P1-016 | Closed | 每个 Level 独立持有 pause/scale/fixed debt/epoch |
| TIME-P1-017 | Partial | active step或pending debt时拒绝 fixed rate change并 bump epoch；没有 live multi-World prepare/commit/migration |
| TIME-P1-018 | Closed | interpolation只读 committed endpoints与真实 debt |
| TIME-P1-019 | Open | App Fixed/LowPower cadence仍在到期后设置 `next_deadline = now + interval`，不保相位且不记录 lateness/miss |
| TIME-P1-020 | Partial | typed context到达 adapter入口；Animation/Script立即降为裸 `f32`，Physics另有独立 fixed_hz/max_substeps contract |

TIME-P1-021..040 的 random、schedule digest、effect journal、replay、checkpoint、telemetry和资格矩阵继续开放；Random currentness 见 Runtime154，Physics dual-clock currentness 见 Runtime99zm，本报告不重复计数。

## 6. P0：当前源码必须先恢复可验证性

### Runtime155-P0-001：hard-cut 后的 untracked scene test 仍调用已删除方法，测试 target 无法完成方法解析

`zircon_runtime/src/scene/tests/ecs_schedule/world_time_controller.rs:62` 调用 `outer.fixed_step_plan().step_count`，并保留“Core compatibility clock has a different timestep”的旧断言。当前 `FrameTimeSnapshot` 在 `core/runtime/time.rs:131-181` 只提供 outer frame index、raw real delta、raw fixed-step budget、discontinuity和 real stamp；全量源码扫描没有该类型的 `fixed_step_plan` extension。

这正是 core virtual/fixed hard-cut 未完成的测试迁移残留。应将测试改为只验证 `outer.fixed_step_budget() == 4`，再通过 Level tick 后的 committed fixed state证明 World 以 1 ms timestep消耗4步；删除 compatibility-clock 文案。修复后必须先运行该 test target，再运行 runtime lib/all-targets。由于文件当前 untracked，本报告不修改它，也不把静态诊断冒充 Cargo execution。

## 7. P1：工程级时间 authority 前必须关闭

### Runtime155-P1-001：outer clock sample 与 frame commit 不在同一 admission lane，并发 tick 可重排真实时间顺序

`CoreHandle::tick_time` 先锁 `frame_clock` 取得 `FrameClockTick`，释放锁后才调用 `advance_time_by_with_discontinuity` 锁 `RuntimeTimeAuthority`。两个线程可按 A sample、B sample、B commit、A commit 执行，导致较晚 sample 的 delta获得较小 outer frame index。`advance_time_by` 又能绕过 frame clock直接竞争 time lock。

应建立 `FrameAdmissionAuthority`，在一个线性化点完成 source sample validation、discontinuity drain、policy selection、outer ID分配和 immutable receipt发布。锁可以分层，但需要 sequence reservation + ordered commit，不能靠调用者“不并发”的隐含约定。加入 controlled two-thread test，强制 sample/commit交错并证明 receipt顺序不反转。

### Runtime155-P1-002：Level 不拒绝重复、乱序或错误 source generation 的 FrameTimeSnapshot

`WorldTimeController::advance` 每次都设置 source generation、推进 Virtual 并累加 Fixed debt；没有保存 last accepted outer frame/runtime identity。`FrameTimeSnapshot` 又是 `Copy`。同一 snapshot被同一 Level消费两次会重复增加 virtual elapsed/debt，旧 snapshot在新 snapshot之后提交也会让 source generation倒退。active-step保护仅是 `debug_assert`，release build不会返回 typed rejection。

应让 `accept_outer_frame` 返回 `Result<WorldFrameAdmission, WorldFrameAdmissionError>`，验证 `{runtime_session_id, source_generation, outer_frame_index}` 严格单调且当前无 active step；duplicate/stale/cross-session均 fail closed并有 receipt/metric。测试覆盖 duplicate、out-of-order、source rollback、active-step reentry、World replacement和多个 Level合法共享同一 outer frame。

### Runtime155-P1-003：raw manual delta 与 raw fixed budget 是公开 mutation API，产品策略不是不可绕过的 authority

`CoreRuntime/CoreHandle::{advance_time_by,tick_time}` 都由调用者传 `u32`，包括0或任意大值；`advance_time_by` 还直接注入任意 `Duration`。dynamic session当前正确使用 profile budget，但任何 Core caller都可绕过 `ProductTimePolicy::validate`。`ProductTimePolicyDigest` production命中仅定义/re-export，没有进入 session receipt、BuildSet、replay或diagnostics。

应把 manual advance放到显式 `ManualFrameController` capability，仅 test/replay/server profile可获得；普通 `tick_frame()` 从 installed immutable `ProductTimePolicy` 读取预算，不接收 raw参数。session creation应返回 policy version/digest/limits，BuildSet/replay header验证同一 digest。需要 runtime-interface capability negotiation，而不是把 policy只留在 Rust内部枚举。

### Runtime155-P1-004：每个 World 获得完整 fixed budget，多 World 总工作量无界放大

`FrameTimeSnapshot::fixed_step_budget` 的注释明确是“each World”可提交的最大步数；WorldDriver对每个 Level独立循环该 plan。LevelManager支持多个 Level，但没有 Runtime 级 total step budget、per-world weight、deadline、round-robin/fairness、starvation/debt age或 over-budget receipt。8步 Client policy在100个 World上可变成800步、2,400个 fixed stages，还不含每 stage系统数。

应引入 `FixedWorkBudgetController`：先收集各 World debt/priority/visibility/server authority，再按 product SLO分配 `WorldFixedStepGrant`；保留 deterministic ordering和最低服务，超预算执行明确的 retain/drop/degrade/fatal policy。预算必须计入 subsystem substep、task fan-out和时间成本，而不只是整数 step count。

### Runtime155-P1-005：World replacement 没有明确 time disposition，旧 World debt 可在新 World继续执行

`reset_runtime_state_after_world_replacement` 清 physics/animation/script frame state并调用 `reset_fixed_interpolation_history`，但保留 virtual/fixed elapsed、overstep、pause、rate和policy generation。replacement发生在 active fixed step时，world generation guard会阻止旧 step commit，这是正确的；然而 abort后相同 debt仍留给新 World，普通 frame间 replacement也同样继承旧 timeline。

不同操作需要不同语义：hot reload可能 preserve timeline，level travel可能 reset，rollback restore需要安装 captured state。应要求 `WorldReplacementTimePolicy::{Preserve,Reset,Restore}`，在持有 World/time lane时原子应用并返回 old/new world generation、clock epochs、discarded/preserved debt和reason。禁止通用 replace API隐式选择。

### Runtime155-P1-006：多个 lifecycle/window discontinuity 在下一 tick 前 last-write-wins，事件历史和处理原因丢失

`FrameClock::rebase_for` 每次递增 generation后执行 `pending_rebase = Some(receipt)`；现有测试连续 rebase两次并只观察第二次。若 suspend、surface recreate、occlusion transition在下一帧前连续到达，较早 cause不进入 snapshot/diagnostics/replay。source generation虽递增，却无法解释中间变更。

应使用 bounded ordered discontinuity journal，或在 admission时生成保留所有 cause/sequence的 `ClockDiscontinuityBatchReceipt`。容量满必须合并为显式摘要或拒绝，不得静默覆盖。每个 cause还需声明 first-delta policy：drop、measure-from-rebase、clamp或 externally supplied。

### Runtime155-P1-007：public ClockSource 的倒退被静默折成零，source contract无法报告 fault或样本身份

`ManualClockSource` 自己拒绝倒退，但 public `ClockSource::monotonic_now -> Instant` 允许任意实现。`FrameClock::tick` 对倒退 sample调用 `saturating_duration_since`，返回0并把 `last_tick`更新到更早时间；下一帧会把被掩盖区间再次计入。没有 source ID、sample sequence、uncertainty、fault receipt或 quarantine。

应改为 `ClockSource::sample() -> Result<ClockSample, ClockSourceError>`，sample携 source generation/sequence/instant。Frame authority对 `< last`、重复 sequence、过大跳变按 policy reject/rebase/quarantine，并将错误传播到 frame admission。系统 monotonic source可用轻量 fast path，但不能牺牲 injected/network/replay source的可诊断性。

### Runtime155-P1-008：关键 generation/tick/elapsed 使用 saturating arithmetic，耗尽后静默冻结并重复身份

`Time::advance_by` 对 elapsed和frame index saturating add；`ClockDomainStamp::bump_epoch`、FrameClock rebase generation、policy generation也 saturating。Fixed frame index到 `u64::MAX` 后，`begin_fixed_step` 的 `saturating_add(1)`持续产生同一个 tick index，成功 commit也不再推进，`SimulationTickId`失去唯一性。World replacement epoch反而在耗尽时 panic，策略不一致。

应定义统一 exhaustion contract：checked increment在 admission前失败，返回 terminal `ClockIdentityExhausted`；或通过新 session/source generation安全 rollover，并证明跨 rollover identity仍唯一。Duration overflow同样必须有 typed fault，不得让 elapsed/debt无声冻结。增加 near-max property tests，而非等待真实运行时长。

### Runtime155-P1-009：没有绑定 World clock domain 的 gameplay Timer 产品

选择集中的 `TaskTimer` 服务于进程 monotonic deadline/callback，UI input timer服务 hover/tooltip/double-click/toast；它们没有 World generation、Virtual/Fixed domain、SimulationTickId、pause/time scale、save/replay或step commit。production全仓没有 gameplay `TimerManager/TimerWheel/Countdown` owner。

应建立 bounded `WorldTimerService`：generation-safe handle、one-shot/repeating、Virtual/Fixed/MonotonicReal明确域、first delay/rate、pause/cancel/reschedule、catch-up/coalesce/skip上限、stable same-deadline order、per-owner quota、serialization/migration和 telemetry。Fixed timer到期只把 event写入当前 step journal，commit后发布；abort必须可重试同一 tick。Editor preview/scrub使用独立 EditorPreview source，不伪造游戏 elapsed。

## 8. P2：质量、可诊断性与合同精度

### Runtime155-P2-001：TimeModule 描述仍宣称 Core 拥有 real/virtual/fixed clocks

`core/runtime/modules/time.rs:15` 的 module description仍是“runtime-owned real, virtual, and fixed clocks”，与当前 Core hard-cut和 Level-local owner相反。应改成 outer monotonic frame authority + default World policy，并在 module descriptor中链接 policy/domain capability版本。

### Runtime155-P2-002：零 delta frame 不写 frame-time/FPS 样本，current diagnostics会保留旧值

`record_time_diagnostics` 在 real delta为0时记录 frame_count后直接返回。manual step、duplicate source sample或rebase后零 delta会让 `time.frame_time/time.fps` 的 current value仍指向旧 frame，消费者无法区分“本帧0”“本帧无样本”“旧值”。应记录 typed validity/reason，或写0 frame-time并令 FPS unavailable；不得静默留下旧 current。

### Runtime155-P2-003：ClockDomainDescriptor 只有 ID/unit，不能表达 owner availability、parent、origin、rate或误差

Registry虽然 versioned，但 descriptor schema无法说明 WorldFixed派生自哪个 WorldVirtual、Render/Input属于哪个 outer frame、Network time的offset/uncertainty、Audio的sample rate、Media/EditorPreview是否实际安装。应加入 owner kind、availability/capability、parent/correlation type、origin、rate representation、precision/error和serialization ID；未安装 domain必须显式 Absent，不能只因 enum存在就被能力探测认为可用。

## 9. 本地参考源码对照

| 参考 | 可吸收的工程边界 | 不应照搬 |
|---|---|---|
| Bevy Time | `TimeUpdateStrategy`显式区分 Automatic/ManualInstant/ManualDuration/FixedTimesteps；Virtual驱动Fixed；每次fixed schedule前推进一次；Timer支持pause/repeat和本tick完成次数 | 默认 fixed loop无产品总预算，fixed time在schedule前推进，不满足Zircon更强的失败后 committed-clock语义 |
| Unreal | TickTaskManager把 World、delta、tick type绑定到 StartFrame/RunTickGroup/EndFrame；TimerManager有generation handle、set/clear/pause/query/next-tick | FApp全局可变时间和传统delegate timer不是Zircon多World/replay事务目标；Unreal tick也不提供通用World rollback |
| Godot | MainTimerSync有accumulator、jitter correction、interpolation、max physics steps；Timer选择idle/physics、one-shot、pause、ignore time scale | cap路径会采用自己的 drop/correction语义；Zircon必须把retain/drop/degrade作为产品policy与receipt，而非硬编码 |
| Fyrox | executor对lag、fixed step和过载fast-forward有明确策略，避免spiral/hang | 使用`f32` lag和大delta fast-forward，不足以作为deterministic/replay authority |
| Unity Graphics VFX | timeline scrub处理backward seek、reinit seed、event order、fixed simulate、max scrub time和大跨度步长调整 | Graphics仓库是VFX/渲染feature参考，不是完整Unity PlayerLoop或全局时间源码；不能据此推断Unity整体clock合同 |

Zircon应保留自己的 World-local committed transaction优势，同时吸收参考引擎已经产品化的 strategy、timer、overload policy、cadence telemetry和preview/scrub模式。

## 10. 目标架构

```text
ClockSourceRegistry
  `- ClockSource::sample -> ClockSample(source, sequence, instant/error)
       `- FrameAdmissionAuthority (single ordered lane)
            |- drain ClockDiscontinuityJournal
            |- install ProductTimePolicyDigest / BuildSet identity
            |- allocate OuterFrameId
            `- FrameAdmissionReceipt
                 `- FixedWorkBudgetController
                      |- grant World A steps/deadline
                      |- grant World B steps/deadline
                      `- report retained/dropped/degraded debt

LevelSystem::accept_outer_frame(receipt)
  |- validate session/source/frame monotonicity exactly once
  |- advance WorldVirtual and propose WorldFixed work
  `- for each grant: SimulationStepTransaction
       |- immutable SystemTickContext
       |- staged World command buffer
       |- staged RNG progress
       |- staged event/effect/network/audio outbox
       |- staged physics/animation/script output
       `- commit all + clock/debt, or abort all + typed receipt

WorldTimerService
  `- domain-bound timer heap/wheel -> stage timer events into transaction

TimeTelemetry
  `- frame/cadence/discontinuity/debt/grant/commit/abort/timer/exhaustion correlation
```

性能目标不是“所有操作都锁一个 central manager”。Outer admission和budget决策是短临界区；每个 World在 immutable grant上执行；timer用bounded heap/wheel；system本地计算和RNG draw保持无锁；commit只发布预分配/分代结果。

## 11. 分层重构计划

### M0：恢复 source/test 可验证性

1. 修复 Runtime155-P0-001，删除所有 core compatibility fixed clock残留。
2. 先跑 focused compile/test，再跑 runtime lib/all-targets；记录 dirty commit和未跟踪文件清单。
3. 不在 source/Cargo基线未闭合时宣称 transaction accepted。

### M1：Frame admission 与 identity

1. 引入 `RuntimeSessionId/OuterFrameId/ClockSampleSequence`。
2. 合并 sample、discontinuity drain、policy selection和 frame receipt顺序。
3. `WorldTimeController::accept_outer_frame` typed reject duplicate/stale/reentrant/cross-session。
4. 用barrier测试并发 sample/commit重排、重复 snapshot和 source rollback。

### M2：Product policy、BuildSet 与多 World budget

1. 普通 frame API不接 raw budget；manual controller需 capability。
2. Product policy version/digest进入 session config、BuildSet、replay header、diagnostics。
3. 增加 Runtime-level debt snapshot和 deterministic grant allocator。
4. 定义 Client/Headless/Editor/Test 的retain/drop/degrade/fatal、deadline、energy和server correctness SLO。

### M3：完整 simulation transaction

1. 在现有 non-cloneable fixed capability上增加 staged World mutation/effect journal。
2. Random lease、physics result、script effect、events/network/audio outbox绑定同一 `SimulationTickId`。
3. `FixedStepFailureReceipt`记录失败stage/system、prior committed count、remaining debt、discarded effects和retry identity。
4. direct mutable Runtime system必须迁移到transactional params或明确标为non-rollback并禁止进入deterministic profile。

### M4：Replacement、rate change 与 cadence

1. World replacement强制显式 Preserve/Reset/Restore time policy。
2. live multi-World policy使用prepare-all/commit-all或明确partial failure receipt。
3. cadence按 previous deadline保持相位，记录lateness/miss/catch-up/drop；模式切换显式 rebase。
4. discontinuity使用bounded journal和per-cause first-delta policy。

### M5：Timer 与 subsystem adapters

1. 先实现 allocation-bounded `WorldTimerService` kernel、stable ordering和generation handle。
2. 再接 Virtual/Fixed/Real domain、pause/rate/repeating/catch-up、save/replay与step journal。
3. Animation/Script/Physics保留完整 tick identity；Physics删除独立 product fixed authority，由Runtime99zm owner实施。
4. EditorPreview/Media/Audio/Network domain只有在真实owner和correlation artifact存在时才标Available。

### M6：资格与性能

1. required tests覆盖duplicate/stale/concurrency/overflow/failure/retry/replacement/rate change/timer storm。
2. deterministic dual-run比较tick/state/effect/RNG/timer digest和first divergence。
3. profile 1/10/100 World、0/1/8/capped steps、1K/100K timers、1/100/1000 systems。
4. 记录admission lock wait/hold、World grant latency、debt age、commit/abort成本、allocations、RSS、energy和p95/p99 hitch。

## 12. 资格门

| Gate | 状态 | 当前证据 / 缺口 |
|---|---|---|
| G1 clean source + focused test compile | Fail | untracked test调用已删除方法，50/53 selected文件dirty |
| G2 versioned clock taxonomy/stamps | Partial | v1 ID/unit/epoch/source generation存在，descriptor/owner不完整 |
| G3 injected/manual clock | Partial | Core可注入，dynamic ABI/seek/fault identity缺失 |
| G4 single ordered frame admission | Fail | frame/time两锁间可重排 |
| G5 duplicate/stale/cross-session reject | Fail | World advance无last accepted identity |
| G6 activation rebase receipt | Pass | activation完成后rebase并携typed receipt |
| G7 ordered discontinuity history | Fail | pending slot last-write-wins |
| G8 public policy validation | Pass | invalid max delta/speed/timestep fail closed |
| G9 product policy authority/digest | Partial | profiles/digest存在，API可绕过且digest未消费 |
| G10 per-World pause/scale/debt | Pass | Level-local controller独立 |
| G11 virtual pause/scale/clamp routing | Pass | default Virtual system与explicit Real分离 |
| G12 fixed proposal/commit/abort | Partial | clock/debt transaction已实现，完整state未实现 |
| G13 World/RNG/effect atomicity | Fail | direct mutations和lease progress可在abort后保留 |
| G14 typed fixed failure receipt | Fail | 只有通用CoreError，dynamic session直接fatal |
| G15 live multi-World policy transaction | Fail | runtime default仅影响后续Level |
| G16 fixed-rate migration | Partial | active/debt reject与epoch bump存在，无迁移policy |
| G17 committed interpolation evidence | Pass | previous/current只在commit更新，fraction来自实际debt |
| G18 Runtime total fixed-work budget | Fail | per-World复制完整预算 |
| G19 debt catch-up/drop/degrade/fatal policy | Fail | 只保留debt，无age/SLO/terminal receipt |
| G20 gameplay Timer product | Fail | 只有task/UI私有timer |
| G21 typed subsystem adapter | Partial | entry有TickContext，Animation/Script降为f32，Physics contract分叉 |
| G22 single Physics/fixed clock | Fail | Runtime99zm PH-P1-005仍Open |
| G23 phase-preserving cadence/lateness | Fail | 到期后now+interval，无miss/lateness |
| G24 bounded correlated TimeTelemetry | Fail | 只有frame count/time/fps，零delta还保留旧current |
| G25 BuildSet/replay/checkpoint binding | Fail | policy/clock/timer identity未进入完整artifact |
| G26 identity exhaustion/rollover | Fail | saturating generation/tick可静默冻结/重复 |
| G27 explicit replacement time disposition | Fail | 只重置interpolation endpoints |
| G28 concurrency/fuzz/fault/soak lane | Fail | 当前无执行证据 |
| G29 cross-platform determinism matrix | Fail | 当前无Windows/Linux/toolchain/thread-count证据 |
| G30 comparative performance/energy evidence | Fail | 无当前dirty source profile或参考引擎同场景数据 |

## 13. 测试与性能证据要求

必须新增的最低 RED 集合：

1. 同一 Level第二次接纳相同 OuterFrameId失败且所有clock/debt不变。
2. 先接纳frame 2再提交frame 1失败；source generation不能倒退。
3. 两个线程强制 A sample/B sample/B commit/A commit，最终receipt仍按sample sequence发布。
4. fixed step在World mutation、event、RNG draw、physics result后失败，retry观察完全相同的pre-step state和随机序列。
5. 100个 World共享一个Runtime总budget，grant总和不超过policy且无永久starvation。
6. replacement的Preserve/Reset/Restore三种策略分别证明debt、epoch和tick identity。
7. 多个discontinuity在一帧前到达，batch保留稳定顺序、cause和处理policy。
8. generation/tick在`u64::MAX - 1`附近产生typed exhaustion或安全rollover，不重复ID。
9. 100K timers同deadline保持stable order、quota、cancel generation和bounded memory；fixed abort不发布timeout。
10. App晚2.5个interval后按产品policy保持相位或明确drop/catch-up，并记录lateness/miss。

性能报告至少输出：frame admission p50/p95/p99、lock wait/hold、World grant compute、fixed begin/commit/abort、timer insert/cancel/expire、debt age、allocations/frame、peak timer bytes、RSS、CPU package energy。比较参考引擎时必须固定World/system/timer数量、fixed rate、hitch输入和render disabled条件；不能用不同功能量证明“优于虚幻”。

## 14. Ownership 与非目标

- Runtime22继续拥有 clock/fixed/determinism/replay parent architecture。
- Runtime154拥有 RandomService、checkpoint和RNG step transaction。
- Runtime99zm拥有 Physics private fixed authority hard-cut和backend step identity。
- zircon_app cadence由 App owner实施，但policy/digest和clock receipt归Runtime公共合同。
- Runtime Interface后续负责external/manual/replay clock capability与version negotiation。
- Tooling、WOC parity和其Rust迁移按用户要求排除，不在本报告扩张。
- 不以复制 Unreal全局FApp、Godot硬编码jitter policy、Fyrox f32 fast-forward或Bevy pre-advanced fixed semantics作为目标。

## 15. 交付判定

当前可称为“World-local time/fixed transaction foundation”，不能称为“工程级时间/确定性产品”，更不能声称性能或表现优于 Unreal。最先允许的实现切片是 Runtime155-P0-001；其后必须先完成 frame admission identity和duplicate/concurrency RED，再进入完整 fixed transaction。Timer、multi-World budget、cadence和replay必须共享相同domain/tick/policy identity，不能各自再建一套临时时钟。
