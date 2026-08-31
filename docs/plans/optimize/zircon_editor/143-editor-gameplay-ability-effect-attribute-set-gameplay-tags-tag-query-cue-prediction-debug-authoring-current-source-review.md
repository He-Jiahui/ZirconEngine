---
title: Editor Gameplay Ability / Effect / AttributeSet / Gameplay Tags / Tag Query / Cue / Prediction / Debug Authoring 当前源码复审
category: zircon_editor
report_id: Editor143
review_date: 2026-08-26
baseline_head: 601472078e848164d2221967c55a77fea2452928
verification_head: 601472078e848164d2221967c55a77fea2452928
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/90-editor-gameplay-ability-effect-attribute-tag-query-cue-prediction-debug-authoring-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_runtime/99zz-runtime-gameplay-ability-effect-attribute-tag-query-attribute-set-aggregator-capture-execution-cooldown-cost-cue-targeting-task-prediction-replication-network-save-scalability-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/56-editor-search-filter-query-index-result-find-usage-reference-navigation-product-integration-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay
  - zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/document
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/play
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities
  - dev/UnrealEngine/Engine/Source/Runtime/GameplayTags
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/editor/debugger/editor_debugger_node.cpp
  - dev/bevy/crates/bevy_asset/src
  - dev/Graphics/Packages/com.unity.shadergraph/Editor
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor143 · Gameplay Ability Authoring 与 Prediction Debug 当前源码复审

## 1. 当前结论

当前工作树仍没有可交付的 Gameplay Ability Editor。Effect、Ability、Tags 三个入口是 retained UI 样机：三份 ZUI 共 **771 行、90 个 node、68 条 route、0 个业务 provider**，固定展示 `GE_HealthRegen`、`GE_DamageFire`、`GA_DashAttack`、`GE_DashAttack_Cost`、`DefaultGameplayTags.ini`、`Server Initiated`、`Ability.Activate` 与 `Character.State.Stunned`。Workbench 初始化仍显式选中 Effect；没有 Gameplay plugin/provider/runtime 时，用户仍会看到可 Save、Compile、Apply、Playtest、Add 与 Rename 的界面。

这些命令没有业务后端。`module_command_feedback.rs` 直接写入 `sample persisted`、`compile queued`、`changes compared`、`applied +50 health preview`、`predicted activation GA_DashAttack` 与 `pending registry/redirect update`；`module_field_edit.rs` 对 Change/Submit 只改 retained control 的 `value/value_text`。九个 generated-bottom row 只切换 selected control、drawer 和 route label，没有 data source、subscription、operation、job、revision fence 或 runtime reader。

产品装配也没有形成。first-party Editor catalog 只有 Navigation 与 Neural，App 只委托该 catalog；`ResourceKind` 的 26 类资源不含 Gameplay Tag、AttributeSet、Effect、Ability 或 Cue；生产源码对 `AbilitySystemComponent`、`GameplayAbilitySpec`、`ActiveGameplayEffect`、`AttributeSet`、`GameplayTagContainer`、`GameplayTagQuery`、`GameplayCue`、`PredictionKey` 的领域类型命中为 0。仓内没有 Gameplay Editor/Runtime package、asset toolkit、operation factory、semantic compiler、artifact、runtime consumer 或 debug provider。

`zircon_runtime::script::vm::gameplay_host` 是真实但不同层级的通用脚本桥。完整目录现有 **16 文件、2,900 行、15 项 test**，覆盖输入、变换、动态 component JSON、spawn/despawn、navigation、scene transition 和简单 HP 操作；`damage_entity`/`heal_entity` 直接改 `script.bindings[*].properties.hp`，实体参数是裸 `u64`，39 类 host callback 主要由宽泛的 `gameplay.entity` 授权。它没有 Definition/Spec、active effect handle、attribute aggregator/capture、tag container/query、activation/commit/end/cancel、authority/prediction/rollback 或 cue lifecycle，不能作为 Runtime151 或 Editor Gameplay 产品完成的证据。

因此 Editor21 的 5 项 canonical P0 仍为 **5 Open / 0 Partial / 0 Closed**；Editor90 的 60 项 P1 仍为 **48 Open / 12 Partial**，12 项 P2 全部 Open；32 项验收门仍为 **28 Fail / 4 Partial**。本轮没有因通用 document、transaction、job、play 或 event 底座存在而错误关闭领域项。目标链路必须是：

```text
versioned Tag / AttributeSet / Effect / Ability / Cue source assets
  -> transactional Gameplay authoring documents with stable identity
  -> one shared deterministic semantic compiler
  -> atomic GameplayBuildSetArtifact + diagnostics/source map/LKG
  -> qualified sandbox / PIE / client-server runtime sessions
  -> generation-qualified prediction/effect/cue trace
  -> provider-backed toolkits, migration, audit and debug products
```

本报告只做 review 与重构规划。MVP `00` 仍在进行且 F0-F5 被阻塞，Gameplay Ability 属于高级功能；本轮未改 production source、未运行 Cargo，也未查询、轮询或等待协调器状态。

## 2. 物理范围与证据等级

### 2.1 当前工作树扫描

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 结论 |
|---|---:|---|
| Gameplay product surface | **13 / 3,984 / 3,719 / 180,571 / 0 / 0** | 三份 ZUI、generated bottom、field/command/navigation/lifecycle 与 template binding |
| Gameplay focused tests | **4 / 2,013 / 1,893 / 72,803 / 23 / 0** | route、字符串、native input/pixel 与 control projection，不证明领域结果 |
| Catalog 与 resource boundary | **4 / 417 / 340 / 13,280 / 2 / 0** | first-party catalog/App composition 与 26 类 `ResourceKind` |
| Generic gameplay script host | **16 / 2,900 / 2,750 / 106,641 / 15 / 0** | host root、实现、nested tests；属于通用脚本/ECS 桥 |
| Unreal GAS/Tags 选定参考 | **20 / 16,975 / 13,773 / 702,425 / 0 / 0** | module、details、graph/schema、tag/query/migration、ASC/effect/prediction/cue/tag runtime |
| Fyrox/Godot/Bevy/Unity 补充参考 | **8 / 7,064 / 6,078 / 266,355 / 0 / 0** | command、undo/debugger、asset loader/event、graph data/undo/VFX model |

计数直接读取物理文件，包含工作树中的 tracked 修改，不以 Git index 覆盖。共享 Editor 的 `asset/document/editing/jobs/play/runtime_event_consumer` 目录也逐项检索了 Gameplay/Ability/Prediction/Cue/AttributeSet/TagQuery adapter，除通用 capability 文案外没有领域实现。

### 2.2 Currentness 与在途修改

冻结基线为 `601472078e848164d2221967c55a77fea2452928`。相关工作树已有其他 Session 或用户的修改：Effect ZUI/generated-bottom 移除了部分预置状态，`componentized_window.rs` 把默认 Effect 选择转入 live initialization，App composition 有重排，gameplay host lifecycle 增强了错误传播和测试。这些改动改善控件或脚本桥的局部工程性，但没有新增 Gameplay domain、provider、factory、artifact 或 trace，故不改变 canonical 状态。

本轮保留所有现有修改，没有覆盖、格式化或回退 production source。`source_recheck_required: true` 表示实施前必须重新冻结三份 Gameplay ZUI、feedback/field/bottom route、catalog/App、`ResourceKind`、script host 与 Runtime151 owner contract。

### 2.3 动态证据边界

按用户要求本轮只做 review，没有运行 Editor/App、asset create/import/save/reopen/cook、sandbox/PIE、client/server、prediction、cue、tag migration、fault/scale/soak/profile 或跨引擎 benchmark。23 项 focused test 只证明控件存在、route 可派发、固定文本会变化和输入可见；不能证明 source durability、compiler parity、runtime authority 或 prediction convergence。

## 3. 当前源码纵向事实

### 3.1 产品入口、catalog 与 asset type

1. Effect/Ability/Tags 是 builtin Workbench 的固定模块，不是 project selection 驱动的 asset toolkit 或插件贡献；初始化直接选择 Effect。
2. `zircon_plugins/first_party_editor_catalog` 仅声明 Navigation/Neural feature 和 registration；`zircon_app` 只做该 catalog 的窄转发。仓内没有 Gameplay package 或 registration。
3. `ResourceKind` 固定为 26 类，缺 Tag、AttributeSet、Effect、Ability、Cue；Content Browser 无 create/open/type/thumbnail/cook role。
4. shared asset type/toolkit registry、document lifecycle 与 operation factory infrastructure 可复用，但没有任何 Gameplay registration 或 adapter。

### 3.2 Surface、field、command 与 bottom truth

1. Ability 为 283 行/34 nodes/25 routes，Effect 为 303 行/35 nodes/28 routes，Tags 为 185 行/21 nodes/15 routes；业务 provider 均为 0。
2. Ability 的 graph/tasks/debug、Effect 的 duration/stack/modifier/capture、Tags 的 source/reference/conflict/redirect 都是固定 props，不是 domain projection。
3. Save/Compile/Diff/Simulate 按当前 UI module 枚举返回固定状态；未显式选择时以 Effect 作为 authority fallback。
4. Effect Apply 没有 target/spec/handle/delta/cue，Ability Playtest 没有 session/world/owner/spec/activation/prediction key，Tag Add/Rename 没有 registry 或文件 adapter。
5. Change 与 Submit 共用直接控件 mutation；没有 command precondition、transaction、dirty/savepoint、schema validation 或 accepted revision。
6. 九个 bottom row 只有 control ID、module/panel/mode label 和 route；点击只打开 drawer 并更新状态文本。

### 3.3 Tag、Attribute、Effect 与 Cue

1. 没有 Tag dictionary owner、source priority/writability、canonical segment、parent closure、redirect chain/cycle、dense network index/hash、container 或 query AST。
2. Reference Scan/Migration Preview 不启动 index/job/staging workspace；Add/Rename 不做冲突、权限、影响分析、journal、rollback 或 crash recovery。
3. 没有 AttributeSet schema、stable attribute ID、base/current/clamp/derived relation、replication/save metadata 或 migration。
4. Effect 没有 typed duration/period/stack/application/removal/immunity、magnitude discriminated model、capture snapshot/live、execution registry 或 deterministic artifact。
5. Cue 只有显示行，没有 asset/notify registry、OnActive/WhileActive/Executed/Removed、resource quality/server policy 或预测去重。

### 3.4 Ability、task、prediction 与 debug

1. 没有 Ability Definition/Spec、grant/instancing/group/activation/commit/end/cancel、cost/cooldown、tag requirement、target data 或 gameplay event payload。
2. Graph 没有 persistent document、stable node/pin/link、typed schema、cardinality/cycle/context validation、palette provider、copy/paste、undo、compiler 或 source map。
3. `Server Initiated` 是静态 dropdown；network capability、authority、安全与 rollback eligibility 不参与 admission。
4. `predicted activation` 没有 key/request/server receipt/accept/reject/catch-up/rollback/reconcile/terminal convergence。
5. generic Play Session、background job 与 runtime event consumer 是 Partial 底座，但没有 Gameplay session selector、consumer manifest、bounded journal、reader lease、gap receipt 或 source navigation。

### 3.5 Runtime script bridge 的准确边界

1. `zr.zircon.gameplay` 注册输入、scene transition、transform、dynamic component、query、combat、spawn/despawn、HUD 和 navigation callback。
2. `gameplay.entity` 同时授权读取、写 component、移动、伤害、治疗、spawn 与 despawn，粒度不足以表达项目/模块/authority/security policy。
3. Entity 从 script `Int` 转为裸 `u64`；当前 world contains check 不等于跨 teardown/reuse/network 的 generation-qualified identity。
4. HP 是动态 JSON property；直接减血并在零值移除 entity，不存在 aggregator、modifier channel、capture、immunity、effect handle、death policy 或 replication receipt。
5. 此桥应作为未来 Gameplay task/host adapter 的受限下层，不应继续扩大为第二套 Ability/Effect/Tag 真值。

### 3.6 测试真实性

1. focused tests 断言 `GE_HealthRegen`、`GA_DashAttack` 与固定 predicted output，实际固化了样机行为。
2. native input/pixel、momentary control 和 route projection 是 UI 基础证据，不能提升 Gameplay canonical 状态。
3. 缺 source round-trip、transaction/undo/recovery、compiler golden/determinism、sandbox teardown、PIE instance、network chaos、migration rollback、provider reload、大字典/大图和 slow-consumer 测试。

## 4. 参考引擎差异与采用边界

| 参考 | 当前源码事实 | Zircon 采用边界 |
|---|---|---|
| Unreal GameplayAbilitiesEditor | module 注册 Attribute/Effect details、ScalableFloat/capture/execution/cue customization、Ability graph/schema、factory、Cue editor/Sequencer 与引用入口 | 同域主参考；建立 runtime type 驱动的 asset/details/graph/audit/debug 分层，不复制 UObject/Slate |
| Unreal GameplayTagsEditor | 独立 Tag/Container/Query customization、picker/search、source/add/rename/cleanup；Query 使用 editable tree 与 commit/cancel，Cue/Tag 操作接 Asset Registry/transaction | Tag registry/query/migration 必须是真实数据产品，destructive workflow 必须有 currentness、transaction 与 rollback |
| Unreal Gameplay runtime | ASC 持有 spec/active effect/replication/prediction；Ability 明确 CanActivate/Try/Commit/Cancel/End；Effect 区分 spec/active instance；PredictionKey 绑定请求、reject/catch-up、rollback；Cue/Tags 有 lifecycle 与网络序列化 | Runtime151 必须先提供 Zircon 唯一 typed domain/build set；Editor 只消费共享 schema/compiler/trace，不复制规则 |
| Unity ShaderGraph/VFX Graph | persistent graph data、validation、controller/blackboard、undo object 与 version sanitize 分离 | 补 Ability/Effect expression graph 的 stable data、validation、undo 和 migration；不作为 Gameplay 语义来源 |
| Godot | EditorUndoRedoManager、Inspector 与 debugger node/plugin 分离 | 采用轻量基础服务分层；跨进程、generation、bounded transport 仍按 Zircon 更严格合同实现 |
| Fyrox | `CommandTrait` 明确 execute/revert，Inspector/asset preview 有 apply/revert 与插件扩展 | 作为 Rust command/property customization 下限；不能替代 shared semantic compiler |
| Bevy | typed AssetLoader、AssetId、AssetEvent 提供加载、身份与变化传播 | 作为资产生命周期下限；Bevy 没有同级首方 GAS Editor，不能为 Zircon 的缺失背书 |

Unreal 是本域唯一一等参考。其 prediction 文档本身也明确指出链式激活回滚存在限制，因此 Zircon 的“优于 Unreal”目标不能靠照搬接口命名；必须以 qualified identity、确定性 artifact、prepared publication、bounded trace、完整 rollback/fault/scale 证据建立可验证优势。

## 5. Owner 与架构边界

1. `zircon_runtime`/Runtime151 唯一拥有 Tag dictionary、Attribute/Effect/Ability/Task/Cue/Prediction/Replication 语义与 artifact；Editor 禁止复制 validator/compiler。
2. `zircon_editor` 拥有 source document、toolkit、transaction adapter、operation、preview/debug controller 和 projection；Workbench 只导航到同一 document/session authority。
3. `zircon_app` 只负责按 project/profile 组合 Runtime/Editor package，不拥有 Gameplay domain。
4. Gameplay 建议形成独立 first-party runtime/editor package，经现有 catalog/contribution/owner-generation 生命周期装配；不能把所有逻辑塞入 builtin Workbench callback。
5. `zircon_runtime::script::vm::gameplay_host` 保持通用脚本桥，未来只通过 typed runtime service handle 调用 Gameplay domain，并收紧 capability/authority。

## 6. Editor21 父 P0 当前重判

| Canonical owner | 状态 | 当前证据与硬切要求 |
|---|---|---|
| `Editor21-P0-01` 默认公开 Effect/Ability/Tags 但无 provider/runtime | Open | 默认 Effect 仍由 live initialization 强制选中；catalog、package、resource kind 与 runtime domain 均缺。M0 先显示 Unavailable。 |
| `Editor21-P0-02` Save/Compile/Diff/Simulate/Apply/Playtest 固定伪成功 | Open | production callback 仍直接返回 persisted/queued/compared/+50/predicted 文案。结果必须只来自 typed terminal receipt。 |
| `Editor21-P0-03` 字段只改 control 属性 | Open | Change/Submit 仍只写 `value/value_text`。必须接 document command、validation、transaction、dirty/save 与 revision。 |
| `Editor21-P0-04` 无网络/预测却宣称 Server/Predicted | Open | `Server Initiated` 与 predicted activation 仍是静态值，PredictionKey 领域类型为 0。Runtime151 前必须禁用。 |
| `Editor21-P0-05` Tag Add/Rename/Scan/Migration 是伪流程 | Open | Add/Rename 只写 pending；bottom row 只更新 route label。真实 registry/index/staging/atomic migration 前必须 Unavailable。 |

## 7. P1 当前源码差距账本

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| `ED-GAS-P1-001` | Open | first-party catalog/App 无 Gameplay package；新增 project-selectable runtime/editor registration、disable/revoke/reload/drain。 |
| `ED-GAS-P1-002` | Open | 无 Tag/AttributeSet/Effect/Ability/Cue asset type；定义 runtime-owned kind/schema/extension/toolkit/cook role/stable identity。 |
| `ED-GAS-P1-003` | Open | 无 create factory/template/naming transaction；先预检 destination/name/policy，再原子提交最小合法 versioned source。 |
| `ED-GAS-P1-004` | Partial | generic toolkit/document lifecycle 存在；仍缺五类 Gameplay toolkit、qualified target、view state、close/reopen。 |
| `ED-GAS-P1-005` | Partial | shared import flow 存在；仍缺 Gameplay adapter、hash/provenance/settings/dependency/reimport conflict/terminal receipt。 |
| `ED-GAS-P1-006` | Open | 无 AuthoringDocument stable field/list/node/pin/link ID、schema version、migration、base/local/accepted revision。 |
| `ED-GAS-P1-007` | Partial | shared transaction/operation gate 存在；Gameplay field/structure edit 未接 typed command、precondition、inverse 与 document scope。 |
| `ED-GAS-P1-008` | Partial | dirty/document/recovery 底座存在；Gameplay document 未注册，也无 recovery schema 或 late-result fence。 |
| `ED-GAS-P1-009` | Partial | save batch 可提供 ack 框架；缺 expected revision、filesystem/source-control receipt 与 artifact current/stale 状态。 |
| `ED-GAS-P1-010` | Open | 缺外部变更、checkout/read-only、multi-user、three-way semantic merge 和 stable-ID conflict staging。 |
| `ED-GAS-P1-011` | Open | Effect 无 shared typed schema/context-sensitive visibility；Details 必须由 runtime schema/validator 驱动。 |
| `ED-GAS-P1-012` | Open | Magnitude 仅静态值；定义 Constant/Scalable/AttributeBased/Custom/SetByCaller discriminated model 与 dependency。 |
| `ED-GAS-P1-013` | Open | Capture 缺 source/target、attribute stable ID、snapshot/live 和 dependency generation；非法 capture 阻止 publish。 |
| `ED-GAS-P1-014` | Open | Modifier 无 stable item ID、typed operator、排序/duplicate/multi-edit、provenance/override 与完整 undo。 |
| `ED-GAS-P1-015` | Open | Duration/Period 无 canonical unit、finite/range/time policy 与 Instant/Duration/Infinite 互斥验证。 |
| `ED-GAS-P1-016` | Open | Stacking 缺 aggregation key、limit/overflow、duration/period refresh、expiration 与 deterministic preview。 |
| `ED-GAS-P1-017` | Open | Application/removal requirement、ongoing inhibition、remove-other 与 immunity 缺 typed authoring/query/diagnostic。 |
| `ED-GAS-P1-018` | Open | Execution Calculation 无 runtime catalog、capture/parameter schema、side-effect/determinism/budget 与 owner generation。 |
| `ED-GAS-P1-019` | Open | Cue 无 asset/notify registry、四类 lifecycle、resource/quality/server policy 与 prediction dedupe preview。 |
| `ED-GAS-P1-020` | Open | Effect Details 无 property customization、conditional field、array item、mixed state 与 transactional batch edit。 |
| `ED-GAS-P1-021` | Open | Effect create 无 versioned template/profile、parent/composition/override、cycle 与 owner-revoke 检查。 |
| `ED-GAS-P1-022` | Open | 无 Tag/Attribute/Effect/Cue/curve/data dependency graph、revisioned usage index 与影响分析。 |
| `ED-GAS-P1-023` | Open | Compile 无 shared semantic compiler、deterministic artifact、diagnostic/source map、dependency hash 与 LKG。 |
| `ED-GAS-P1-024` | Open | Effect preview 无隔离 world/owner/attribute/tag/seed/time；Apply/Remove/Reset 无 handle/delta/cue/terminal receipt。 |
| `ED-GAS-P1-025` | Open | Effect 三类 bottom view 无 typed stream、revision、filter/export/source navigation、reader lease 与 budget。 |
| `ED-GAS-P1-026` | Open | Ability 无 Definition/Spec、grant/instancing/group、tag/input/event/net/security policy model。 |
| `ED-GAS-P1-027` | Open | Ability graph 是固定流程；建立 persistent stable-ID graph、selection、copy/paste/comment/find/viewport metadata/transaction。 |
| `ED-GAS-P1-028` | Open | Graph schema 缺 execution/data/event/target/attribute/tag pin type、cardinality、cycle、phase 与 provider validation。 |
| `ED-GAS-P1-029` | Open | Task palette 无 runtime extension catalog；需 descriptor/schema/owner generation、unavailable/reload/migration。 |
| `ED-GAS-P1-030` | Open | Activate/Commit/End/Cancel 无可视语义与编译规则；诊断重复 commit、无终点、未取消 latent task。 |
| `ED-GAS-P1-031` | Open | Cost/Cooldown 是固定资产名/4 秒；改为 effect picker、artifact generation、affordability/commit/cooldown trace。 |
| `ED-GAS-P1-032` | Open | Net execution/security 不看项目能力；只有 provider 与 rollback contract 合格时才能开放 predicted。 |
| `ED-GAS-P1-033` | Open | Ability tag 字段无统一 dictionary/picker/query generation、exact/container/query 区分和引用跳转。 |
| `ED-GAS-P1-034` | Open | Target Data 无 Entity/Location/Hit schema、range/LOS/team/filter、provider 与 server validation。 |
| `ED-GAS-P1-035` | Open | Gameplay Event 无 event/payload schema、producer-consumer reference graph 与 payload compatibility validation。 |
| `ED-GAS-P1-036` | Open | Timeline 不消费 latent task/sandbox/PIE trace 或 qualified time domain。 |
| `ED-GAS-P1-037` | Open | Cancel/interrupt/group/priority/replace/block 关系不可编辑，缺循环、并发与 cleanup validation。 |
| `ED-GAS-P1-038` | Open | Animation/Navigation/Physics/Audio/VFX task 无 broker handle、provider generation、cancel/reload/currentness。 |
| `ED-GAS-P1-039` | Partial | generic Play Session/runtime event host 存在；缺 session/world/owner/avatar/spec/activation selector 与 retire fence。 |
| `ED-GAS-P1-040` | Open | Prediction debug 无 key/dependency/request/server receipt/reject/catch-up/correction/rollback/reconcile。 |
| `ED-GAS-P1-041` | Open | Compile Log/Diff 无 source/artifact revision、node/pin/property location 与 base/local/remote semantic diff。 |
| `ED-GAS-P1-042` | Partial | generic Play Controller/job cancel 可复用；Gameplay Playtest 仍无 identity、timeout/failure/terminal/teardown。 |
| `ED-GAS-P1-043` | Open | Tags registry 无 provider/dictionary generation 与 name/parent/source/owner/reference/status projection。 |
| `ED-GAS-P1-044` | Open | Tag source 缺 project/plugin/native/generated priority、path/format/owner/read-only/load/error 与 write adapter。 |
| `ED-GAS-P1-045` | Open | Tag hierarchy 无 parent closure、implicit parent、breadcrumb、stable selection、lazy tree 与 virtualization。 |
| `ED-GAS-P1-046` | Open | Tag search/filter 无 typed query、source/owner/status/usage 维度、paged result、cancel/stale receipt。 |
| `ED-GAS-P1-047` | Open | Add Tag 无 canonicalization、duplicate/case/parent/permission/depth/concurrent preflight 与 transaction。 |
| `ED-GAS-P1-048` | Open | Rename 无 redirect chain/cycle/conflict/source permission/reference impact、atomic commit 与 rollback receipt。 |
| `ED-GAS-P1-049` | Open | Delete/Cleanup 无完整 reference index、native/indirect/generated 分类、dry-run/backup/recovery。 |
| `ED-GAS-P1-050` | Open | Reference Scan 无 source/scene/config/generated provider、progress/cancel/index generation、定位与导出。 |
| `ED-GAS-P1-051` | Open | Migration 无 per-file staging/diff/conflict/cook impact、durable journal、atomic apply 与 rollback。 |
| `ED-GAS-P1-052` | Open | 无 Tag Query Editor；需 Any/All/None/Exact AST、drag/reorder、budget、text/graph 双视图与 runtime preview。 |
| `ED-GAS-P1-053` | Open | Tags Compile Log 无 parse/redirect/closure/dense index/network dictionary/recook diagnostic 与 source location。 |
| `ED-GAS-P1-054` | Open | 无 network dictionary hash/index/bit width/redirect generation/target build compatibility；mismatch 应阻止 playtest。 |
| `ED-GAS-P1-055` | Partial | 九类 route/mode/drawer lifecycle 存在；缺 typed provider/subscription/currentness/filter/export/slow-consumer。 |
| `ED-GAS-P1-056` | Partial | scheduler 有 admission/progress/cancel/shutdown；compile/scan/migration/audit/playtest 未提交 revision-fenced job。 |
| `ED-GAS-P1-057` | Partial | workspace route/control state 存在；active owner 仍由 selected checkbox/fallback Effect 推断，未统一 document/session。 |
| `ED-GAS-P1-058` | Partial | native input/focus/momentary control 有基础；复杂 graph/tree/table/dialog、screen reader、locale、numeric storage 未达标。 |
| `ED-GAS-P1-059` | Open | 大 dictionary/graph/list/diagnostic/trace 无 paging/delta/virtualization 与 entries/bytes/time/retention budget。 |
| `ED-GAS-P1-060` | Partial | 23 项 focused test 证明 UI，但固化固定业务文案；需 schema/transaction/compiler/provider/fault/scale 产品测试。 |

## 8. P2 高阶能力

| ID | 状态 | 目标 |
|---|---|---|
| `ED-GAS-P2-001` | Open | Magnitude/Execution expression graph、常量折叠、单位/范围、复杂度与循环分析。 |
| `ED-GAS-P2-002` | Open | Ability set/combo/input window/hold-release-chord/subgraph/template/versioned fragment。 |
| `ED-GAS-P2-003` | Open | 多客户端 predicted/server/simulated proxy 对齐时间轴、packet/receipt/correction/convergence。 |
| `ED-GAS-P2-004` | Open | 跨项目/plugin Tag migration、headless CI audit、policy gate、signed receipt 与恢复。 |
| `ED-GAS-P2-005` | Open | stable-ID three-way semantic diff/merge 与 graph conflict resolution。 |
| `ED-GAS-P2-006` | Open | Attribute/Effect curve/data registry、level sweep、敏感性/范围与 artifact-bound preview。 |
| `ED-GAS-P2-007` | Open | Cue Sequencer、VFX/Audio track、pool/concurrency/distance/quality/server 预算预览。 |
| `ED-GAS-P2-008` | Open | 批量表格、data-driven generation、template instance 与 schema migration 工具。 |
| `ED-GAS-P2-009` | Open | 团队 permission/review/approval、actor/scope audit 与 destructive rollback。 |
| `ED-GAS-P2-010` | Open | Ability/Effect/TagQuery/Task/Cue CPU、内存、带宽、rollback 成本 profiler。 |
| `ED-GAS-P2-011` | Open | activation contradiction、effect cycle、infinite stack/period、unreachable task 与数值爆炸 audit。 |
| `ED-GAS-P2-012` | Open | 第三方 property/graph/task/query/cue ecosystem 的 versioned schema、lease、sandbox/reload/compat admission。 |

## 9. 分层重构顺序

### M0：产品真相硬切

无 Gameplay runtime/editor provider 时显示 Unavailable；删除 fixed Save/Compile/Diff/Apply/Playtest/Add/Rename success 与默认 Effect authority。保留 momentary-control 修正，但不能把它算成领域完成。

### M1：Runtime151 shared domain 与 build set

先完成 Tag dictionary/query、AttributeSet/aggregator、Effect definition/spec/active instance、Ability definition/spec/task、Cue、Prediction/Replication 的唯一 schema/compiler/runtime service；产出 generation-qualified deterministic GameplayBuildSetArtifact、diagnostic/source map 与 LKG。

### M2：Package、asset 与 document foundation

建立 first-party Gameplay runtime/editor package、catalog/App composition、resource kind/source format、factory/importer/toolkit；接 shared document/transaction/dirty/save/autosave/recovery/source-control 与 stable identity。

### M3：Tag Registry、Query 与 migration

交付多 source dictionary、hierarchy/picker/search/query、reference index、add/rename/delete/cleanup、staging workspace、durable journal、atomic migration/rollback 和 network dictionary compatibility。

### M4：AttributeSet 与 Effect toolkit

完成 typed Attribute schema、Effect details/magnitude/capture/stack/execution/immunity/cue authoring、dependency graph、shared compile 与 isolated deterministic preview。

### M5：Ability graph、task 与 policy

完成 stable-ID graph/schema/palette、Definition/Spec、activation/commit/end/cancel、cost/cooldown、tag/target/event、cross-system task broker、net/security admission 与 compiler/source map。

### M6：Qualified playtest 与 prediction debug

用 sandbox/PIE/client-server session 建立 owner/avatar/spec/activation identity、request/server receipt/reject/catch-up/rollback/reconcile、cue dedupe、timeout/terminal/teardown 与 revision fence。

### M7：Provider-backed bottom views 与规模资格

九类 panel 消费 typed operation/trace stream，具有 reader lease、gap/overflow、filter/export/source jump、slow-consumer 和 retention budget；Tag/graph/list/trace 完成 paging/delta/virtualization。

### M8：Fault、soak 与竞争验收

覆盖 source corruption、compile/disk failure、job cancel、late result、plugin reload、world/session teardown、network loss/reorder、dictionary mismatch 和 recovery。最后使用同内容、场景、seed、硬件、构建与质量设置与 Unreal 对比 authoring latency、runtime correctness、CPU/内存/带宽及 prediction convergence。

## 10. 验收门禁

| Gate | 状态 | 验收条件 |
|---|---|---|
| `G-01` | Fail | 无 provider/runtime 时只显示 Unavailable，不输出 success/queued。 |
| `G-02` | Fail | App 按 project selection 装配、禁用、reload Gameplay Editor。 |
| `G-03` | Fail | Tag/AttributeSet/Effect/Ability/Cue 可 Create/Open/Save/Reopen。 |
| `G-04` | Partial | shared transaction 存在；Gameplay edit 尚未接 undo/redo/dirty/recovery。 |
| `G-05` | Partial | shared dirty/save 底座存在；Gameplay 无 durability/source revision receipt。 |
| `G-06` | Fail | Editor/runtime/cook 使用同一 schema/compiler/artifact hash。 |
| `G-07` | Fail | Tag source/hierarchy/redirect/container/query/dense index 与 Runtime151 一致。 |
| `G-08` | Fail | Add/Rename/Delete/Cleanup/Scan/Migration 覆盖冲突与 rollback。 |
| `G-09` | Fail | 大 registry tree/search/picker/reference result virtualized 且有预算。 |
| `G-10` | Fail | Attribute schema/default/clamp/metadata/stable ID/reload migration 通过。 |
| `G-11` | Fail | 非法 duration/stack/modifier/capture/execution 组合由同源 schema 拒绝。 |
| `G-12` | Fail | Effect Compile 产生真实 diagnostic/dependency/hash/generation/LKG。 |
| `G-13` | Fail | Effect sandbox Apply/Remove/Reset 产生真实 handle/delta/tag/cue。 |
| `G-14` | Fail | Effect bottom views 只显示 revision-bound 真实数据并可定位 source。 |
| `G-15` | Fail | Ability graph stable ID、typed edge、copy/paste、undo、compiler/source map 通过。 |
| `G-16` | Fail | Task palette 来自 runtime extension，处理 missing/revoke/reload/migration。 |
| `G-17` | Fail | CanActivate/Try/Commit/End/Cancel、cost/cooldown/tag 与 runtime 一致。 |
| `G-18` | Fail | typed target/payload/server validation 与引用关系可编译、预览。 |
| `G-19` | Fail | cross-system task 的 broker handle/cancel/provider generation 正确。 |
| `G-20` | Fail | playtest 有 qualified session/owner/activation、cancel/timeout/terminal/teardown。 |
| `G-21` | Fail | prediction 有真实 key/request/receipt/reject/rollback/reconcile 并最终收敛。 |
| `G-22` | Fail | 网络/rollback 能力不足时 Predicted/Server 选项禁用。 |
| `G-23` | Fail | Cue 四类 lifecycle、资源、quality/server policy 与预测去重通过。 |
| `G-24` | Fail | 九类 Gameplay view 有 typed data source/subscription/slow-consumer 策略。 |
| `G-25` | Partial | shared scheduler 存在；Gameplay 长操作尚未提交 job 或验证 shutdown。 |
| `G-26` | Fail | late result 不能覆盖新 document/artifact/world generation。 |
| `G-27` | Fail | owner revoke 后 toolkit/task/provider/trace 安全 retire。 |
| `G-28` | Fail | production 不含 sample persisted、+50 health 或固定 predicted success。 |
| `G-29` | Partial | 基础 focus/input 存在；复杂控件和 locale 未完成。 |
| `G-30` | Fail | missing source/provider、compile/disk/runtime/network/project close 诚实终结。 |
| `G-31` | Fail | Windows Editor、client/server、目标平台证据绑定 source/build/artifact。 |
| `G-32` | Fail | 同内容/质量/硬件/网络的可复现实验达阈值后才可声称优于 Unreal。 |

## 11. 禁止的临时修补

1. 禁止只把固定业务文案改得更像真实日志，或注册永远成功的空 operation factory。
2. 禁止把 `ResourceKind::Data`、动态 JSON 或 `script.bindings.hp` 重新命名为 Ability/Effect/AttributeSet 后宣称领域完成。
3. 禁止 Editor 与 Runtime 各自维护 Tag、Effect、Ability 或 prediction validator/compiler。
4. 禁止把 ZUI node/route、selected state、drawer row、focused test 数量算成 provider/toolkit/domain 完成。
5. 禁止让 Workbench checkbox、固定 fallback Effect 或显示文本决定 active document/session authority。
6. 禁止用裸 `u64` entity、无 generation 的 handle 或宽泛 `gameplay.entity` 承载安全敏感 Gameplay 操作。
7. 禁止在无 network provider/rollback contract 时暴露 Predicted、Server Initiated 或成功 convergence 文案。
8. 禁止 Tag rename/delete/migration 在无完整 reference index、staging、journal、rollback 时可执行。
9. 禁止 preview 直接修改 authoring World，或让 sandbox/PIE/shipping 使用不同 schema/compiler/artifact。
10. 禁止无限 trace/list、无 reader lease/gap receipt 的 debug stream，或每帧全量重建大 Tag/graph projection。
11. 禁止以 Unreal 的接口数量作为完成标准；正确性、故障、规模、soak 和同条件基准必须可复现。
12. 禁止在 MVP gate 未开放时绕过依赖实施大规模高级功能；本报告当前只提供可执行架构顺序。

## 12. 本轮产出与实施前置

本轮新增 Editor143 current-source review，并更新 Editor 索引、根索引和覆盖矩阵；未修改 Runtime、Editor、App、plugin 或 tests。未发现适用于本域的 `failure-*.md` handoff，未创建或关闭跨计划失败记录。tooling 不在当前目标内；协调器状态按用户要求未查询、未轮询、未等待。

实施从 M0 开始，但 M1 的 Runtime151 shared domain 是后续 Editor 产品闭环的硬前置。所有实现 Session 必须重新冻结 current HEAD、相关工作树与 owner 计划，并逐门提交 Windows Editor/client-server/current-artifact 证据；在 G-32 之前不得声称性能或表现优于 Unreal。
