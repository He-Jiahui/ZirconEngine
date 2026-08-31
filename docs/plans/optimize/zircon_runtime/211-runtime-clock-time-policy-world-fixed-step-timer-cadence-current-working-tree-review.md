---
title: Runtime Clock、Time Policy、World Fixed Step、Timer 与 Cadence 当前工作树工程化差距
category: zircon_runtime
report_id: Runtime211
review_date: 2026-08-31
baseline_head: working-tree
observed_head: f31fd06f69fdaedb70a0a56fe6d0268de1af83a6
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_incomplete
source_recheck_required: true
tooling_scope: excluded_by_user_request
coordination_tracking: skipped_by_user_request
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/103-runtime-clock-time-policy-world-fixed-step-timer-cadence-current-source-review.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/22/2026-08-24-fixed-step-transaction-architecture-and-performance-plan.md
  - docs/plans/optimize/zircon_runtime/22/2026-08-18-virtual-delta-validation-pending.md
related_reports:
  - docs/plans/optimize/zircon_runtime/99zm-physics-world-review.md
  - docs/plans/optimize/zircon_runtime/210-runtime-random-authority-stream-checkpoint-replay-consumer-performance-current-working-tree-review.md
related_code:
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/runtime/clock_source.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/scene/world_time
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/fixed_step_failure.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs
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

# Runtime Clock、Time Policy、World Fixed Step、Timer 与 Cadence 当前工作树工程化差距

## 1. 结论

当前时间子系统不是临时的单 delta 封装。Core 已收敛为 monotonic outer-frame clock 与后续 Level 的默认策略；每个 Level 独立持有 Virtual/Fixed、pause、scale、debt、policy generation 与 committed interpolation。`WorldFixedStep` 是不可复制 capability，`SimulationTickId` 带 world generation、fixed epoch 与 tick index；三个 fixed stage 全部成功后才提交 clock/debt。dynamic session 也已按 Client/Headless/Editor/Test 安装 8/16/4/1 的 fixed budget。旧 Runtime155-P0-001 所指的 stale test 已迁移，当前测试只从 `WorldTimeSnapshot` 读取 fixed plan；`FixedStepFailureReceipt` 也已补齐失败 phase、tick、system、已提交步数与剩余 debt。

这些进展仍不足以称为 Unreal 同等级、更不足以声称性能优于 Unreal：

- `tick_time` 在 `FrameClock` 锁内采样，释放后才在另一把锁内分配 outer frame；并发调用可以让 sample 顺序和 commit/ID 顺序相反。
- Level 已拒绝 duplicate、out-of-order 与无 discontinuity 的 skipped frame，但 snapshot 没有 Runtime/session identity；source generation 可倒退，active-step reentry 只有 `debug_assert`。
- fixed begin/commit/abort 只覆盖 clock/debt。World mutation、deferred command、RNG、physics、script 与外部 effect 没有共同 transaction。
- `fixed_step_budget` 仍复制给每个 World。N 个 World 的最坏工作量是 `N * budget * fixed stages * systems/substeps`，没有 Runtime 总预算、fairness、debt age 或 deadline。
- Product policy/profile/digest 已存在且 dynamic session 确实消费 profile，但公开 `advance_time_by(delta, budget)` 与 `tick_time(budget)` 仍可绕过策略；digest 在 production 只被定义/re-export，没有进入 BuildSet、session receipt、replay 或 diagnostics。
- World replacement 只清 interpolation history 和 single-step request，继续保留旧 World 的 Virtual/Fixed elapsed、debt、pause、rate 与 policy；因此按当前状态转移，新 World 可继续消费旧 debt。
- discontinuity 仍是单个 pending slot；custom `ClockSource` 倒退仍由 `saturating_duration_since` 静默折为零；generation/tick/elapsed 耗尽仍由 saturating arithmetic 静默冻结。
- App cadence 的 coalescing、runtime deadline、focus/occlusion 与 idle suppression 是真实能力，但 LowPower/FixedInterval 到期后仍设置 `next_deadline = now + interval`，既漂移相位，也没有 lateness/missed-period/catch-up/drop receipt。
- 引擎仍没有绑定 World domain 的 gameplay Timer。`TaskTimer` 是 process monotonic control-plane timer；UI timers 是输入私有状态，都不能替代可保存、可回放、随 pause/scale、在 fixed abort 后可重试的 Timer 产品。

本报告不新增 canonical finding，只刷新 Runtime155 的唯一账目：旧 P0 为 **1 Closed / 0 Open**；9 项 P1 为 **7 Open / 2 Partial / 0 Closed**；3 项 P2 为 **3 Open**。30 道资格门为 **16 Fail / 9 Partial / 5 Pass**。Tooling 按用户要求排除，本轮也未查询、轮询、等待或实时跟踪协调器。

## 2. 冻结范围与 currentness

fingerprint 口径为 lower-case repo-relative path、文件 SHA-256，按路径排序后以 `path<TAB>hash` 和 LF 拼接，再计算 SHA-256。

| 范围 | files | lines | non-empty | bytes | tests | ignored | unsafe tokens | HEAD / index / dirty | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| Zircon clock/time/fixed/timer/cadence + product/adapter/tests | **59** | **14,078** | **12,700** | **497,761** | **122** | **2** | **10** | **32 / 59 / 54** | `a3ea68a1e43d3b606f096ed6ba2e55f1666e54f9bc43769860d5b9eb715c9c36` |
| Unreal/Bevy/Godot/Fyrox/Unity Graphics references | **19** | **17,688** | **15,405** | **647,189** | **28** | **0** | **4** | n/a | `08ffddccf7822e023402c09520dc540dc6e85f53d3a9a23d6ae7bdcce5d71cdf` |

Zircon 选择集是可复算的精确集合：

1. 全递归 `zircon_runtime/src/core/framework/time/**/*.rs` 与 `zircon_runtime/src/scene/world_time/**/*.rs`。
2. Core owner：`clock_source.rs`、`frame_clock.rs`、`time.rs`、`time/product_policy.rs`、`handle/time.rs`、`runtime.rs`、`modules/time.rs`、`tasks/timer.rs`。
3. Scene owner：`level_system.rs`、`level_manager_lifecycle.rs`、`world_driver.rs`、`fixed_step_failure.rs`、`tick_context.rs`、`schedule_runner.rs`、`tick_policy.rs`、`runtime_scene_system.rs`、`scene_system_registry.rs`。
4. Tests：runtime `tests/time.rs`，scene `fixed_update.rs`、`world_driver.rs`、`world_time_controller.rs`。
5. Product：dynamic session `construction/profile/state/events`；App `frame_cadence/frame_loop` 与 lifecycle `events/state/transitions`；Runtime Interface `frame_shape/session/operation`。
6. Adapter/negative controls：Physics `settings/world_step_plan/physics_runtime_enabled`、Animation runtime、Script scene system、UI input timers。

19 个 reference 文件精确为 Bevy `lib.rs/time.rs/virt.rs/fixed.rs/timer.rs/stopwatch.rs/main_schedule.rs`；Unreal `App.h/App.cpp/TickTaskManager.cpp/TimerManager.h/TimerManager.cpp`；Godot `main_timer_sync.h/.cpp/main.cpp/timer.h/.cpp`；Fyrox `executor.rs`；Unity Graphics VFX control-track mixer。Unity Graphics 只用作 feature-level scrub/preview 参考，不能代表完整 Unity PlayerLoop。

当前 HEAD 为 `f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`。59 个输入虽全部进入 index，但只有 32 个存在于 HEAD，54 个 dirty；`real.rs` 的 tracked deletion继续作为 `Real -> MonotonicReal` hard-cut 迁移证据。因此 `source_recheck_required` 保持 true，当前在途工作树不能冒充 clean-checkout acceptance。

本轮是 review-only，没有修改 Rust/Cargo/ABI/tests/UI，也没有运行 Cargo、Miri、loom、fuzz、跨平台 replay、fault/soak、功耗或 benchmark。旧 managed validation 不冒充当前 dirty source 的执行证据。

## 3. 当前可保留的底座

| 能力 | 当前证据 | 保留条件 |
|---|---|---|
| Core/World 分层 | Core 只拥有 MonotonicReal outer input；Virtual/Fixed 为 Level-local | 不恢复全局兼容 virtual/fixed clock |
| domain identity | versioned registry、stable ID/unit、epoch/source generation | descriptor 必须补 owner/parent/availability/correlation，enum 不等于 installed capability |
| injected source | `ClockSource` 与拒绝倒退的 `ManualClockSource` | sample 返回 identity/sequence/result，并由 frame admission 线性化 |
| activation rebase | session activation 后 rebase，带 typed receipt | 多 cause 不得 last-write-wins，first-delta policy进入 receipt |
| virtual policy | pause、relative speed、max delta，checked conversion | profile/digest成为不可绕过的 session/BuildSet authority |
| outer admission checks | duplicate、out-of-order、skip-without-discontinuity typed reject | 加入 runtime/session/source monotonicity和release-mode reentry拒绝 |
| fixed capability | non-cloneable step + world/fixed/tick identity | clock、World、RNG、effect、physics共享一个 transaction |
| fixed failure receipt | phase/tick/system/committed count/debt/world generation | source也 typed；记录 staged/discarded effects与retry identity |
| committed interpolation | endpoints只随 commit更新，fraction来自实际 debt | endpoint绑定 committed World snapshot，不只绑定时间 ID |
| product profiles | Client 8、Headless 16、Editor 4、Test 1；64 Hz、250 ms | 需要 SLO、总预算、BuildSet/replay/diagnostic binding |
| App cadence | coalescing、runtime deadline、focus/occlusion、idle suppression | 保持 deadline phase并产出lateness/miss/degrade receipt |

## 4. 当前 owner 与断路图

```text
ClockSource
  `- FrameClock mutex: sample + take one pending rebase
       `- unlock
            `- RuntimeTimeAuthority mutex: allocate OuterFrameId + snapshot(raw budget)
                 |- Level A validates outer index -> full per-World fixed budget
                 `- Level B validates outer index -> same full budget again

LevelSystem / WorldTimeController
  |- Virtual/Fixed policy, pause, scale, debt
  |- duplicate/out-of-order/skipped-frame rejection
  `- WorldDriver fixed loop
       |- begin WorldFixedStep
       |- FixedFirst / FixedUpdate / FixedPostUpdate
       `- commit or abort clock/debt only

ProductTimePolicyDigest -- defined and tested, absent from BuildSet/replay/session diagnostics
TaskTimer -------------- process monotonic deadline primitive
UiInputTimerState ------ input/UI private deadline state
World gameplay Timer --- absent
```

目标不是一把全局时间锁。outer sample/admission 需要唯一线性化顺序；预算决策发布 immutable grant；World 并行执行本地 staged transaction；最终只在短 commit lane 内一起提交 clock、World、RNG 与 effects。

## 5. Runtime22 既有 finding 状态

| Existing ID | 当前状态 | 当前工作树证据 |
|---|---|---|
| TIME-P0-001 | Source root closed / validation pending | Update 默认 Virtual，pause/scale/clamp已进入 typed tick route；本轮未执行动态验证 |
| TIME-P0-002 | Partial | 逐步 begin/commit/abort 与 typed failure receipt存在；完整 World/RNG/effect/physics事务仍缺失 |
| TIME-P1-001 | Partial | Registry v1、ID/unit/epoch/source generation存在；真实 owner/parent/availability仍不完整 |
| TIME-P1-002 | Partial | `Real` 已 hard-cut 为 `MonotonicReal`；WallUtc/Calendar service仍无 concrete owner |
| TIME-P1-003 | Closed | Core snapshot只含 outer evidence；World snapshot形成 virtual/fixed proposal |
| TIME-P1-004 | Partial | outer source可注入；profiling/task/UI clock inventory与声明产物仍不完整 |
| TIME-P1-005 | Partial | Core有manual source；Runtime Interface/App无external/replay strategy、version或seek capability |
| TIME-P1-006 | Closed | activation成功后 rebase，receipt存在 |
| TIME-P1-007 | Partial | lifecycle cause已 typed；单 pending slot覆盖更早 cause |
| TIME-P1-008 | Closed | virtual scale先校验/限制再转换，不再由极大有限值触发转换panic |
| TIME-P1-009 | Closed at public policy boundary | policy transaction fail closed；raw frame APIs仍是另一条绕过面 |
| TIME-P1-010 | Partial | version/profile/digest存在；digest未消费，raw budget可绕过 |
| TIME-P1-011 | Partial | debt/whole steps/ratio可见；无age、catch-up/drop/degrade/fatal policy |
| TIME-P1-012 | Closed | total debt和modulo interpolation fraction已分离 |
| TIME-P1-013 | Source root closed | profiles分化为8/16/4/1；SLO/benchmark/energy仍是资格缺口 |
| TIME-P1-014 | Closed | `SystemTickContext`带stage/domain/outer/sim tick/delta/elapsed/world generation |
| TIME-P1-015 | Closed | tick policy与registry admission拒绝非法stage/domain组合 |
| TIME-P1-016 | Closed | 每个Level独立持有pause/scale/debt/epoch |
| TIME-P1-017 | Partial | active/debt时拒绝rate change并bump epoch；没有迁移或live multi-World transaction |
| TIME-P1-018 | Closed | interpolation只读committed endpoints和实际debt |
| TIME-P1-019 | Open | App到期后仍`now + interval`，无phase/lateness/miss合同 |
| TIME-P1-020 | Partial | typed context到达adapter入口；Script降为裸float，Physics另有fixed_hz/max_substeps |

TIME-P1-021..040 的 Random、schedule digest、effect journal、replay、checkpoint、telemetry与资格矩阵继续由 Runtime22/Runtime210 等 owner 追踪，本报告不重复编号。

## 6. Runtime155 finding 刷新

### Runtime155-P0-001：stale fixed-plan test

**状态：Closed at source / validation pending。**

旧报告记录的 `outer.fixed_step_plan()` 调用已不存在。当前 `world_time_controller.rs` 测试只通过 `WorldTimeSnapshot::fixed_step_plan()` 观察 Level-local fixed proposal，并从 outer snapshot读取 `fixed_step_budget()`。这关闭了旧方法解析断口；但本轮没有运行 Cargo，且相关选择集高度 dirty，因此不能写成动态 acceptance。

### Runtime155-P1-001：sample 与 outer-frame commit 不在同一 admission lane

**状态：Open。** `CoreHandle::tick_time` 在 `handle/time.rs:48-67` 先锁 frame clock取 sample，释放后才锁 time authority。A sample、B sample、B commit、A commit仍是合法交错；`advance_time_by`还可以绕过source lane直接竞争outer ID。

必须引入 `FrameAdmissionAuthority`：在一个可证明线性化的流程中完成 sample validation、discontinuity drain、installed policy选择、ID reservation与ordered publication。实现可以分锁，但必须有sequence reservation/ordered commit及barrier并发测试。

### Runtime155-P1-002：World frame admission只完成了局部身份校验

**状态：Partial。** `WorldTimeController::validate_outer_frame` 已 typed reject duplicate、out-of-order和无discontinuity的skip，这是对旧报告的真实修复。但 snapshot没有Runtime/session ID；source generation只被直接写入World clock，可从较大值倒退；任何discontinuity都放行任意frame gap；`advance`对active fixed step只做`debug_assert`，release可先推进并记录outer frame，再在后续begin失败。

应以 `{RuntimeSessionId, ClockSourceId, source_generation, sample_sequence, OuterFrameId}` 构造不可伪造的 admission receipt；World接纳必须在任何状态mutation之前验证exact-once、strict source monotonicity、允许的gap cause和active-step空闲，并对所有拒绝保持bitwise不变。

### Runtime155-P1-003：ProductTimePolicy已接线，但仍不是不可绕过的authority

**状态：Partial。** dynamic session构造时先应用profile policy，`tick_frame`也读取installed session policy budget。这比旧报告完整。问题是 public `advance_time_by(Duration,u32)` 和 `tick_time(u32)`仍允许任意delta/budget；`ProductTimePolicyDigest` production grep只命中定义/re-export，没有BuildSet、replay、session receipt、diagnostic consumer。

普通入口应改为无raw参数的 `tick_frame()`，只读immutable installed policy。manual advance必须由Test/Replay/ServerExternal profile显式领取 capability。policy version/digest/limits要进入session creation receipt、BuildSet、checkpoint/replay header与telemetry。

### Runtime155-P1-004：每个World获得完整budget

**状态：Open。** outer snapshot注释和实现仍把同一 `u32`预算交给每个World；WorldDriver逐World执行全部step。多World总工作量无Runtime上限、deadline、weight、round-robin、starvation或debt-age证据。

引入 `FixedWorkBudgetController`，先收集各World debt/priority/visibility/authority，再按deterministic order发布 `WorldFixedStepGrant`。Client/Headless/Editor/Test分别定义retain/drop/degrade/fatal与最低服务，grant同时约束physics substep、task fan-out和时间成本。

### Runtime155-P1-005：replacement time disposition仍隐式继承

**状态：Open。** `reset_after_world_replacement` 只清previous/current interpolation和single-step，保留Virtual/Fixed elapsed、debt、pause、rate、policy generation。新World因此可执行旧World债务。

所有replacement/travel/hot-reload/rollback入口必须要求 `WorldReplacementTimePolicy::{Preserve,Reset,Restore}`，在World/time lane内原子完成并返回old/new world generation、clock epochs、preserved/discarded debt、reason与checkpoint identity。

### Runtime155-P1-006：discontinuity仍last-write-wins

**状态：Open。** `FrameClock::rebase_for` 以 `pending_rebase = Some(receipt)`覆盖较早cause。多个suspend/resume/surface/occlusion事件在下一帧前发生时，历史、次序和每个first-delta policy丢失。

替换为bounded ordered journal或typed batch receipt。容量满只能产出明确overflow/coalescing摘要或拒绝，不能静默覆盖。

### Runtime155-P1-007：custom ClockSource倒退仍被折零

**状态：Open。** `ManualClockSource`自己拒绝倒退，但public trait只返回`Instant`；`FrameClock::tick`使用`saturating_duration_since`。任意实现的倒退样本变成零delta且last_tick回退，后续区间可能重复计入。

改为 `sample() -> Result<ClockSample, ClockSourceError>`，携source ID/generation/sequence/instant/uncertainty。admission对倒退、重复sequence与异常jump执行reject/rebase/quarantine并保留receipt。

### Runtime155-P1-008：identity与elapsed耗尽仍静默冻结

**状态：Open。** outer index、policy/source generation、fixed tick、elapsed/debt多处仍使用saturating arithmetic。达到上限后ID重复或状态冻结，不会产生terminal failure。

统一采用checked successor与 `ClockIdentityExhausted`，或通过新session/source generation做可证明唯一的rollover。所有mutation必须在耗尽检查后发生，并增加near-max property/fault tests。

### Runtime155-P1-009：World gameplay Timer仍缺失

**状态：Open。** `TaskTimer`与UI timer都没有World generation、Virtual/Fixed domain、SimulationTickId、pause/scale、save/replay或step commit语义。

建立bounded `WorldTimerService`：generation-safe handle、one-shot/repeating、Virtual/Fixed/MonotonicReal domain、first delay/rate、pause/cancel/reschedule、stable same-deadline order、per-owner quota、catch-up/coalesce/skip cap、serialization/migration与telemetry。Fixed timer只向当前step journal写事件，commit后发布，abort后同tick可重试。

### Runtime155-P2-001：TimeModule描述错误

**状态：Open。** `modules/time.rs:15`仍宣称Core拥有real/virtual/fixed clocks，与Level-local hard-cut冲突。描述应改为outer monotonic frame authority + default World policy，并列出真实capability版本。

### Runtime155-P2-002：零delta诊断保留旧current value

**状态：Open。** zero-delta路径记录frame count后返回，frame-time/FPS current仍指向旧frame。应发布typed validity/reason，或写0 frame-time并将FPS标为unavailable。

### Runtime155-P2-003：ClockDomainDescriptor过薄

**状态：Open。** descriptor只有ID/unit，不能表达owner、availability、parent、origin、rate、precision/error或correlation。未安装的Audio/Network/Media/EditorPreview等domain不能因enum存在就被当成能力。

## 7. 参考引擎逐文件对照

| 参考 | 可吸收的工程边界 | Zircon必须更严格之处 |
|---|---|---|
| Bevy Time/App | `TimeUpdateStrategy`区分Automatic/ManualInstant/ManualDuration/FixedTimesteps；Virtual驱动Fixed；每次fixed schedule前推进；Timer有pause/repeat/本tick完成次数/serde | Bevy默认fixed loop没有Zircon需要的Runtime总预算，也不是失败后完整state rollback模型 |
| Unreal TickTaskManager | StartFrame绑定World、delta与tick type，RunTickGroup有明确phase，EndFrame收口；并行queue与newly-spawned runaway cap属于工程级调度 | 需要吸收owner/phase/runaway protection，但不能复制全局mutable `FApp`或假定Unreal已有通用World rollback |
| Unreal TimerManager | World/game-thread owner、generation handle、set/clear/pause/query/next-tick、heap、per-frame guard与elapsed call-count处理 | Zircon Timer还要绑定domain/session/tick/checkpoint和transaction journal；传统delegate接口不是最终ABI |
| Godot MainTimerSync/Timer | accumulator、jitter correction、interpolation、max physics steps；Timer区分idle/physics、one-shot、pause、ignore time scale | drop/correction必须成为versioned product policy和receipt，不能埋在算法常量里 |
| Fyrox executor | fixed lag loop、throttle与过载fast-forward避免spiral/hang | `f32` lag和大delta fast-forward不足以作为跨平台determinism/replay authority |
| Unity Graphics VFX | scrub分chunk、backward seek重建、seed/reinit/prewarm、fixed simulate与max scrub time warning | 只借鉴feature-level preview/scrub操作；不据此推断完整Unity clock或PlayerLoop |

参考源码证明了strategy、tick phase、timer product、overload policy和scrub control必须是显式产品能力；它们没有替Zircon解决跨World总预算、full-state transaction或BuildSet-bound replay。

## 8. 目标架构

```text
ClockSourceRegistry
  `- ClockSource::sample -> ClockSample(source, generation, sequence, instant/error)
       `- FrameAdmissionAuthority (single ordered lane)
            |- drain ClockDiscontinuityJournal
            |- bind ProductTimePolicyDigest + BuildSet
            |- allocate OuterFrameId
            `- FrameAdmissionReceipt
                 `- FixedWorkBudgetController
                      |- deterministic World grants
                      `- retained/dropped/degraded debt receipts

LevelSystem::accept_outer_frame(receipt)
  |- validate session/source/frame exactly once before mutation
  |- advance WorldVirtual and propose WorldFixed work
  `- SimulationStepTransaction per grant
       |- typed SystemTickContext
       |- staged World commands + component deltas
       |- staged RNG progress
       |- staged event/effect/network/audio outbox
       |- staged physics/animation/script output
       `- commit all + clock/debt, or abort all + typed receipt

WorldTimerService
  `- bounded domain timer heap/wheel -> stage timeout events in step transaction

TimeTelemetry
  `- frame/cadence/discontinuity/debt/grant/commit/abort/timer/exhaustion correlation
```

性能原则是短 admission lane、immutable grant、World-local并行staging、allocation-bounded timer和短commit publication；不是把所有timer/system调用串到中央manager。

## 9. 分层重构计划

### M0：冻结当前迁移与可执行基线

1. 为已迁移的stale test运行focused compile/test，并记录当前HEAD/index/dirty输入。
2. 清除Core compatibility fixed clock残留，修正TimeModule文案。
3. 在current working tree重检后才升级source-closed项，不引用历史validation替代当前证据。

### M1：Frame admission与完整identity

1. 引入 `RuntimeSessionId/ClockSourceId/ClockSampleSequence/OuterFrameId`。
2. 线性化sample、discontinuity drain、policy binding、ID allocation和publication。
3. World admission在mutation前typed rejectduplicate/stale/cross-session/source rollback/reentrant/gap-policy mismatch。
4. 增加barrier控制的并发交错、source fault和near-max identity tests。

### M2：Product policy、BuildSet与多World budget

1. 普通frame API删除raw budget/delta；manual path只通过capability开放。
2. policy digest进入session receipt、BuildSet、replay/checkpoint和diagnostics。
3. Runtime收集World debt并用deterministic allocator发布总预算grant。
4. 为四种profile定义latency/correctness/power/debt-age/degrade SLO。

### M3：完整simulation transaction

1. 扩展现有non-cloneable fixed capability为staged World mutation/effect transaction。
2. Random、Physics、Script、events/network/audio输出绑定同一`SimulationTickId`。
3. receipt记录失败stage/system、prior commits、discarded effects、remaining debt与retry identity。
4. direct mutable system迁移到transactional params，或从deterministic profile fail closed。

### M4：replacement、rate、discontinuity与cadence

1. replacement强制Preserve/Reset/Restore time policy。
2. live multi-World policy使用prepare-all/commit-all，失败返回完整per-World receipt。
3. cadence按previous deadline保持phase，记录lateness/miss/catch-up/drop；模式切换显式rebase。
4. discontinuity改bounded journal和per-cause first-delta policy。

### M5：Timer与subsystem adapters

1. 实现allocation-bounded timer kernel、generation handle、stable ordering和quota。
2. 连接Virtual/Fixed/Real domain、pause/rate/repeating/catch-up、save/replay和step journal。
3. Script/Animation/Physics保留typed tick identity；Physics私有fixed authority按Runtime99zm hard-cut。
4. EditorPreview/Media/Audio/Network只有真实owner/correlation artifact存在时才能标Available。

### M6：资格与性能

1. required lane覆盖duplicate/stale/concurrency/overflow/failure/retry/replacement/rate/timer storm。
2. deterministic dual-run比较tick/state/effect/RNG/timer digest与first divergence。
3. benchmark矩阵覆盖1/10/100 World、0/1/8/capped steps、1K/100K timers、1/100/1000 systems。
4. 输出admission lock wait/hold、grant latency、debt age、commit/abort、allocations、RSS、energy与p95/p99 hitch。

## 10. 资格门

| Gate | 状态 | 当前证据 / 缺口 |
|---|---|---|
| G1 clean source + focused test compile | Partial | stale方法调用已静态关闭；59个输入中54 dirty，本轮未执行Cargo |
| G2 versioned clock taxonomy/stamps | Partial | v1 ID/unit/epoch/source generation存在，descriptor/owner不完整 |
| G3 injected/manual clock | Partial | Core可注入，dynamic ABI/seek/fault identity缺失 |
| G4 single ordered frame admission | Fail | frame/time两锁之间仍可重排 |
| G5 duplicate/stale/cross-session reject | Partial | duplicate/order/skip已拒绝；session/source/reentry未闭合 |
| G6 activation rebase receipt | Pass | activation完成后rebase并携typed receipt |
| G7 ordered discontinuity history | Fail | pending slot仍last-write-wins |
| G8 public policy validation | Pass | invalid max delta/speed/timestep fail closed |
| G9 product policy authority/digest | Partial | profile已生产接线；raw API可绕过且digest未消费 |
| G10 per-World pause/scale/debt | Pass | Level-local controller独立 |
| G11 virtual pause/scale/clamp routing | Pass | default Virtual与explicit MonotonicReal分离 |
| G12 fixed proposal/commit/abort | Partial | clock/debt逐步事务存在，完整state未覆盖 |
| G13 World/RNG/effect atomicity | Fail | mutation与external progress可在abort后保留 |
| G14 typed fixed failure receipt | Partial | fixed receipt已存在；source/whole-state rollback/effect evidence不完整 |
| G15 live multi-World policy transaction | Fail | Runtime default只影响后续Level |
| G16 fixed-rate migration | Partial | active/debt reject与epoch bump存在，无迁移policy |
| G17 committed interpolation evidence | Pass | endpoints只在commit更新，fraction来自实际debt |
| G18 Runtime total fixed-work budget | Fail | 每个World复制完整预算 |
| G19 debt catch-up/drop/degrade/fatal policy | Fail | 只有retained debt，无age/SLO/terminal receipt |
| G20 gameplay Timer product | Fail | 只有process task timer与UI私有timer |
| G21 typed subsystem adapter | Partial | typed入口存在；Script降float，Physics contract分叉 |
| G22 single Physics/fixed clock | Fail | `fixed_hz/max_substeps`私有authority仍存在 |
| G23 phase-preserving cadence/lateness | Fail | due后`now + interval`，无miss/lateness |
| G24 bounded correlated TimeTelemetry | Fail | current diagnostics不覆盖zero delta、debt/discontinuity/cadence miss |
| G25 BuildSet/replay/checkpoint binding | Fail | policy/clock/timer identity未进入完整artifact |
| G26 identity exhaustion/rollover | Fail | saturating generation/tick可能冻结或重复 |
| G27 explicit replacement time disposition | Fail | replacement只清interpolation/single-step |
| G28 concurrency/fuzz/fault/soak lane | Fail | 当前无执行证据 |
| G29 cross-platform determinism matrix | Fail | 无Windows/Linux/toolchain/thread-count证据 |
| G30 comparative performance/energy evidence | Fail | 无同场景Unreal对比或当前dirty source profile |

合计：**16 Fail / 9 Partial / 5 Pass**。任何P0重新出现、G4/G13/G18/G25未关闭、required correctness lane未通过时，都不得宣称时间子系统工程级完成。

## 11. 必需测试与性能证据

最低RED/qualification集合：

1. 两线程强制A sample、B sample、B commit、A commit，最终receipt仍按sample sequence发布。
2. 同一World duplicate、out-of-order、cross-session、source rollback、active-step reentry全部typed reject且状态bitwise不变。
3. 多个discontinuity同帧前到达，保留顺序/cause/policy；overflow有明确摘要。
4. fixed step在World mutation、RNG draw、physics result、script/event effect后失败，retry看到相同pre-state和随机序列。
5. 100个World共享一个Runtime总budget，grant总和不超policy且不存在永久starvation。
6. replacement Preserve/Reset/Restore分别证明debt、epoch与tick identity。
7. generation/tick接近`u64::MAX`时typed exhaustion或安全rollover，不重复identity。
8. 100K timers同deadline保持stable order、quota、cancel generation与bounded memory；fixed abort不发布timeout。
9. cadence晚2.5个interval时按profile保持phase或明确drop/catch-up，并记录lateness/miss。
10. 同BuildSet双跑比较每tick state/effect/RNG/timer digest并定位first divergence。

性能报告至少包含frame admission p50/p95/p99、lock wait/hold、World grant compute、fixed begin/commit/abort、timer insert/cancel/expire、debt age、allocations/frame、peak timer bytes、RSS、CPU package energy。与Unreal比较必须固定World/system/timer数量、fixed rate、hitch输入、render开关和功能量；不能用功能缺失带来的低成本声称性能更优。

## 12. Ownership与禁止旁路

- Runtime22继续拥有clock/fixed/determinism/replay parent architecture。
- Runtime210拥有RandomService/checkpoint与RNG step transaction；本报告只要求共同commit边界。
- Runtime99zm拥有Physics私有fixed authority hard-cut和backend step identity。
- App owner实施cadence scheduler，但policy/digest/clock receipt属于Runtime公共合同。
- Runtime Interface后续承载external/manual/replay clock capability、pause/step/scrub operation与version negotiation。
- Editor只能消费Runtime operation/receipt实现pause、single-step、scrub和domain inspector，不得复制第二套time authority。
- Tooling/WOC Rust迁移前继续排除。
- 禁止以调用者承诺单线程、扩大per-World budget、清空debt、吞掉clock regression、`debug_assert`、saturating ID、私有Timer或私有Physics clock作为修复。

## 13. 交付判定

本轮交付是current-working-tree review与分层重构计划，不是实现完成证明。当前可以保留Core/World分层、typed tick context、World-local policy、逐步clock/debt transaction、partial frame admission和typed fixed failure receipt；不能保留可重排outer admission、可绕过policy、每World复制预算、隐式replacement debt继承、last-write-wins discontinuity、静默source regression/identity saturation、private subsystem clock或缺失的gameplay Timer产品。

下一实现批次应从M0/M1开始，先把身份、线性化与可执行基线固定，再做M2总预算和M3完整transaction。Timer、Editor scrub、replay和性能比较都依赖这些owner边界，不能用并行临时实现提前伪造完成度。
