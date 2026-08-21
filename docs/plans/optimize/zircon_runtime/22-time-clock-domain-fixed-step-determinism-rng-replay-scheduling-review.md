---
related_code:
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/framework/time/clock.rs
  - zircon_runtime/src/core/framework/time/fixed.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
  - zircon_runtime/src/core/framework/time/real.rs
  - zircon_runtime/src/core/framework/time/virtual_clock.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/ecs/system/native/runtime_scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/scene/ecs/schedule_runner
  - zircon_runtime/src/scene/ecs/commands/worker_command_buffer.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/script
  - zircon_plugins/animation/runtime/src
  - zircon_plugins/physics/runtime/src
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/support.rs
  - zircon_plugins/ai/runtime/src
  - zircon_plugins/particles/runtime/src/simulation/rng.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
  - zircon_plugins/particles/runtime/src
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs
  - zircon_runtime_interface/src
  - zircon_editor/src
  - examples/woc/native/crates/woc_parity/src/rng.rs
tests:
  - zircon_runtime/src/core/framework/tests
  - zircon_runtime/src/core/runtime/tests
  - zircon_runtime/src/scene/ecs/schedule_runner/tests
  - zircon_runtime/src/scene/tests/ecs_schedule
  - zircon_runtime/src/scene/tests/ecs_systems
  - zircon_plugins/ai/runtime/src/tests
  - zircon_plugins/particles/runtime/src/tests
  - zircon_app/src/entry/tests/runtime_entry_source_guards
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_tooling/11-woc-parity-oracle-trace-golden-differential-replay-evidence-review.md
  - docs/plans/optimize/zircon_tooling/22-magic-constant-sentinel-threshold-timeout-capacity-budget-policy-convergence-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
reference_engines:
  - dev/bevy/crates/bevy_time/src/time.rs
  - dev/bevy/crates/bevy_time/src/virt.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/bevy/crates/bevy_time/src/real.rs
  - dev/godot/main/main_timer_sync.h
  - dev/godot/main/main_timer_sync.cpp
  - dev/godot/core/os/time.h
  - dev/godot/core/math/random_pcg.h
  - dev/godot/core/math/random_number_generator.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformTime.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/App.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/RandomStream.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/TickTaskManagerInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/DemoNetDriver.h
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Camera/HDCamera.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/HDRenderPipeline.PostProcess.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricLighting/HDRenderPipeline.VolumetricLighting.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 22 · Time、Clock Domain、Fixed Step、Determinism、RNG、Replay 与 Scheduling 工程化差距

## 1. 结论

Zircon 的时间基础不是空壳。`Time<Real>`、`Time<Virtual>`、`Time<Fixed>`已经借鉴 Bevy 分离真实、游戏和固定时间；虚拟时钟具备暂停、相对速度和最大帧增量，Fixed 时钟保留 overstep 并限制单帧步数。Scene schedule 用拓扑计划、稳定 system order/ID/step rank 排序，worker command buffer 又按编译期确定的 key 合并，重复 key 会被拒绝。ECS 状态与多处命令路径广泛使用 `BTreeMap/BTreeSet`。这些都是应保留的工程方向，不应在重构时退回单一 `delta_seconds` 或用线程完成顺序决定 world mutation。

但当前产品执行链没有遵守这套时间模型。`RuntimeTimeClocks::advance_by`先从真实增量推进 Virtual/Fixed，而`WorldDriver`随后却把`advance.real_delta()`转换为`f32`传给普通`Update/PostUpdate`。因此 Virtual 的 pause、time scale 和 250 ms clamp只约束 Fixed accumulation；Animation 的`PostUpdate`、脚本`Update`及其他普通系统仍按未经缩放的真实增量运行。应用被遮挡时 cadence可降为1秒，resume也没有时钟rebase，这条旁路会直接把1秒原始delta送入普通游戏逻辑。当前“暂停”不是全产品暂停，“time scale”也不是全world时间尺度。

Fixed 路径有第二个硬语义问题。`Time<Fixed>::drain_steps`在任何 Fixed schedule执行之前，已按本帧全部step count一次性增加elapsed和frame index；返回值只有`FixedStepPlan`计数/总消耗。`WorldDriver`再重复执行同一Fixed stage，但每个子步查询到的是同一个最终`fixed_time()`，没有独立`SimulationTickId`、逐步快照或commit点。若第N步失败，时钟仍已推进全部计划步；若系统记录事件、RNG、网络输入或snapshot，无法证明它属于哪一个真实子步。这不是诊断缺口，而是会破坏确定性状态转移、回放和失败恢复的核心语义错误。

RNG 与 replay 也没有形成引擎合同。AI weighted selector用`DefaultHasher(tree_id,node_id,tick)`生成选择，没有world/entity/master seed；同tick的不同实例会得到同一选择，算法又不是显式版本化的跨BuildSet格式。Particles另有CPU自定义`u64` RNG和GPU hash/salt/frame算法，WOC parity再维护一套Mulberry RNG；三者都没有共享算法ID、stream key、draw index、snapshot和迁移语义。仓内没有发现生产代码直接调用`thread_rng`这类无控制全局随机源，这是可保留的基础，但“各模块自己可复现”不等于“引擎可回放”。

本篇拥有引擎级Clock Authority、clock-domain taxonomy、frame/fixed tick identity、逐步fixed commit、time mutation transaction、RNG stream/schema、通用input journal/checkpoint/replay/divergence和跨系统时间关联。Runtime08A继续拥有Physics双fixed clock；Runtime08C拥有Animation subsystem clock/scrub；Runtime08F拥有AI产品可达性和其局部seed finding；Runtime09H1拥有render temporal history；Runtime11A拥有UI timer/zero timestamp P0；Editor07拥有Play/PIE控制产品；Tooling11拥有WOC parity oracle。本篇不重复这些条目。本轮登记 **2项P0、40项P1和12项P2**，均未实施。

## 2. 审查边界、方法与 currentness

### 2.1 物理扫描

本轮对七个产品家族中的tracked Rust做production-like词法inventory：排除路径中明显的tests/benches/examples/fixtures/generated/vendor/target，并在纯`#[cfg(test)]`尾部前截断。结果覆盖12,170文件、约1,170,797行前缀代码；`Instant::now`有385处/125文件，`SystemTime`或`UNIX_EPOCH`有188处/50文件，Duration constructor有331处/103文件，`delta_seconds`有176处/43文件，`frame_index`有1,140处/111文件，seed/random信号有181处/34文件。`HashMap`类型信号834处/286文件，`HashSet`237处/128文件，`BTreeMap`2,216处/673文件，`BTreeSet`679处/281文件。

这些都是复核入口而不是缺陷计数：宏展开、父级cfg、build script和第三方内部不在静态前缀模型中；`frame_index`/`random`包含渲染、测试辅助与业务命名；`HashMap`只在输出顺序进入snapshot/digest/network/replay时才需要canonicalization，不能机械替换为树结构。确认项来自逐调用链阅读，而不是关键字外推。

### 2.2 深读调用链

1. `FrameClock::tick -> RuntimeTimeClocks::advance_by -> RuntimeTimeAdvance -> RuntimeSessionState::tick_frame -> WorldDriver::tick_level`。
2. `Time<Real> -> Time<Virtual> -> Time<Fixed>::drain_steps -> FixedStepPlan -> FixedUpdate/Update/PostUpdate`。
3. App interactive/headless/background cadence、resume/occlusion路径与dynamic session profile。
4. Native schedule plan、parallel executor result order、worker command buffer merge与ECS ordered containers。
5. Animation、script、physics、AI、particle、runtime UI、editor input metadata与WOC parity对时间/RNG的消费。
6. Runtime Interface/App ABI中pause/scale/manual clock/fixed tick/replay surface的存在性。

本轮源revision为`ae2be3d865a937b9ed368bf965592045346c64e3`。`zircon_runtime/src/dynamic_api/session/profile.rs`在审查时有其他session的在途修改，所以本篇标记`source_recheck_required: true`；报告没有覆盖或吸收该修改。动态验证仍被既有Editor、Hub、WOC和plugin构建阻断，且这是一轮review-only工作，本轮没有重跑不可能抵达时间产品语义的相同Cargo/npm lane。

## 3. 当前可保留的工程基础

| 基础 | 当前证据 | 保留条件 |
|---|---|---|
| 三时钟模型 | `Time<Real/Virtual/Fixed>`分离，Virtual有pause/speed/max delta | 修正consumer语义并引入typed domain，不删除分层 |
| Fixed overstep | accumulator保留未消费债务，单帧step有cap | 改为逐step advance/commit，并公开debt policy/telemetry |
| 稳定schedule | order、system ID、step rank及拓扑plan决定运行顺序 | 顺序必须进入BuildSet/trace；并行side effect仍受effect contract约束 |
| 稳定worker merge | worker buffers按deterministic key排序，duplicate拒绝 | key必须绑定system/tick/world generation并有冲突receipt |
| Ordered ECS状态 | 多处`BTreeMap/BTreeSet`用于状态和命令路径 | snapshot/hash/serialization必须声明canonical order，不机械全局替换 |
| 有界Virtual delta | 默认最大250 ms，避免Fixed无限catch-up | 上限成为profile policy，discontinuity与dropped/debt必须可观测 |
| 显式局部seed | AI/Particles/WOC没有直接依赖线程全局RNG | 收敛为versioned stream authority，保留可重现能力 |
| Temporal history底座 | Graphics已有per-camera history/generation/reset局部机制 | 消费统一frame/tick/discontinuity ID，不把render frame冒充sim tick |

## 4. 参考实现给出的边界

### 4.1 Bevy

Bevy `Time<Real/Virtual/Fixed>`不仅分类型，也在Fixed schedule期间把generic `Time`切换到当前逐步Fixed clock；Fixed每运行一步才前进一个timestep，并可显式discard overstep。Real第一次更新只建立起点、delta为零；`TimeUpdateStrategy`允许manual duration/instant注入。这给出的关键合同是“时钟类型必须跟实际执行域一致”和“可测试的外部clock source”，不是照抄其API名称。

### 4.2 Unreal

Unreal `FApp`区分current/last/delta/idle/game/fixed/current frame time，Tick Task Manager明确StartFrame、pause frame、tick groups和EndFrame。`FRandomStream`保存initial/current seed并可重置/序列化状态；DemoNetDriver显式拥有frame、checkpoint、scrub、fast-forward和replay task状态。Zircon需要同等级的状态身份、控制点和回放operation，而不是只有`delta_seconds`和一个统计frame count。

### 4.3 Godot

Godot `MainTimerSync`协调fixed/idle steps和jitter，OS time与游戏时间职责分离；PCG和RandomNumberGenerator公开seed/state/randomize语义。可借鉴的是“同步算法、随机算法和状态是版本化产品合同”，而不是把系统wall time塞进游戏随机种子。

### 4.4 Fyrox 与 Unity Graphics

Fyrox executor用`Instant`、fixed timestep和lag循环驱动插件/脚本/UI，是产品loop基线，但不足以单独定义跨平台确定性。Unity Graphics的`HDCamera`、PostProcess和Volumetric Lighting维护per-camera frame/history、current/previous delta、reset history和有效性；它证明图形时间是simulation clock的消费者并有独立render identity，不能作为全局游戏clock authority。

## 5. P0：产品时间与Fixed状态转移硬错误

### TIME-P0-001 · 普通Update绕过Virtual时钟，pause、time scale和hitch clamp不控制产品游戏逻辑

`RuntimeTimeClocks::advance_by`正确计算`virtual_delta = min(real_delta * effective_speed, max_delta)`并用它积累Fixed，但`RuntimeTimeAdvance`没有公开virtual delta；`WorldDriver`只能取`advance.real_delta()`，随后把它传给每个非Fixed stage。Animation在`PostUpdate`直接消费该`context.delta_seconds`，脚本Update也在同一路径。因此Virtual paused时普通系统仍执行并收到非零delta；time scale不会缩放普通系统；后台1秒cadence或resume长间隔可越过250 ms保护直接进入动画/脚本/游戏逻辑。

重构必须先把`FrameTimeSnapshot`作为单帧不可变输入，至少包含raw real、clamped virtual、effective speed、pause/discontinuity、outer frame ID和mutation generation。每个schedule stage声明消费哪个ClockDomain；默认游戏Update消费Virtual，FixedUpdate消费逐步Fixed，允许明确的Real-time系统只能通过metadata注册并进入审计。pause期间执行`RunWhenPaused`/editor/diagnostic系统应走独立pause stage，不能靠传零delta或继续全量Update猜测。resume、occlusion、device recreation和debug break要产生ClockDiscontinuity并按profile选择rebase、clamp、debt或reject。

验收必须覆盖：pause后Animation/script/gameplay状态不变；Real-time diagnostic仍按策略运行；0.25x/2x time scale对Update和Fixed累计一致；1秒遮挡/30秒suspend后不注入巨型Virtual delta；所有stage trace能证明实际消费域。现有Runtime11A UI timer P0由其报告修复，本条只要求它最终消费统一domain。

### TIME-P0-002 · Fixed clock在执行前预推进整批步骤，子步身份、失败原子性与回放状态均不真实

`Time<Fixed>::drain_steps`先调用`take_steps`，再一次性增加`delta=timestep`、`elapsed += timestep*steps`和`frame_index += steps`。`WorldDriver`随后按`step_count`循环执行Fixed stage。结果是本帧全部子步读取相同的最终elapsed/frame index；第1步与第8步无法区分。任何第N步system error都发生在时钟已经宣称全部步骤完成之后，remaining debt、event/RNG/network command归属和snapshot boundary都不能恢复。`FixedStepPlan::overstep_fraction()`还把大于一步的债务clamp到1，进一步隐藏cap后的真实积压。

重构必须改成`begin_fixed_step -> SimulationTickContext -> execute schedule -> commit_fixed_step`。每步只推进一个timestep并生成稳定`SimulationTickId(world_generation, fixed_epoch, tick_index)`；失败时输出包含已提交/未提交步骤、effect journal和remaining debt的typed receipt。若系统允许不可回滚外部effect，必须在commit后由outbox发布；若step失败，不得把未来tick的clock、RNG draw、network ack或snapshot index提前公开。插值上下文单独携previous/current committed snapshot和unconsumed fraction，不通过伪造Fixed elapsed实现。

验收必须故障注入每个Fixed stage和每个step：第N步失败只能提交前N-1步；每步看到连续且唯一的tick ID、elapsed、RNG stream position；重试/回放产生相同state/effect digest；max-step cap后的debt完整可观测，且policy明确是保留、丢弃、降级还是终止。

## 6. P1：Clock Authority 与时间控制面

| ID | 当前差距 | 需要重构 |
|---|---|---|
| TIME-P1-001 | 没有Canonical ClockDomain taxonomy，monotonic、UTC、virtual game、fixed sim、input、render、audio、network、media/editor preview混用 | 建立versioned `ClockDomainRegistry`，每个时间值携domain、unit、epoch、source generation |
| TIME-P1-002 | `Real`注释称wall-clock，但实现来自monotonic `Instant` | 将其定义为MonotonicReal；UTC/calendar单独由WallClock service提供，禁止互换 |
| TIME-P1-003 | `RuntimeTimeAdvance`只暴露real delta和fixed plan，没有virtual delta、pause、scale或discontinuity | 替换为不可变`FrameTimeSnapshot`和typed accessors |
| TIME-P1-004 | 385个`Instant::now`/188个wall-time信号没有统一可注入source | owner层注入ClockSource；profiling局部Instant可保留，但必须声明domain和不可进入simulation state |
| TIME-P1-005 | dynamic runtime/interface/app ABI没有manual/external clock strategy | 提供test/replay/server-authoritative clock adapter及capability/version协商 |
| TIME-P1-006 | `FrameClock`在CoreRuntime构造时启动，首tick包含构造、加载和等待时间 | session activation成功后显式rebase；首帧delta策略进入receipt |
| TIME-P1-007 | resume/suspend/occlusion/window recreation没有ClockDiscontinuity | lifecycle hook向ClockAuthority提交typed discontinuity并记录处理策略 |
| TIME-P1-008 | Virtual speed先乘再clamp，极大有限值可能在clamp前overflow/panic | 先验证有界policy或使用checked/saturating conversion，返回typed config error |
| TIME-P1-009 | time setters靠assert验证，错误profile或外部值会panic | `TimePolicyTransaction`做validate/prepare/commit/reject并输出generation receipt |
| TIME-P1-010 | 64Hz、250ms、8 steps等默认值没有BuildSet/profile owner | 由versioned ProductTimePolicy统一生成client/server/editor/test配置与digest |

## 7. P1：Fixed Step、World 与 Schedule 语义

| ID | 当前差距 | 需要重构 |
|---|---|---|
| TIME-P1-011 | max-step cap只保留隐式debt，没有catch-up/drop/degrade/fatal policy | 公开`FixedDebtPolicy`、debt duration/steps/age和每产品上限 |
| TIME-P1-012 | `overstep_fraction`clamp到0..1，掩盖多步债务 | 分离interpolation fraction与total debt，禁止用clamped fraction做健康指标 |
| TIME-P1-013 | 所有dynamic profile共用8步，client/server/editor没有需求差异 | profile按latency、simulation correctness、power和service SLO定义policy |
| TIME-P1-014 | `RuntimeSceneSystemContext`只有core/level/`f32 delta_seconds` | 引入typed `SystemTickContext`，携stage、domain、outer frame、sim tick、delta/elapsed、world generation |
| TIME-P1-015 | system metadata不声明ClockDomain或pause behavior | 注册时要求`TickPolicy`，编译schedule时验证非法域和stage组合 |
| TIME-P1-016 | 多world共享CoreRuntime clocks，没有world pause/scale/epoch | 建立WorldTimeController；全局real source与world virtual/fixed state分层 |
| TIME-P1-017 | fixed-rate运行中修改没有overstep rebase/migration合同 | 事务化rate change，定义preserve time/debt、epoch rollover与network/replay兼容 |
| TIME-P1-018 | 没有previous/current committed state和interpolation alpha合同 | World snapshot/extract消费`FixedInterpolationContext`，禁止读取预推进future state |
| TIME-P1-019 | App cadence晚点时重置为`now+interval`，缺phase/lateness/catch-up语义 | cadence scheduler记录deadline、lateness、miss并按产品策略保持相位或明确重基 |
| TIME-P1-020 | Physics、Animation、script等各自只拿delta或另有局部clock | 用adapter消费统一TickContext；局部substep需声明parent tick、substep ID和commit边界 |

## 8. P1：RNG 与确定性合同

| ID | 当前差距 | 需要重构 |
|---|---|---|
| TIME-P1-021 | 没有引擎RandomService、算法ID、master seed或stream hierarchy | 建立`RandomAlgorithmId + RandomStreamKey + RandomState`，算法/version进入BuildSet |
| TIME-P1-022 | AI selector只hash tree/node/tick，不含world/entity/instance seed | stream key加入world/entity generation、system/purpose和authoring seed，避免实例锁步相关 |
| TIME-P1-023 | AI使用`DefaultHasher`，不是稳定序列化算法合同 | 换成明确、测试向量固定、兼容策略已定义的算法；禁止std实现细节进入replay格式 |
| TIME-P1-024 | Particle CPU与GPU随机算法/seed salt/frame语义不同 | 定义effect-level parity目标、CPU/GPU算法ID和允许误差；跨backend迁移显式拒绝或转换 |
| TIME-P1-025 | WOC parity维护第三套RNG authority | WOC可保留domain stream，但必须由引擎registry登记算法、seed、state与version |
| TIME-P1-026 | RNG没有draw index、fork/counter和snapshot接口 | 每stream可序列化state/counter；fork只由stable key派生，不能依赖执行完成顺序 |
| TIME-P1-027 | schedule稳定排序未绑定BuildSet/system graph digest | 编译产出`ScheduleBuildReceipt`，replay header验证system IDs、edges、stage和policy digest |
| TIME-P1-028 | 并行系统可能通过IO/task/callback执行不可排序side effect | 所有simulation effect写入tick-scoped journal/outbox，commit后按稳定key发布 |
| TIME-P1-029 | 无unordered collection进入snapshot/hash/network的统一canonicalization规则 | 定义CanonicalStateEncoding；只在边界排序/编码，并对浮点、NaN、-0、map/set给规则 |
| TIME-P1-030 | “确定性”没有作用域，无法区分same-process、same-build、cross-platform或bitwise | 建立`DeterminismProfile`与支持矩阵；物理/浮点不满足bitwise时明确状态级/容差级合同 |

## 9. P1：Replay、Checkpoint 与可观测性

| ID | 当前差距 | 需要重构 |
|---|---|---|
| TIME-P1-031 | 通用runtime没有versioned replay bundle/header/schema | `ReplayManifest`绑定BuildSet、world/schema、clock/RNG/schedule/input/effect版本和digests |
| TIME-P1-032 | 没有统一input/event/command ordered journal | admission后将输入映射到principal/world/tick/sequence并记录reject/drop/resync receipt |
| TIME-P1-033 | 没有engine world checkpoint、增量snapshot或seek index | 建立bounded checkpoint store、delta chain、retention和random-access seek |
| TIME-P1-034 | Physics/Animation/AI/network/render没有共同tick correlation | 统一OuterFrameId、SimulationTickId、RenderFrameId及domain correlation，不混用计数器 |
| TIME-P1-035 | replay没有兼容/迁移/拒绝策略 | header admission验证schema/algorithm/system graph/build，支持显式migrator或fail-closed |
| TIME-P1-036 | 无record/playback clock authority和fast-forward/scrub operation | ReplayClock可manual step、batch fast-forward、pause/seek/cancel，并有operation receipt |
| TIME-P1-037 | 无逐tick state/effect digest与首次分叉诊断 | 定义canonical digest tree，输出first divergent tick/path/system/stream而非只报最终不等 |
| TIME-P1-038 | diagnostics只见raw frame/fps/fixed count，缺virtual、debt、discontinuity和cadence miss | 发布bounded TimeTelemetry，含domain delta、debt age、mutation、lateness、drop/degrade原因 |
| TIME-P1-039 | 没有同BuildSet双跑确定性required lane | 同一input/seed运行两次并比较tick/state/effect/RNG digests，artifact保留首次分叉 |
| TIME-P1-040 | 没有跨平台/toolchain/hitch/suspend/time-mutation资格矩阵 | required matrix覆盖Windows/Linux、debug/release、thread counts、hitch、pause、scale、rate change和failure injection |

## 10. P2：长期能力与体验

| ID | 当前差距 | 需要重构 |
|---|---|---|
| TIME-P2-001 | elapsed/f32 consumer长期精度和wrap策略未定义 | 定义高精度内部表示、shader/UI下转换和可测试wrap epoch |
| TIME-P2-002 | reverse time没有明确支持或拒绝边界 | 默认游戏clock拒绝负速；只在Replay/Editor专用domain实现反向seek |
| TIME-P2-003 | 没有entity/local clock或time dilation hierarchy | 在全局/world合同稳定后，引入有界层级与组合规则，避免每组件自由乘delta |
| TIME-P2-004 | UTC/calendar/timezone与simulation time没有独立产品服务 | 建立CalendarClock/locale adapter，绝不用于固定模拟顺序或随机seed |
| TIME-P2-005 | Editor没有clock-domain inspector和debt/discontinuity图 | 提供只读timeline、domain correlation、policy generation及异常定位 |
| TIME-P2-006 | Replay缺scrub/bookmark/branch comparison UX | Editor消费Replay operation/receipt，不自行复制snapshot或clock authority |
| TIME-P2-007 | RNG seed/stream缺authoring与调试UX | Inspector显示seed来源、algorithm、stream key和draw counter，shipping隐藏敏感内容 |
| TIME-P2-008 | deterministic fuzz seed corpus没有生命周期 | 失败seed最小化、去重、绑定BuildSet并进入长期regression corpus |
| TIME-P2-009 | GPU temporal frame与simulation tick关联只在局部实现 | Render capture记录sim/render/history IDs及reset cause，支持跨帧因果追踪 |
| TIME-P2-010 | audio/media/device clocks没有drift与resample合同 | 建立独立clock adapters、drift telemetry和同步策略，不强行锁到Fixed clock |
| TIME-P2-011 | network remote clock/RTT/drift compensation不在统一taxonomy | 网络报告实现同步算法，本篇提供domain/epoch/correlation接口与replay记录 |
| TIME-P2-012 | 没有time-travel debugger断点/条件步进合同 | 在checkpoint/digest成熟后实现system/tick breakpoint和read-only historical inspection |

## 11. 目标架构

```text
PlatformMonotonicSource     WallUtcSource      External/ReplaySource
          |                     |                       |
          +---------- ClockSource Registry ------------+
                                |
                         ClockAuthority
                                |
          +---------------------+----------------------+
          v                     v                      v
   RealFrameClock        WorldVirtualClock       Product Cadence
          |                     |
          +---------- FrameTimeSnapshot
                                |
                  begin_fixed_step / commit_fixed_step
                                |
                    SimulationTickContext
                                |
       ScheduleBuildReceipt -> Systems -> Tick Effect Journal
                                |                  |
                         committed state        Outbox
                                |
        RandomStream Registry / Input Journal / State Digest
                                |
                 Checkpoint Store + Replay Manifest
                                |
              Playback / Seek / Divergence Receipt
```

核心类型：

- `ClockDomainId`：区分MonotonicReal、WallUtc、WorldVirtual、WorldFixed、Input、Render、Audio、Network、Media和EditorPreview；携epoch/source generation。
- `FrameTimeSnapshot`：一次outer frame内不可变，包含raw/clamped delta、pause/speed、discontinuity、cadence deadline/lateness和Fixed debt proposal。
- `SimulationTickContext`：每个已开始但未提交的Fixed step唯一，包含world generation、tick ID、timestep、previous committed tick、RNG epoch和effect scope。
- `TimePolicyTransaction`：pause/scale/fixed rate/max delta/debt policy统一validate/prepare/commit，变更只在明确frame/tick boundary生效。
- `RandomStreamKey`：由algorithm、master seed generation、world/entity/system/purpose稳定派生；state/counter可snapshot和迁移。
- `ReplayManifest`：绑定BuildSet、schemas、schedule、clock/RNG algorithms、initial snapshot、input journal、checkpoints和expected digest tree。

禁止建立一个锁住所有系统的“GlobalTimeManager”。ClockAuthority拥有源和frame snapshot，WorldTimeController拥有world pause/scale/fixed state，Render/Audio/Network保留各自domain adapter；它们通过typed correlation和policy receipt协作，而不是共享可随时修改的全局浮点数。

## 12. 重构里程碑

### M0 · 冻结错误语义并建立可失败回归

- 为两项P0写当前失败的source/behavior tests：Virtual pause/scale绕过、Fixed批量预推进和第N步失败。
- 标注Real/Virtual/Fixed、outer frame/fixed tick以及各consumer实际domain。
- 生成time/RNG/replay inventory和owner manifest；不修改Physics/UI/WOC既有owner finding。
- 将未资格化pause/time scale/replay能力标记Unavailable，不让UI或descriptor继续暗示完整支持。

### M1 · Clock source、snapshot 与policy transaction

- 实现ClockSource registry、MonotonicReal/WallUtc分离和manual/replay adapter。
- 以`FrameTimeSnapshot`替代`RuntimeTimeAdvance`的弱表面，普通游戏Update改用Virtual delta。
- App lifecycle发出discontinuity并在activation/resume显式rebase。
- ProductTimePolicy进入BuildSet，invalid mutation返回typed rejection。

### M2 · 逐步Fixed transaction

- 把`drain_steps`拆成debt proposal与逐步begin/commit。
- 给每步分配SimulationTickId、state/effect/RNG scope；失败只提交已完成步骤。
- 引入interpolation context、debt policy和spiral telemetry。
- Physics等substep adapter声明parent tick，不允许第二clock静默推进world。

### M3 · World/system clock domains

- 扩展System metadata和TickContext；compile schedule验证stage/domain/pause policy。
- 建立WorldTimeController、world epoch、pause/scale/rate transaction。
- Render/Input/Audio/Network/Media通过typed adapter关联outer/sim/render IDs。
- Editor07消费同一API实现pause/resume/single-step，不复制clock owner。

### M4 · RNG与canonical state

- 选择并版本化默认RNG算法、测试向量、stream hierarchy和fork规则。
- 迁移AI、Particles、WOC domain streams，明确CPU/GPU parity和旧state处理。
- 定义CanonicalStateEncoding与per-tick digest tree。
- 并行side effects收敛到tick journal/outbox。

### M5 · Replay/checkpoint/divergence

- 建立ReplayManifest、input journal、checkpoint/delta store和seek index。
- 实现record/playback/manual step/fast-forward/scrub/cancel receipts。
- 在first divergent tick输出system/state path/RNG/effect诊断。
- 接入network/physics/animation/AI和render capture correlation。

### M6 · 产品资格与性能

- 建立same-BuildSet double-run、跨thread-count和跨平台/toolchain矩阵。
- 执行hitch/suspend/occlusion/time-scale/fixed-rate/failure injection/long soak。
- 衡量snapshot/journal/digest开销并按profile分级，不能为“确定性”无界复制world。
- Editor clock/replay inspector和shipping diagnostics消费同一bounded telemetry。

## 13. 验收矩阵

| 资格面 | 必须证明 |
|---|---|
| pause | Virtual game state、Animation、script与Fixed不推进；显式RunWhenPaused系统按策略运行 |
| scale | 0、0.25、1、2及边界值在Update/Fixed累计一致；非法/过大值typed reject而非panic |
| lifecycle | first activation、occlusion、suspend/resume、debug break和window recreation均产生可审计discontinuity |
| fixed identity | 每个step有连续唯一tick ID和elapsed；同outer frame的8步不可读取同一最终clock |
| failure atomicity | 任一stage/step失败只保留已commit state/effect/RNG，remaining debt准确 |
| debt | cap、hitch、overload下steps/duration/age可观测；drop/degrade/fatal符合profile |
| world isolation | 两个world可独立pause/scale/rebase，不能互相改变epoch或tick |
| schedule | graph digest、system order和worker merge在同BuildSet稳定；duplicate/conflict fail-closed |
| RNG | 固定向量、stream fork、snapshot/restore、draw counter和算法迁移有positive/negative tests |
| CPU/GPU | Particle随机行为符合声明的bitwise/state/statistical parity级别，未支持迁移显式拒绝 |
| replay | record、checkpoint、seek、fast-forward、resume和完整playback产生相同state/effect digest |
| divergence | 单bit input/state/RNG变化定位首个tick与owner，不只返回最终hash mismatch |
| canonical state | unordered map/set、float特殊值、entity generation和schema version编码稳定 |
| cross-platform | 支持的determinism profile在Windows/Linux、debug/release、不同worker数通过；不支持范围写明 |
| performance | journal/snapshot/digest在规模梯度内满足CPU、memory、latency预算，pressure可降级且不破坏truth |
| observability | Frame/Simulation/Render/Input/Audio/Network IDs可相关，telemetry有界且不会反向改变simulation |
| product closure | App/Editor/server通过同一Clock/Replay ABI控制真实session，无fixture、字符串或本地旁路 |

任何“固定输入跑一次没崩”、FPS稳定、schedule排序单测、局部seed可复现、WOC golden clone或一份render capture都不能单独证明引擎确定性和回放资格。

## 14. 依赖、所有权与禁止旁路

- Runtime22是Clock Authority、Fixed tick transaction、RNG schema和通用Replay manifest的canonical owner。
- Runtime02拥有task/executor生命周期；它必须接收TickContext/effect scope，不另造simulation ordering。
- Runtime05拥有World/ECS状态和snapshot实现；它按Runtime22的tick/checkpoint合同提供canonical state。
- Runtime08A/08C/08F分别拥有Physics/Animation/AI局部语义，不重复引擎clock/RNG owner。
- Runtime09H1拥有per-camera temporal history；RenderFrameId关联SimulationTickId但不等同。
- Runtime11A拥有UI timer/input timestamp缺口；最终接入ClockDomainRegistry，不在本篇重复其P0。
- Runtime Interface定义稳定Clock/Replay DTO与ABI，App处理platform cadence/lifecycle，Editor07提供产品控制UX。
- Tooling11拥有WOC oracle/differential evidence；Tooling22拥有常量放置；Tooling23拥有通用panic/error；本篇只定义时间专用policy/result。
- 不允许system直接用`Instant::now`/`SystemTime`改变simulation state，不允许以frame count作为随机seed，不允许以unordered iteration决定state/effect顺序。
- 不允许为追求bitwise确定性承诺未经平台/编译器/浮点/physics验证的范围；support matrix必须先于性能或“超过Unreal”声明。

## 15. 本轮记录

本轮只新增review与索引计划，没有修改Runtime、App、Editor、Plugin、Interface、tests、Cargo manifest、lockfile或workflow。报告保留三时钟、stable schedule、deterministic worker merge和局部seed基础，确认2项产品级P0，并将40项P1、12项P2分配到Clock Authority、Fixed transaction、World/System domain、RNG/Determinism、Replay/Checkpoint和Qualification。

没有运行动态测试。此前Editor、Hub、WOC native/npm和plugin metadata的既有阻断未变化，重跑不能抵达本篇产品行为；`dynamic_api/session/profile.rs`仍是外部在途源码。实施M0时必须先在current BuildSet重取fingerprint，添加两项可失败回归，再按M1-M6逐层验证，不能直接从当前静态报告宣称功能已修复、性能已提升或已超过Unreal。
