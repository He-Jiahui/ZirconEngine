---
title: Runtime-wide State、NextState、Transition、Hook、History、Schedule、Scope 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime125
review_date: 2026-08-23
baseline_head: 6ce24f25e46d8f370aa5b5d4e8487f53103b43c0
observed_head: f79dc502a1e8db5f7cbcc17fbeb297af1e193f7e
baseline_epoch: 375
supersedes:
  - docs/plans/optimize/zircon_runtime/48-runtime-wide-state-next-state-transition-hook-history-schedule-scope-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/state
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/tests/state.rs
  - zircon_runtime/src/tests/state/hook_index.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs
  - zircon_app/src/tests/prelude.rs
  - docs/zircon_runtime/core/state.md
tests:
  - zircon_runtime/src/core/runtime/handle/states.rs::tests
  - zircon_runtime/src/tests/state.rs
  - zircon_runtime/src/tests/state/hook_index.rs
  - zircon_runtime/src/tests/prelude.rs::runtime_prelude_exports_state_contracts
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs::runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry
  - zircon_app/src/tests/prelude.rs::app_prelude_includes_runtime_prelude_foundations
plan_sources:
  - docs/plans/optimize/zircon_runtime/48-runtime-wide-state-next-state-transition-hook-history-schedule-scope-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/02/2026-07-22-state-hook-canonical-index.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
reference_engines:
  - dev/bevy/crates/bevy_state/src/app.rs
  - dev/bevy/crates/bevy_state/src/condition.rs
  - dev/bevy/crates/bevy_state/src/state/resources.rs
  - dev/bevy/crates/bevy_state/src/state/states.rs
  - dev/bevy/crates/bevy_state/src/state/transitions.rs
  - dev/bevy/crates/bevy_state/src/state/sub_states.rs
  - dev/bevy/crates/bevy_state/src/state/computed_states.rs
  - dev/bevy/crates/bevy_state/src/state_scoped.rs
  - dev/bevy/crates/bevy_state/src/state_scoped_events.rs
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeModule/Public/StateTreeEvents.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeModule/Public/StateTreeExecutionTypes.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeModule/Public/StateTreeExecutionContext.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeTestSuite/Private/StateTreeTransitionTest.cpp
  - dev/Fyrox/fyrox-animation/src/machine/mod.rs
  - dev/Fyrox/fyrox-animation/src/machine/state.rs
  - dev/Fyrox/fyrox-animation/src/machine/transition.rs
  - dev/Fyrox/fyrox-animation/src/machine/event.rs
  - dev/Fyrox/fyrox-animation/src/machine/layer.rs
  - dev/godot/scene/animation/animation_node_state_machine.h
  - dev/godot/scene/animation/animation_node_state_machine.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/IDebugDisplaySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettingsUI.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99z · Runtime-wide State Current Source Review

## 1. 结论

Runtime48 的主要裁决在当前源码中仍成立。`core::framework::state` 的 12 个 production 文件确实提供了 typed current/next、显式 identity transition 与 `PendingIfNeq` suppression、按 state/state-pair 哈希桶匹配 hook、`exit -> transition -> enter` 顺序，以及锁内冻结 dispatch、锁外运行 callback。这些是可保留的局部算法基础，不是空壳。

但完整产品能力仍为 **0**。排除 facade 定义、inline tests、`zircon_runtime/src/tests` 与 `zircon_app/src/tests` 后，`init_state`、`set_next_state`、`apply_state_transition`、history query 和 hook registration 没有任何 App、World、Session、Editor、plugin、script、network、save 或 replay production caller。Runtime03 已完成 `First -> PreUpdate -> FixedFirst -> FixedUpdate -> FixedPostUpdate -> Update -> PostUpdate -> Last -> RenderExtract` 九阶段 schedule，但没有 state request freeze、resolve、apply、publish 或 cleanup system。当前状态能力仍是调用者手工驱动的一把全局 `Mutex<StateRegistry>`，不是 runtime-owned state service。

核心正确性和生命周期也没有收敛：单个 `NextState<T>` 仍然静默 last-writer-wins；`insert_state` 覆盖已有 current 时仍伪造 `None -> Some(new)`；repeated init 仍返回形似 transition 的 synthetic DTO；registry 读写和 hook registration 会隐式创建永久 slot；history 是无界 `Vec` 且每次查询全量 clone；hook 没有 token、owner generation、unregister、admission close 或 quiescence；callback panic 在 current/history 已提交后截断 fan-out；poisoned mutex 被 `into_inner()` 当作健康状态继续服务；`TypeId + Any + type_name` 不能承担 schema、ABI、持久化或 hot-reload identity。

本轮确认两项窄进展，但都不足以关闭 finding。公开 repeated-init 回归已经从过时源码字符串断言改为行为断言，module doc 也新增 Runtime48 truth freeze，明确 capability 仅供实验和测试。因此 `STATE-P1-045` 与 `STATE-P1-046` 从 Open 改判为 **Partial**。然而 handle-local test 仍用 `include_str!().contains()` 固定实现形状，Runtime15 structure guard 仍要求当前源码中不存在的 `self.lock_states().init_state::<T>(...)`，module doc 仍把 `into_inner()` 描述为“poison-safe”并保留历史 Cargo 结果。其余 46 项 P1 没有 current-source closure。

当前裁决为 **0 项本地 P0；46 P1 Open、2 P1 Partial、0 Closed；12 P2 Open；33 Gate Fail、1 Gate Partial、2 Gate Pass**。本轮新增 finding 为 0，只对 Runtime48 的现有 48 项 P1、12 项 P2 和 36 项资格门做 current-source 重判。本文不把 Bevy/Fyrox 自身的 last-writer 或 silent-drop 行为当成 Zircon 的工程目标，也不把“回调锁外运行”“测试可调用”或“poison 后不 panic”写成优于 Unreal 的证据。

本轮只做静态 review 和文档记录，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行 Cargo、真实 frame loop、并发争用、plugin unload、panic fault injection、100k transition soak 或 benchmark。MVP 未完成，`source_recheck_required` 保持 true。

## 2. 审查边界与物理冻结

### 2.1 Focused 集合

| 范围 | 文件 / 行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| `core/framework/state` production | 12 / 519 / 14,817 / 0 / 0 | `bbdd1e225d24259cfe03af8bb3b41777767bc478f97afc2fa26146069f7f9826` |
| runtime facade / owner wiring | 5 / 871 / 30,460 / 3 / 0 | `faad7d1b8433487fc2f573a04bfb51987ab09cddca5c275a53d44964875f40dc` |
| dedicated state tests | 2 / 320 / 10,361 / 9 / 0 | `29606afa4f4c08c85718068ec3c1bf22d4da6fbf75bb3d26a7370041984001f3` |
| Zircon focused total | 19 / 1,710 / 55,638 / 12 / 0 | `064b0849b7932242470ed26efa0b5a6a48c124c1af79b0a79f115465459683c2` |
| selected five-engine evidence | 23 / 13,973 / 560,367 / cross-language | `e68f180ebb4cfc7546c222833172e47d08f30c91b6ca7ffba6885f7c5d8c11cf` |

fingerprint 算法与旧报告保持一致：仓库相对路径转 `/`、小写、ordinal 排序去重；每项编码为 `lowercase-path + NUL + lowercase per-file SHA-256`，以 LF 连接且末尾无 LF，再对 UTF-8 payload 计算 SHA-256。它只冻结本轮读取集合，不是 runtime identity、ABI、artifact 或 release identity。

与 Runtime48 旧冻结相比，12 个 canonical state production 文件的 519 行 / 14,817 bytes 完全未变；focused total 增加 20 行 / 647 bytes，来自 facade/owner 的无关 `active_module_order` wiring 与 repeated-init 测试/格式调整，没有形成 state architecture 进展。

### 2.2 Currentness、HEAD drift 与工作树

- Session 注册基线是 `6ce24f25e46d8f370aa5b5d4e8487f53103b43c0` / epoch 375；最终验证前共享主检出前进到 `f79dc502a1e8db5f7cbcc17fbeb297af1e193f7e`。只读 diff 证明这段 HEAD drift 没有触及 19 个 focused state 文件或 module doc；两级 optimize index 的既有并发改动被保留并在 lease 后做了局部更新。
- 本轮读取到 `docs/zircon_runtime/core/state.md`、`runtime.rs`、`core_runtime_state.rs` 与 `src/tests/state.rs` 的其他会话/用户改动，并保留它们；本文不回退、不暂存、不改写这些路径。
- Runtime02 child 只完成 hook 哈希索引的静态实现，明确未改变 history retention，且 Cargo compile 被外域错误阻塞；本文只承认当前 source shape，不升级为 compile/performance acceptance。
- Runtime03 九阶段 schedule 已完成，但全仓没有 state schedule edge；“已有 scheduler”只减少重构范围，不关闭 Runtime48 integration。
- 参考 revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`；Unreal snapshot 位于 Zircon root tracking 下，以 per-file hash 和 observed root HEAD 冻结。

### 2.3 产品调用链

```text
CoreRuntime facade
  -> CoreHandle
  -> Mutex<StateRegistry>
  -> HashMap<TypeId, Box<dyn Any>>
  -> StateMachine<T> { current, next, Vec<event>, hook index }
  -> caller manually invokes apply_state_transition<T>()
  -> callbacks run outside registry lock

  -X-> Runtime03 schedule / frame barrier
  -X-> App bootstrap or shutdown
  -X-> World / Session / PIE / LocalPlayer scope
  -X-> Editor / plugin / ZrVM / dynamic API
  -X-> save / replication / replay / diagnostics product
```

`zircon_runtime/src/tests/prelude.rs` 和 `zircon_app/src/tests/prelude.rs` 只证明 public surface 可编译；它们创建独立 `CoreRuntime` 后立即手工 apply，不是产品 frame trace。全仓同名 Animation State Machine、ECS `SystemParam::init_state` 与 Tauri state 命中属于其他领域，不能计为这套 Runtime state facade 的 consumer。

## 3. 当前实现逐层事实

### 3.1 Owner 与 public surface

| 事实 | 当前实现 | 工程裁决 |
|---|---|---|
| behavior owner | machine、registry、journal、hook index 全在 `core::framework::state` | 违反固定架构；行为必须迁到 `core::runtime::state`，framework 只留 neutral contracts |
| runtime storage | `CoreRuntimeInner.states: Mutex<StateRegistry>` | 没有 service descriptor、construct/activate/drain/close、health 或 capability |
| facade | CoreRuntime/CoreHandle/core facade/prelude 重复公开全部 direct mutation API | experimental surface 过宽；目标只公开 register/request/snapshot/subscription/receipt |
| registration | read/write/hook 通过 `machine_mut` 隐式创建 slot | 拼错或 stale type 也永久占位；必须显式 descriptor registration、unknown fail-close |
| scope | registry 只按 process-local `TypeId` 分片 | 无 Runtime/World/Session/Player/PIE owner 与 generation 隔离 |

### 3.2 Request、transition 与一致性

| 事实 | 当前实现 | 风险 |
|---|---|---|
| pending | 每类型一个 `NextState<T>` | 多 writer 静默覆盖，锁获取顺序决定结果，无 producer/priority/sequence |
| apply point | 任意 caller 随时手工 apply | 同一 frame 不同 caller 可观察不同事实，late request 没有定义 |
| init | created 时发布 `None -> Some`；already present 时 facade 合成同形 DTO | synthetic DTO 冒充 publication，没有 Created/AlreadyPresent receipt |
| insert | 覆盖 current 后仍记录 `None -> Some(new)` | previous 丢失，真实 exit/transition 跳过，审计与 replay 事实错误 |
| removal | target 固定为 `T` | 不能表达 `Some -> None`、deactivate、retire 或 generation replacement |
| snapshot | current、next、events 分别加锁并 clone | 无同代 current/previous/pending/last outcome snapshot |

### 3.3 Journal、dispatch 与 failure

| 事实 | 当前实现 | 风险 |
|---|---|---|
| history | machine 永久保存 `Vec<StateTransitionEvent<T>>` | 内存随 transition 单调增长，没有 entries/bytes/age hard cap |
| query | `state_transition_events()` clone 全量 history | 成本与历史总长成正比，无 cursor、batch、lag/gap |
| payload movement | current/next/event/history/return/hook snapshot 多次 clone `T` | 无 value size、clone bytes、hash、redaction 预算 |
| hook lookup | 哈希桶定位匹配项 | 可保留；但每次仍 clone 三个 `Vec<Arc<Fn>>` |
| subscription | registration 返回 `()` | 无 revoke/drop token、owner generation、in-flight drain、leak census |
| callback error | opaque `Fn` 直接运行 | panic 在 commit 后截断剩余 fan-out并 unwind，无聚合 failure receipt |
| poison | `unwrap_or_else(|p| p.into_inner())` | partial mutation 可能继续服务且 health 仍伪装正常 |

### 3.4 Test 与 doc 证据债务

- dedicated tests 只覆盖 init、单 pending、identity suppression、hook 顺序、orthogonal type 和 repeated init；没有 multiwriter、overwrite、remove、retire、unregister、panic、reentrancy、concurrency、bounded history、many-type scale 或 product frame。
- `existing_state_init_reuses_the_registry_lock` 仍以 `include_str!` 和 `.contains()` 锁实现形状。
- Runtime15 structure guard 仍要求 `self.lock_states().init_state::<T>(T::default())`，而当前 init 为 `let mut states = self.lock_states(); states.init_state...`，因此该静态断言与 current source 冲突。
- module doc 的 Runtime48 truth freeze 是有效进展；但“poison-safe”把“不 panic”误写成“数据一致”，历史 Cargo 结果也不是当前 build receipt。

## 4. 五引擎参考差异

| 参考 | 已核对的工程合同 | Zircon 应吸收 | 明确不照搬 |
|---|---|---|---|
| Bevy `bevy_state` | `StatesPlugin` 在 startup `PreStartup` 前和每帧 `PreUpdate` 后运行 `StateTransition`；四阶段为 dependent/exit/transition/enter；有 `State/PreviousState/NextState` resource、message cursor、run condition、remove、computed/substate dependency 和 state-scoped entity/message cleanup | schedule 接线、previous/current、cursor、dependency order、run condition 与 scope cleanup | `NextState` 仍允许后写覆盖；`insert_state` 覆盖时也清消息并写 `None -> Some`。Zircon 不复制这两个弱语义 |
| Unreal StateTree | transition request 带 priority、fallback、active frame/state source；event 有 tag/payload/origin，queue hard cap 64、支持 consume 与 phase clear；execution instance有 Start/Tick/Stop、run status和 opt-in recorded transition | request provenance/priority、有限队列、instance generation、运行状态、phase lifetime 与可选调试记录 | StateTree 是层级行为执行框架，不是通用 App state 的一对一模板 |
| Godot animation state machine | Resource definition 与 per-scene unique playback 分离；playback 维护 start/travel/next/stop request、current/fading/path；transition有 priority/switch/xfade/reset；process消费请求并做 validation/signals | definition/instance分离、request-at-process-boundary、路径/validation、实例唯一性和显式运行状态 | 只用于 Animation adapter；不把 StringName graph、blend或teleport语义塞进通用 RuntimeStateService |
| Fyrox animation machine | reflected/visited state、transition、layer；state有 enter/leave actions，transition有 condition/time/blend，layer有 active state/transition；实际 layer event queue容量 2048 | authored graph/instance、有限事件、显式 active transition 与可观察生命周期 | queue 满时静默 drop，且默认 queue 可为 `u32::MAX`；不能作为 Zircon terminal receipt/gap policy |
| Unity Graphics | selected source是 rendering debug settings：typed settings collection、lazy UI、reset、register/unregister和panel dispose；本地 SRP repo没有通用 application state owner | 仅作为局部渲染状态与UI生命周期边界证据 | 不能从 Graphics mirror 推断闭源 Unity Gameplay/Animator，也不能用 renderer settings 替代 runtime application state |

共同差异不在“类数量”，而在 definition、instance、request、schedule、scope、event lifetime、failure 和 diagnostics 都有明确 owner。Zircon 可以用更紧凑的 slot、arena、cursor 和 immutable dispatch table获得更低成本，但不能删除 identity、receipt、boundedness 和 lifecycle 后宣称性能领先。

## 5. Canonical owner 边界

| 事实 | Canonical owner | Runtime125 / Runtime48 只拥有的纵切面 |
|---|---|---|
| schedule阶段表、system access/conflict、parallel executor | Runtime03 | 声明 state freeze/resolve/publish barrier和state run condition接线，不另建scheduler |
| module/service construct/activate/drain/close与health | Runtime46 / Runtime01 | `RuntimeStateService`加入统一生命周期和health，不复制lifecycle coordinator |
| Runtime/World/Session/Player/PIE identity与generation | Runtime24、Runtime05、Runtime38 | descriptor消费qualified scope key；slot/request/journal绑定scope generation |
| clock/frame/tick/replay identity | Runtime22 | request与receipt记录现有clock identity，不自建时间线 |
| operation admission/cancel/deadline/terminal outcome | Runtime41 | state request用轻量receipt并可关联operation，不复制Operation Service |
| plugin/native callback lease、unload/quiescence | Runtime07、Runtime46、Tooling35 | state subscription token接入统一owner generation和drain |
| history/hook snapshot性能预算 | PERF-MVP-320 | bounded journal/cursor、steady dispatch allocation和100k workload |
| GameState、Animation、AI、Gameplay authored semantics | 各domain owner | 只提供adapter；通用state service不吸收blend/tree/ability authority |
| Editor authoring、PIE UI与inspection | `zircon_editor` owner | Editor只读catalog/snapshot/receipt，不成为runtime truth owner |

## 6. Runtime48 P1 当前裁决

状态只表示当前 focused source 是否满足旧账目标；Partial 不代表可发布。

| ID | 状态 | 当前源码证据 |
|---|---|---|
| STATE-P1-001 | Open | 排除测试和facade后production caller仍为0 |
| STATE-P1-002 | Open | `apply_state_transition`仍由caller手工触发，Runtime03无state barrier |
| STATE-P1-003 | Open | machine/registry/journal/hook behavior仍归`core::framework::state` |
| STATE-P1-004 | Open | `CoreRuntimeInner`仍只持裸`Mutex<StateRegistry>` |
| STATE-P1-005 | Open | CoreRuntime/CoreHandle/core/prelude仍公开direct insert/apply/history全表面 |
| STATE-P1-006 | Open | 未注册type的write/hook仍隐式创建slot |
| STATE-P1-007 | Open | 无catalog、owner、enabled、policy或health snapshot |
| STATE-P1-008 | Open | 无World/Session/Player/PIE scope key或generation |
| STATE-P1-009 | Open | 单`NextState`仍静默last-writer-wins |
| STATE-P1-010 | Open | request无producer/module/system identity |
| STATE-P1-011 | Open | 无priority、arbitration policy或stable tie-breaker |
| STATE-P1-012 | Open | 并发lock acquisition仍可决定最终pending value |
| STATE-P1-013 | Open | Pending/PendingIfNeq覆盖与suppression无显式outcome |
| STATE-P1-014 | Open | `insert_state`仍伪造`None -> new`并跳过exit/transition |
| STATE-P1-015 | Open | repeated init仍返回synthetic transition-shaped DTO |
| STATE-P1-016 | Open | setter仍返回`()`，无request ID或terminal receipt |
| STATE-P1-017 | Open | 无`Some -> None` remove/deactivate transition |
| STATE-P1-018 | Open | machine无reset/retire/re-register generation |
| STATE-P1-019 | Open | hook registration无token/unregister |
| STATE-P1-020 | Open | `'static Arc<Fn>`仍可永久强持module/world/plugin资源 |
| STATE-P1-021 | Open | hook与owner generation、admission close、quiescence无关 |
| STATE-P1-022 | Open | exit不清理state-scoped entity/resource/task/subscription |
| STATE-P1-023 | Open | 无computed/substate dependency graph和cycle rejection |
| STATE-P1-024 | Open | scheduler无`in_state/state_changed` typed condition |
| STATE-P1-025 | Open | history仍为永久增长`Vec` |
| STATE-P1-026 | Open | history query仍clone全量，无cursor/gap |
| STATE-P1-027 | Open | event无sequence/frame/tick/clock/scope generation |
| STATE-P1-028 | Open | event无request/producer/cause/outcome/error |
| STATE-P1-029 | Open | `T`仍在current/next/history/return/hook路径反复clone |
| STATE-P1-030 | Open | 每次dispatch仍clone三组matching `Vec<Arc<Fn>>` |
| STATE-P1-031 | Open | opaque callback无resource access、affinity、budget或trace |
| STATE-P1-032 | Open | callback panic仍在commit后截断fan-out并unwind |
| STATE-P1-033 | Open | registry identity仍为process-local `TypeId + Any` |
| STATE-P1-034 | Open | blanket `StateSpec`仍阻止显式descriptor contract |
| STATE-P1-035 | Open | stable/display/debug name仍混为compiler `type_name` |
| STATE-P1-036 | Open | 无reflect/codec/schema/version/migration/unknown policy |
| STATE-P1-037 | Open | dynamic API、ZrVM、native plugin无ABI-safe state contract |
| STATE-P1-038 | Open | payload无size/clone/hash/log/redaction budget |
| STATE-P1-039 | Open | hook key仍是完整T值且无definition generation |
| STATE-P1-040 | Open | 无variant catalog、legal graph、guard或definition validation |
| STATE-P1-041 | Open | poisoned registry仍`into_inner()`后伪装healthy |
| STATE-P1-042 | Open | 无request/apply/suppression/queue/history/hook latency指标 |
| STATE-P1-043 | Open | previous仍只能从全history推断 |
| STATE-P1-044 | Open | current/next/history分别锁，无法读取同代snapshot |
| STATE-P1-045 | Partial | public repeated-init test已改行为断言；inline shape test和stale Runtime15 `.contains()` guard仍在 |
| STATE-P1-046 | Partial | module doc已有Runtime48 truth freeze；“poison-safe”和历史build声明仍不current |
| STATE-P1-047 | Open | panic/reentrancy/concurrency/overwrite/remove/retire/unregister/retention/scale矩阵仍缺 |
| STATE-P1-048 | Open | 无App/World/PIE/plugin/save/replay端到端consumer或shutdown证据 |

## 7. Runtime48 P2 当前裁决

| ID | 状态 | 当前源码证据 |
|---|---|---|
| STATE-P2-001 | Open | 无derive/registration ergonomics和编译期descriptor诊断 |
| STATE-P2-002 | Open | 无declarative guard/condition合同 |
| STATE-P2-003 | Open | 无delayed/deadline/cancel transition |
| STATE-P2-004 | Open | 无transition graph introspection或runtime visualization |
| STATE-P2-005 | Open | 无opt-in save/checkpoint policy |
| STATE-P2-006 | Open | 无opt-in replication/prediction/rollback adapter |
| STATE-P2-007 | Open | 无replay/rewind inspection和deterministic receipt stream |
| STATE-P2-008 | Open | 无基于Task/Operation owner的async enter/exit orchestration |
| STATE-P2-009 | Open | 无heatmap、latency percentile或branch coverage |
| STATE-P2-010 | Open | 无property/model-based transition validation |
| STATE-P2-011 | Open | 无独立editor-authored StateGraph/StateTree domain |
| STATE-P2-012 | Open | 无Animation/AI/Gameplay state snapshot/receipt adapter |

## 8. 目标架构

```text
core::framework::state
  StateTypeKey / StateScopeKey / StateDescriptor
  StateRequest<T> / StateSnapshot<T>
  StateTransitionReceipt<T> / StateSubscriptionToken
                      |
                      v
core::runtime::state
  RuntimeStateService
    descriptor catalog + scoped generational slots
    bounded per-phase request inboxes
    deterministic resolver + O(1) snapshot
    bounded cursor journal + owned subscriber registry
    lifecycle / health / diagnostics
                      |
                      v
Runtime03 schedule
  FreezeRequests -> Resolve -> Exit -> Commit -> Transition -> Enter -> ScopeCleanup
                      |
                      v
App / World / Session / Player adapters
  real product definitions, run conditions, teardown and traces
```

### 8.1 Contract 与 identity

- `StateDescriptor` 必须声明 stable type key、schema version、scope kind、initial/replace/remove policy、arbitration、journal budget、payload/redaction policy和owner generation。
- `StateRequest` 必须带 request ID、qualified scope、producer/system、producer generation、clock/frame/tick、mode、priority、target/cause；Rust `TypeId` 只可作为已注册slot的进程内加速索引。
- `StateRequestReceipt` 必须区分 Applied、IdentityApplied、Removed、Suppressed、Superseded、Rejected、Failed 和 AppliedWithHandlerFailures；每个 admitted request恰有一个terminal outcome。
- current/previous/pending summary/last outcome由一次generation-stamped snapshot读取，不依赖journal retention。

### 8.2 Schedule 与 failure

1. scheduler system和外部ingress只提交request，不直接修改current。
2. Runtime03声明唯一frame barrier；是否另有fixed-clock barrier必须由descriptor/clock policy显式选择，不能由任意caller决定。
3. barrier冻结batch；late request进入下一轮并立即得到deferred receipt。
4. resolver按arbitration、priority、stable producer order和sequence裁决全部request，不静默覆盖。
5. exit阶段后一次提交current/previous/generation，再运行transition、enter和scope cleanup；依赖state按leaf-to-root exit、root-to-leaf enter。
6. handler panic/error在engine boundary隔离并聚合；已提交事实不伪回滚，service health和owner quarantine可观察。
7. owner close顺序固定为close admission、drain in-flight、revoke subscriptions、retire slot/journal、release code/resource lease。

### 8.3 Boundedness 与性能

- inbox和journal同时有entry、declared bytes、age/deadline hard budget；overflow/eviction返回typed outcome或gap。
- cursor query成本与unread batch成正比，不再保留全量clone convenience API。
- stable subscription table发布immutable generation；steady transition不创建三组临时Vec，payload默认是小型discriminant/qualified key。
- diagnostics off近零bookkeeping；on时采样request、resolve、queue、journal、handler latency/failure和scope cleanup，不永久保存敏感payload。
- 性能资格必须比较相同正确性、相同producer数量、相同retention和相同硬件下的CPU p50/p95、alloc/RSS和tail latency，不能通过删语义获得“领先”。

## 9. 硬切范围

1. 不保留 `framework::state::StateRegistry/StateMachine` compatibility re-export，也不创建 `state2`。
2. 不让旧 direct insert/apply facade 与新 request service 双写或长期并存。
3. 不在 Editor、World、plugin 或 App 再复制私有通用 state registry。
4. 不以 poison recovery、不 panic、锁外 callback 或 prelude 编译代替 consistency/lifecycle 资格。
5. 不保留无界 history、全量clone query或无token hook。
6. 不把 `TypeId`、`type_name`、裸指针、closure identity或registration order写入ABI/持久化/主诊断键。
7. 不自动持久化、复制、replay或Editor暴露所有state；这些能力必须descriptor opt-in。
8. 不把Animation blend graph、AI StateTree或Gameplay Ability状态机吸收到通用application state core。
9. 不用source `.contains()`、测试数量或路径存在替代行为、fault、product和teardown证据。
10. 未通过相同workload的Windows current-source基准前，不宣称性能或表现优于Unreal/Bevy/Godot/Fyrox/Unity。

## 10. 分层重构路线

### M125-0 · Truth、RED tests 与 deletion matrix

- 冻结全部public facade、test caller、潜在product state、scope、writer、history consumer和hook owner；建立old path deletion matrix。
- 修正 stale Runtime15 structure guard和“poison-safe”文档；保留当前behavior test但删除implementation `.contains()`资格。
- 先写 overwrite、multiwriter、panic fan-out、poison quarantine、unbounded history和zero production consumer RED tests。

### M125-1 · Contract / Runtime owner hard cut

- framework只留descriptor/request/snapshot/receipt/token；machine、registry、resolver、journal、subscription迁到folder-backed `core::runtime::state`。
- 删除blanket public `StateSpec`和隐式slot创建；建立显式registration、catalog、duplicate/schema/unknown typed error。
- `RuntimeStateService`接入Runtime46 construct/activate/drain/close和health；不留compat module或双facade。

### M125-2 · Schedule、admission 与 deterministic resolver

- 将request freeze/resolve/publish接入Runtime03声明barrier，late request进入下一轮并有receipt。
- 建立bounded inbox、producer identity、priority/arbitration、stable sequence和每request terminal outcome。
- 将init/replace/remove/identity/suppression语义硬切为真实receipt；禁止`None -> Some`伪覆盖。

### M125-3 · Scope、dependency 与 subscription lifecycle

- slot/request/subscription绑定Runtime/World/Session/Player scope generation；双runtime、双world、PIE完全隔离。
- 提供token/revoke/capture policy/admission close/in-flight drain/quiescence，并接入统一plugin code/resource lease。
- 实现state-scoped entity/resource/task/subscription cleanup和acyclic dependent state ordering。

### M125-4 · Snapshot、journal、diagnostics 与性能止损

- 建立O(1) current/previous/pending/last-outcome snapshot和bounded cursor journal；lag/eviction显式gap。
- immutable subscription generation或复用scratch消除steady transition三Vec allocation；冻结payload clone/hash/redaction budget。
- 完成PERF-MVP-320的1/1k/100k、60/120 Hz、stable registration、slow/lagging consumer和diagnostics on/off矩阵。

### M125-5 · Product integration 与 ABI adapter

- 选择至少一个真实App/World/Session state接bootstrap、frame gate、scoped cleanup、reopen和shutdown；删除test-only product illusion。
- dynamic API、ZrVM和native plugin只在真实consumer出现后增加ABI-safe descriptor/request/snapshot/receipt adapter。
- 完成save/replication/replay默认关闭、opt-in policy和sensitive state redaction验证。

### M125-6 · Failure、soak 与发布资格

- 覆盖concurrent conflict、late request、queue full、handler panic/reentrancy、poison、retire/re-register、hot reload A-B-A、plugin unload和100k transition soak。
- Windows MVP先给current-source managed Cargo、真实frame trace、alloc/RSS/tail latency和shutdown leak census；其他平台按平台资格补齐。
- 36项资格门全部通过后才能关闭Runtime48/125账本；可选computed/substate/authoring/adapters在基础资格后独立实施。

## 11. 资格门当前状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| ST-G01 | Pass | 12/5/2 focused文件清单、指标和fingerprint已按当前working tree重建 |
| ST-G02 | Fail | production caller仍为0，无host-to-receipt trace |
| ST-G03 | Fail | framework仍拥有machine/registry/journal/callback behavior |
| ST-G04 | Fail | 无RuntimeStateService lifecycle和health |
| ST-G05 | Fail | 无双runtime/双World/PIE scope隔离 |
| ST-G06 | Fail | unknown write/hook仍隐式创建slot |
| ST-G07 | Fail | state resolution仍不在Runtime03 barrier |
| ST-G08 | Fail | late request无下一轮receipt语义 |
| ST-G09 | Fail | 多writer仍静默覆盖 |
| ST-G10 | Fail | 无priority和registration-order-independent tie-breaker |
| ST-G11 | Fail | 无并发确定性receipt/journal证据 |
| ST-G12 | Fail | request无fixed/update/async clock identity |
| ST-G13 | Fail | repeated init仍返回transition-shaped DTO |
| ST-G14 | Fail | replace伪造初始化且无remove |
| ST-G15 | Fail | identity/suppression/supersession/rejection矩阵不完整 |
| ST-G16 | Fail | 无同代current/previous/last outcome snapshot |
| ST-G17 | Fail | 无dependency cycle rejection和leaf/root顺序 |
| ST-G18 | Fail | 无state-scoped cleanup |
| ST-G19 | Fail | journal无entries/bytes/age硬预算 |
| ST-G20 | Fail | 无consumer cursor lag/eviction gap |
| ST-G21 | Fail | steady read仍clone全history |
| ST-G22 | Fail | transition仍clone三组hook Vec |
| ST-G23 | Fail | payload clone/hash/log bytes无预算/redaction |
| ST-G24 | Fail | 无diagnostics on/off成本与60/120 Hz归因 |
| ST-G25 | Fail | identity仍依赖TypeId/type_name |
| ST-G26 | Fail | 无descriptor duplicate/schema/variant typed failure |
| ST-G27 | Fail | 无ABI-safe descriptor/request/snapshot/receipt |
| ST-G28 | Fail | 无hot reload A-B-A stale isolation |
| ST-G29 | Fail | persistence/replication/replay opt-in policy未实现 |
| ST-G30 | Fail | Debug/event/trace无敏感payload policy |
| ST-G31 | Fail | 无hook revoke、in-flight和quiescence receipt |
| ST-G32 | Fail | handler panic仍截断fan-out |
| ST-G33 | Fail | poisoned/partial mutation registry仍伪装healthy |
| ST-G34 | Fail | unit/property/concurrency/fault/soak/product矩阵未建立 |
| ST-G35 | Partial | Runtime48 truth freeze和repeated-init行为说明已current；poison-safe和source guard仍stale |
| ST-G36 | Pass | frontmatter、链接、finding/gate计数、LF/BOM/trailing-space与scoped `git diff --check`由本轮文档验证 |

## 12. 验收约束

后续实施必须先从 correctness、owner 和 lifecycle RED evidence开始，而不是继续扩展当前 facade。任何里程碑都要列出旧类型、旧方法、旧re-export、旧test guard和旧doc的删除目标，并在同一切片hard cut；不能用compat shim、双写或Editor私有truth维持过渡。性能主张必须同时给出bounded request/journal、完整receipt/lifecycle语义和同workload分布，单独减少锁次数、clone或callback scan都不能宣称优于Unreal。

本报告是 review complete、implementation pending。Runtime125 文档完成不等于用户的全工程 review 目标完成，也不改变MVP、Runtime48或PERF-MVP-320的实施状态。

## 13. 状态与产出记录

| Milestone | Status | Date | Evidence |
|---|---|---|---|
