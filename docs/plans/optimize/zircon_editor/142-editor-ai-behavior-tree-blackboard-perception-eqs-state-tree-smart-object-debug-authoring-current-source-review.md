---
title: Editor AI / Behavior Tree / Blackboard / Perception / EQS / StateTree / Smart Object / Debug Authoring 当前源码复审
category: zircon_editor
report_id: Editor142
review_date: 2026-08-26
baseline_head: b41b0c0b9da31eb4d19e3f086d6027f745f11a38
verification_head: 601472078e848164d2221967c55a77fea2452928
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/89-editor-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-debug-authoring-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/100-runtime-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-task-navigation-network-save-scalability-editor-product-integration-current-source-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/zircon_plugins/06-ai.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
related_code:
  - zircon_plugins/ai/editor
  - zircon_plugins/ai/runtime
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/scene/viewport
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_runtime/src/core/framework/ai
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor
  - dev/UnrealEngine/Engine/Plugins/AI/EnvironmentQueryEditor
  - dev/UnrealEngine/Engine/Source/Editor/GameplayDebugger
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeEditorModule
  - dev/UnrealEngine/Engine/Plugins/Runtime/SmartObjects/Source/SmartObjectsEditorModule
  - dev/Fyrox/fyrox-impl/src/utils/behavior
  - dev/Fyrox/editor/src/scene/commands/graph.rs
  - dev/godot/scene/gui/graph_edit.cpp
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/editor/debugger/editor_debugger_node.cpp
  - dev/bevy/crates/bevy_asset/src
  - dev/Graphics/Packages/com.unity.shadergraph/Editor
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor142 · AI Authoring 与 Runtime Debug 当前源码复审

## 1. 当前结论

当前物理工作树中的 AI 并非空壳，也不能判定为工程级产品。Runtime 已有 dense compiled behavior tree、18 项标准节点目录、typed dense Blackboard、slot generation、changed-slot observer、owner execution gate、subtree abort、Navigation/Animation/Script 集成 host、受限 hearing backlog、pair cursor、可选 physics sight query、行为 tick 与 debug event。Editor 也已有 asset/import/toolkit、graph/palette、operation descriptor、session/sequence/World mirror、基础 Perception overlay geometry。这些是应保留并继续工程化的底座。

但从项目选择到真实作者化、编译、场景激活、运行和调试的产品链仍然断裂。AI Editor 继续导入仓内已不存在的 `ViewportToolModeDescriptor`，并调用不存在的 `register_viewport_tool_mode`；first-party Editor catalog 仍只有 Navigation 与 Neural，App 又只委托该 catalog。五个公开 AI operation 没有任何 `OperationCommandFactory`，Behavior Tree 与 Perception ZUI 没有业务 provider/controller，默认 Workbench 仍显示固定的 `BT_Enemy`、`AI_Guard_01` 和固定成功反馈。

Runtime 当前已经安装 `ai.perception_tick` 与 `ai.behavior_tick`，但生产源码没有调用 `register_behavior_tree`、`register_blackboard_schema`，也没有首次 `AiAgentTickRequest` 将场景实体激活为 agent。换言之，它现在是“有调度循环、无产品入口”。Vampire 示例仍以通用 Data TOML、场景字符串和脚本镜像形成第二套决策真值；Editor 也没有消费 Runtime compiler 或 generation-qualified artifact。

高级域仍是物理缺失：AI package、framework、Editor 和 App 的生产源码中没有 EQS、StateTree 或 Smart Object 实现。manifest 将 Perception 标成 `complete` 也不成立：当前只有 Sight/Hearing，steady tick 仍收集全 World 节点并遍历 receiver×source；physics query 缺失或失败时 Sight 选择 fail-open 可见；行为 LOD 由 active camera 距离驱动，无法作为 server/headless 或多视口权威语义。

因此 Editor20 的 5 项 canonical P0 仍为 **5 Open / 0 Partial / 0 Closed**；Editor89 的 60 项 P1 当前仍为 **48 Open / 12 Partial**，12 项 P2 仍全部 Open；32 项验收门为 **26 Fail / 6 Partial**。当前目标架构必须保持单一真值链：

```text
versioned AI source assets
  -> transactional authoring documents with stable identities
  -> shared deterministic semantic compiler
  -> atomic AI build-set publication + diagnostics/source map
  -> per-World Runtime execution + task/query scheduler
  -> generation-qualified bounded debug journal
  -> real graph/blackboard/query/overlay product controllers
```

本报告只做 review 和重构规划。MVP 总计划仍在进行且 F0-F5 被阻塞，AI 属于高级功能；本轮没有实施源码、没有运行 Cargo，也没有查询、轮询或等待协调器状态。

## 2. 物理范围、currentness 与证据等级

### 2.1 冻结范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 当前证据 |
|---|---:|---|
| AI Editor package | **12 / 2,396 / 2,223 / 83,869 / 18 / 4** | Cargo、两份 ZUI、registration、mirror、overlay 与 allocation tests |
| AI Runtime package | **84 / 21,634 / 19,893 / 777,751 / 217 / 42** | compiler、catalog、executor、Blackboard、manager、Perception、plugin registration 与 tests |
| neutral AI framework | **9 / 1,040 / 956 / 31,239 / 0 / 0** | manager、descriptor、tick、snapshot、event、ID 与 error contract |
| 产品入口选择集 | **7 / 1,130 / 1,019 / 51,053 / 2 / 0** | manifest、Editor catalog/App composition、两份 Workbench 与固定反馈 |
| 选定参考实现 | **24 / 27,267 / 23,255 / 975,902 / 2 / 0** | Unreal 五个 AI Editor 域及 Fyrox/Godot/Bevy/Unity Graphics 补充边界 |

统计读取当前物理文件，包括已修改与 untracked 测试，不以 Git index 覆盖工作树。AI Editor 数量与 Editor89 冻结一致；AI Runtime 当前为 84 文件、21,634 行，反映 executor、abort、Blackboard、Perception 与 allocation evidence 的在途增量。

### 2.2 Currentness

冻结基线为 `b41b0c0b9da31eb4d19e3f086d6027f745f11a38`。选定范围内存在用户或其他 Session 的 tracked 修改和多组 untracked allocation/topology test，本轮全部保留，没有回退、格式化或覆盖。实施前必须重新冻结 AI Editor/Runtime、catalog/App、Workbench、manifest、owner 计划和 HEAD；本报告设置 `source_recheck_required: true`。

当前 in-flight 修改包含真实性能与生命周期改进，但不能用文件数或 benchmark test 声明产品关闭。尤其是 release-only tests 处于 `#[ignore]`，本轮没有执行它们，也没有验证当前旧 Editor API 是否导致 plugin workspace 编译失败。

### 2.3 证据边界

本轮证据为静态当前源码审查：逐文件读取 AI Editor、AI Runtime 核心路径、framework contract、产品装配、Workbench 与选定参考源码。没有运行 Editor/App、PIE、import/cook、save/reopen、runtime trace、overlay、EQS、StateTree、Smart Object、fault/scale/soak/profile 或跨引擎 benchmark。

现有 unit/scenario/allocation test 只能证明局部结构。Closed/Pass 必须同时具备 current source、真实产品入口、成功与失败路径、bounded scale、lifecycle teardown 和 old-authority deletion 证据。

## 3. 当前实现事实

### 3.1 Package、catalog 与 App 选择

1. `plugin.toml` 声明 Client/Server/Editor Host、BT/Blackboard/Perception 与 AI Editor module，maturity 为 experimental；Perception 的 `complete` 与当前语义不符。
2. `first_party_editor_catalog/Cargo.toml` 和 `catalog.rs` 仍只提供 Navigation/Neural feature、dependency 与 registration；AI 不在默认 Editor 选择链。
3. `zircon_app/src/entry/first_party_editor_plugins.rs` 只委托该 catalog，没有 AI 第二入口，也不应新增第二入口掩盖 catalog 缺口。
4. AI Editor registration 具备 Behavior Tree asset/import/toolkit、graph editor、18-node palette、四个 BT command、Perception Debug surface、Toggle command 与三类 runtime event consumer。
5. 当前 registration 仍使用已删除的 viewport tool API；即使把 crate 加进 catalog，静态合同漂移也必须先硬切到 `SceneModeRegistration`、factory 与 provider。

### 3.2 Source、document、graph 与 operation

1. Behavior Tree 只有 descriptor；没有 versioned source DTO、unknown-field policy、migration、source revision、typed dependency graph 或真实 importer byte parser。
2. Blackboard、Perception Config、EQS、StateTree、Smart Object 没有独立 source kind、factory、toolkit、cook role 或 migration owner。
3. Behavior Tree ZUI 仍以无 provider Table 表示 palette/Blackboard，以两个业务 `Space` 表示 graph 与 inspector，且没有事件绑定；Perception ZUI 只有无 provider agent Table。
4. Import/Open/Validate/Compile/Toggle Overlay 五个 operation 只有 descriptor。AI package 对 `OperationCommandFactory` 为零注册，host 会确定返回 `MissingFactory`。
5. shared transaction、operation factory、job、scene mode 与 viewport provider 是可复用基础；AI 没有把它们接成 dirty/save/undo/redo/compile/publish/controller 生命周期。
6. palette 来源于 Runtime 标准节点目录是正确方向，但当前只有 ID/display/category，没有 authoring parameter schema、pin/attachment rule、owner generation 或第三方 node reload contract。

### 3.3 Runtime compiler、executor 与产品激活

1. compiler 产出前序 dense node、parent/subtree range 与 implementation slot，并通过 immutable `Arc<[CompiledBehaviorTree]>` generation 共享；这是应保留的结构。
2. `compile_subtree` 与 executor 的 `evaluate_node` 仍递归，没有 depth/node/time/bytes/reentrancy budget。Parallel、TimeLimit、Loop 等节点尚未形成完整 latent child lifecycle。
3. abort 路径已有按 compiled node index 排序的 observer request、parent lookup、integration task abort 与 subtree target abort，是比旧审查更具体的进展。
4. `MoveTo` 写 typed Navigation destination 并消费 arrived/no-path report；`PlayAnimation` 写 player parameter；`ScriptTask` 通过 weak bridge 调 VM。三者仍缺统一 task ticket、completion generation、cancel acknowledgement、timeout 和 reload fencing。
5. `SetBlackboard`、`EmitEvent` 仍可落到静态 `evaluate_task` 结果；Service 仍可由 `service_result` 决定结果，不能视作真实 effect/service lifecycle。
6. plugin registration 确实安装 Perception-before-Behavior 的 Update systems，并在行为 tick 中构造 integration host 与 debug snapshot。
7. 生产源码没有调用 tree/schema registration，也没有首次 agent tick 激活场景实体；后续 `tick_active_agents` 只会遍历已经存在的 `active_behavior_trees`。因此 scene-to-runtime activation 仍断裂。

### 3.4 Blackboard、World、并发与 Perception

1. Blackboard 已有 typed dense layout、按类型分区 storage、slot generation、deterministic entry cache、changed-slot queue 与 observer lookup；这些属于 Partial 基础。
2. 对外仍以完整 `Vec<AiBlackboardEntry>` snapshot 同步；没有 schema version/migration/inheritance/redirect/default、writer provenance、network/save/replay policy 或 transactional batch receipt。
3. 单个 `Arc<Mutex<AiRuntimeState>>` 承载所有 World。tick 会在锁内移除 agent Blackboard/instance，锁外执行后重新插入；同 agent 并发 tick 缺 execution ownership，可出现双跑与 last-writer-wins。
4. agent identity 仍是 `(WorldHandle, EntityId)` 裸整数，没有 entity generation、Brain owner、artifact generation 或 world teardown state machine。
5. Perception 每 tick 通过 `world.node_records()` 收集 receiver/source，再以全局 256 pair budget 和 round-robin cursor 扫描笛卡尔积；只有 item 上限，没有 time/physics/bytes/latency SLO。
6. hearing backlog 有 1,024 容量、age 与 ingest limit，是实际 bounded ingress；Sight/静态 Hearing 与 debug stream 尚未统一到同一 scheduler/backpressure contract。
7. physics provider 缺失、禁用、reload 或调用错误时 `is_occluded` 返回 `None`，当前分支会刷新 Sight 为可见。这是显式 fail-open，不是工程级 degraded truth。
8. 行为 LOD 由 active camera 到 agent 距离决定；headless/server、split-screen、多 viewport、replay 和 authority simulation 没有稳定输入 owner。

### 3.5 Runtime debug mirror 与 overlay

1. Runtime 每个 agent report 只有一个 `active_node` 和最终 status；每 tick 至多形成一条 node result，不能表达 active path、parallel additional nodes、decorator abort、service/task transition 或因果时间线。
2. Runtime debug snapshot 每帧为所有 active agent 克隆 tree、Blackboard、Perception 和 Perception debug 数据，没有 reader subscription、field mask、backpressure 或 gap receipt。
3. `AiPieMirror` 有 play-session、sequence、World fence和 snapshot replacement；`AiBtNodeResultMirror` 有按 World/Entity/node 分层与 borrowed lookup。这些仍缺 entity/program/schema/source generation 与 bounded history。
4. snapshot pruner 只保留 `frame.report.active_node`，会丢失并行/附加节点；mirror 没有 breakpoint、step、rewind、instance selector 或 source-map jump。
5. overlay 能绘制 agent sphere、sight cone、hearing circle 和 stimulus，过滤非有限输入并预分配；但容量估算与构建双扫描，没有 per-view filter/culling/LOD、primitive/bytes/time budget、overflow quality 或 cache。
6. overlay controller 没有注册为真实 viewport provider；catalog 返回 registration report 后也没有把 plugin-owned mirror state 暴露给产品 controller。

### 3.6 Workbench、示例与高级 AI 域

1. 两份默认 AI Workbench 共 414 行、48 个 control/node、26 个 event table 和 32 条 route，但数据来自固定 ZUI/property 文本与 match 分支。
2. 固定反馈包括 `BT_Enemy sample persisted`、`selector branch is reachable` 与 `AI_Guard_01 simulation tick 00:12.4`；它们不读取 asset/compiler/runtime receipt。
3. Vampire 示例的行为树仍是通用 Data TOML，与当前 descriptor schema 不兼容；Scene 保存字符串，脚本复制决策过程，形成多 authority。
4. 当前生产源码对 EQS、StateTree、Smart Object 为零实现；不能用 Workbench 文案、palette 节点或 manifest capability 代替 domain source/compiler/runtime/editor。
5. 现有测试覆盖 descriptor、mirror fence、overlay geometry、dense Blackboard、observer、abort、integration 和 Perception 局部行为；没有 App catalog、asset-to-scene activation、PIE authoring/debug、network/save/replay、1K agent 或长期 soak。

## 4. 参考引擎差异与采用边界

| 参考 | 当前核对的工程事实 | Zircon 必须吸收的边界 |
|---|---|---|
| Unreal BehaviorTreeEditor | 独立 BT/Blackboard factory、asset definition、graph/schema/subnode、Undo/Redo、Save/UpdateAsset、Find/Diff；debugger 管理 PIE begin/end/pause、instance、active path、breakpoint、step in/over/out/back 与 subtree 切换 | AI authoring/debug 主参考；吸收职责、状态机和 source/runtime mapping，不复制 UObject/Slate 实现 |
| Unreal EnvironmentQueryEditor | 独立 factory、graph/schema、option/generator/test、`UpdateAsset`、Profiler tab、stats load/save 与 graph overlay | EQS 必须有 query artifact、run identity、item score、失败原因、time-sliced runtime 与 profiler，不能用静态预览文字替代 |
| Unreal StateTree Editor | 独立 editor data/schema/view model、property binding compiler、compiler manager/log、compile-all commandlet、async diff、outliner/events view 与 standalone host | StateTree 是独立 source/compiler/toolkit/debug domain，不能塞成 BT palette 的几个节点 |
| Unreal SmartObjects Editor | definition factory、asset editor/toolkit、view model、slot/details、preview scene、viewport hit testing、transactional gizmo editing、component visualizer 与 World Partition builder | Smart Object 拥有 definition/slot/claim/use/collection 生命周期，不得降级为 tag 或 Blackboard key |
| Unreal GameplayDebugger | editor mode/toolkit 与 runtime category/extension 分离，具备 activate/deactivate、input/category、replication 与 runtime projection | 建立可关闭、可授权、可限流、可断线恢复的 debug provider，不让 UI 直接锁 Runtime manager |
| Fyrox | behavior tree 是 serializable Pool/Handle 组合且仍递归；Editor scene graph mutation 使用具名 `CommandTrait` 的 execute/revert | 借鉴 stable handle/serialization 与 reversible command；其轻量 BT 不能降低 Unreal 级 AI 产品门槛 |
| Godot | `EditorUndoRedoManager` 按 history create/commit/undo/redo/save-state；`EditorDebuggerNode` 管 session stop/pause/step/breakpoint、remote tree 与 plugin capture；GraphEdit 提供通用连接 UI | 借鉴 history owner、debug session/plugin 和通用 graph widget；Godot 本地树不是本域的 AI semantic compiler |
| Bevy | `AssetLoader`、`LoadContext`、labeled/nested dependency 与 Added/Modified/Removed/Unused/LoadedWithDependencies typed event | 作为 typed asset identity/lifecycle 下限；Bevy 无同级首方 AI Editor，不能证明 Zircon 的 authoring 缺失合理 |
| Unity Graphics | ShaderGraph 有 persistent `GraphData`、importer、GraphEditorView、BlackboardController action 和 complete-object Undo；DebugManager 有 data/panel register/unregister/reset | 只补共享 graph/blackboard/debug 生命周期；Graphics 不是 AI domain reference，AI 语义仍由 Zircon Runtime 共享 compiler 定义 |

Unreal 是本域主参考，其他四类参考只补共享基础或指出轻量实现边界。Zircon 应在 Rust ownership、immutable generation、per-World isolation、bounded transport、prepared publication 和 server authority 上形成自己的工程合同。

## 5. 父 P0 当前重判

| Canonical ID | 状态 | 当前证据与硬切要求 |
|---|---|---|
| `AI-ED-P0-001` | Open | `ViewportToolModeDescriptor`/`register_viewport_tool_mode` 在当前仓内无定义；改为 `SceneModeRegistration` + 真实 factory/provider，并加入 required compile lane。 |
| `AI-ED-P0-002` | Open | first-party Editor catalog 只有 Navigation/Neural，App 只委托该 catalog；接入 AI feature/dependency/registration/selection 并验证 disable/unload。 |
| `AI-ED-P0-003` | Open | 五个可见 operation 均无 factory，host 确定返回 `MissingFactory`；补 typed payload/factory/receipt，未实现时必须 Unavailable。 |
| `AI-ED-P0-004` | Open | graph/Blackboard/Perception/mirror/overlay 均无产品 controller/provider；descriptor、Table、Space 和 builder 不能计为产品。 |
| `AI-ED-P0-005` | Open | 默认 Workbench 继续固定 AI 数据与成功反馈；先标 sample/Unavailable，最终硬切到真实 toolkit 与 runtime projection。 |

## 6. Canonical P1 重判

| Canonical ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| `AI-ED-P1-001` | Open | Behavior Tree 无 versioned source DTO、schema/unknown policy、stable asset identity 和 migration。 |
| `AI-ED-P1-002` | Partial | 已有 asset type/importer/toolkit descriptor；缺 bytes 解析、source revision、provenance、create/reimport 与真实 toolkit factory。 |
| `AI-ED-P1-003` | Open | Blackboard 无独立 asset type、factory、importer、toolkit、inheritance 和 redirect。 |
| `AI-ED-P1-004` | Open | Perception Config 无 source asset、sense config schema、platform override、validation 和 cook role。 |
| `AI-ED-P1-005` | Open | EQS 无 source asset、query option/generator/context/test schema、factory 和 toolkit。 |
| `AI-ED-P1-006` | Open | StateTree 无 source asset、state/transition/condition/binding schema、factory 和 toolkit。 |
| `AI-ED-P1-007` | Open | Smart Object Definition/Slot 无 source asset、scene reference、factory、toolkit 和 collection artifact。 |
| `AI-ED-P1-008` | Open | AI 各 source 间无 typed dependency graph、cycle policy、redirect、missing/LKG 诊断。 |
| `AI-ED-P1-009` | Open | 无 create template、命名/路径冲突、目录事务与 rollback。 |
| `AI-ED-P1-010` | Open | 无 import/reimport settings、source hash、cancel、dry-run、dependency 与原子提交。 |
| `AI-ED-P1-011` | Partial | GraphEditor/18-node palette descriptor 存在；无 document/controller/canvas 消费，也无 authoring parameter schema。 |
| `AI-ED-P1-012` | Open | 无 stable node/edge/pin/auxiliary/service/decorator identity 和 tombstone/redirect。 |
| `AI-ED-P1-013` | Open | 无 BT root/composite/task/decorator/service/subtree attachment 与连线规则验证。 |
| `AI-ED-P1-014` | Open | palette 不含参数/pin/category version、owner generation、capability 和第三方 reload 语义。 |
| `AI-ED-P1-015` | Open | 无 graph selection、marquee、move/connect/delete、focus、multi-select 和 keyboard routing。 |
| `AI-ED-P1-016` | Open | 无 copy/paste/duplicate、stable remap、external reference repair 与跨文档策略。 |
| `AI-ED-P1-017` | Open | 无 Find、semantic search、jump-to-node、reference search 和 diagnostic navigation。 |
| `AI-ED-P1-018` | Open | 无 source/revision diff、node correspondence、conflict projection 和 merge policy。 |
| `AI-ED-P1-019` | Open | Blackboard 无 key type/default/category/inheritance/override/rename-impact 编辑。 |
| `AI-ED-P1-020` | Open | Inspector 仍是 `Space`，无 typed property、conditional field、multi-edit、unit 和 validation。 |
| `AI-ED-P1-021` | Partial | shared transaction/history/journal 基础存在；AI 没有 command、inverse delta、merge scope 或 dirty wiring。 |
| `AI-ED-P1-022` | Open | 无 save token、atomic write、source CAS、external-change conflict 和 last-known-good。 |
| `AI-ED-P1-023` | Open | 无 autosave/recovery/session lock/readonly 与 crash reopen。 |
| `AI-ED-P1-024` | Partial | shared operation factory 及 `MissingFactory` 失败路径真实存在；AI 五个 operation 没有 factory/payload/receipt。 |
| `AI-ED-P1-025` | Open | Validate 没有调用共享 Runtime semantic compiler，也没有 source-bound diagnostics。 |
| `AI-ED-P1-026` | Open | Compile 没有 prepared build set、dependency digest、source map 或 deterministic bytes。 |
| `AI-ED-P1-027` | Open | 多 BT/BB/Perception/EQS 依赖不能形成单一 atomic AI build-set publication。 |
| `AI-ED-P1-028` | Partial | shared background job/admission 基础存在；AI 无 cancel/progress/deadline/owner drain/commit executor。 |
| `AI-ED-P1-029` | Open | Editor preview、PIE、cook/shipping 未证明使用同一 compiler/artifact。 |
| `AI-ED-P1-030` | Open | 无 compile generation/LKG/install acknowledgement/currentness 和 stale result discard。 |
| `AI-ED-P1-031` | Partial | typed runtime event consumer、manifest 与 host session 基础存在；未接产品 reader/controller。 |
| `AI-ED-P1-032` | Partial | mirror 具备 session/sequence/World 隔离与 snapshot replacement；identity/generation/currentness 仍不足。 |
| `AI-ED-P1-033` | Open | BtNodeResult 每 tick 唯一 active node，不能表示 active path、parallel nodes 和完整 node lifecycle。 |
| `AI-ED-P1-034` | Open | 无 decorator observer/abort range、service/task begin/end/cancel/fault trace。 |
| `AI-ED-P1-035` | Open | 无 Blackboard delta/watch、value provenance、timestamp、schema generation 和敏感值策略。 |
| `AI-ED-P1-036` | Open | 无 bounded trace journal、gap/overflow receipt、retention tier、reader lease 和 slow-reader policy。 |
| `AI-ED-P1-037` | Open | 无 PIE instance selector、pause/step/step-back、历史游标和 session 切换状态机。 |
| `AI-ED-P1-038` | Open | 无 breakpoint create/remove/enable/disable、condition、hit count 和 runtime acknowledgement。 |
| `AI-ED-P1-039` | Open | 无 program/schema/source revision 到 graph node 的 generation-qualified source map。 |
| `AI-ED-P1-040` | Partial | node mirror 按 World/Entity 分层并支持无分配 borrowed lookup；producer/prune 仍只保留单 active node。 |
| `AI-ED-P1-041` | Open | catalog 返回 registration 后无稳定 debug state access handle，plugin 实例 getter 无法被产品 controller 使用。 |
| `AI-ED-P1-042` | Open | 无远端/多进程 agent debug、capability 授权、privacy/redaction、disconnect/reconnect 和 clock calibration。 |
| `AI-ED-P1-043` | Partial | overlay 有 finite 过滤、精确预分配和基础 FOV/hearing/stimulus 几何；仍双扫描且无预算/裁剪/quality。 |
| `AI-ED-P1-044` | Partial | shared scene mode factory 与 viewport provider lifecycle 存在；AI 仍调用旧 API 且未注册 provider。 |
| `AI-ED-P1-045` | Open | overlay 无 selected agent/sense/team/affiliation/filter、show flag 和 per-view state。 |
| `AI-ED-P1-046` | Open | overlay 无 frustum/distance/LOD、primitive/bytes/time budget、overflow 和 cache reuse。 |
| `AI-ED-P1-047` | Open | Perception Debug 无 agent 列表 provider、selection、stimulus history、forget/age/quality/failure 原因。 |
| `AI-ED-P1-048` | Open | EQS 无 run request、preview world、item shape/score、failed test 解释和 query profiler。 |
| `AI-ED-P1-049` | Open | StateTree 无 hierarchy/transition/condition/binding editor、compiler log、simulation/diff/debugger。 |
| `AI-ED-P1-050` | Open | Smart Object 无 slot/details/viewport visualizer、claim/use runtime projection 和 World collection workflow。 |
| `AI-ED-P1-051` | Open | 无 AI preview sandbox 的 isolated World、deterministic seed/time/input、reset 与 mutation boundary。 |
| `AI-ED-P1-052` | Open | 无 Navigation task/query 关联、path/cost/result/abort trace 和 Nav Editor 跳转。 |
| `AI-ED-P1-053` | Open | 无 Animation/Script/Gameplay task handle 状态、跨域 diagnostic 与 owner revoke 可视化。 |
| `AI-ED-P1-054` | Open | 无 network authority、server/client observation、prediction/replay/save currentness UI。 |
| `AI-ED-P1-055` | Partial | manifest 声明 capability/event/editor artifact；默认 catalog、产品 lifecycle 和 acceptance 尚未兑现。 |
| `AI-ED-P1-056` | Open | plugin manager active snapshot 遗漏 factory/provider 等贡献，reload/read-model 与直接注册路径不一致。 |
| `AI-ED-P1-057` | Open | Workbench 两份 AI workspace 和 generated bottom panel 仍是独立静态 authority，未硬切真实 toolkit。 |
| `AI-ED-P1-058` | Partial | focused tests 覆盖 descriptor、mirror、world/stale、geometry 和 allocation；缺产品/故障/规模矩阵。 |
| `AI-ED-P1-059` | Open | 无 1/10/100 文档、1K/10K node、1K agent、trace/overlay/update 延迟与内存预算资格。 |
| `AI-ED-P1-060` | Open | 无编译失败、source 损坏、plugin unload、session churn、queue overflow、panic/device loss 与长期 soak。 |

## 7. Canonical P2 重判

| Canonical ID | 状态 | 目标 |
|---|---|---|
| `AI-ED-P2-001` | Open | 具备 semantic diff/merge、review comment 和 stable identity 的多人 AI 资产协作。 |
| `AI-ED-P2-002` | Open | 大型 BT/StateTree 的虚拟化 graph、分层 LOD、局部布局和增量编译。 |
| `AI-ED-P2-003` | Open | 运行时 trace 与 source revision 的时间旅行、fork 与 deterministic replay。 |
| `AI-ED-P2-004` | Open | 分布式/远端 server AI 观察、权限、加密、redaction 和多 session 聚合。 |
| `AI-ED-P2-005` | Open | EQS 离线数据集、回归比较、统计分布和 query optimization advisor。 |
| `AI-ED-P2-006` | Open | Perception 热力图、occlusion/visibility 解释、历史衰减和跨 agent 聚合。 |
| `AI-ED-P2-007` | Open | StateTree/BT/EQS/Smart Object 跨资产 semantic refactor 与安全 rename。 |
| `AI-ED-P2-008` | Open | 第三方 AI node/schema/editor extension 的 sandbox、version negotiation 与 hot reload。 |
| `AI-ED-P2-009` | Open | 基于 production trace 的 profile-guided tree/query 优化建议，必须可解释且不自动改真值。 |
| `AI-ED-P2-010` | Open | 大世界分区 AI authoring、streaming preview、World Partition Smart Object collection 和跨 cell 诊断。 |
| `AI-ED-P2-011` | Open | 同 AI source/scenario/seed 下与 Unreal 的行为、debug 可见性和性能竞争基准。 |
| `AI-ED-P2-012` | Open | 分布式 fault/scale/soak farm，覆盖 compile/reload/session/network/save/replay 与 Editor 交互分位。 |

## 8. 验收门当前状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| `AI-ED-G01` | Fail | AI Editor 当前静态引用已删除 Viewport Tool API，没有 required compile 证据。 |
| `AI-ED-G02` | Fail | Project 选择 AI 时 first-party Editor catalog/App 不返回 AI registration。 |
| `AI-ED-G03` | Partial | asset/import/toolkit descriptor 存在；create/import/open/reimport/save/reopen 产品链未达标。 |
| `AI-ED-G04` | Fail | 五个 operation 均无 factory，真实 host 会返回 `MissingFactory`。 |
| `AI-ED-G05` | Fail | BT/Blackboard/Perception/EQS/StateTree/Smart Object source schema 与 migration 未建立。 |
| `AI-ED-G06` | Fail | AI document dirty/transaction/undo/redo/autosave/recovery 未建立。 |
| `AI-ED-G07` | Partial | graph/palette descriptor 存在；真实 graph controller/canvas/selection/edit/save 不存在。 |
| `AI-ED-G08` | Fail | Blackboard 独立资产、inheritance、key rename 影响与 live value 未闭合。 |
| `AI-ED-G09` | Fail | Validate/Compile 未调用共享 semantic compiler，也无 deterministic artifact/source map。 |
| `AI-ED-G10` | Fail | 多资产 build set 不能 prepared/atomic publish 或回退 LKG。 |
| `AI-ED-G11` | Fail | Editor preview/PIE/cook/shipping compiler 与 artifact parity 无证据。 |
| `AI-ED-G12` | Partial | shared command/factory/transaction/job 基础存在；AI 未消费。 |
| `AI-ED-G13` | Fail | plugin manager snapshot 与 direct registration 对 factory/provider 贡献不一致。 |
| `AI-ED-G14` | Fail | AI plugin disable/unload/reload 没有 document/reader/job/provider drain 与 terminal receipt。 |
| `AI-ED-G15` | Fail | Behavior Tree surface 仍含业务 `Space` 和无 provider Table。 |
| `AI-ED-G16` | Fail | Perception surface agent Table 无 provider/selection/history。 |
| `AI-ED-G17` | Partial | typed event、session/sequence/World mirror 真实存在；generation/source currentness 不足。 |
| `AI-ED-G18` | Fail | trace 无法表达 active path、parallel nodes、abort、service/task 和 Blackboard delta。 |
| `AI-ED-G19` | Fail | breakpoint、step、step-back、instance selector 和 timeline 未建立。 |
| `AI-ED-G20` | Fail | runtime/editor source-map 及 hot reload generation handoff 不存在。 |
| `AI-ED-G21` | Fail | trace 无 bounded retention、gap/overflow、reader lease 和 slow-reader 资格。 |
| `AI-ED-G22` | Fail | 远端/多进程 debug 授权、clock、disconnect/reconnect 未建立。 |
| `AI-ED-G23` | Partial | overlay geometry、finite 过滤、预分配和 shared provider substrate 存在；AI provider/lifecycle/budget 缺失。 |
| `AI-ED-G24` | Fail | EQS graph/run/item score/profiler 不存在。 |
| `AI-ED-G25` | Fail | StateTree compiler/editor/diff/simulation/debugger 不存在。 |
| `AI-ED-G26` | Fail | Smart Object definition/slot/editor/visualizer/collection 不存在。 |
| `AI-ED-G27` | Fail | static Workbench 仍显示固定 AI asset、trace 与成功反馈。 |
| `AI-ED-G28` | Fail | App/Editor/PIE 端到端产品 lane 不存在。 |
| `AI-ED-G29` | Fail | compile/source/plugin/session/overflow/device fault matrix 不存在。 |
| `AI-ED-G30` | Partial | 18 项 focused test 和 4 项 ignored allocation test 存在；均未覆盖产品装配与主要生命周期。 |
| `AI-ED-G31` | Fail | 1K/10K node、1K agent、trace/overlay P50/P95/P99 及内存预算无证据。 |
| `AI-ED-G32` | Fail | 长时间 PIE/reload/open-close/network/save/replay/overlay soak 与 Unreal 竞争基准不存在。 |

## 9. 分层重构顺序

### M0：编译、catalog 与 truthfulness 硬切

用 `SceneModeRegistration`、真实 factory/provider 替换旧 viewport API；把 AI runtime/editor feature 纳入唯一 first-party catalog/App selection，并加 required compile/startup/disable/unload lane。五个 operation 和两份静态 Workbench 在实现前统一显示 Unavailable/sample，删除固定成功 authority。同步修复 plugin manager snapshot 对 factory/provider 的 materialization 缺失。

### M1：Source Asset、Scene Binding 与依赖合同

定义 Behavior Tree、Blackboard、Perception Config、EQS、StateTree、Smart Object 的 versioned source、stable ID、migration、typed dependency、create/import/reimport 与 cook role。建立 Brain/Agent scene component、artifact handle/generation、spawn/despawn/reload projection；Runtime 与 Editor 只能共享一套 schema/compiler。

### M2：Transactional Document 与 Graph 基础

建立 immutable document revision、stable node/edge/pin/auxiliary ID、selection/controller/canvas/Inspector/Blackboard、typed command/inverse delta、dirty/save/autosave/recovery/conflict、copy/paste/search/diff。复用共享 Editor transaction/history，禁止插件自造第二套简化 undo。

### M3：Shared Compiler 与 Atomic Build Set

Editor Validate/Compile、PIE、cook 和 shipping 统一调用 Runtime semantic compiler；补 admission budgets、deterministic artifact、dependency digest、source map、diagnostic identity、prepared publication、CAS commit、LKG 与 stale result discard。

### M4：Per-World Runtime、Task 与 Perception Scheduler

拆分 immutable registry 与 per-World/per-agent state，建立 execution lease、entity generation 和 teardown state machine；递归 evaluator 改为 budgeted explicit stack。Navigation/Animation/Script/Event/Blackboard effect 全部使用 typed ticket、completion、cancel、timeout 与 owner generation。Perception 改为 incremental source/listener projection、spatial index、统一 items/time/bytes/query budget 与 fail-close/degraded receipt。

### M5：真实 Toolkit 与产品 controller

为 BT/Blackboard/Perception/EQS 建立 factory-backed toolkit、pane data source、operation controller 和 typed receipt，把 ZUI 的无 provider Table/Space 替换为真实 projection。Workbench 只导航到同一 toolkit，不保存第二份 AI 状态。

### M6：Bounded Runtime Debug 与 Perception Overlay

Runtime 发布 active path、parallel node、condition/abort、service/task、Blackboard delta、Perception/EQS 的 bounded journal；identity 包含 session/world/entity/program/schema/source generation。实现 reader lease、field mask、gap/overflow/quality、disconnect 与 hot reload fence。Editor 完成 instance selector、breakpoint、step/history、source jump 和 provider-backed overlay 的 per-view filter/culling/LOD/budget/cache。

### M7：EQS、StateTree 与 Smart Object 独立产品域

分别完成 EQS graph/runtime query/profiler、StateTree hierarchy/binding/compiler/diff/simulation/debugger、Smart Object definition/slot/view model/viewport/visualizer/claim-use/World collection。它们复用资产、文档、编译与调试基础，但保持独立 owner。

### M8：Fault、Scale、Soak 与竞争资格

覆盖 source 损坏、migration、compile cancel、publication failure、plugin reload、session churn、queue overflow、remote disconnect、world teardown 与 device loss；建立 1/10/100 文档、1K/10K node、1K agent、trace/overlay P50/P95/P99、内存与长期 soak。最后用同 source/scenario/seed 与 Unreal 比较作者化、debug 可见性、运行正确性和性能。

## 10. 禁止的临时修补

1. 禁止只改旧 viewport API 名称，却不提供 scene mode factory、overlay provider 与生命周期。
2. 禁止只给 catalog 加 AI 分支，却不验证 compile、selection、disable/unload 和产品 controller。
3. 禁止为空 operation 注册固定 success factory，或继续用 Workbench match 文案冒充 receipt。
4. 禁止把 descriptor、palette 数量、空 Table、`Space` 或静态 row 算作 graph/toolkit 完成。
5. 禁止 Editor 复制 Runtime 验证/编译逻辑，或让 Preview 与 shipping 消费不同 artifact。
6. 禁止以裸 World/Entity u64、单 active node 和每帧全量 snapshot 作为长期 debug 协议。
7. 禁止 UI 直接持有 Runtime manager 锁，或建立无 reader lease、无 gap receipt 的无限 trace。
8. 禁止只靠固定 256 pair budget 声称 Perception 可扩展，也禁止 physics unavailable 时静默判定可见。
9. 禁止让 active camera 决定 server/headless 的 AI authority tick。
10. 禁止把 EQS、StateTree、Smart Object 压成 BT palette 中的伪节点或 Blackboard key。
11. 禁止保留通用 Data TOML、场景字符串、脚本镜像与 compiled AI artifact 的多重 authority。
12. 禁止用 historical milestone、manifest `complete`、unit test 数量或 ignored benchmark 替代当前产品证据。

## 11. 本轮产出与实施前置

本轮新增 Editor142 current-source review，并更新 Editor 索引、根索引和覆盖矩阵；未修改 production Runtime、Editor、App、plugin 或 tests。tooling 不在当前目标内；协调器状态按用户要求未查询、未轮询、未等待。

实施必须从 M0 开始，并与 Runtime152、Plugins15/06 的 source/compiler/world runtime/package owner 同步收敛。由于 MVP `00` 尚在进行且 F0-F5 被阻塞，本报告当前只提供高级功能的可执行重构顺序，不授权跳过 MVP gate 实施 AI 大规模代码改造。
