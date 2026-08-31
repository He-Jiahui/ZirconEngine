---
title: Editor AI、Behavior Tree、Blackboard、Perception 当前工作树 Authoring/Graph/Debug/Overlay/Workbench 工程化复审
category: zircon_editor
report_id: Editor234
review_date: 2026-08-30
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/142-editor-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-debug-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/89-editor-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-debug-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/174-runtime-ai-current-working-tree-world-agent-behavior-tree-blackboard-perception-execution-debug-review.md
related_code:
  - zircon_plugins/ai/editor
  - zircon_plugins/ai/runtime
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/extension/store/batch.rs
  - zircon_editor/src/scene/modes
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai
tests:
  - zircon_plugins/ai/editor/src/tests.rs
  - zircon_plugins/ai/editor/src/overlay/allocation_tests.rs
  - zircon_plugins/ai/editor/src/runtime_mirror/lookup_allocation_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/89-editor-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-debug-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/142-editor-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-debug-authoring-current-source-review.md
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/zircon_plugins/06-ai.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeDebugger.cpp
  - dev/UnrealEngine/Engine/Plugins/AI/EnvironmentQueryEditor/Source/EnvironmentQueryEditor/Private/EnvironmentQueryEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeEditorModule
  - dev/Fyrox/editor/src/scene/commands/graph.rs
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/editor/debugger/editor_debugger_node.cpp
  - dev/bevy/crates/bevy_tasks/src/lib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
---

# Editor234 · AI Authoring、Graph、Debug、Overlay 与 Workbench 当前工作树复审

## 1. 结论

AI editor 目前有可保留的注册和数据投影底座：`zircon_plugins/ai/editor` 当前 12 个文件、2,382 行、83,179 bytes、18 个测试属性、4 个 ignored；两份 AI Workbench ZUI 为 478 行、26,885 bytes。`plugin.rs` 能登记 AI drawer、Behavior Tree asset/importer/toolkit/graph/palette、四个行为树 operation 和 perception debug surface；`runtime_mirror.rs` 能按 play session、delivery sequence、World/Entity 校验并拒绝 stale/cross-world frame；overlay controller 能把 FOV、听觉半径和 stimulus 连接投影为 `SceneGizmoOverlayExtract`。这些是工程化起点，不应被删除。

但是这仍是 descriptor/mirror shell，不是 Unreal 级 Editor 产品。最硬的当前阻断是 `overlay.rs:2,127-138` 仍 import `ViewportToolModeDescriptor` 并调用 `register_viewport_tool_mode`，而当前 core Editor contract 已经是 `SceneModeRegistration`/`register_scene_mode`（`zircon_editor/src/core/editor_extension.rs:23,300-325`），因此该插件与当前 API 不兼容或无法由默认构建消费。`first_party_editor_catalog` 也没有 AI provider；App 只委托该 catalog，所以 AI Editor 不会从 manifest selection 进入默认 Editor Host。

即便绕过注册阻断，四个 BT operation、importer、toolkit、graph editor、palette 和 perception toggle 仍只有描述符，没有 `OperationCommandFactory`、document/controller、transaction、编译 job、artifact receipt、PreviewWorld 或 runtime attach。Mirror 只保存最新 debug frame/active node 与每节点最后结果，Workbench 根节点默认 collapsed，`BT_Enemy`、`AI_Guard_01`、`Sniper_Perception`、固定时间和固定反馈均是静态 fixture。EQS、StateTree、Smart Object、AI profiler、breakpoint/step/replay 没有真实 Editor domain。

本轮登记 **20 项新增 P1、8 项新增 P2、18 个资格门**，不新增唯一 P0；旧 Editor20/89/142 的历史 owner 继续保留。报告只做 review/refactor plan，不运行 Cargo、UI automation、PIE 或 tooling。

## 2. 审查范围与证据

### 2.1 当前物理冻结

| 范围 | 文件 | 行 | bytes | tests | ignored |
|---|---:|---:|---:|---:|---:|
| AI editor plugin（Rust/TOML/ZUI） | 12 | 2,382 | 83,179 | 18 | 4 |
| AI runtime mirror/overlay allocation tests | included above | included | included | included | included |
| AI Workbench behavior/perception | 2 | 478 | 26,885 | 0 | 0 |
| first-party editor catalog | 4 | 251 | 8,978 | 6 | 0 |
| runtime framework AI boundary | 9 | 1,040 | 31,239 | 0 | 0 |

统计按当前工作树文件逐项读取，未把整个 retained host 目录的 80 文件、17,760 行误算为 AI 实现；只引用其中 AI action/feedback/navigation 命中点。

### 2.2 关键证据定位

1. `zircon_plugins/ai/editor/src/plugin.rs:33-56` 用 `authoring_plugin!` 声明 Experimental AI editor plugin，并镜像 runtime manifest；`:64-78` 通过 `expect` 取得两个 runtime mirror；`:97-116` 只登记 drawer/template/surface/contribution batch。
2. `plugin.rs:120-182` 的 batch 注册 import/open/validate/compile 四个 command descriptor、`.btree.toml` importer、asset contribution/toolkit、graph editor 和 compile operation；没有 factory、document、controller、job 或 artifact consumer。
3. `plugin.rs:187-214` 从 runtime `standard_node_catalog()` 生成 palette；每个 `GraphNodeDescriptor` 只有 id/display/category（无参数/pin/factory/side-effect/generation）。
4. `overlay.rs:2` import 已不存在的 `ViewportToolModeDescriptor`；`:105-139` 注册 UI template、authoring surface、toggle command 并调用已删除的 `register_viewport_tool_mode`。当前 core registry 的公开入口是 `register_scene_mode` 和 `register_viewport_overlay_provider`（`zircon_editor/src/core/editor_extension.rs:300-338`）。
5. `runtime_mirror.rs:47-135` 的 `AiPieMirror` 以 `(world,entity)` BTreeMap 保留每帧一个 `AiBehaviorDebugFrame`；只检查 session/sequence/World，未保存 program generation、active path、node transitions、budget或 trace cursor。
6. `runtime_mirror.rs:189-288` 的 node result mirror 以 `(world,entity)->node_id` 只保留每节点最后一个 `BtNodeResultEvent`；snapshot prune 只按照当前 active_node 集合删除旧高亮；`:323-391` 用三个 `Arc<Mutex<...>>` consumer state 注册，缺 reader/backpressure/loss/resync。
7. `zircon_plugins/first_party_editor_catalog/src/catalog.rs:35-50` provider 只有 Navigation/Neural；`zircon_app/src/entry/first_party_editor_plugins.rs:15-38` 只委托 catalog，不存在 AI 第二入口。
8. `zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_behavior_workspace.zui:29` 根 workspace collapsed；`:89-170` 固定 Selector/Attack/node rows/`Runtime Trace: branch changed to Attack`；`:200-236` 固定 `BB_Enemy`、`AIController_Enemy`、`Running`。
9. `workbench_perception_workspace.zui:28` 根 workspace collapsed；`:87-167` 固定 `AI_Guard_01`、`Sniper_Perception`、`74 deg target visible`、`Noise_Maker_BP 1200 cm` 和 `00:12.345`；`:188-232` 固定 Guard config、Line of Sight On、Team Filter All。
10. `module_navigation.rs:452-458,486-521` 只把 validate/simulate 和 blackboard/AI/state/config/LOS/team edit/commit 映射到 control id 或 boolean route；没有 typed document intent、target selection 或 command receipt。
11. `module_command_feedback.rs:287-294,341-348` 直接返回 “Behavior tree validated”/“Perception simulation running” 与固定输出文本；这不是 runtime operation、compile job、PIE simulation 或 error projection。
12. `zircon_plugins/ai/editor/src/tests.rs` 的主要测试验证 descriptor/palette/manifest、mirror stale/session/world 行为和 overlay 几何；没有真实 import/open/compile/save/reopen、factory、SceneMode lifecycle、PreviewWorld、runtime attach 或 UI interaction 测试。

## 3. 已有底座与边界

| 底座 | 当前价值 | 不能被误称为 |
|---|---|---|
| descriptor batch | 清楚列出 asset type、operation、toolkit、graph/palette、capability | 可执行 operation 或真实 asset editor |
| session/sequence mirror | 能隔离 PIE session、stale delivery 和跨 World 数据 | 完整 debugger、trace、replay 或 lossless stream |
| overlay geometry | FOV/hearing/stimulus 的有限浮点过滤和 capacity 估算 | 正式 SceneMode/overlay provider、selection/picking 或实时 World consumer |
| runtime catalog palette | 与 18 个标准 node descriptor 共享命名 | 可编辑 graph schema、pin/parameter factory 或 compiled artifact source map |
| ZUI layout | 有 behavior/perception 页面骨架 | 动态 provider、实时状态、Editor document 或 authoritative data |

## 4. P1 差异与重构合同

### 4.1 Catalog、API 与资源

| ID | 状态 | 当前差异 | 必须重构 |
|---|---|---|---|
| ED-AI-P1-001 | Open | overlay 使用已删除 `ViewportToolModeDescriptor/register_viewport_tool_mode` | 迁移到 `SceneModeRegistration` + 正式 overlay provider，增加 compile/lifecycle test，禁止保留兼容影子 API |
| ED-AI-P1-002 | Open | first-party editor catalog 没有 AI feature/dependency/provider，App 也无第二入口 | 增加 AI editor provider、manifest selection、target/capability admission、disable/unload receipt；缺 provider不得静默返回空 Vec |
| ED-AI-P1-003 | Open | plugin registration 每次构造 `AiEditorPlugin`，mirror通过 expect取得，未绑定长生命周期 host/provider | 由 Editor extension owner 持有 generation-qualified provider/session；注册、撤销、reload、PIE stop 使用同一 owner lease |
| ED-AI-P1-004 | Open | 两份 `plugins://ai/editor/*.zui` 资源虽然存在，但没有资源 mount/admission/parse/版本测试 | 将 template/resource URI纳入 package manifest 与 mount registry，测试缺失、版本不兼容和回滚 |
| ED-AI-P1-005 | Open | operation descriptor有 payload schema/menu path，但无 factory/handler；共享 dispatch 会产生 fake success 或 MissingFactory | 每个 operation 接 typed `OperationCommandFactory`、target/session/generation、progress/cancel/terminal receipt |
| ED-AI-P1-006 | Open | importer 只声明 `.btree.toml`，无 source decode、diagnostic span、dependency/cook artifact | 引入 `BehaviorTreeSourceDocument`、版本迁移、依赖 graph、原子 import/reimport、last-good artifact |

### 4.2 Behavior Tree、Blackboard 与 Perception authoring

| ID | 状态 | 当前差异 | 必须重构 |
|---|---|---|---|
| ED-AI-P1-007 | Open | graph editor/palette 没有 document/session/controller；无法编辑节点、连线、root、subtree或 auxiliary/service | 建立 stable document revision、node/edge/pin identity、selection model、graph controller、validation diagnostics与可逆 transaction |
| ED-AI-P1-008 | Open | palette只投影 id/display/category，runtime descriptor也没有 pin/parameter/default/side-effect schema | 由同一 versioned node schema生成 palette、details、pin compatibility、asset picker、capability/owner generation |
| ED-AI-P1-009 | Open | 没有 Blackboard asset/toolkit、key schema editor、default/inheritance/rename/migration UI | 建立 Blackboard source asset、typed key model、schema revision/redirect、preview values 和 migration diagnostics |
| ED-AI-P1-010 | Open | Perception surface 没有 receiver/source/sense configuration document，Workbench 的 Team/LOS 只是值文本 | 建立 SenseConfig document、affiliation/team/tag/range/FOV/age、provider validation、world binding与 transaction |
| ED-AI-P1-011 | Open | EQS/StateTree/Smart Object 在 editor package 中无 asset type、factory、graph/schema、compile domain | 作为独立 Editor modules实现 source/schema/compiler/toolkit/debugger，不塞入 BT palette 或共享静态页面 |
| ED-AI-P1-012 | Open | 没有 save/reopen/source control/merge/recovery 证据；descriptor 不带 source revision/digest | 接入 Editor document/transaction/savepoint、atomic write、reopen equivalence、conflict/redirect 和 last-good recovery |

### 4.3 Runtime attach、debug 与 overlay

| ID | 状态 | 当前差异 | 必须重构 |
|---|---|---|---|
| ED-AI-P1-013 | Open | mirror只保留每 agent 最新 frame，缺 program/entity generation、active path、condition/abort/task history | 消费 Runtime174 的 versioned trace：session/world/program generation、node enter/exit、abort、task ticket、timing、budget与loss receipt |
| ED-AI-P1-014 | Open | node result mirror只保留每节点最后 event，snapshot prune依赖单 active_node，无法表达 parallel/subtree或已结束节点 | 使用 bounded ring/delta projection、per-instance path、terminal/expired marker、reader cursor、resync和筛选预算 |
| ED-AI-P1-015 | Open | 三个 consumer 用 `Arc<Mutex>`，没有订阅者 backpressure、容量/bytes/age限制或断线恢复 | 由 Runtime event consumer coordinator 分配 owner/session lease，统一 cursor、drop/overflow、resync和 shutdown |
| ED-AI-P1-016 | Open | overlay只生成圆弧/线段/pick shape；没有正式 scene mode factory、selected agent、World binding、provider capability或撤销 | 注册真实 `SceneModeRegistration`/overlay provider；selection、World/PIE、visibility/filter/picking 与 render extract 都 generation-qualified |
| ED-AI-P1-017 | Open | no AI profiler/breakpoint/step/branch/subtree inspector/EQS score/Perception heatmap | 建立可暂停/单步/断点的 debugger session，时间线/火焰图/Blackboard diff/sense-query heatmap，所有视图受 debug budget 与授权控制 |
| ED-AI-P1-018 | Open | 无 PreviewWorld/PIE attach、standalone server-client、world selection 或 runtime command bridge | 使用 Editor-owned PreviewWorld/PIE session，runtime service 通过 ticket attach/detach；不能让 UI 直接锁 manager |

### 4.4 Workbench 与产品验证

| ID | 状态 | 当前差异 | 必须重构 |
|---|---|---|---|
| ED-AI-P1-019 | Open | behavior/perception workspace 默认 collapsed，固定样例与固定文本反馈遮蔽真实状态 | 由 provider state/render model 驱动可见性、selection、node/agent list、diagnostics和空/错误/加载状态；删除固定样例真值 |
| ED-AI-P1-020 | Open | `module_command_feedback.rs` 对 validate/simulate 直接返回成功/queued 文本，action routes没有真实 command receipt | 所有按钮绑定 typed operation、job progress/cancel/error/artifact/runtime output；没有 handler 时必须显示不可用诊断 |

## 5. P2：竞争性能力

| ID | 目标 | 前置条件 |
|---|---|---|
| ED-AI-P2-001 | 大规模 virtualized graph/agent/Blackboard/Perception lists | stable IDs、delta projection、viewport budget |
| ED-AI-P2-002 | graph diff/merge/migration/source map | versioned document/compiler artifact与node correspondence |
| ED-AI-P2-003 | breakpoint/watchpoint/step-in/over/out/back 与 branch replay | runtime trace ticket、pause/cancel/replay input |
| ED-AI-P2-004 | EQS visualizer（generator/context/test/item score/partial） | EQS runtime query service和time-sliced receipt |
| ED-AI-P2-005 | StateTree/Smart Object独立 toolkit、preview与debugger | source/compiler/runtime domain闭合 |
| ED-AI-P2-006 | remote multiplayer AI debugger、record/replay与权限审计 | event transport、session identity、privacy/redaction、bounded remote capture |
| ED-AI-P2-007 | multi-user collaborative graph/lock/conflict resolution | document transaction、source control、stable path/merge |
| ED-AI-P2-008 | AI quality dashboard、fault/soak/scale archive | deterministic scenario corpus、1K/10K agents、多 World 与 cross-platform receipt |

## 6. 参考编辑器对照

| 参考 | 本轮采用的工程约束 | Zircon 当前差距 |
|---|---|---|
| Unreal BehaviorTreeEditor | 独立 asset/schema/factory、graph schema、details、UpdateAsset、Undo/Redo、Find/Diff 与 debugger | Zircon只有 descriptor/palette/operation名，无 document/controller/factory/trace debugger |
| Unreal BehaviorTreeDebugger | PIE session、pause/step、instance stack、active path、breakpoint 与 runtime category | Zircon mirror只保留 frame/active node，不能暂停、单步或重建 path |
| Unreal EnvironmentQueryEditor | generator/option/test graph、compile log、query profiler、item score/partial result | Zircon没有 EQS source/runtime/editor domain |
| Unreal StateTree Editor | typed binding compiler、instance data、async execution context、compile-all diagnostics | Zircon没有 StateTree module，仅作为未来 P2 |
| Fyrox graph/commands | serializable graph、stable handles、`CommandTrait` execute/revert | Zircon尚未把 graph editing接到 reversible document transaction |
| Godot `EditorUndoRedoManager`/Debugger | history owner、save-state、session pause/step、remote capture | Zircon没有 AI document history 或 runtime debugger session |
| Bevy tasks | owned/scope task execution边界 | Editor operation没有 job/cancel/owner ticket |
| Unity Graphics DebugManager | provider/panel register/unregister/reset 与可控 debug data source | Zircon consumer mutex无 provider lifecycle、reader backpressure与reset contract |

## 7. 分层重构路线

### M0 · API 与 provider truth

把 overlay 迁到 current SceneMode/overlay API；将 AI 接入 first-party editor catalog 和 App target；为 template URI、capability、operation factory 和 missing provider 增加失败测试；清理静态成功反馈。

### M1 · Source document 与 graph

建立 BehaviorTree/Blackboard/Perception source document、stable node/edge/key IDs、schema/pin/parameter model、transaction/undo/save/reopen、import/reimport/migration与 compiler job/artifact receipt。palette、details、validation、find/diff都消费同一 schema。

### M2 · Runtime attach 与 debugger

由 Editor PreviewWorld/PIE session持有 runtime attach ticket；mirror消费 Runtime174 的 generation-qualified trace/delta；加入 pause/step/breakpoint、Blackboard diff、sense/query heatmap、selection/World filter与 bounded remote capture。

### M3 · Workbench truth与高级 domain

替换 collapsed/fixed Workbench 为 provider-driven state；空、loading、fault、permission、stale、no-reader 状态均显式表达。BT稳定后分别建设 EQS、StateTree、Smart Object toolkit，不复用静态 BT 页面。

### M4 · 资格与删除旧权威

通过 import/cook/save/reopen、runtime tick/abort/reload、PIE/standalone、overlay/picking、trace loss/resync、fault/scale/soak 与 cross-platform acceptance 后，删除固定 sample、fake feedback、旧 viewport API 与 descriptor-only fallback。

## 8. 资格门

| Gate | 必须证明 |
|---|---|
| ED-AI-G01 | AI editor 在 current core API 下可编译，使用 SceneMode/overlay provider 正式注册并可撤销 |
| ED-AI-G02 | manifest selection -> first-party editor catalog -> App Host provider 闭合，缺 provider显式失败 |
| ED-AI-G03 | 两份 ZUI 通过 package mount、version、missing-resource、fallback 测试 |
| ED-AI-G04 | BT import/open/validate/compile 每个 operation 都有 factory、target、receipt、cancel/error |
| ED-AI-G05 | source document、stable node/edge/pin/key identity、transaction/undo/save/reopen 可回放 |
| ED-AI-G06 | runtime node schema与Editor palette/details/validation使用同一 generation-qualified schema |
| ED-AI-G07 | Blackboard/Perception authoring具有typed config、migration、dependency、diagnostic与scene binding |
| ED-AI-G08 | PreviewWorld/PIE attach/detach 不泄漏 Runtime manager、World 或 task ticket |
| ED-AI-G09 | runtime mirror验证 session/world/program/entity generation，并能处理 stale/drop/resync |
| ED-AI-G10 | trace显示 active path、condition、abort、task transition、timing、budget与error，而非单 active node |
| ED-AI-G11 | node result/Perception overlay为 bounded delta，支持 selection、World/PIE、filter、picking与撤销 |
| ED-AI-G12 | debugger支持 pause/step/breakpoint/branch/subtree，并有权限、超时和恢复路径 |
| ED-AI-G13 | Workbench所有按钮绑定真实 operation/job/runtime output，静态成功/queued文本已删除 |
| ED-AI-G14 | EQS/StateTree/Smart Object 各自有 source/compiler/runtime/editor owner，不借 BT 伪造 |
| ED-AI-G15 | save/reopen/source-control/merge/recovery 与 runtime artifact generation 一致 |
| ED-AI-G16 | runtime/native/editor event schema、cursor、loss receipt、privacy/permission统一 |
| ED-AI-G17 | 1K/10K agent、深图、双 World、PIE、远程调试、fault/soak和UI virtualization有 receipt |
| ED-AI-G18 | 删除旧 `ViewportToolModeDescriptor`、fixed sample、fake feedback、descriptor-only fallback 后仍有完整产品闭环 |

## 9. 明确不接受的修复

1. 不以新增 ZUI、control id、route 或 palette descriptor关闭 Editor 功能。
2. 不以测试中构造 mirror/overlay、固定 `BT_Enemy` 文案或 “validated/simulation running” 关闭真实 runtime/debug。
3. 不复制 Runtime 的 Scene/Asset/Task/Blackboard/Perception authority到 Editor；Editor只持有 document/session/projection。
4. 不把 18 个局部测试、descriptor registration 或 overlay capacity benchmark写成 import/cook/PIE/scale资格。
5. 不在旧 API、无 factory 或无 artifact receipt 仍存在时宣称 AI Editor 已完成。

## 10. 状态

本轮只新增本报告、索引与 coverage 记录，没有修改 Editor、Runtime、Cargo、ABI、ZUI、测试或 tooling。Editor234 的 20 项 P1、8 项 P2、18 个资格门均为 Open/Fail 的重构计划；运行时对应 owner 为 [Runtime174](../zircon_runtime/174-runtime-ai-current-working-tree-world-agent-behavior-tree-blackboard-perception-execution-debug-review.md)。
