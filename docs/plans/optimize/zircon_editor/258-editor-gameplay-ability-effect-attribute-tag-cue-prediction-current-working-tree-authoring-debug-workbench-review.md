---
title: Editor Gameplay Ability、Effect、Attribute、Tag、Cue 与 Prediction 当前工作树 authoring、debug 与 Workbench 边界复审
category: zircon_editor
report_id: Editor258
review_date: 2026-08-31
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/235-editor-gameplay-ability-effect-attribute-tag-cue-prediction-current-working-tree-authoring-debug-workbench-review.md
  - docs/plans/optimize/zircon_editor/143-editor-gameplay-ability-effect-attribute-set-gameplay-tags-tag-query-cue-prediction-debug-authoring-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/198-runtime-gameplay-ability-effect-attribute-tag-cue-prediction-current-working-tree-authority-artifact-execution-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_effect_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_tags_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/document
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/play
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayAbilitiesEditor/Private/GameplayAbilitiesEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayAbilitiesEditor/Private/GameplayAbilityGraph.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayAbilitiesEditor/Private/GameplayAbilityGraphSchema.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayAbilitiesEditor/Private/GameplayAbilitiesBlueprintFactory.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayAbilitiesEditor/Private/GameplayAbilityAudit.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayAbilitiesEditor/Private/GameplayEffectDetails.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Private/SGameplayTagWidget.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Private/SAddNewGameplayTagWidget.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Private/SRenameGameplayTagDialog.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Private/SCleanupUnusedGameplayTagsWidget.cpp
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/editor/editor_debugger_node.cpp
  - dev/bevy/crates/bevy_asset/src
  - dev/Graphics/Packages/com.unity.shadergraph/Editor
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor258 · Gameplay authoring、validation、preview 与 prediction debug 当前工作树差距

## 1. 结论

当前 Editor 的 Gameplay Ability、Effect、Tags 三个 Workbench 仍不是可交付的 authoring product。当前工作树三份 `.zui` 共 828 行、46,761 bytes；本轮 focused 选择集为 231 个去重文件、50,464 行、46,024 非空行、1,784,203 bytes、350 个测试声明、0 个 ignored marker，指纹 `6ea08036eb8d5283b215804418f5558ce2f343e2e3dea3be6a92058de9cc8bbf`。共享 module_navigation.rs、module_command_feedback.rs、module_field_edit.rs 虽加入控件单节点查找、属性行、滚动容器和更窄的命令路由，仍没有 Gameplay provider、document、operation、compiler、artifact 或 runtime session consumer。

三份 workspace 根节点仍以 `props = { visibility = "collapsed" }` 发布。Ability 固定 GA_DashAttack、Server Initiated、GE_DashAttack_Cost、Activate -> Cost -> Montage -> Damage -> End 和 1.22s Ability Activated；Effect 固定 GE_HealthRegen、GE_DamageFire、10 秒 duration、1 秒 period、Health +10.0 和 Simulation Output: +50 health over 10 seconds；Tags 固定 DefaultGameplayTags.ini、Ability.Activate、Character.State.Stunned 与 2 errors / 8 warnings / 6 infos。这些仍是展示数据，不是从资源或 runtime snapshot 读取的状态。

共享桥进一步确认了假闭环：当前 `module_navigation.rs:304-371` 仍把 Apply/Playtest/Add/Rename 和属性 Change/Submit 映射为字符串 action；`module_command_feedback.rs:376-404` 只对 Save/Browse/Compile/Diff/Simulate 返回静态模块文案，Apply/Playtest 的显式成功文案虽已删除，但 Simulate/Save/Compile/Diff 仍不连接后端；`module_field_edit.rs:17-35` 对 Change/Submit 只 mutation control 的 `value` 和 `value_text`。因此改名、改 magnitude、Apply 或 Playtest 不会产生 document revision、transaction、compile job、runtime request 或 prediction receipt。

Editor143 的 canonical P0 仍是 5 Open / 0 Partial / 0 Closed；Editor90 账本仍不能因通用 document/undo/job/play 底座存在而关闭。本次重新核对后仍为 24 项 P1（21 Open / 3 Partial / 0 Closed）、10 项 P2（10 Open）和 22 道资格门（20 Fail / 2 Partial / 0 Pass），不新增独立 P0。当前 ZUI 响应式布局与桥路由优化是可保留底座，但不改变 domain owner 缺失。

## 2. 当前证据

### 2.1 ZUI 是 fixture，不是 resource-backed editor

- Ability：`workbench_ability_workspace.zui:24-27,69-94,121-171,203-255` 仍直接写入 collapsed root、GA_DashAttack、Playtest、phase matrix、graph text、timeline、Server Initiated 与 4.00s cooldown；新增 Overlay/ScrollableBox/PropertyEditorRow 与 responsive props 只改善布局，没有 collection binding、asset handle、graph model 或 diagnostics source map。
- Effect：`workbench_effect_workspace.zui:24-28,69-121,148-241,267-344` 仍直接写入 GE_HealthRegen、GE_DamageFire、duration/period/policy、modifier rows、Gameplay Cue 行和 +50 health preview；新增 search/scroll/property rows 没有 AttributeSet/aggregator/capture/execution document。
- Tags：`workbench_tags_workspace.zui:24-28,67-150,175-224` 仍直接写入 collapsed root、DefaultGameplayTags.ini、两个 tag 行、固定 validation 数字、redirect/owner 字段；没有 registry provider、canonical id、rename transaction 或 usage index。

### 2.2 路由与反馈没有业务后端

- `module_navigation.rs:1-18,124-156,168-218,304-371` 列出模块 surface、row/action 与 field route；当前移除了旧 panel-tab/command-control 双查找，但 route 仍只返回 control id/布尔判定，不创建 domain operation。
- module_navigation.rs:478-515 将 edit/commit 识别为字段动作，但没有 typed property path、document id、revision、selection generation 或 operation target。
- `module_command_feedback.rs:231-242,247-370,376-404` 仍以模块枚举和静态字符串生成 Save/Compile/Diff/Simulate status/output；Apply/Playtest 显式假成功分支虽被移除，但 generic Simulate 等仍可在无 backend 时显示“running”。
- `module_field_edit.rs:17-35,41-59` 仅写 retained control 属性并 refresh；它没有构造 undo command，也没有在提交时调用 asset/document/runtime service。

### 2.5 当前工作树增量

- `workbench_ability_workspace.zui:24-255`、`workbench_effect_workspace.zui:24-344`、`workbench_tags_workspace.zui:24-224` 新增 Overlay、ScrollableBox、responsive tier、density token、PropertyEditorRow、dropdown options 与 input flags，改善布局与输入语义，但根节点仍 collapsed，业务值仍硬编码。
- `module_command_feedback.rs:44-69,211-242` 将 selected/checked 解析合并为单节点查找，并保留静态 module output；`module_navigation.rs:28-371` 拆分 row group、移除旧 panel tab route，但 action 仍未产生 document/operation receipt。
- `zircon_app/src/entry/first_party_editor_plugins.rs:10-40` 与 `zircon_plugins/first_party_editor_catalog/src/catalog.rs:10-54` 当前 provider 解析更显式、去重更稳定，但 provider 集合仍只有 Navigation/Neural，没有 Gameplay provider。

### 2.3 Catalog、asset 与 App 装配缺失

zircon_plugins/first_party_editor_catalog/src/catalog.rs 当前 provider 分支只有 Navigation 与 Neural；zircon_app/src/entry/first_party_editor_plugins.rs 只委托该 catalog。ResourceKind 和现有 editor asset/toolkit 也没有 Gameplay Ability、Effect、AttributeSet、Tag、Cue 类型入口。没有 importer/factory、dependency graph、semantic compiler、artifact cache、source map、LKG、diagnostic panel、preview-world adapter 或 runtime attach provider。

### 2.4 Debug/preview 不可证明

当前没有 Gameplay debug provider、per-World mirror、prediction timeline、active effect inspector、attribute capture trace、tag query explanation、cue event log、network reconciliation view 或 deterministic replay diff。WorkBench 的 Playtest 不创建 isolated PreviewWorld/PIE session，也没有 cancel/timeout/teardown/receipt；静态 output 反而会掩盖 runtime 尚不存在的状态。

## 3. 与参考编辑器的差异

Unreal GameplayAbilitiesEditor 将 module startup、asset actions、Blueprint factory、graph/schema、details customization、audit 与 compile diagnostics 分成可测试的 editor owner；GameplayTagsEditor 提供树状 registry、new/rename/redirect/cleanup、source ownership、validation 和 reference cleanup。Zircon 只有三份 layout asset 和共享字符串路由，缺少这些 owner 边界。

Fyrox 的 command 模型与 Godot UndoRedo/Debugger 代码说明 editor mutation 必须是可命名、可合并、可撤销、可序列化且有对象生命周期 fence 的 operation；Zircon field bridge 直接 mutation control，无法 save/reopen、undo/redo 或 crash recovery。Bevy asset loader 与 Unity ShaderGraph/VisualEffectGraph editor 则展示 source graph、dependency、compiled artifact、property binding 和 preview/runtime parity 的分离；当前 Ability graph row 只是一个 value_text。

## 4. P1 差距与重构任务

| ID | 当前问题 | 重构结果 / 验收 |
|---|---|---|
| ED-GAS-01 | 无 Gameplay editor provider | 增加 manifest、feature、first-party catalog、App composition、dist/ABI registration；provider 缺失时 Workbench 不显示假入口。 |
| ED-GAS-02 | 无资源类型/工厂 | 为 Ability/Effect/AttributeSet/Tag source/Cue 定义 ResourceKind、factory、import/reimport、subasset identity、dependency scan 与 unsupported diagnostic。 |
| ED-GAS-03 | fixture ZUI | 将 workspace 绑定到 typed document/query snapshot；去掉固定样例、collapsed 默认和硬编码 validation/output。 |
| ED-GAS-04 | 无 document/session owner | 建立 GameplayDocument、selection/lease、revision、object generation、dirty state、save/reopen 与 last-good source。 |
| ED-GAS-05 | 字段 edit 不产生 operation | 所有 Change/Submit 生成 typed command，支持 merge group、undo/redo、savepoint、reject receipt 和 document revision fence。 |
| ED-GAS-06 | 无 graph model/schema | Ability graph 使用 stable node/edge id、pin/type schema、cycle/latent/cancel validation、copy/paste/redirect 与 source-map。 |
| ED-GAS-07 | Effect inspector 无 semantic model | duration/period/policy/stack/modifier/capture/execution/cue 使用 typed property paths 和 schema-driven details；编辑器与 runtime compiler 同源。 |
| ED-GAS-08 | AttributeSet 不存在 | 提供 attribute definition、base/current/final、clamp、replication/save metadata 和 capture preview；禁止用字符串 Health 行替代。 |
| ED-GAS-09 | Tag registry 不存在 | 提供 hierarchical tree、canonical id、source ownership、redirect/rename transaction、usage/reference index 和 conflict diagnostics。 |
| ED-GAS-10 | 无 Tag Query builder | 将 query 组合编译为 typed bytecode，显示 match explanation、missing/redirected tag 和 bounded complexity。 |
| ED-GAS-11 | 无 shared compiler | editor compile 与 runtime load 使用同一 deterministic compiler/artifact/hash/source map；失败保留 last-good artifact。 |
| ED-GAS-12 | 无 diagnostics model | 诊断必须含 code、severity、source span、object id、fix-it、generation；不能固定显示 2/8/6 数字。 |
| ED-GAS-13 | Apply 无 preview transaction | Effect preview 在 isolated PreviewWorld 中执行 runtime artifact，带 snapshot/rollback、target selection、clock、budget 和 receipt。 |
| ED-GAS-14 | Playtest 无 PIE/authority | Ability playtest 创建可取消的 Preview/PIE session，显示 admission/cost/cooldown/task/cue/prediction/reconcile timeline。 |
| ED-GAS-15 | 无 runtime debug provider | 提供 per-world mirror、active effect/attribute/tag/cue snapshots、generation/path/trace loss、sampling 与 bounded history。 |
| ED-GAS-16 | 无 prediction inspector | 显示 prediction key、input receipt、speculative delta、server ack/reject、rollback reason 和 replay correlation。 |
| ED-GAS-17 | 无 network/save views | 为 replication baseline/late join/relevancy、save participant/migration、checksum divergence 提供可查询 artifact 和 diff。 |
| ED-GAS-18 | 无 job lifecycle | import/compile/preview/validation 进入统一 JobService，支持 admission、cancel、progress、timeout、shutdown drain 和 failure artifact。 |
| ED-GAS-19 | 无 search/reference navigation | tag/effect/ability/cue usage 进入 indexed search，结果带 asset/document revision 与 source location；不能只搜索展示文本。 |
| ED-GAS-20 | 无 collaboration/conflict model | source ownership、file lock/lease、merge/rebase、redirect migration 与 external change detection 形成可恢复 workflow。 |
| ED-GAS-21 | 无 conformance tests | 增加 factory/catalog、document undo/save/reopen、compiler parity、tag redirect/query、preview rollback、prediction/replay UI automation。 |
| ED-GAS-22 | 无 scale/soak evidence | 对 100K assets、10K tag queries、1K graph nodes、长时间 preview、network churn 测量 P95/P99 和内存上限。 |
| ED-GAS-23 | shared route namespace 污染 | 把 module route/action/control 分层并版本化，禁止 generic string route 直接决定 domain mutation。 |
| ED-GAS-24 | 产品状态假阳性 | Apply/Playtest/Save/Add/Rename 只有拿到真实 operation/runtime receipt 后才能呈 success；否则显示 unavailable/diagnostic。 |

## 5. P2 产品化

P2 共 10 项：1) graph minimap/layout persistence；2) tag usage heatmap；3) effect audit report；4) attribute capture timeline；5) cue preview controls；6) network bandwidth overlay；7) replay diff export；8) localization/display metadata；9) telemetry redaction；10) artifact cache eviction/accessibility command coverage。它们必须读取真实 document/artifact/snapshot，不得继续扩展 fixture 文案。

## 6. 资格门

| Gate | 当前 | 必须证明 |
|---|---|---|
| E1 provider/catalog/App | Fail | Gameplay provider 可解析、可装配、可禁用且无假入口。 |
| E2 resource/factory | Fail | 五类 source asset 可创建、导入、重载、保存并保留 identity。 |
| E3 document/operation | Fail | 每次 mutation 有 revision、undo/redo、savepoint、reject receipt。 |
| E4 compiler parity | Fail | editor/runtime 共用 artifact/hash/source map。 |
| E5 diagnostics | Fail | 诊断来自 compiler/validator，source span 与 fix-it 可导航。 |
| E6 tag registry | Fail | hierarchy、redirect、rename、query、usage、conflict 可重开复现。 |
| E7 effect/attribute authoring | Fail | modifier/capture/stack/policy 与 typed AttributeSet 对齐。 |
| E8 ability graph | Fail | graph/schema/task/cancel/cost/cooldown 编译可执行。 |
| E9 preview transaction | Fail | isolated world、rollback、clock、budget、cancel 和 receipt 闭合。 |
| E10 PIE/authority | Fail | Playtest 显式区分 editor/preview/server/client authority。 |
| E11 debug mirror | Fail | generation、loss、history、trace 与 runtime snapshot 一致。 |
| E12 prediction/replay UI | Fail | accept/reject/reconcile/rollback 可追踪并可导出。 |
| E13 network/save UI | Fail | replication/save artifact 可查询、比较、迁移。 |
| E14 job lifecycle | Fail | compile/preview/validation 支持 cancel、shutdown、failure artifact。 |
| E15 stale/conflict | Fail | 外部修改、stale selection、provider unload fail-closed。 |
| E16 reference search | Fail | tag/effect/ability/cue usage 结果带 revision/source location。 |
| E17 route boundary | Partial | retained route/control 底座存在，但 domain operation target 缺失。 |
| E18 shared editor substrate | Partial | document/undo/job/play 可复用，但尚未接 Gameplay owner。 |
| E19 scale | Fail | 大型 registry/graph/preview 的 P99 与内存上限未证明。 |
| E20 fault recovery | Fail | crash/reopen, compile fail, network loss, preview abort 无 last-good 恢复证据。 |
| E21 product truthfulness | Fail | 静态成功文案仍可在无 backend 时出现。 |
| E22 test evidence | Fail | 当前测试只覆盖 retained surface，不覆盖 Gameplay product path。 |

## 7. 实施顺序

先完成 ED-GAS-01/02/04/05/11/12，建立 provider、资源、document/operation、compiler、诊断和 LKG；再完成 ED-GAS-06/07/08/09/10/19，形成 graph/effect/attribute/tag semantic authoring；随后完成 ED-GAS-13/14/15/16/17/18，连接 PreviewWorld、PIE、runtime mirror、prediction/network/save 与 job lifecycle；最后移除 fixture、收紧 route namespace、加入 scale/fault/UI automation。Runtime175 的 domain artifact 与 receipt 是本报告的前置依赖。

本轮仅完成 Editor258 review/index/coverage 文档，没有修改 editor、runtime、tests、Cargo、ABI 或 ZUI；工作树中已有的 ZUI、bridge、asset/document/play 与 catalog 修改均保留并按当前边界评价。没有运行 Editor、Cargo、UI automation、PreviewWorld、PIE、save/reopen、fault、scale、soak 或 benchmark；按用户要求未查询、轮询、等待或实时跟踪协调器。
