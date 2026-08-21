---
related_code:
  - zircon_runtime/src/core/framework/state/mod.rs
  - zircon_runtime/src/core/framework/state/hook.rs
  - zircon_runtime/src/core/framework/state/hook_index.rs
  - zircon_runtime/src/core/framework/state/machine.rs
  - zircon_runtime/src/core/framework/state/next_state.rs
  - zircon_runtime/src/core/framework/state/on_enter.rs
  - zircon_runtime/src/core/framework/state/on_exit.rs
  - zircon_runtime/src/core/framework/state/on_transition.rs
  - zircon_runtime/src/core/framework/state/registry.rs
  - zircon_runtime/src/core/framework/state/state.rs
  - zircon_runtime/src/core/framework/state/state_spec.rs
  - zircon_runtime/src/core/framework/state/state_transition_event.rs
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
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/38-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/35-ownership-graph-shared-weak-borrow-lease-callback-subscription-raii-cycle-detach-leak-isolation-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/02/2026-07-22-state-hook-canonical-index.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
reference_engines:
  - dev/bevy/crates/bevy_state/src/app.rs
  - dev/bevy/crates/bevy_state/src/condition.rs
  - dev/bevy/crates/bevy_state/src/state/resources.rs
  - dev/bevy/crates/bevy_state/src/state/states.rs
  - dev/bevy/crates/bevy_state/src/state/transitions.rs
  - dev/bevy/crates/bevy_state/src/state/sub_states.rs
  - dev/bevy/crates/bevy_state/src/state/computed_states.rs
  - dev/bevy/crates/bevy_state/src/state_scoped.rs
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeModule/Public/StateTreeEvents.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeModule/Public/StateTreeExecutionTypes.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeModule/Public/StateTreeExecutionContext.h
  - dev/Fyrox/fyrox-animation/src/machine/mod.rs
  - dev/Fyrox/fyrox-animation/src/machine/state.rs
  - dev/Fyrox/fyrox-animation/src/machine/transition.rs
  - dev/Fyrox/fyrox-animation/src/machine/event.rs
  - dev/Fyrox/fyrox-animation/src/machine/layer.rs
  - dev/godot/scene/animation/animation_node_state_machine.h
  - dev/godot/scene/animation/animation_node_state_machine.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 48 · Runtime-wide State、NextState、Transition、Hook、History、Schedule、Scope 与产品接入工程化差距

## 1. 结论

`zircon_runtime::core::framework::state` 不是空壳。当前 12 个 production 文件实现了按 Rust 类型隔离的 current/next state、显式与同值抑制两种 pending transition、初始/普通 transition event、`OnExit -> OnTransition -> OnEnter` 顺序、按 state/state-pair 哈希桶定位 hook，以及锁内冻结 dispatch、锁外执行 callback 的重入边界。`CoreRuntime`、`CoreHandle` 和 prelude 也公开了完整 typed facade。这些能力是可保留底座，不能在后续重构中退回字符串状态表、全表扫描或锁内 callback。

但它目前仍是一个手动调用的进程内 typed callback registry，不是进入 Zircon 产品帧循环的状态服务。全仓 production caller 反查只有 `zircon_app/src/tests/prelude.rs`；App、Editor、Runtime world、dynamic session、plugin 和 examples 都没有真实消费者。`set_next_state` 后必须由调用者自己选择时机执行 `apply_state_transition`，现有 `First -> PreUpdate -> Fixed* -> Update -> PostUpdate -> Last -> RenderExtract` 权威阶段表没有任何 state system、barrier 或 publication edge。文档称其为“Bevy-aligned application state”，但目前只复制了 DTO 和 callback 顺序，没有 Bevy 的 schedule、message cursor、previous state、dependent state、run condition、state-scoped cleanup 和 product composition。

更关键的是生命周期与正确性仍未闭合：每个类型只有一个 last-writer-wins `NextState<T>`；request 没有 owner、priority、sequence、cause 或 admission receipt；`insert_state` 覆盖已有 current 时仍记录伪造的 `None -> Some(new)`，跳过真实 exit/transition；history 是永久增长的 `Vec`，每次读取完整 clone；hook 只能注册不能撤销，闭包可永久强持 plugin/module 资源；callback panic 会发生在 current 和 history 已提交之后，后续 hook 被截断且没有结构化 terminal receipt；poisoned registry 被静默视为健康；`TypeId + Any + type_name` 既不可持久化，也不能跨 DLL、脚本、动态 API 或热重载代际稳定识别。

当前测试与文档还存在已证实的 current-source 矛盾。`CoreHandle::init_state` 已在一个 registry guard 内调用 `states.state::<T>().map(State::into_inner)`；`zircon_runtime/src/tests/state.rs::state_handle_init_existing_state_uses_direct_projection` 却仍要求旧的 `match self.state::<T>()` 源文本并禁止另一条旧表达式，静态上必然失败。`docs/zircon_runtime/core/state.md` 同样把已不存在的“双锁 direct match”写成现状。测试数量因此不能被当作当前实现已经通过的证据。

本报告登记 **0 个新增 P0、48 个 P1、12 个 P2**。Runtime48 拥有通用应用/运行时状态服务的 scope、request admission、transition receipt、schedule 接线、bounded journal、hook lifecycle 和产品资格；Runtime03 继续拥有阶段表与 executor，Runtime05/38 拥有 World/GameState 产品语义，Runtime07/46 与 Tooling35 拥有 module/plugin generation、callback lease 和 unload 通则，Runtime22/24/41 分别拥有 clock/replay identity、qualified identity 与 operation receipt 基础，PERF-MVP-320 继续拥有 state history/hook snapshot 性能预算。本文不把通用 application state 扩张成动画、AI 或 Gameplay StateTree，也不重复建立这些父 owner 的 P0。

本轮只做 review，没有修改 production、tests、Cargo 或 reference source；没有运行 Cargo、真实帧循环、多线程竞争、plugin unload、soak 或 benchmark。性能结论只陈述代码路径和缺失预算，不宣称 Zircon 当前快于或慢于 Unreal、Bevy、Godot、Fyrox 或 Unity Graphics。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 范围 | 文件 / 行 / bytes | `#[test]` / ignored | 状态 |
|---|---:|---:|---|
| `core/framework/state` production | 12 / 519 / 14,817 | 0 / 0 | 逐文件完整读取；工作树 clean |
| runtime facade / owner wiring | 5 / 861 / 30,031 | 3 / 0 | 逐文件完整读取；`runtime.rs` 成文前已有共享工作树改动 |
| dedicated state tests | 2 / 310 / 10,143 | 9 / 0 | 逐文件完整读取；未执行 |
| focused total | 19 / 1,690 / 54,991 | 12 / 0 | fingerprint `70a97bb241ae70381c39ab1e098e9cc786abe80af45fe404f473a8dcb7ac6b15` |
| integration evidence | runtime/app prelude、poison guard、module doc、全仓 caller search | 静态 source tracing | 不计入 focused fingerprint |

fingerprint 按相对路径小写排序，将每项编码为 `path + NUL + per-file SHA-256`，以 LF 连接后再次计算 SHA-256。它只标识本次读取集合，不是 ABI、artifact key 或 release identity。本报告基线 HEAD 为 `bea1acf91b909525ab1759e2c800858b0eda6528`；相关 state 集合最近可定位提交为 `322a03acfec7c8527cec593a4165af3ae31437b5`（2026-08-01）。共享工作树中 `zircon_runtime/src/core/runtime/runtime.rs` 已有其他会话/用户改动，本文读取并指纹化当前内容，但不覆盖、暂存或回退该改动。

### 2.2 产品调用链反查

对 `zircon_app`、`zircon_editor`、`zircon_plugins`、`zircon_hub`、`zircon_runtime_interface` 与 `examples` 搜索全部 state facade 调用，唯一命中是 `zircon_app/src/tests/prelude.rs` 的 `init_state/set_next_state/apply_state_transition` 编译面测试。排除 tests 后，非测试 production caller 为 **0**：

```text
CoreRuntime / CoreHandle public facade
  -> StateRegistry<TypeId, Box<dyn Any>>
  -> StateMachine<T> { current, next, events, hooks }
  -> manual apply_state_transition<T>()
  -> callback dispatch outside Mutex
  -X-> App frame loop / Runtime03 schedule
  -X-> World / dynamic session / gameplay framework
  -X-> Editor / plugin / script / network / save / replay
```

这意味着 public/prelude 可见性只证明“第三方 Rust 代码可以调用”，不能证明引擎已经拥有 application state 产品能力。任何实现里程碑必须先选择一个真实 App/Runtime composition owner 和至少一个产品 state type，再谈 feature complete。

### 2.3 当前 source、test 与文档冲突

| 项 | 当前源码 | 测试/文档声明 | 裁决 |
|---|---|---|---|
| repeated `init_state` lock | 一个 `let mut states = self.lock_states()` guard 内执行 init 与 current projection | dedicated test 和 module doc 要求 `match self.state::<T>()` direct projection | test/doc stale；不能声称 current gate 通过 |
| repeated init event | 不记录 history、不 dispatch hook，却返回新建的 `None -> current` DTO | 文档称“idempotent and returns current value” | 返回值形状无法区分 `Created` 与 `AlreadyExists`，且看起来像真实 transition |
| `insert_state` overwrite | 直接替换 current，记录 `None -> new` 并只匹配 enter | 文档只列公开方法，没有说明 bypass | 不是普通 transition；必须改为显式 bootstrap/replace policy |
| poison recovery | `into_inner()` 后继续服务 | 文档把它描述为 poison-safe | 只能证明不 panic，不能证明 partial mutation 后 registry 健康 |

### 2.4 已有 owner 与不重复范围

| 已有 owner | 继续负责 | Runtime48 只负责 |
|---|---|---|
| Runtime01 / Runtime46 | module/service lifecycle、activation/deactivation、shutdown、descriptor 与 service generation | state service 如何加入 lifecycle 与如何撤销本域对象 |
| Runtime02 / Runtime03 | event/task 公共基础、九阶段 frame schedule 与 executor | state request freeze/resolve/publish 对这些基础的接线 |
| Runtime05 / Runtime38 | World scope、GameInstance/GameMode/GameState、Play/product composition | 通用 state scope contract 与 typed product consumer |
| Runtime07 / Tooling35 | plugin reload、callback subscription token、capture policy、quiescent unload | state hook token/owner/generation 的领域落地 |
| Runtime22 / Runtime24 | frame/tick/clock/RNG/replay与 qualified identity 通则 | state request/event/journal 所需组合 identity |
| Runtime41 | admission、cancel、deadline、terminal receipt 公共 operation schema | lightweight state request outcome 与 operation 关联 |
| PERF-MVP-320 / Runtime07 performance | history bytes/clones、hook snapshot alloc、100k transitions 与 60/120 Hz budget | bounded journal 和 reusable dispatch 的正确性合同 |

## 3. 应保留的工程基础

1. `State<T>`、`NextState<T>` 与 state-specific label 都是 typed surface，避免把业务状态降级为不受 schema 约束的字符串。
2. 每个 Rust state type 拥有独立 `StateMachine<T>`，不同维度不会共享 current/next/history 容器。
3. `Pending` 与 `PendingIfNeq` 区分显式 identity transition 和仅在值变化时触发，为后续 request mode 提供了最小语义基础。
4. hook 查找已使用 `HashMap<T, Vec<_>>` 与 state-pair 二级哈希桶，当前 transition 不再扫描全部注册项。
5. dispatch 在 registry lock 内冻结 event 与匹配 hook，在锁外执行 callback，避免 hook 读取 state 或提交下一请求时直接自锁。
6. exit、transition、enter 的顺序固定，并对初始化、identity suppression 和多个正交 state type 有行为测试。
7. current state 与事件值是 owned snapshot，不向调用者泄漏 registry 内部借用。
8. Runtime03 已有权威阶段表和明确 stage 内排序基础，Runtime48 不需要另建第二套 scheduler，只需声明接入位置和 barrier。

这些能力必须迁移到 runtime-owned service，而不是保留 `framework` 行为 owner、另建 `state2`，或让各 App/World/plugin 再复制一份私有状态机。

## 4. P0 裁决

本轮不新增 P0。当前 state facade 没有 production caller，尚无证据表明它已经造成已发布持久数据损坏、跨 DLL use-after-unload 或产品权限绕过。测试与文档矛盾、无界 history、不可撤销 hook 和手动调度都按 P1 进入产品化阻塞。实施时若发现已有 plugin hook 在 DLL 卸载后仍可调用，或 save/network 已持久化 `TypeId/type_name` 并发生错误恢复，必须回到 Runtime07/24/40/46 的数据完整性与 native lifetime owner 上调 P0，而不是在本报告提前虚构事故。

## 5. P1：产品化前必须闭合的工程差距

### 5.1 Product Reachability、Ownership 与 Scope

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| STATE-P1-001 | facade 在所有 product crate/example 中只有测试 caller | 选择至少一个真实 App/Runtime mode state，接入 bootstrap、frame 和 shutdown；无 consumer 时保持 capability unavailable | Runtime48 + App01/Runtime38；production trace 可从 host 到 transition receipt |
| STATE-P1-002 | `set_next_state` 后由任意 caller 手工决定何时 apply | 将 state resolution 接入 Runtime03 权威 schedule，固定 request freeze、resolve、publish 与 cleanup barrier | Runtime48/Runtime03；每个 frame/tick 的 apply 点唯一 |
| STATE-P1-003 | `core::framework::state` 同时拥有 DTO、machine、hook index 与 registry 行为 | framework 只保留 neutral contract；machine/registry/journal/dispatch 迁入 `core::runtime` folder-backed owner | Runtime02/48；依赖方向和 module audit 通过 |
| STATE-P1-004 | `CoreRuntimeInner` 直接持有全局 `Mutex<StateRegistry>`，没有 service descriptor/lifecycle | 建立 runtime-owned `RuntimeStateService`，参与 construct/activate/drain/close，发布 health/capability | Runtime46/48；多 runtime 实例隔离和 terminal receipt 通过 |
| STATE-P1-005 | CoreRuntime、CoreHandle、core facade、prelude 同时广泛公开实验 API | 冻结目标 public surface，硬切重复/无策略入口；只公开 request、snapshot、subscription 与 receipt | Runtime48；API inventory 无旁路 apply/insert |
| STATE-P1-006 | 任意 read/write/hook 都会隐式创建 machine，拼错类型也形成永久空 slot | 类型必须显式注册 descriptor；未知 type request 返回 typed rejection，不隐式占用 registry | Runtime48；unknown type/duplicate registration fault 通过 |
| STATE-P1-007 | registry 无枚举、catalog、owner 或 enabled 状态，产品无法证明有哪些 state | 发布只读 `StateCatalogSnapshot`，列 stable type key、scope、owner generation、policy 与 health | Runtime42/46/48；capability report 与 registry 一致 |
| STATE-P1-008 | 所有 state 都是整个 CoreRuntime 进程级 scope，无法表达 World、session、PIE 或 local player | descriptor 显式选择 Runtime/World/Session/Player 等受支持 scope，slot key 带 owner/generation | Runtime05/24/38/48；双 World/PIE A-B-A reopen 无串状态 |

### 5.2 Request Admission、Ordering 与 Transition Semantics

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| STATE-P1-009 | 每个类型只有一个 `NextState<T>`，同阶段多 writer 静默 last-writer-wins | 使用有界 request inbox 或 per-system command buffer；所有 request 都有 outcome | Runtime48；冲突请求无静默覆盖 |
| STATE-P1-010 | request 没有 producer/module/system identity | request 携带 qualified producer、owner generation 与可选 operation ID | Runtime24/41/48；stale owner 请求 fail-close |
| STATE-P1-011 | 没有 priority、policy 或 tie-breaker | descriptor 定义 SingleWriter/HighestPriority/ExplicitArbiter 等策略，禁止隐式注册顺序 | Runtime48；反转 producer 注册顺序结果不变 |
| STATE-P1-012 | 并发线程的 lock acquisition 决定最终 queued value | scheduler producer 使用稳定 system order + local sequence；外部 ingress 使用唯一 admission sequence 并记录来源 | Runtime03/48；重复并发回放得到相同 receipt 序列 |
| STATE-P1-013 | `PendingIfNeq` 只在 apply 时看 current，之前的 `Pending`/覆盖关系无显式裁决 | request mode 与 supersession rule 进入 resolver，保留 Suppressed/Superseded 原因 | Runtime48；混合 set/set-if-different 矩阵通过 |
| STATE-P1-014 | `insert_state` 覆盖已有 current，却记录 `None -> new` 并跳过真实 exit/transition | 将 bootstrap 与 replace 分成显式命令；replace 必须产生真实 previous/current 或被拒绝 | Runtime48；overwrite 不得伪造初始化 |
| STATE-P1-015 | repeated `init_state` 返回形似 transition 的 synthetic event，但不记录、不 dispatch | 返回 `StateInitializationReceipt::{Created,AlreadyPresent}`，不得冒充 event | Runtime48；重复初始化无假 transition |
| STATE-P1-016 | setter 返回 `()`，没有 accepted/rejected/superseded/cancelled/deadline receipt | 每个 request 返回/关联稳定 ID；resolver 发布 terminal `StateRequestReceipt` | Runtime41/48；每个 accepted request 恰有一个 terminal outcome |

### 5.3 Lifecycle、Removal、Dependency 与 State Scope

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| STATE-P1-017 | transition 只能进入 `Some(T)`，无法表达 `Some -> None` | 支持明确 Remove/Deactivate，并发布 exit 与 removal receipt | Runtime48；remove 后 current 缺席且 exit 顺序正确 |
| STATE-P1-018 | machine 一旦创建只能随整个 runtime drop，没有 reset/retire | descriptor owner 可 drain、remove slot、回收 journal/hook，并阻止旧 generation 请求 | Runtime46/48；retire/re-register 不复活旧数据 |
| STATE-P1-019 | hook registration 不返回 token，无法 unregister | 引入 `StateSubscriptionToken`/RAII revoke，token 绑定 state scope 与 owner generation | Tooling35/Runtime48；drop/revoke 后不再调用 |
| STATE-P1-020 | `'static Arc<Fn>` 可永久强持 plugin/module/world 资源 | 声明 capture policy，默认弱引用/窄 lease；泄漏审计报告 remaining roots | Tooling35/Runtime07/48；terminal 后 LeakCensus 归零 |
| STATE-P1-021 | hook 与 plugin/module generation、admission close、quiescence 无关 | unload 先关闭注册/dispatch admission，再等待 in-flight，撤销 generation，最后卸载代码 | Runtime07/46/48；DLL unload fault matrix 无 stale callback |
| STATE-P1-022 | exit 不能自动清理 state-scoped entity/resource/task/subscription | 提供 state-scope owner 与 cleanup system，在 exit 后、enter 前/后按裁决运行 | Runtime05/48；离开 state 无 scoped survivor |
| STATE-P1-023 | orthogonal state 互不知晓，没有 sub/computed dependency graph 与顺序 | 仅对明确需要的 dependent state 注册 acyclic dependency depth，exit leaf-to-root、enter root-to-leaf | Runtime48；cycle registration 被拒绝，顺序稳定 |
| STATE-P1-024 | 系统不能声明 `in_state/state_changed` run condition，只能在 callback 中手写分支 | scheduler 提供 typed read condition 与 transition-changed condition，纳入 access/conflict graph | Runtime03/05/48；disabled system 不执行且无隐藏锁 |

### 5.4 Event Journal、Dispatch、Failure 与 Data Movement

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| STATE-P1-025 | 每个 machine 永久保存全部 transition event | 使用有界 journal，配置 entry/bytes/age，并定义 PreserveTerminal/DropOldest/Reject 策略 | PERF-MVP-320/Runtime48；100k transition 内存有上界 |
| STATE-P1-026 | `state_transition_events()` 每次 clone 全历史 | 改为 snapshot cursor/read batch/drain；lag 返回 gap receipt，禁止全量 convenience API 留在 public surface | Runtime02/48；查询成本与 unread batch 成正比 |
| STATE-P1-027 | event 没有 sequence、frame/tick、clock domain 或 timestamp | journal record 带 monotonic sequence、scope generation、frame/tick/clock identity | Runtime22/24/48；跨 fixed/update trace 可排序 |
| STATE-P1-028 | event 没有 request ID、producer、cause、reason、outcome 或 error | 统一 request/transition receipt，区分 Applied/Identity/Removed/Suppressed/Rejected/Failed | Runtime41/48；诊断不再推断原因 |
| STATE-P1-029 | `T` 在 current、next、event、history、return value和hook snapshot中反复 clone | 冻结适用 payload 预算；大 payload 使用 immutable shared value/compact key，记录 clone bytes | PERF-MVP-320/Runtime48；规模门证明 hot path clone budget |
| STATE-P1-030 | 每次 dispatch 为 exit/transition/enter 各 clone 一个 `Vec<Arc<Fn>>` | 稳定注册后发布 immutable slot/slice 或复用 scratch；保留锁外执行 | PERF-MVP-320/Runtime48；steady transition snapshot alloc=0 |
| STATE-P1-031 | callback 是 opaque `Fn`，无 scheduler access、依赖、thread affinity、budget 或 tracing | product hook 改为 scheduler system/owned subscriber；声明资源访问、顺序、执行域和预算 | Runtime02/03/48；并行冲突与超时可观测 |
| STATE-P1-032 | current/history 先提交，任一 hook panic 会截断后续 hook并向 caller unwind | engine boundary containment panic/error，完成既定 fan-out，发布 committed-with-handler-failures receipt 并按 owner policy quarantine | Runtime01/48；panic/reentrancy/fan-out fault matrix 通过 |

### 5.5 Type Identity、Schema、ABI 与 Definition Contract

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| STATE-P1-033 | `TypeId + Any` 只在当前 Rust process/build 有意义 | runtime 内可继续用 TypeId 加速，但公开/持久/跨边界使用 stable `StateTypeKey` + schema version | Runtime24/48；不同 build/reload 不序列化 TypeId |
| STATE-P1-034 | blanket `impl<T> StateSpec for T` 使类型不能自定义 descriptor/state name | 删除 blanket public contract，使用显式 derive/registration 或 sealed adapter | Runtime48；duplicate key/schema mismatch 编译或注册失败 |
| STATE-P1-035 | 默认 `state_name()` 使用 compiler `type_name` 且当前业务路径不消费 | display name、stable key、Rust debug name 分离；compiler name 只作诊断附加项 | Runtime24/48；rename 不破坏持久 identity |
| STATE-P1-036 | 无 reflect/serde/schema/migration/unknown-value policy | descriptor 可选注册 reflection、codec、version/migration 与 unknown handling | Runtime04/48；roundtrip、old/new reader matrix 通过 |
| STATE-P1-037 | dynamic API、ZrVM、native plugin ABI 都无法注册/查询/请求 state | 定义 ABI-safe descriptor/request/snapshot/receipt；Rust generic facade 只作本地 typed adapter | Interface05/Runtime07/43/48；跨 DLL/VM fixture 通过 |
| STATE-P1-038 | `Clone + Eq + Hash + Debug` 接受任意昂贵/敏感 payload，没有大小、日志或 redaction 策略 | 限制 state 为紧凑 discriminant/qualified key，或 descriptor 声明 clone/hash/redaction budget | Runtime03/48；敏感值不进入 Debug 日志，规模超限被拒绝 |
| STATE-P1-039 | `OnEnter/Exit/Transition` 以完整 state value 为哈希 key，没有 definition generation | hook key 绑定 stable variant key、descriptor generation 与 scope；旧定义不能匹配新 slot | Runtime24/48；hot reload A-B-A 不误触发 |
| STATE-P1-040 | 没有可枚举 variant、合法 transition graph、guard 或 definition validation | descriptor 提供可选 finite definition/validator；开放值 state 明确标为 non-enumerable | Runtime48；非法 edge、missing initial、cycle/duplicate key 有 typed error |

### 5.6 Consistency、Diagnostics、Documentation 与 Verification

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| STATE-P1-041 | poisoned mutex 用 `into_inner()` 静默继续，无法区分 healthy 与 partial mutation | registry mutation 使用 prepare/commit guard；poison/logic panic 标 health，quarantine 或重建，发布诊断 | Runtime01/03/48；poison fault 不伪装健康 |
| STATE-P1-042 | 没有 request、apply、suppression、queue、history、hook latency 或 failure 指标 | 发布 per-scope bounded metrics/trace，diagnostics off 近零 bookkeeping | Runtime03/48；60/120 Hz workload 可归因 |
| STATE-P1-043 | previous state 只能从完整 history clone 后猜最后事件 | slot 直接维护 O(1) current/previous/last receipt snapshot，并定义 init/remove 后语义 | Runtime48；读取不依赖 journal retention |
| STATE-P1-044 | `state()`、`next_state()`、history 分别加锁，调用方无法取得同代一致快照 | 提供 generation-stamped `StateSnapshot<T>`，一次读取 current/previous/pending/last outcome | Runtime24/48；并发 apply 下 snapshot 内部一致 |
| STATE-P1-045 | dedicated source-text test 要求已不存在实现，静态上必然失败 | 删除实现字符串锁定，改为行为测试：重复 init 只取一次 guard、无 transition/hook/history side effect | Runtime48；current test 可执行并通过 |
| STATE-P1-046 | module doc 把 stale direct-match 与“poison-safe”写成已验证现状 | 按 current source 重写文档，区分 implemented/reviewed/pending 和未运行 Cargo | Runtime48 + docs maintenance；doc currentness guard 通过 |
| STATE-P1-047 | 测试缺 panic、reentrancy、并发 conflict、overwrite、remove、retire、unregister、retention 与 many-type scale | 建立 unit/property/concurrency/fault/soak matrix，删除以 `.contains()` 为核心的实现形状断言 | Runtime48；矩阵全部绑定 source/build receipt |
| STATE-P1-048 | 没有 App/World/PIE/plugin/save/replay 的端到端消费者和 shutdown 证据 | 至少一个真实产品 state 完成 init/request/apply/gate/scoped cleanup/reopen/close 链 | Runtime38/48；非测试 frame trace 与双实例隔离通过 |

## 6. P2：后续能力与维护性债务

| ID | 能力差距 | 后续方向 | 边界 |
|---|---|---|---|
| STATE-P2-001 | 无 derive/registration ergonomics 与编译期诊断 | 提供 `#[derive(States)]` 等价的 Zircon macro 或 codegen，生成 descriptor/variant key | 不能掩盖 runtime registration failure |
| STATE-P2-002 | 无 declarative transition guard/condition | 在 finite definition 上允许纯读 guard，声明 access 与失败原因 | 复杂 gameplay logic 仍留在系统，不塞进通用 DSL |
| STATE-P2-003 | 无 delayed/deadline transition | request 可选 target tick/time domain 和 cancellation | 时钟语义必须复用 Runtime22 |
| STATE-P2-004 | 无 transition graph introspection/visualization | 发布 read-only graph、active edge、recent receipt 给 devtools | 不能让 Editor 成为运行时真值 owner |
| STATE-P2-005 | 无 opt-in save/checkpoint persistence | descriptor 声明 capture/restore/migration policy | 默认 runtime state 不自动持久化 |
| STATE-P2-006 | 无 opt-in network replication/prediction | 由 network owner 映射 qualified state receipt、authority 和 rollback | 不能把所有 local application state 自动复制 |
| STATE-P2-007 | 无 replay/rewind inspection | journal receipt 可进入 deterministic replay evidence；debug rewind 使用隔离实例 | 不把 debug history 变成无界 production history |
| STATE-P2-008 | 无 async enter/exit orchestration | 通过 Task/Operation owner组合异步准备、cancel、timeout 和 terminal barrier | 不在 state core 自建线程池/future runtime |
| STATE-P2-009 | 无 transition heatmap、latency percentile 与 branch coverage | diagnostics reader 按需聚合，不在 hot path永久保存明细 | diagnostics off 保持近零成本 |
| STATE-P2-010 | 无 property/model-based transition验证 | 从 finite definition 生成 sequence、冲突和 invariant 测试 | 只作为验证工具，不进入 product runtime |
| STATE-P2-011 | 无 editor-authored statechart asset | 若产品需要，建立独立 authored StateGraph/StateTree domain 和 compiler artifact | 不把通用 App state 强行升级为 Unreal StateTree |
| STATE-P2-012 | 无 animation/AI/gameplay state adapter | 以桥接 snapshot/receipt 组合现有 Animation、AI、Gameplay owner | 各领域保留自己的 transition/blend/authority 语义 |

## 7. 参考引擎对照与适用边界

| 参考 | 已核源码事实 | Zircon 应吸收 | 明确不照搬 |
|---|---|---|---|
| Bevy `bevy_state` | `StatesPlugin` 把 `StateTransition` 放在 startup 前与每帧 `PreUpdate` 后；transition schedule 分 dependent/exit/transition/enter 四阶段；State/PreviousState/NextState 是资源；event 使用 Message reader cursor；有 run conditions、computed/substates、state-scoped entity | application state 的 schedule 接线、previous/current、cursor、dependent order、scope cleanup 与显式 registration | 不必引入 Bevy ECS 或完全复制其 last-writer `NextState`; Zircon 目标应补 producer receipt 与 bounded policy |
| Unreal StateTree | transition request 带 priority、source frame/state；event 有 tag/payload/origin，queue `MaxActiveEvents = 64` 且支持 consume/phase clear；execution context有 active frame、run status、instance data、direct-transition scope、可选 recorded transition/debug | request provenance/priority、有限队列、实例代际、运行状态、可选调试记录和直接迁移作用域 | StateTree 是层级行为执行框架，不应作为通用 App state 的一对一模板 |
| Fyrox animation machine | reflected/serializable state/transition/layer；transition 有条件、时间、blend；state 有 enter/leave actions；event queue 有固定容量；layer有 active state/transition和 debug | authored graph 的 definition/instance 分离、有限事件、可观察 transition 生命周期 | 只适用于 Animation adapter；不能把 pose/blend/parameter 语义塞入 RuntimeStateService |
| Godot animation state machine | Resource 定义 state/transition；支持 nested/grouped、priority、switch mode、xfade/reset、start/travel/next/stop request、每实例唯一 playback、validation 与 signals | definition/playback instance 分离、请求状态、路径/优先级、实例唯一性与 validation | 只适用于 Animation/authoring；不把 StringName graph 当作跨域 state identity |
| Unity Graphics | 本地 `dev/Graphics/Packages` 对 `state machine/StateTransition` 的全包搜索无通用 application state owner；源码范围是 SRP/render graph/command/render state | 作为负向边界：Graphics 不能为 gameplay/application state 完成度背书 | 不从 renderer package 推测闭源 Unity Gameplay/Animator/Editor 行为 |

参考共同点不是“类更多”，而是 definition、runtime instance、request、event、schedule、scope、failure 和 diagnostics 各有明确 owner。Zircon 可以采用更紧凑的数据结构和更低分配实现，但不能通过删除这些语义来制造“比 Unreal 更快”的结论。

## 8. 目标架构

### 8.1 所有权与层次

```text
core::framework::state
  StateTypeKey / StateScopeKey / StateDescriptor
  StateRequest<T> / StateSnapshot<T>
  StateTransitionReceipt<T> / StateSubscriptionToken
              |
              v
core::runtime::state
  RuntimeStateService
    descriptor catalog
    scoped typed slots
    bounded request inboxes
    resolver + journal + subscriber registry
    lifecycle / health / diagnostics
              |
              v
Runtime03 schedule
  FreezeRequests -> Resolve -> Exit -> Commit/Transition -> Enter -> ScopeCleanup
              |
              v
App / World / Session / Player product adapters
  real state definitions, run conditions, scoped resources, product traces
```

`core::framework` 只保留跨 owner 可共享的 typed contract；`core::runtime` 拥有存储、锁、队列、resolver、journal、subscription 和 lifecycle。World/GameState、Animation、AI、Gameplay 不共享一个万能 state machine，只通过稳定 request/snapshot/receipt 接口组合。

### 8.2 核心合同草案

```rust
pub struct StateDescriptor {
    pub type_key: StateTypeKey,
    pub schema_version: u32,
    pub scope_kind: StateScopeKind,
    pub arbitration: StateArbitrationPolicy,
    pub journal_budget: StateJournalBudget,
    pub owner: OwnerGeneration,
}

pub struct StateRequest<T> {
    pub id: StateRequestId,
    pub scope: StateScopeKey,
    pub producer: ProducerKey,
    pub producer_generation: u64,
    pub mode: StateRequestMode,
    pub priority: i32,
    pub target: Option<T>,
    pub cause: StateTransitionCause,
}

pub enum StateRequestOutcome {
    Applied,
    IdentityApplied,
    Removed,
    Suppressed,
    Superseded,
    Rejected,
    Failed,
}
```

具体字段由实现计划定稿，但 identity、scope、producer、sequence、cause、outcome 与 owner generation 不能再缺失。Rust `TypeId` 可以作为服务内部已注册 slot 的加速索引，不能出现在 schema、日志主键、ABI 或持久化文件中。

### 8.3 调度、提交与失败语义

1. scheduler 系统和外部 ingress 只提交 request，不直接改 current。
2. frame/tick barrier 冻结本轮 request；late request 明确进入下一轮。
3. resolver 按 descriptor policy、priority、producer order 和 sequence 产生每个 request 的 outcome。
4. transition 发布 exit phase，随后一次 commit current/previous/generation，再发布 transition 与 enter phase；state-scoped cleanup 顺序必须由 descriptor/系统依赖固定。
5. handler panic/error 在引擎边界被隔离并聚合；已经 commit 的 transition 返回 `AppliedWithHandlerFailures` 等可诊断事实，不回写伪 rollback。
6. owner close 会停止 admission、等待 in-flight、撤销 subscriptions、retire slot/journal，再允许 plugin/module/DLL 卸载。

### 8.4 Journal、snapshot 与性能模型

- current/previous/pending summary/last outcome 是 O(1) generation-stamped snapshot，不依赖历史。
- journal 使用 entries + declared bytes + age 三维预算和 consumer cursor；lag/eviction 产生 gap receipt。
- stable registration 后 hook/subscriber lookup 不全表扫描，steady transition 不创建三组临时 Vec/Arc RMW。
- state value 默认是小型 discriminant/qualified key；大 payload 放在所属 domain resource，由 state 只引用稳定 key。
- diagnostics off 不永久记录 stack/payload；diagnostics on 的 detailed trace 有采样、redaction 与 retention。

## 9. 硬切范围与禁止方案

1. 不保留 `framework::state::StateRegistry/StateMachine` compatibility re-export；行为 owner 迁移时一次改完调用点。
2. 不新建 `state2`、`app_state_manager` 或 Editor 私有 state registry 与旧实现并存。
3. 不把手工 `apply_state_transition` 继续作为产品主路径；测试/工具若需显式 pump，必须调用同一 schedule/service entry。
4. 不以 `Mutex` 存在、poison recovery或锁外 callback 作为完整并发/失败资格。
5. 不保留无界 `Vec` history 或返回整段 clone 的 public convenience API。
6. 不序列化 `TypeId`、`type_name`、裸指针、closure identity 或 registration order。
7. 不允许无 token、无 owner generation、无 quiescence 的 `'static` hook 跨 plugin/module reload。
8. 不把 callback panic 吞掉后返回普通 Applied，也不尝试假装回滚已执行的外部 side effect。
9. 不把所有 state 自动持久化、复制、replay 或暴露给 Editor；这些能力由 descriptor opt-in。
10. 不把通用 application state 变成动画 blend graph、AI StateTree 或 Gameplay Ability 状态机；这些领域只使用 adapter。
11. 不用 source `.contains()`、路径存在、prelude 编译或测试数量代替行为、并发、产品和 teardown 证据。
12. 不宣称性能优于 Unreal/Bevy/Fyrox/Godot/Unity，除非同 workload、同硬件、同正确性和统计协议全部通过。

## 10. 重构里程碑

### M0 · Truth Freeze 与 stale evidence 清理

- 修正 failing source-text test 和 `docs/zircon_runtime/core/state.md`；
- 冻结真实 public caller、state type、scope、writer、history consumer 与 hook owner 清单；
- 建立 current-source inventory/fingerprint，保持 production 行为不变。

### M1 · Contract / Runtime Owner 硬切

- `framework` 留 descriptor/request/snapshot/receipt/token；machine/registry/journal 迁 `core::runtime::state`；
- 删除 blanket `StateSpec` 与实验 facade 旁路，建立显式 registration/catalog；
- 将 state service 接入 Runtime46 lifecycle/health。

### M2 · Schedule、Admission 与 Deterministic Resolver

- 接入 Runtime03 stage/barrier，冻结 request batch；
- 建立 bounded inbox、producer identity、priority/arbitration、sequence 和 terminal receipt；
- 修正 init/insert/remove/identity/supersession 语义。

### M3 · Subscription、Scope 与 Unload

- callback 迁 scheduler system/owned subscriber；
- token/revoke/capture policy/generation/quiescence 全链闭合；
- 完成 state-scoped cleanup、machine retire 和 dependent state ordering。

### M4 · Snapshot、Journal、Diagnostics 与性能止损

- current/previous snapshot O(1)，history 改 bounded cursor journal；
- stable hook dispatch allocation 收敛；
- 发布 request/apply/lag/drop/failure/latency 指标并完成 PERF-MVP-320 workload。

### M5 · Product Integration 与 Failure Matrix

- 至少一个 App/World/Session state 进入真实 frame loop；
- 完成双 runtime、双 World、PIE、plugin reload、shutdown、panic、queue full、stale generation、100k transition soak；
- dynamic API/VM/plugin adapter 只在真实需求与 ABI gate 下启用。

### M6 · 可选高级能力

- 按产品需求逐项启用 computed/substate、persistence、replication、replay、async orchestration 和 devtools graph；
- Animation/AI/Gameplay authored state graph 各自立项，不回灌通用 state core。

## 11. 验收矩阵

| Gate | 验收内容 |
|---|---|
| ST-G01 | 12/12 state production、5/5 facade/owner、2/2 dedicated test 文件清单与 fingerprint 可重建 |
| ST-G02 | production caller 不再为 0；host -> request -> schedule -> receipt trace 可达 |
| ST-G03 | framework 中无 machine/registry/journal/lock/callback 行为 owner |
| ST-G04 | RuntimeStateService 参加 construct/activate/drain/close，health 可读 |
| ST-G05 | 双 CoreRuntime、双 World、PIE session state 完全隔离 |
| ST-G06 | 未注册/disabled state fail-close，不隐式创建 slot |
| ST-G07 | state resolution 只在 Runtime03 声明 barrier 发生 |
| ST-G08 | late request 明确进入下一轮且 receipt 可观察 |
| ST-G09 | 多 writer 不静默覆盖，每个 request 有 terminal outcome |
| ST-G10 | priority/tie-breaker 与 producer 注册顺序无关 |
| ST-G11 | 并发重复执行产生相同 outcome/journal 序列 |
| ST-G12 | fixed/update/async ingress 带正确 clock/frame/tick identity |
| ST-G13 | init Created/AlreadyPresent 不冒充 transition |
| ST-G14 | replace 不伪造 `None -> Some`，remove 产生 `Some -> None` |
| ST-G15 | identity、suppression、supersession、rejection 矩阵完整 |
| ST-G16 | current/previous/last outcome snapshot 同代一致 |
| ST-G17 | sub/computed dependency cycle 被拒绝，exit/enter 顺序稳定 |
| ST-G18 | state-scoped entity/resource/task/subscription 退出后归零 |
| ST-G19 | journal entries/bytes/age 有硬预算，100k transition 内存有界 |
| ST-G20 | consumer cursor lag/eviction 返回 gap，不静默漏事件 |
| ST-G21 | steady read 不 clone 全历史，成本与 unread batch 成正比 |
| ST-G22 | stable hook registration 下每 transition snapshot alloc=0 |
| ST-G23 | state payload clone/hash/log bytes 有预算与 redaction |
| ST-G24 | diagnostics off 近零 bookkeeping，on 时 60/120 Hz 指标可归因 |
| ST-G25 | stable StateTypeKey/schema/version 不依赖 TypeId/type_name |
| ST-G26 | descriptor duplicate/schema mismatch/unknown variant typed fail |
| ST-G27 | ABI/VM/plugin 只传 ABI-safe descriptor/request/snapshot/receipt |
| ST-G28 | hot reload A-B-A 不匹配旧 hook、request、slot 或 journal |
| ST-G29 | persistence/replication/replay 默认关闭且 opt-in policy 明确 |
| ST-G30 | sensitive state 不通过 Debug/event/trace 泄露 |
| ST-G31 | hook token revoke/drop 后无新调用，in-flight 有 quiescence receipt |
| ST-G32 | handler panic不截断其余既定 fan-out，失败聚合且状态事实一致 |
| ST-G33 | poisoned/partial mutation registry 进入 quarantine/recovery，不伪装 healthy |
| ST-G34 | unit/property/concurrency/fault/soak/product tests 绑定 current source/build receipt |
| ST-G35 | module doc、计划、public API 与 current source 无 stale direct-match 声明 |
| ST-G36 | Markdown frontmatter、链接、finding/gate计数、LF/BOM/trailing-space与 `git diff --check` 全通过 |

## 12. 本轮状态

| 项 | 状态 |
|---|---|
| Zircon focused 19 文件逐文件 review | complete |
| product caller 与 current test/doc 矛盾反查 | complete |
| Bevy / Unreal / Fyrox / Godot / Unity Graphics 参考边界 | complete |
| 0 P0 / 48 P1 / 12 P2 / 36 gates 登记 | complete |
| production / tests / Cargo 修改 | none |
| RuntimeStateService 重构与产品接入 | pending |
