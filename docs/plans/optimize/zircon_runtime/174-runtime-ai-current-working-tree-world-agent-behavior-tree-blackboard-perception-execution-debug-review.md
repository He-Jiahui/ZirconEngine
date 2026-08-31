---
title: Runtime AI、Behavior Tree、Blackboard、Perception 当前工作树 World/Agent/Execution/Debug 工程化复审
category: zircon_runtime
report_id: Runtime174
review_date: 2026-08-30
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/100-runtime-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-task-navigation-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/234-editor-ai-current-working-tree-authoring-graph-debug-overlay-workbench-review.md
related_code:
  - zircon_runtime/src/core/framework/ai
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime
  - zircon_plugins/ai/editor
  - zircon_plugins/ai/dist
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
tests:
  - zircon_plugins/ai/runtime/src
  - zircon_plugins/ai/editor/src/tests.rs
  - zircon_plugins/ai/dist/src/lib.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/100-runtime-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-task-navigation-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/142-editor-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-debug-authoring-current-source-review.md
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/performance/01/2026-07-30-runtime-framework-animation-ai-navigation-tasks-static-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BehaviorTreeComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BehaviorTreeComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BlackboardComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BlackboardComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/Perception/AIPerceptionSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Perception/AIPerceptionSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/Perception/AISense_Sight.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Perception/AISense_Sight.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/EnvironmentQuery/EnvQueryManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/EnvironmentQuery/EnvQueryManager.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeModule/Public/StateTreeExecutionContext.h
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeDebugger.cpp
  - dev/UnrealEngine/Engine/Plugins/AI/EnvironmentQueryEditor/Source/EnvironmentQueryEditor/Private/EnvironmentQueryEditor.cpp
  - dev/Fyrox/fyrox-impl/src/utils/behavior/mod.rs
  - dev/godot/scene/2d/navigation/navigation_agent_2d.cpp
  - dev/godot/scene/3d/navigation/navigation_agent_3d.cpp
  - dev/bevy/crates/bevy_tasks/src/lib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
---

# Runtime174 · AI 当前工作树复审

## 1. 结论

当前 AI runtime 有真实的算法底座，不是可以用几行 `if/else` 代替的演示代码：`zircon_plugins/ai/runtime` 当前为 84 个文件、21,634 行、777,751 bytes、217 个 `#[test]`、42 个 `#[ignore]`；framework AI contract 为 9 个文件、1,040 行、31,239 bytes；`dist` 为 2 个文件、115 行、4,087 bytes。编译器已经产出带 parent、child range、abort flags、subtree target 和 implementation slot 的不可变 tree，manager 还保留了 `Arc<[CompiledBehaviorTree]>` generation；Blackboard 有 dense layout/store、changed-slot observer；Perception 有一次 records pass、pair cursor、bounded hearing ingress；节点目录有 owner/revoke gate，局部 Navigation/Animation/Script host 与 debug event 也存在。这些应作为后续重构的底座。

但它仍不是工程级 AI Runtime。当前真实路径是“手工注册 descriptor/schema -> 直接调用 manager -> 每 World 共享大锁 -> 每 agent 递归求值 -> 把若干结果投影为 snapshot”。没有 `source asset -> importer/cook -> immutable artifact -> Scene/ECS Brain/Agent -> per-World lifecycle -> cancellable task/query -> network/save/replay -> editor debug` 的单一真值链。`plugin.toml` 把 perception 标为 `complete`（第 47-48 行），与实际只实现 Sight/Hearing、无 spatial index、物理缺失时 fail-open 的实现不符。

本轮是 current-source refresh，旧 Runtime08F、Runtime100、Plugins15 的历史 finding 不删除；只把已经完成的局部优化（compiled generation Arc、单次 records pass、targeted snapshot）从旧结论中区分出来。登记 **28 项新增 P1、8 项新增 P2、20 个资格门**，不新增唯一 P0。所有实现仍 pending；本报告不运行 Cargo、benchmark 或 tooling lane。

## 2. 审查范围与证据

### 2.1 当前物理冻结

| 范围 | 文件 | 行 | bytes | tests | ignored |
|---|---:|---:|---:|---:|---:|
| `zircon_runtime/src/core/framework/ai` | 9 | 1,040 | 31,239 | 0 | 0 |
| `zircon_plugins/ai/runtime` | 84 | 21,634 | 777,751 | 217 | 42 |
| `zircon_plugins/ai/editor`（关联边界） | 12 | 2,382 | 83,179 | 18 | 4 |
| `zircon_plugins/ai/dist` | 2 | 115 | 4,087 | 2 | 0 |
| `zircon_plugins/ai/plugin.toml` | 1 | 122 | 4,495 | 0 | 0 |
| first-party runtime catalog | 6 | 1,289 | 46,335 | 11 | 0 |
| first-party editor catalog | 4 | 251 | 8,978 | 6 | 0 |
| AI Workbench 两份 ZUI | 2 | 478 | 26,885 | 0 | 0 |

统计为当前工作树逐文件读取的物理值；测试属性包含 release-only ignored benchmark，不代表已执行或取得产品资格。未把整个 `zircon_app/src/entry` 或整个 Workbench callback 目录冒充 AI 专属实现，而是在下面只引用 AI 直接命中的注册与反馈点。

### 2.2 关键证据定位

1. `zircon_plugins/ai/plugin.toml:7-12,33-48` 声明 maturity 为 experimental、支持 client/server/editor host，并将 runtime/plugin、behavior tree、blackboard 标为 partial，却将 perception 标为 complete；仅声明 `ai.perception_source` 与 `ai.perception_receiver` 两个组件（第 63-80 行）。
2. `zircon_plugins/ai/runtime/src/plugin.rs:137-153` 的 runtime descriptor 与 manifest 同样把 perception 标成 complete；这不是编译器或场景激活证据。
3. `zircon_plugins/first_party_runtime_catalog/src/lib.rs:31-38` 有 AI runtime provider 分支；`zircon_plugins/first_party_editor_catalog/src/catalog.rs:35-50` 只有 Navigation 与 Neural editor provider，AI 没有 editor route；App 入口只委托该 catalog（`zircon_app/src/entry/first_party_editor_plugins.rs:15-38`）。
4. `zircon_plugins/ai/runtime/src/manager.rs:18-20` 的核心 state 是单一 `Arc<Mutex<AiRuntimeState>>`，catalog 另用 `Arc<RwLock<...>>`；`manager/state.rs:13-25` 把所有 tree/schema/blackboard/perception/instance/report map 放进同一 state。
5. `manager/tick.rs:45-137` 在锁内取得 generation、遍历所有 tree 的 implementation slots 并收集 owner lease，然后 remove agent 的 blackboard/instance，锁外执行，最后在 `:249-273` 重新 insert；这是可重入保护不足的 remove/evaluate/reinsert 状态机，不是 per-agent actor。
6. `manager/tick.rs:334-390` 只按 `active_behavior_trees` 构造请求，LOD 未 tick 时累加 pending delta；请求的 blackboard 是空 Vec，真实数据来自 store；循环随后串行调用每一个 agent。
7. `behavior_tree/catalog.rs:57-157` 的 node context 只有 parameters、只读 blackboard、perception 和 delta；descriptor 只有 id/display/category/semantics/recheck/factory，没有参数 schema、pins、side-effect、线程、资源或 latent metadata。
8. `behavior_tree/executor.rs:54-86,220-410` 仍以 `Vec<String>` stack 和递归 `evaluate_node` 求值，未见深度、节点、wall-time、内存或 callback fuel budget；`:399-400` 把 SetBlackboard/EmitEvent 合并到通用 task 分支。
9. `behavior_tree/executor/selector.rs:327-365` 通过静态 `TASK_RESULT_PARAMETER_KEY` 推断 child status；`executor.rs:771-812` 通过 `service_result` 和 `result` 参数决定 service/wait 结果。
10. `behavior_tree/nodes/integration.rs:43-112,114-190` 的集成 context 只有 node、parameters、entity、delta、started；`write_nav_target(None)` 在 `:176-188` 写入实体当前位置，`:199-249` 以历史 `NavAgentTickReport` 和 float epsilon 匹配完成；`:292-348` 每次 ScriptTask tick 同步调用 VM；PlayAnimation 在 `:252-289` 写参数并立即 Succeeded。
11. `perception/scan.rs:110-223` 先收集 receiver/source，再按 `receivers.len() * sources.len()` 建 Cartesian pair cursor；预算只有 `max_pairs_per_frame=256`（第 20-71 行），没有 spatial、query、wall、bytes 或 oldest-latency budget。
12. `perception/scan.rs:225-285` Sight 的 occlusion provider 返回 `None` 时进入 fallback，直接刷新可见 stimulus；`:321-344` 每 tick 仍从 `world.node_records()` 生成 samples。单次 records pass是改进，但不等于增量 registry。
13. `perception/components.rs:53-113` 只有 Source 的 bitflag channels/strength 与 Receiver 的 sight FOV/range、hearing radius、forget seconds；contract 虽枚举 Damage/Touch/Custom，生产 scan 只实现 Sight/Hearing。
14. `registration.rs:313-406` 感知系统通过 sound/animation event、physics interface、`replace_world_perception_snapshots` 推送 snapshot；`:409-512` 行为系统按 active camera 到 agent 的距离决定 LOD，随后为每个报告构造 debug frame 并发送完整 `AiBehaviorDebugSnapshot`。
15. `manager/snapshot.rs:8-36` 的 full snapshot 用 HashSet 合并四张 map，顺序不确定并 clone tree descriptor；targeted snapshot（`:40-58`）是已存在的局部优化，但 debug producer 仍复制每个活动 agent 的完整 blackboard/perception。
16. `zircon_plugins/ai/dist/src/lib.rs:20-46` 明确 `is_stateless=true`、state schema 0、command/event manifest 为空，invoke/save/restore/unload/bridge/on_host_ready 均为 None；它证明 ABI metadata 可导出，不证明 native package 可独立运行 AI。
17. `zircon_runtime/src/core/framework/ai/manager.rs:9-40` 的公共 trait 只有同步 register/set/get/tick/full snapshot，无 world close、agent lifecycle、generation、cancel ticket、fault receipt 或 async query contract；`tick.rs:8-67` 的 report 只有 world/entity/status/active_node/diagnostic。

## 3. 已有底座与应保留方向

| 底座 | 当前真实能力 | 重构时必须保留的性质 |
|---|---|---|
| compiled tree | `compile.rs` 已做 id/root/parent/cycle/reachability 校验并生成 dense node | 保留 immutable artifact 与 slot indexing；增加版本、digest、limits、source map、last-good generation |
| immutable generation | `AiRuntimeState.compiled_behavior_tree_generation` 用 Arc slice，且有 release-only paired benchmark | 保留 Arc/COW；改为 active generation pin/retire，不回到每 tick deep clone |
| Blackboard dense store | schema layout、typed slots、changed-slot observer、Dynamic fallback | 保留 typed storage；把 fallback限制到 legacy/tooling，补事务、默认/继承、stable key、save/net policy |
| owner-aware catalog | node slot、owner lease、revoke listener 与 execution gate | owner 必须贯穿 program/task/sense/query/debug，撤销要 quiesce 并等待 ticket 终态 |
| perception ingress | single records pass、event cursor、bounded hearing backlog、forgetting | 从全量采样迁移到 component lifecycle + spatial candidate + 多维预算 |
| targeted debug projection | `runtime_snapshots_for_agents` 已可按世界/实体投影 | 保持只对订阅者投影，改为 delta/ring/trace receipt |

## 4. P1 差异与重构合同

### 4.1 World、Asset、Agent 与 provider

| ID | 状态 | 当前差异 | 必须重构 |
|---|---|---|---|
| RT-AI-P1-001 | Open | manifest 宣称 client/server/editor host，runtime catalog 仅在可选 feature 链接时可见，editor catalog没有AI provider | 建立 target/profile/provider matrix；required AI 缺 provider 时启动失败并给出 capability receipt，不能静默跳过 |
| RT-AI-P1-002 | Open | 只有 `ai.perception_source`/`receiver` 组件，没有 Brain/Agent/BehaviorTree/Blackboard typed component | 由 Scene/ECS component 投影 `AiAgentHandle`、program/schema reference、enable/disable 与 owner lifecycle |
| RT-AI-P1-003 | Open | `register_behavior_tree`/`register_blackboard_schema` 主要由 API 和测试调用，未发现 asset-to-manager production caller | 建立 load -> validate -> compile -> cache -> register -> activate -> retire 的唯一 service，并返回 generation receipt |
| RT-AI-P1-004 | Open | `.btree.toml` compiler API存在，但没有 importer/cook artifact consumer；示例场景只保存字符串 tree id | 版本化 source asset、依赖图、cook artifact、stable asset handle 和 scene migration；字符串只能是迁移输入 |
| RT-AI-P1-005 | Open | `AiRuntimePlugin` 保存 `Arc<DefaultAiManager>`，但没有 world close/entity despawn/agent disable 的 teardown API | 建立 per-World `AiWorld`、agent registry、despawn/scene unload/PIE stop 幂等清理与 stale generation 拒绝 |
| RT-AI-P1-006 | Open | native dist 是 stateless metadata shell，不能 save/restore/unload 运行态 | 明确 source/runtime/native 三种 carrier parity；若 dist只做代理，manifest必须标为 metadata-only并禁止独立能力声明 |

### 4.2 Manager、identity、并发与执行预算

| ID | 状态 | 当前差异 | 必须重构 |
|---|---|---|---|
| RT-AI-P1-007 | Open | 一个大 Mutex 承载所有 World、agent、tree、schema、perception、debug，跨 World 互相阻塞 | 分离 immutable catalog、per-World scheduler、per-agent state、bounded ingress；锁顺序和 contention budget写入合同 |
| RT-AI-P1-008 | Open | Entity/World/tree/schema/agent ID 仍是裸 u64 或 tuple；无 generation/epoch | 引入 world-qualified stable handles、asset/program generation、entity retirement；跨 World、复用和 stale reference 必须 fail-close |
| RT-AI-P1-009 | Open | tick 从 map remove 状态、锁外 evaluate、再 insert；同 agent 并发 tick 可能覆盖 instance/blackboard/report | 为 agent 提供 single-flight lease 或 mailbox；重复 tick返回 Busy/Coalesced receipt，禁止最后写入者覆盖 |
| RT-AI-P1-010 | Open | 每个 tick 都从所有 compiled tree 收集 implementation slots 并取得所有 owner lease，而非只针对目标 tree | 编译 artifact 持 owner set；按目标 program generation获取最小 lease，支持 hot reload 的 active pin |
| RT-AI-P1-011 | Open | executor使用递归 `evaluate_node` 与可增长 `Vec<String>` stack，无深度/节点/time/alloc/fuel上限 | 改显式 execution stack + admission limits + per-tick budget；超限产生可定位 partial/timeout/fault terminal，不 panic或无限递归 |
| RT-AI-P1-012 | Open | LOD 只由 active camera 距离驱动，headless无 camera 默认 Full；pending delta 没有最大积累/substep政策 | 使用 simulation significance/authority policy，和 camera 解耦；规定 max accumulated delta、substep、replay/server determinism |
| RT-AI-P1-013 | Open | 公共 `AiManager` 是同步 CRUD/tick/full snapshot，不支持 cancellation、async completion、world shutdown 或 fault state | 收敛 `AiRuntimeService` 生命周期 API：register/activate/tick/cancel/stop/reload/query/debug，全部带 generation、receipt、deadline |

### 4.3 Behavior Tree 语义与 gameplay side effect

| ID | 状态 | 当前差异 | 必须重构 |
|---|---|---|---|
| RT-AI-P1-014 | Open | Node context没有 World/command sink/typed task broker；descriptor无参数 schema、pins、side-effect、线程/确定性 metadata | 定义 versioned node schema：输入输出 pin、默认/范围、Blackboard read/write、resource/interface dependency、thread/latent/abort/restart/determinism/debug policy |
| RT-AI-P1-015 | Open | SetBlackboard、EmitEvent、UpdateBlackboardDistance通过 `result`/`service_result` 读取结果，未产生承诺中的写入或事件副作用 | 节点只能通过 typed Blackboard transaction/event command sink产生 effect receipt；删除 result 参数绕过，service拥有 interval/phase/deactivate语义 |
| RT-AI-P1-016 | Open | Wait可被静态 result绕过，缺 clock domain、pause/time scale、wake deadline | 使用 runtime clock ticket 与 wake queue；记录 pause/step/replay/save 语义和超时原因 |
| RT-AI-P1-017 | Open | Parallel/selector abort 基础存在，但完成分支时没有统一 cancel/ack barrier，不能证明所有 sibling latent work 已停止 | 明确 parallel child lifecycle、completion threshold、background policy；terminal transition按确定顺序 abort/cancel并等待 bounded ack |
| RT-AI-P1-018 | Open | MoveTo通过动态 JSON property、event store presence和 destination float equality 猜测请求；clear target写当前位置 | 使用 NavigationTaskBroker request/generation、acceptance radius、filter、repath、partial/no-path/stuck outcome；clear必须是真实 optional removal |
| RT-AI-P1-019 | Open | PlayAnimation 触发一次后立即 Succeeded，没等待 montage/state/graph completion、cancel或 reload generation | 使用 AnimationTaskBroker ticket、notify/finish/interrupt、resource generation、owner affinity和 stop ack |
| RT-AI-P1-020 | Open | ScriptTask同步调用 VM；若返回 Running，下一帧仍可能重复调用；无 fuel/deadline/cancel/reload fence | 让脚本返回 durable continuation/ticket，调用受 owner lease、fuel、wall/memory budget 和 cancellation 约束，late completion 必须被拒绝 |
| RT-AI-P1-021 | Open | subtree/abort/debug只保留 active node 和字符串 diagnostic，无法重建 search、condition、abort、task transition | 输出 versioned execution trace：program generation、instance path、node enter/exit、condition value、abort reason、task ticket、timing 和 drop receipt |

### 4.4 Blackboard、Perception 与 debug

| ID | 状态 | 当前差异 | 必须重构 |
|---|---|---|---|
| RT-AI-P1-022 | Open | schema key/value type仍主要是字符串，dense store steady path在边界上仍用完整 Vec entries；值域仅六类，无 default/inheritance/object/resource/weak entity | compiler生成 stable key handle/slot/type token；增加 nullable/entity generation、asset/object/tag/enum、default/inheritance与 migration；普通 tick只能做 typed delta |
| RT-AI-P1-023 | Open | Blackboard set/get没有 writer identity、authority、replication/save/replay/debug policy，也没有多 writer conflict transaction | 统一 phase/priority/conflict policy 的 batch mutation；observer只见完整 revision，记录 node/task/script provenance |
| RT-AI-P1-024 | Open | Perception receiver×source 全笛卡尔积，固定 256 pair 只限制消费数；没有 spatial index、time/query/byte/alloc budget和最大感知延迟 | 维护 source/listener registry 与 spatial cells；按 importance/channel/range生成候选，预算 time/pair/query/bytes并报告 oldest latency/truncation |
| RT-AI-P1-025 | Open | Sight provider 为 None 时 fail-open 刷新可见，物理桥接失败会穿墙；无显式 unknown/degraded policy | dependency admission必须定义 required/degrade/fail-close；返回 `Unknown/Unsupported` 与诊断，不把缺依赖当成功 |
| RT-AI-P1-026 | Open | components只含 Sight/Hearing 基本参数；没有 team/affiliation/tag/socket/offset、dominant sense、lost/expired reason、source generation | 引入 versioned SenseRegistry/SenseConfig/Stimulus：成功与丢失转移、confidence、age、affiliation、source/listener generation、Damage/Touch/Custom provider |
| RT-AI-P1-027 | Open | 感知 snapshot 全量 clone；行为系统每 tick 发送 full debug frame，即使没有 Editor reader；full map projection HashSet 顺序不定 | 订阅者启用后才投影；使用 bounded ring/delta、items/bytes/age/sample budget、reader cursor/resync，所有事件有 world/program/entity generation |
| RT-AI-P1-028 | Open | runtime contract 只有最后 report/active_node，node event最多一条；没有 trace profiler、budget overrun、drop/backpressure和 replay capture | 建立 runtime AI telemetry provider，与 Editor mirror共享 schema、cursor、loss receipt、sampling与 privacy policy |

## 5. P2：竞争性能力

| ID | 目标 | 前置条件 |
|---|---|---|
| RT-AI-P2-001 | StateTree/data-oriented execution | P1 execution stack、artifact generation、typed memory 完成后批量 transition |
| RT-AI-P2-002 | Utility AI/HTN 与可解释 plan repair | Blackboard/EQS/trace 共享，不再复制 decision authority |
| RT-AI-P2-003 | EQS source/compiler/generator/test/score/query cache | query ticket、time slicing、candidate budget、Editor debugger 全闭合 |
| RT-AI-P2-004 | Smart Object definition/slot claim/use | stable world identity、claim ticket、contention、navigation/animation handshake |
| RT-AI-P2-005 | Mass/crowd/large-agent significance | spatial index、server authority、representation swap、批量 scheduler |
| RT-AI-P2-006 | squad/faction shared knowledge | affiliation/privacy/replication/save schema先稳定 |
| RT-AI-P2-007 | network prediction/replay/savegame | deterministic clock/RNG、ticket serialization、generation migration |
| RT-AI-P2-008 | AI quality and scale corpus | 1K/10K agent、多 World、深树、latent、multi-sense、fault/soak、cross-platform evidence |

## 6. 参考引擎对照边界

| 参考 | 本轮采用的工程约束 | Zircon 当前差距 |
|---|---|---|
| Unreal `BehaviorTreeComponent` | instance stack、pause/restart/stop、latent task message、node memory、branch abort 与调度生命周期 | Zircon有递归 evaluator 和部分 abort，但没有统一 instance/task/message/cancel barrier |
| Unreal `BlackboardComponent` | typed key memory、observer registration、parent/default 数据与销毁通知 | Zircon有 dense store/observer，但公共 API仍是 string/Vec，缺 schema migration、authority 和 teardown |
| Unreal `AIPerceptionSystem`/`AISense_Sight` | listener/source/sense lifecycle、aging/forgetting、affiliation、可配置 query 与 provider | Zircon只做两种 sense、全量采样、Cartesian budget，LOS 失败还 fail-open |
| Unreal `EnvQueryManager` | query id、querier cleanup、abort、cache、time slicing、item/test cost | Zircon没有生产 EQS/query domain |
| Unreal BT/EQS Editor | 独立 asset/schema/factory、graph transaction、compile log、PIE debugger/profiler | Zircon只在 editor crate登记 descriptor；详见 Editor234 |
| Unreal StateTree | execution context、instance data、compiled bindings、async execution context | Zircon只把 StateTree列为未来 P2，没有 runtime/source实现 |
| Fyrox behavior | serializable tree、Pool/Handle、可复用 context 与组合节点 | 可借鉴稳定 handle/serialization；不能把其轻量递归模型当作性能上限 |
| Godot NavigationAgent | typed navigation target、path/velocity/avoidance lifecycle | Zircon MoveTo直接写 dynamic JSON并读取历史 event，缺 request/ack/cancel |
| Bevy tasks | 明确 task pool、scope/ownership、可控执行边界 | Zircon AI manager 不接统一 task scheduler/fuel/deadline |
| Unity Graphics DebugManager | provider/panel register/unregister/reset 与 bounded debug ownership | Zircon debug snapshot无 reader gate、trace budget 和 provider lifecycle |

## 7. 分层重构路线

### M0 · Truth、provider 与 fail-close

冻结 manifest/descriptor/catalog/App target matrix；将 perception `complete` 改为与证据相符的 partial；明确 NativeDynamic metadata-only；删掉 silent missing provider；为所有输入、LOS、script、navigation failure 定义 explicit diagnostic。

### M1 · World/Agent/Asset authority

建立 `AiAssetSource -> AiCompiledProgram -> AiWorld -> AiAgentInstance`，scene component 绑定 program/schema generation；world/entity despawn、PIE stop、plugin unload、asset reload 共用 teardown/last-good 状态机。

### M2 · Budgeted executor 与真实 task

保留 dense compiled tree/Arc generation，迁移显式 stack、node/time/fuel/alloc budget；以 typed command sink、Blackboard transaction、event bus 和 task broker替换 `result`/`service_result`。Navigation、Animation、Script 都必须返回 generation-qualified ticket并有 cancel/ack。

### M3 · Blackboard 与 Perception

引入 stable key slots、schema version/default/inheritance/migration、writer policy；source/listener lifecycle 接入 spatial index 与候选缓存，统一 sense/event ingress、time/bytes/query budget 和 fail-close LOS。

### M4 · Debug、Replay 与 Editor closure

runtime trace、snapshot delta、cursor/resync、loss receipt、profiler和replay输入进入同一 schema；Editor234 的 asset graph/provider/PIE overlay 消费同一 generation，禁止静态文本冒充 runtime 状态。

### M5 · 竞争性 domain 与资格

在 P1 关闭后再实现 EQS、StateTree、Smart Object、Mass AI、shared knowledge；建立 deep-tree、1K/10K agent、multi-World、server/headless、reload、fault、save/network/replay 与 cross-platform 资格档案。没有同负载证据，不得声称性能超过 Unreal。

## 8. 资格门

| Gate | 必须证明 |
|---|---|
| RT-AI-G01 | client/server/editor target 对 AI provider、feature、capability 一致，缺失显式失败 |
| RT-AI-G02 | source asset 可 import、依赖可解析、cook 生成 immutable artifact 与 digest |
| RT-AI-G03 | scene instantiate 自动激活 Agent，despawn/World close/PIE stop 幂等清理 |
| RT-AI-G04 | program/schema/entity/world generation 全部 stale-safe、cross-world fail-close |
| RT-AI-G05 | manager 不以单 global lock 串行无关 World，single-flight agent tick 有证明 |
| RT-AI-G06 | executor 有 depth/node/time/fuel/alloc budget，超限有 terminal receipt |
| RT-AI-G07 | selector/parallel/abort 对每个 active latent task 都有 cancel/ack barrier |
| RT-AI-G08 | SetBlackboard/EmitEvent/Service 产生真实 typed effect，不接受 result 伪成功 |
| RT-AI-G09 | Wait/Navigation/Animation/Script 使用 clock/task broker，支持 timeout/cancel/reload |
| RT-AI-G10 | Blackboard schema、slot、default、migration、writer/replication/save policy 可回放 |
| RT-AI-G11 | Perception source/listener 增量注册，spatial candidate 与公平/最大延迟可量化 |
| RT-AI-G12 | 所有 budget 同时限制 pair/query/time/bytes/alloc，丢弃可观察 |
| RT-AI-G13 | LOS provider 缺失或失败不会刷新为 visible，unknown/degraded 可诊断 |
| RT-AI-G14 | Sense 配置包含 affiliation、age、success/lost/expired reason 和 generation |
| RT-AI-G15 | LOD 与 camera 解耦，server/replay/headless 与 client 同一 schedule policy |
| RT-AI-G16 | debug 只有订阅才启用，trace/snapshot bounded，cursor/resync/loss 可验证 |
| RT-AI-G17 | runtime、native dist、Editor mirror 使用同一 event/schema/generation contract |
| RT-AI-G18 | plugin revoke、asset reload、script reload、World teardown 无 dangling task/instance |
| RT-AI-G19 | EQS/StateTree/Smart Object 只有在真实 source/compiler/runtime/editor 具备后才标 partial/complete |
| RT-AI-G20 | 1K/10K agent、深树、latent、multi-sense、fault/soak、server 与 cross-platform 有可重复 receipt |

## 9. 明确不接受的修复

1. 不以增加几个 enum、标准节点名字、静态 Workbench 文案或 `result` 参数关闭行为语义。
2. 不把一次 `Arc` clone 优化、一次 records pass 或 217 个测试属性写成产品链完成。
3. 不通过扩大 256 pair、pending delta 或全量 snapshot 容量掩盖缺 spatial、time、bytes、reader 和 lifecycle。
4. 不在 AI package 内复制 Scene、Navigation、Animation、Script、Asset、Editor、Network 或 Save authority。
5. 不在没有同负载、同平台和 fault/reload 证据时声称优于 Unreal。

## 10. 状态

本轮只新增本报告、后续索引与 coverage 记录；没有修改 Runtime、Editor、Cargo、ABI、fixture、测试或 tooling。Runtime174 的 28 项 P1、8 项 P2、20 个资格门均为 Open/Fail 的重构计划，实施前必须重新冻结 source fingerprint、BuildSet 和目标平台矩阵。Editor 侧对应问题由 [Editor234](../zircon_editor/234-editor-ai-current-working-tree-authoring-graph-debug-overlay-workbench-review.md) 负责，不能用本报告代替其 UI/authoring owner。
