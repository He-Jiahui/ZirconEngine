---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay
  - zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
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
  - zircon_runtime/src/script/vm/gameplay_host
plan_sources:
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_runtime/99zz-runtime-gameplay-ability-effect-attribute-tag-query-attribute-set-aggregator-capture-execution-cooldown-cost-cue-targeting-task-prediction-replication-network-save-scalability-editor-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities
  - dev/UnrealEngine/Engine/Source/Runtime/GameplayTags
  - dev/Graphics/Packages/com.unity.shadergraph/Editor
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor
  - dev/godot/editor
  - dev/Fyrox/editor/src
  - dev/bevy/crates/bevy_asset/src
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
review_id: Editor90
---

# 90 · Editor Gameplay Ability / Effect / Attribute / Tag Query / Cue / Prediction / Debug Authoring 当前源码工程化差距

## 1. 当前结论

冻结于 `2026-08-25T17:44:19+08:00`、HEAD `8ee9411db24b7b4bdaf3fe028194642a7557c0b6` 的当前物理工作树仍没有可交付的 Gameplay Ability Editor。当前能看到的 Effect、Ability、Tags 三个模块是完成度较高的 retained UI 样机，不是由 Gameplay domain、资产文档、语义编译器或运行时会话驱动的工程产品。

三份 Gameplay ZUI 合计 **771 行、90 个 node、68 条 route、0 个 provider**。它们固定展示 `GE_HealthRegen`、`GE_DamageFire`、`GA_DashAttack`、`GE_DashAttack_Cost`、`DefaultGameplayTags.ini`、`Server Initiated`、`+50 health`、`Ability.Activate` 与 `Character.State.Stunned`。`componentized_window.rs` 在初始化时仍显式选中 Effect，并调用 `apply_workbench_module_workspace("workbench.module.effect.select")`；当前在途改动只是把 ZUI 中预置的 checked/selected 状态移到 live-control 初始化，并把命令按钮改为 momentary control，没有改变默认产品真相。

所有业务动作仍由 `module_command_feedback.rs` 写固定结果。Save 写入 `sample persisted`，Compile 写入 `compile queued`，Diff 写入 `changes compared`，Effect Apply 写入 `applied +50 health preview`，Ability Playtest 写入 `predicted activation GA_DashAttack`，Tag Add/Rename 只写 `pending registry/redirect update`。字段 Change/Submit 经 `module_field_edit.rs` 只修改控件 `value/value_text` 并刷新 surface，不创建 document command，不进入 transaction/dirty/save，也不做任何 Gameplay schema validation。

共享 Editor 已有真实可复用底座：asset type/toolkit registry、document lifecycle、transaction/dirty registry、operation factory、background jobs、Play Session、runtime plugin event consumer。这些只能将部分重构项判为 Partial，不能把 Gameplay 产品判为存在。`ResourceKind` 的 26 类资源不含 Tag、AttributeSet、Effect、Ability、Cue；builtin asset toolkit 只为 UI 与 Animation 资源提供专用打开路径；first-party Editor catalog 仍只有 Navigation 与 Neural；生产源码中 `AbilitySystemComponent`、`GameplayAbilitySpec`、`ActiveGameplayEffect`、`AttributeSet`、`GameplayTagContainer`、`GameplayTagQuery`、`GameplayCue`、`PredictionKey` 精确命中总数仍为 **0**。

因此，Editor21 的 5 项父 P0 当前仍为 **5 Open / 0 Partial / 0 Closed**。本报告不重复新增 P0；60 项 canonical P1 重判为 **48 Open / 12 Partial**，12 项 P2 均为 Open；32 项验收门为 **28 Fail / 4 Partial**。目标链路必须收敛为：

```text
versioned Tag / AttributeSet / Effect / Ability / Cue source assets
  -> transactional Gameplay authoring documents
  -> shared deterministic semantic compiler
  -> atomic GameplayBuildSetArtifact + diagnostics/source map
  -> sandbox / PIE / network-qualified runtime sessions
  -> generation-qualified trace, prediction and cue projections
  -> real Editor toolkits, tag migration and debugging products
```

## 2. 当前物理范围与证据边界

### 2.1 冻结范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 当前证据 |
|---|---:|---|
| Gameplay Workbench surface | **7 / 1,913 / 1,678 / 116,900 / 0 / 0** | 三份Gameplay ZUI、模块组合、generated bottom与toolbar；fingerprint `36eeb30c1fad7ca4edfb72ee7cb691177d2be5da7ce4eae7ed1114256ae8dff0` |
| Retained route与binding | **12 / 3,409 / 3,308 / 141,204 / 2 / 0** | preview action registry、module/extension field/feedback/navigation及bottom route；fingerprint `c511edd9d42ef7960f91ecb87d90cbdd95690a020ea2e12d68ab5a1723249cd2` |
| Gameplay focused surface tests | **4 / 2,013 / 1,893 / 72,803 / 23 / 0** | 路由、字段、fixed feedback、native input/pixel与projection；fingerprint `aeeb10cd3f71b3310de574664bfba622b76fd059acd9d3c5c345fd59913c251d` |
| 共享Editor基础选择集 | **27 / 7,030 / 6,337 / 237,976 / 21 / 1** | asset/toolkit、document、transaction/dirty、operation、job、play与runtime event；fingerprint `362e9f5de5b8977923dba577e0a4473abd995081e84bb4f6b4e19294c4c41525` |
| Catalog与asset边界 | **6 / 592 / 493 / 19,419 / 8 / 0** | App委托、first-party Editor catalog与26类ResourceKind；fingerprint `55a616e4adc5018873869eec25202a48f0387b749df202d624224060577cea09` |
| Generic gameplay script host | **11 / 1,803 / 1,696 / 70,608 / 5 / 0** | combat/component/input/lifecycle/navigation/scene/transform/value callbacks；fingerprint `6df9c0ab1771adea0691310749db314eeb67d81b391df66da42cb3aeb699b6c0` |
| Unreal Gameplay Editor参考 | **44 / 22,001 / 18,014 / 882,183 / 1 / 0** | Ability/Effect/Attribute/Cue/Tag/Query authoring、runtime contract与query tests；fingerprint `91d3edcdb466296f3885a2ef204f880ef4f6513c3a224cb5fb61c40cbc9170a0` |
| Unity Graphics、Godot、Fyrox、Bevy参考 | **14 / 17,109 / 14,732 / 658,416 / 2 / 0** | graph data/validation/undo、inspector/debugger、command和asset loader；fingerprint `14c3b2eacea1b53da211f2d0f0931871b5ea4913028471a7991270e181987c65` |

Zircon selected 合计 **67 个文件、16,760 行、15,405 非空行、658,910 bytes、59 项 test declaration、1 项 ignored**。参考选择集合计 **58 个文件、39,110 行、32,746 非空行、1,540,599 bytes、3 项 test declaration**，aggregate fingerprint 为 `746742b837351c133a32f3acca339cd0ecb60805da9656a566b71aedb46ba610`。计数读取物理文件；fingerprint 按 repo-relative path 与 bytes 计算。

### 2.2 Currentness 与并发工作树

selected 范围存在其他 Session/用户的在途修改：Effect Apply、toolbar 与 generated-bottom Open 的预置 selected/checked 被移除；module/extension 命令改为 momentary control；初始化选择状态转入 bridge；generic gameplay host lifecycle 增加错误传播调整。这些变化改善控件状态语义，但没有新增 Gameplay domain、provider、operation factory、artifact或runtime trace，故不关闭任何父 P0。

当前工作树广泛 dirty，本轮没有覆盖、格式化或回退任何 production source。由于 selected 源码仍可能变化，实施 Session 必须重新冻结三份 Gameplay ZUI、`module_command_feedback.rs`、`module_field_edit.rs`、`componentized_window.rs`、generated-bottom routes、catalog、`ResourceKind` 与 Runtime151 前置合同。

### 2.3 动态证据边界

按用户要求本轮只做 review，没有运行 Cargo、Editor/App、asset create/import/save/reopen/cook、sandbox/PIE、client/server、prediction、cue、tag migration、fault/scale/soak/profile或跨引擎 benchmark。现有 23 项 focused test 主要证明控件可投影、route 可派发、字符串会变化、native input/pixel 可见；它们不证明任何 Gameplay 资产持久化、编译或运行时结果。

## 3. 当前实现的纵向事实

### 3.1 产品入口、catalog与ownership

1. Effect/Ability/Tags 是 builtin Workbench 的固定顶栏模块，不是 asset toolkit 或插件贡献。初始化代码直接选择 Effect，因此没有 provider 时产品仍展示可编辑、可保存、可编译的假象。
2. `zircon_plugins/first_party_editor_catalog` 的 feature、dependency 与 registration map 只有 Navigation/Neural；`zircon_app` 仅委托这一 catalog。仓内也没有 Gameplay/Ability Editor plugin package。
3. `ResourceKind` 与 builtin registry 不含 Tag、AttributeSet、Effect、Ability、Cue。三模块无法通过 Content Browser create/open，也没有 source extension、importer、thumbnail、toolkit或cook role。
4. generic gameplay script host 只是 39 类脚本 callback 的承载面，处理 entity/component/HP/input/navigation/lifecycle 等动态操作；它不提供 GameplayBuildSet、Tag dictionary、effect spec、ability activation或prediction contract。

### 3.2 Surface、field与command truth

1. 三份 Gameplay ZUI 分别为 Ability 283 行/34 nodes/25 routes、Effect 303 行/35 nodes/28 routes、Tags 185 行/21 nodes/15 routes，业务 provider 均为0。
2. Ability 的 Graph、Tasks、Debug tab 只切换 selected control；流程固定为 `Activate -> Cost -> Montage -> Damage -> End`，Timeline 固定为 `1.22s Ability Activated OK`。
3. Effect 的 duration/period/policy/modifier/capture/dependency graph/attribute preview 全是 ZUI props；Apply 不构造 target、spec、effect handle、attribute delta或cue receipt。
4. Tags 的 registry/source/reference count/conflict/validation/redirect/owner 全是固定行；search 只是 field value，Add/Rename 不调用 registry 或文件适配器。
5. `module_field_edit.rs` 对 edit/commit 使用同一条直接控件 mutation 路径；Submit 没有比 Change 多任何 validation、transaction、savepoint或acknowledgement。
6. `module_command_feedback.rs` 按当前 selected module 枚举固定字符串；当没有明确选择时回退 Effect。业务 owner 由 UI checkbox 猜测，而不是 active document/toolkit/session identity。

### 3.3 Asset、document、transaction与compiler

1. shared asset registry、toolkit route、document lifecycle、transaction engine、dirty registry与save batch是真实底座，应复用而非重写。
2. Gameplay 没有任何 versioned source DTO、schema version、stable field/node identity、unknown-field policy、dependency/provenance、migration或artifact hash。
3. 没有 Gameplay create/import/reimport/open operation factory；也没有 AttributeSet、Effect、Ability graph、Cue或Tag Query document controller。
4. Save/Compile/Diff/Simulate 等 action 没有 operation request/receipt、job ticket、source revision fence或terminal outcome。共享 operation/job 系统尚未被三模块消费。
5. Runtime151 已确认 production domain 类型为0，因此 Editor 无法调用共享 semantic compiler。当前 Effect/Ability/Tag 规则只能是展示字符串，不存在 Editor/runtime/cook parity。

### 3.4 Gameplay Tags、Query与migration

1. 没有 dictionary owner、source adapter、project/plugin/native/generated priority、read-only/source-control state或dictionary generation。
2. 没有 canonical segment validation、parent closure、redirect chain/cycle、dense network index/hash、container或query AST。
3. Reference Scan 与 Migration Preview 是 generated-bottom 的 route metadata；点击只更新 selected row、module/panel/mode文本并打开 drawer，不启动扫描或 staging workspace。
4. Add/Rename 没有影响分析、冲突预检、跨文档事务、durable journal、rollback receipt或crash recovery。此类 destructive workflow 当前必须 disabled/unavailable。

### 3.5 Attribute、Effect与Cue authoring

1. 没有 AttributeSet schema、attribute stable ID、base/current value、clamp、derived relation、replication/save metadata或schema migration。
2. Effect 没有 typed duration/period/stack/application/removal/immunity model；ZUI 不能表达 Unreal 参考中的 context-sensitive property visibility。
3. Magnitude 没有 ScalableFloat、AttributeBased、CustomCalculation、SetByCaller 等 discriminated model；capture 没有 source/target、snapshot/live或dependency currentness。
4. Execution 没有 calculation registry、valid capture definition、conditional effect、passed-in tag、预算或owner generation。
5. Gameplay Cue 只有 `Gameplay Cues` 行，没有 cue asset、notify type、OnActive/WhileActive/Executed/Removed lifecycle、VFX/Audio/Animation引用、预测去重或dedicated-server policy。
6. Effect preview 没有隔离 sandbox/world、seed/time/level/context、apply/remove handle、reset/teardown或可复现 trace。

### 3.6 Ability graph、task与network debug

1. Ability graph 没有 document、stable node/pin/link ID、schema、cardinality/cycle/context validation、palette provider、copy/paste、undo、compiler或source map。
2. Ability definition/spec、grant/instancing/activation/commit/end/cancel、cost/cooldown、owned/required/blocked tags、target data与gameplay event payload均无 typed model。
3. `Server Initiated` 是静态 dropdown 文本；runtime/network capability 不参与 option admission。没有 authority、owner connection、security policy或rollback eligibility。
4. Playtest 的 `predicted activation` 不含 Play Session、World、owner/avatar、spec、activation generation、PredictionKey、client request、server receipt、reject/catch-up、rollback/reconcile或终态。
5. generic Play Session 与 runtime event consumer 可作为将来底座，但当前没有 Gameplay consumer manifest、trace event、reader lease、bounded history、gap receipt或debug controller。

### 3.7 Generated bottom、jobs与测试真实性

1. 九类 Gameplay row 只有 route、control ID、module/panel/mode label。没有 data source、subscription、typed item identity、source/artifact/session generation、filter/export或定位 source。
2. generated-bottom lifecycle 只控制 drawer/panel visibility；隐藏/关闭没有释放 Gameplay reader，因为根本不存在 reader。
3. shared jobs 支持 admission/progress/cancel/shutdown 等通用能力，但 compile/reference scan/migration/audit/playtest 均未提交 Gameplay job。
4. focused tests把 `GA_DashAttack` 编辑成 preview 字符串、断言 fixed predicted output、检查 route 与像素。新 momentary-control test只证明按钮不保持selected/checked，不证明命令业务真实。
5. 没有 source round-trip、transaction/undo/recovery、compiler golden、artifact determinism、sandbox teardown、PIE instance、network chaos、tag migration rollback、provider reload、大字典/大图或slow-consumer测试。

## 4. 参考引擎差异与采用边界

| 参考 | 已核对的工程事实 | Zircon必须吸收的边界 |
|---|---|---|
| Unreal GameplayAbilitiesEditor | 独立Editor module依赖AssetTools、PropertyEditor、GraphEditor、GameplayTags、ContentBrowser、SourceControl、Sequencer；注册Ability asset/factory、Effect/Attribute/Magnitude/Execution details、graph pin/schema、audit与Cue editor/debug callbacks | 本域主参考；吸收runtime type驱动的asset/detail/graph/audit/debug职责分离，不复制UObject/Slate实现 |
| Unreal GameplayTagsEditor | 独立asset definition、tag/container/query customization、picker/search、source/add/rename/cleanup widget；Query编辑使用editable tree、transaction、commit/cancel；cleanup等待Asset Registry并显示source/unused结果 | Tag registry/query/migration必须是真实数据产品；引用扫描、重命名、清理需要source/currentness/transaction，而不是静态row |
| Unreal Gameplay runtime | ASC、spec、ability、effect、attribute、prediction、cue manager与tag manager提供Editor可绑定的同源类型和调试回调 | Zircon必须先完成Runtime151的typed owner/build set；Editor不得私建第二套语义 |
| Unity ShaderGraph / VFX Graph | persistent GraphData、graph validation、controller/blackboard、compiled data、undo cursor与validation controller分离 | 作为Ability/Effect表达式图共享authoring架构参考；领域规则仍由Gameplay shared compiler定义 |
| Godot | UndoRedo、Inspector与EditorDebugger是分离的基础服务，调试数据不由Inspector静态文本冒充 | 复用轻量事务/检查器/调试器分层；不降低generation、bounded transport和跨进程PIE要求 |
| Fyrox | Editor command有execute/revert边界，Inspector由类型编辑器/handler扩展 | 用于Rust typed command与property customization下限；不能替代Gameplay semantic compiler |
| Bevy | typed Assets/AssetLoader与asset event提供身份、加载和变化传播 | 用作资产生命周期下限；Bevy没有同级首方GAS Editor，不能证明authoring缺失是合理设计 |

Unreal 是 Gameplay Ability/Tags/Cue Editor 的首要参考；Unity Graphics 只补图数据、validation、undo/controller；Godot/Fyrox/Bevy只补共享 Editor 基础。Zircon应保留 Rust ownership、generation identity、prepared publication、bounded transport与跨进程 runtime gateway优势，不复制UObject指针、全局单例或Slate具体控件。

## 5. Editor21 父 P0 当前重判

| Canonical owner | 状态 | 当前证据与硬切要求 |
|---|---|---|
| `Editor21-P0-01` 默认公开Effect/Ability/Tags但无Gameplay provider/runtime | Open | live initialization仍显式选择Effect；catalog无Gameplay，ResourceKind无领域资产。M0先显示Unavailable，真实plugin/toolkit/runtime装配后再开放。 |
| `Editor21-P0-02` Save/Compile/Diff/Simulate/Apply/Playtest固定伪成功 | Open | `module_command_feedback.rs`仍含sample persisted、queued、+50 health和predicted activation。删除固定业务成功分支，结果只来自typed operation receipt。 |
| `Editor21-P0-03` 字段只改control属性 | Open | `module_field_edit.rs`仍只写`value/value_text`并refresh。引入Gameplay document command、validation、transaction、dirty/save与accepted revision projection。 |
| `Editor21-P0-04` 无network/prediction却宣称Server/Predicted | Open | `Server Initiated`与predicted输出仍为静态字符串，精确PredictionKey类型0命中。Runtime151网络门禁前禁用并解释。 |
| `Editor21-P0-05` Tag Add/Rename/Scan/Migration伪流程 | Open | Add/Rename仅pending文本，bottom row仅route metadata。建立registry/index/staging/atomic migration/rollback前必须Unavailable。 |

## 6. P1 当前源码差距账本

### `ED-GAS-P1-001` [Open] first-party Editor catalog没有Gameplay入口

新增独立Gameplay Editor package、feature/dependency/registration与project selection；App只通过统一catalog装配，支持disable/revoke/reload和state drain。

### `ED-GAS-P1-002` [Open] 没有Tag、AttributeSet、Effect、Ability和Cue asset type

扩展runtime唯一resource kind/source schema和Editor asset registry，定义extension、presentation、thumbnail、toolkit、cook role与stable identity。

### `ED-GAS-P1-003` [Open] 没有Create factory、template和命名事务

Create先做destination/name/source-policy预检，生成versioned最小合法文档；commit失败不得留下半文件、registry残片或假打开tab。

### `ED-GAS-P1-004` [Partial] 没有Gameplay Open toolkit与document session

generic toolkit route和document lifecycle已存在；仍需五类领域toolkit、qualified document target、selection/view state及close/reopen生命周期。

### `ED-GAS-P1-005` [Partial] 没有Gameplay import/reimport、provenance和settings合同

共享asset import flow可复用；Gameplay尚无source adapter、content hash、settings revision、dependency、reimport conflict或cancel/terminal receipt。

### `ED-GAS-P1-006` [Open] Gameplay AuthoringDocument没有stable field/node identity与source revision

定义versioned source DTO、stable field/list/node/pin/link ID、unknown-field policy、migration和base/local/accepted revision。

### `ED-GAS-P1-007` [Partial] Gameplay编辑没有transactional command与undo/redo

通用transaction/operation gate已存在，但三模块字段不消费它。所有结构/属性编辑必须经typed command、precondition、inverse/restore与document scope。

### `ED-GAS-P1-008` [Partial] Gameplay文档未接dirty、autosave、recovery和close decision

shared dirty registry/document lifecycle是真实底座；Gameplay document尚未注册，也没有recovery schema和late-result fence。

### `ED-GAS-P1-009` [Partial] Save没有领域durability acknowledgement与artifact状态

共享save batch可提供ack框架；仍需expected source revision、filesystem/source-control结果、artifact stale/current状态和失败恢复。

### `ED-GAS-P1-010` [Open] 没有外部变更、source control、multi-user和merge策略

建立watch/import revision、three-way domain diff、checkout/read-only、conflict staging及stable-ID merge；不能让控件字符串覆盖外部文件。

### `ED-GAS-P1-011` [Open] Effect没有共享typed schema和context-sensitive property visibility

Details由runtime schema生成，duration policy、period、stack、execution等字段按语义显示/禁用并复用同一validator。

### `ED-GAS-P1-012` [Open] Magnitude只有常量字符串

定义Constant/Scalable/AttributeBased/Custom/SetByCaller discriminated model、curve/data source、coefficient/pre/post、tag key和dependency。

### `ED-GAS-P1-013` [Open] Attribute capture缺source/target、snapshot/live和dependency检查

编辑capture owner、attribute stable ID、snapshot timing与qualified dependency generation；无效capture阻止compile/publish。

### `ED-GAS-P1-014` [Open] Modifier列表无stable ID、排序、duplicate、multi-edit和provenance

列表项使用stable identity与typed operator，所有增删重排可撤销；显示template/inherited/local来源和override状态。

### `ED-GAS-P1-015` [Open] Duration/Period无单位、range、time policy和互斥验证

使用canonical时间单位与finite/range检查，Instant/Duration/Infinite和period组合由shared schema约束。

### `ED-GAS-P1-016` [Open] Stacking缺key、overflow、refresh/reset和expiration语义

编辑source/target aggregation key、limit、overflow effect、duration/period refresh、expiration与deny policy，并提供确定性preview。

### `ED-GAS-P1-017` [Open] Effect application/removal requirements与immunity不可编辑

提供Tag Query/attribute/policy条件、ongoing inhibition、remove-other、immunity source与失败诊断，引用统一dictionary generation。

### `ED-GAS-P1-018` [Open] Execution Calculation没有registry、schema、预算和owner

由runtime extension catalog提供calculation descriptor、captures、parameters、side-effect contract、determinism/budget和owner generation。

### `ED-GAS-P1-019` [Open] Gameplay Cue没有asset picker和生命周期预览

建立Cue asset/notify registry，覆盖OnActive/WhileActive/Executed/Removed、资源依赖、预测去重、quality/LOD和server policy。

### `ED-GAS-P1-020` [Open] Effect Details没有property customization或批量编辑

按typed schema提供picker、conditional field、array item、multi-selection common/mixed state和transactional batch edit。

### `ED-GAS-P1-021` [Open] Effect创建没有模板、继承/组合和项目策略

Create menu来自versioned template/profile，显式记录parent/composition与override；检测循环、失效模板和plugin owner revoke。

### `ED-GAS-P1-022` [Open] Effect依赖与引用扫描缺失

维护Tag/Attribute/Effect/Cue/curve/data dependency graph，增量索引source/artifact revision并支持usage定位和影响分析。

### `ED-GAS-P1-023` [Open] Compile没有shared semantic compiler、artifact或LKG

Editor、runtime与cook调用同一compiler；产生deterministic artifact、diagnostic/source map、dependency hash与last-known-good。

### `ED-GAS-P1-024` [Open] Effect preview没有隔离sandbox和可复现实例

构造独立world/owner/attribute/tag上下文与seed/time profile，Apply/Remove/Reset返回handle、delta、cue和终态receipt。

### `ED-GAS-P1-025` [Open] Effect底部面板没有provider

Attribute Delta/Validation/Compile Log消费revision-bound typed stream，支持filter/export/source navigation、reader lease与retention budget。

### `ED-GAS-P1-026` [Open] Ability没有Definition/Spec/activation policy模型

编辑grant、instancing、activation group、required/blocked/owned tags、input/event、net/security policy，并直接使用runtime schema。

### `ED-GAS-P1-027` [Open] Ability graph只是静态文本流程

建立persistent graph document、stable node/pin/link、selection、copy/paste、comment、find、viewport metadata与transaction。

### `ED-GAS-P1-028` [Open] Graph schema缺pin type、cardinality、cycle和context validation

连接前验证execution/data/event/target/attribute/tag类型、单多连接、cycle、owner、phase与provider capability。

### `ED-GAS-P1-029` [Open] Ability Task palette没有runtime extension catalog

palette由task descriptor/schema/owner generation驱动，支持category/search/favorite/unavailable/reload和schema migration。

### `ED-GAS-P1-030` [Open] Activate/Commit/End/Cancel阶段没有可视语义与编译规则

定义entry/terminal/commit/cancel路径，诊断重复commit、无终点、latent task未取消和未处理失败分支。

### `ED-GAS-P1-031` [Open] Cost/Cooldown只有固定资产名和4秒

使用Effect picker与resolved artifact generation，preview affordability、commit timing、cooldown tag和真实剩余时间。

### `ED-GAS-P1-032` [Open] Net Execution/Security policy没有项目能力约束

只有network/prediction provider可用且graph满足rollback contract时才开放Predictive；显示authority和security影响。

### `ED-GAS-P1-033` [Open] Ability Tag字段没有统一picker/query

activation/owned/cancel/block tag都消费同一dictionary generation，区分exact/container/query并支持引用跳转。

### `ED-GAS-P1-034` [Open] Target Data没有type/provider/filter/server validation authoring

定义Entity/Location/Hit等target schema、range/LOS/team/filter和authority policy，preview展示server拒绝原因。

### `ED-GAS-P1-035` [Open] Gameplay Event没有schema、payload与订阅关系

event tag、payload fields、sender/target/listener使用typed definition与reference graph，compile检查producer/consumer和payload mismatch。

### `ED-GAS-P1-036` [Open] Ability timeline没有真实latent task与simulation clock

timeline消费sandbox/PIE trace，展示task wait/event/finish/cancel、effect/cue/prediction阶段及qualified time domain。

### `ED-GAS-P1-037` [Open] Cancel/interrupt/activation-group关系不可编辑和验证

提供group/priority/replace/cancel/block规则，compiler检测互相取消循环、非法并发与不可达cleanup。

### `ED-GAS-P1-038` [Open] Cross-system task没有asset/provider/currentness检查

Animation/Navigation/Physics/Audio/VFX引用真实broker handle与provider generation；unavailable/reload/cancel不能返回Ability成功。

### `ED-GAS-P1-039` [Partial] 没有PIE owner/spec/activation实例选择器

generic Play Session和runtime event host存在；仍需按session/world/owner/avatar/spec/activation generation选择、retire与currentness fence。

### `ED-GAS-P1-040` [Open] Prediction debug没有key、receipt、rollback与reconciliation

记录local PredictionKey、dependent operations、server receive/accept/reject/catch-up、correction delta和最终收敛。

### `ED-GAS-P1-041` [Open] Ability Compile Log与Diff没有source/artifact revision

诊断定位node/pin/property；Diff明确base/local/remote与authoring/compiled变化，拒绝固定`changes compared`。

### `ED-GAS-P1-042` [Partial] Playtest没有完整operation lifecycle与sandbox teardown

generic Play Controller/job cancellation可复用；Gameplay按钮仍只写字符串。实现session/world/owner/activation ID、cancel/timeout/failure/terminal与teardown。

### `ED-GAS-P1-043` [Open] Tags registry没有真实provider与dictionary generation

Tree/Table投影name/parent/source/owner/references/status，project/plugin切换以atomic generation替换。

### `ED-GAS-P1-044` [Open] Tag source管理缺priority、writability与load state

显示project/plugin/native/generated source、path/format/owner/priority/read-only/error，写操作路由到正确adapter。

### `ED-GAS-P1-045` [Open] Tag hierarchy没有lazy tree与large-registry导航

实现parent closure、breadcrumb、exact/implicit parent、stable selection、lazy load和virtualized expansion。

### `ED-GAS-P1-046` [Open] Tag search/filter缺source、owner、status与usage维度

编译typed query，异步分页结果绑定dictionary/reference-index generation，支持cancel与stale receipt。

### `ED-GAS-P1-047` [Open] Add Tag缺规范化、重复、父级、权限和命名策略验证

预览canonical name/parent chain，拒绝大小写冲突、非法段、只读source、超限深度与并发重复，commit进入transaction。

### `ED-GAS-P1-048` [Open] Rename缺redirect lifecycle、冲突/循环检查和原子提交

预检目标、redirect chain、source权限与引用影响；staging成功后跨文档commit并产生migration/rollback receipt。

### `ED-GAS-P1-049` [Open] Delete/Cleanup Unused能力缺失

基于完整reference index区分unused/native/indirect query/runtime-generated tag，支持dry-run、selection、confirmation、backup与恢复。

### `ED-GAS-P1-050` [Open] Reference Scan没有索引、进度、取消与定位

扫描source/assets/scenes/config/generated references，记录unknown providers与index generation，结果可跳到asset/property并导出。

### `ED-GAS-P1-051` [Open] Migration Preview/Apply没有staging workspace与rollback

展示per-file before/after、redirect或hard rewrite、冲突和cook影响；apply使用跨文档transaction或durable journal。

### `ED-GAS-P1-052` [Open] 缺少Gameplay Tag Query编辑器

提供Any/All/None/Exact嵌套AST、drag/reorder、depth/node budget、文本/图形双视图和runtime preview。

### `ED-GAS-P1-053` [Open] Tags Compile Log没有真实dictionary/cook diagnostics

展示source parse、redirect resolve、parent closure、dense index、network dictionary与dependent recook结果，定位source line/tag ID。

### `ED-GAS-P1-054` [Open] 没有network dictionary compatibility视图

显示dictionary hash、index count、bit width、redirect generation和目标server/build兼容性；mismatch阻止prediction playtest。

### `ED-GAS-P1-055` [Partial] 九类bottom panel没有统一operation/trace provider

route、mode与drawer lifecycle已存在；仍需typed view model/data source/subscription/currentness/filter/export和slow-consumer策略。

### `ED-GAS-P1-056` [Partial] Gameplay长操作未接Background Job lifecycle

shared scheduler已有admission/progress/cancel/shutdown；compile/scan/migration/audit/playtest必须提交revision-fenced job并隔离late result。

### `ED-GAS-P1-057` [Partial] 模块、document、selection与bottom state未统一

现有UI route能切workspace并保存控件状态，但active owner仍由selected checkbox/fallback Effect决定；改为toolkit/document/session route。

### `ED-GAS-P1-058` [Partial] Accessibility、键盘、localization与数值输入未达业务资格

native input/focus和momentary button已有测试；复杂graph/tree/table/dialog、错误关联、screen reader、locale bundle与canonical numeric storage仍缺。

### `ED-GAS-P1-059` [Open] 大资产没有virtualization、delta projection与预算

Tag tree/reference、modifier/task、diagnostic/trace按分页/delta/virtualization更新，并设entries/bytes/time/reader-retention预算。

### `ED-GAS-P1-060` [Partial] 测试把固定文案当业务成功

23项focused test证明surface/route/input/pixel但固化假结果。替换为schema/transaction/compiler/provider/product/fault/scale合同测试。

## 7. P2 高阶能力

### `ED-GAS-P2-001` [Open] 复杂Magnitude/Execution表达式图与静态分析

增加曲线、数据注册、捕获链、条件modifier/execution graph，并做常量折叠、范围/单位、复杂度和循环分析。

### `ED-GAS-P2-002` [Open] 高级Ability组合、combo/input trigger与graph fragment

支持ability set、combo/window、hold/release/chord、subgraph/template和versioned fragment引用。

### `ED-GAS-P2-003` [Open] 多客户端prediction对比与网络时间轴

同屏对齐client predicted、server authoritative和simulated proxy trace，显示packet/receipt/correction与收敛时间。

### `ED-GAS-P2-004` [Open] 跨项目/插件Tag迁移、CI审计与自动修复

提供headless audit/migration dry-run、policy gate、signed receipt、分批提交与失败恢复。

### `ED-GAS-P2-005` [Open] three-way semantic Diff/Merge与graph冲突解决

按stable node/field/tag ID比较base/local/remote，提供结构化冲突和transactional merge。

### `ED-GAS-P2-006` [Open] Attribute/Effect曲线与平衡数据联动

集成Curve/Data Registry、level sweep、敏感性分析、范围告警和引用更新，preview绑定artifact hash。

### `ED-GAS-P2-007` [Open] Cue Sequencer、VFX/Audio轨与资源预算预览

投影Cue生命周期、并发、池、距离/quality LOD和dedicated-server边界。

### `ED-GAS-P2-008` [Open] 批量表格、数据驱动生成与schema migration工具

支持grid authoring、template instance、bulk validation/version migration，产物仍是可审计typed asset。

### `ED-GAS-P2-009` [Open] 团队权限、review/approval与安全策略

Tag migration、network/security policy和跨资产重写支持审批、actor/scope记录和rollback receipt。

### `ED-GAS-P2-010` [Open] 内容级CPU/内存/带宽/rollback成本profiler

按Ability/Effect/Tag Query/Task/Cue归因并保留历史趋势与同场景对照。

### `ED-GAS-P2-011` [Open] 离线平衡、不可达、循环与数值爆炸审计

扫描activation contradiction、effect cycle、infinite stack/period、unreachable task和极值，输出可定位诊断。

### `ED-GAS-P2-012` [Open] 第三方property/graph/task/query/cue authoring生态

建立versioned schema、owner lease、sandbox、reload/compat/capability admission，禁止插件注入假业务成功反馈。

## 8. 分层重构顺序

### M0：产品真相硬切

Gameplay provider/runtime缺失时三模块显示Unavailable；删除fixed Save/Compile/Apply/Playtest/Add/Rename结果与默认Effect假authority。保留momentary-control修正，但不能把它当领域完成。

### M1：领域资产与document foundation

交付Tag/AttributeSet/Effect/Ability/Cue resource kind、source schema、factory/importer/toolkit、qualified document session、transaction/dirty/save/recovery和source-control contract。

### M2：Gameplay Tags与Query作者链

对接Runtime151 dictionary/source/redirect/query compiler，完成registry/picker/add/rename/delete/cleanup/reference scan/migration staging与network hash视图。

### M3：AttributeSet与Effect作者链

完成attribute schema、Effect typed details、modifier/capture/execution/duration/period/stack/requirements/cue、shared compiler、artifact/LKG和sandbox preview。

### M4：Ability与Task graph

完成Definition/Spec、graph document/schema/palette、activation/commit/cancel、cost/cooldown、event/target/task、compiler/source map与semantic diff。

### M5：Cue与跨系统provider

对接Animation/Navigation/Physics/Audio/VFX broker、Cue lifecycle/Sequencer/resource budget，处理provider missing/revoke/reload/cancel。

### M6：Playtest、PIE与prediction debug

建立session/world/owner/spec/activation selector、bounded trace、PredictionKey/receipt/rollback/reconcile、network capability admission和sandbox teardown。

### M7：Bottom views、job与规模资格

九类view接真实provider，完成job shutdown、revision fence、大dictionary/graph、consumer stall、accessibility/localization和性能预算。

### M8：静态Workbench硬切与产品验收

默认入口只打开真实toolkit；静态样例若保留必须标记non-persistent showcase且不能提供业务命令。Windows Editor/client/server/目标平台证据全部绑定current build/artifact。

## 9. 验收门禁

- **G-01 [Fail] 默认真相**：无Gameplay provider/runtime时显示Unavailable且不输出成功/queued。
- **G-02 [Fail] 产品装配**：App按project selection装配、禁用和reload Gameplay Editor。
- **G-03 [Fail] Asset类型**：Tag/AttributeSet/Effect/Ability/Cue可Create/Open/Save/Reopen。
- **G-04 [Partial] Document事务**：共享transaction存在；Gameplay编辑尚未接入undo/redo/dirty/recovery。
- **G-05 [Partial] 持久化终态**：共享dirty/save底座存在；Gameplay无durability/source revision receipt。
- **G-06 [Fail] Shared compiler**：Editor/runtime/cook使用同一schema、diagnostic和artifact hash。
- **G-07 [Fail] Tag字典**：source/hierarchy/redirect/container/query/dense index与Runtime151一致。
- **G-08 [Fail] Tag操作**：Add/Rename/Delete/Cleanup/Scan/Migration覆盖冲突与rollback。
- **G-09 [Fail] Tag规模**：大registry tree/search/picker/reference结果virtualized且有预算。
- **G-10 [Fail] Attribute编辑**：schema/default/clamp/metadata/stable ID/reload migration通过。
- **G-11 [Fail] Effect Details**：所有非法duration/stack/modifier/capture/execution组合由同源schema拒绝。
- **G-12 [Fail] Effect artifact**：Compile产生真实diagnostic/dependency/hash/generation/LKG。
- **G-13 [Fail] Effect preview**：sandbox Apply/Remove/Reset产生真实handle、delta、tag和cue。
- **G-14 [Fail] Effect bottom views**：三类view只显示revision-bound真实数据并可定位source。
- **G-15 [Fail] Ability graph**：stable ID、typed edge、copy/paste、undo、compiler/source map通过。
- **G-16 [Fail] Task catalog**：palette来自runtime extension，处理missing/revoke/reload/migration。
- **G-17 [Fail] Activation语义**：CanActivate/Try/Commit/End/Cancel、cost/cooldown/tag与runtime一致。
- **G-18 [Fail] Target/Event**：typed target/payload/server validation和引用关系可编译/预览。
- **G-19 [Fail] Cross-system task**：broker handle、cancel与provider generation正确。
- **G-20 [Fail] Ability playtest**：有qualified session/owner/activation、cancel/timeout/terminal/teardown。
- **G-21 [Fail] Prediction debug**：真实key/request/receipt/reject/rollback/reconcile最终收敛。
- **G-22 [Fail] Net policy admission**：网络能力缺失或不可rollback时Predicted/Server选项禁用。
- **G-23 [Fail] Cue authoring**：四类lifecycle、资源、quality/server policy与预测去重通过。
- **G-24 [Fail] Bottom provider**：九类Gameplay view有typed data source/subscription/slow-consumer策略。
- **G-25 [Partial] Job lifecycle**：共享scheduler存在；Gameplay长操作尚未提交job或验证shutdown。
- **G-26 [Fail] Revision fence**：late result不能覆盖新document/artifact/world generation。
- **G-27 [Fail] Plugin reload**：owner revoke后toolkit/task/provider/trace安全retire。
- **G-28 [Fail] 无固定业务结果**：production不含sample persisted、+50 health或固定predicted success。
- **G-29 [Partial] Accessibility/I18n**：基础focus/input存在；复杂业务控件和locale尚未完成。
- **G-30 [Fail] 故障矩阵**：missing source/provider、compile/disk/runtime/network/project close诚实终结。
- **G-31 [Fail] Current-build证据**：Windows Editor、client/server与目标平台结果绑定source/build/artifact。
- **G-32 [Fail] 性能声明**：同内容/质量/硬件/网络的可复现实验达到阈值后才可声称优于Unreal。

## 10. 跨owner前置与硬切政策

- Runtime151拥有Tag/Attribute/Effect/Ability/Task/Cue/Prediction/Replication的唯一运行时语义；Editor90只拥有authoring、operation、toolkit与debug projection，不复制runtime规则。
- Editor02/21/56/63拥有共享document、search/reference和transaction父问题；本报告只登记Gameplay adapter与产品闭环，不重复其P0。
- 不保留静态Workbench与真实Gameplay toolkit两个可写authority；M8后前者只能是明确non-persistent showcase。
- 不保留“先改control、稍后同步document”的双轨。control仅投影draft/accepted revision，业务truth只在versioned AuthoringDocument。
- 不保留fixed success/queued fallback。factory/provider/runtime缺失必须返回typed Unavailable或terminal error。
- 不在Runtime151的authority/prediction/replication门禁前开放相关UI；隐藏或禁用比伪造成功更符合产品要求。
- tooling按用户要求排除，后续Rust迁移另立owner，不在本报告扩展范围。

## 11. 当前验证结论

本轮完成67个Zircon selected文件、16,760行、658,910 bytes和58个参考selected文件、39,110行、1,540,599 bytes的静态审查。只新增review与索引记录，没有修改production code，也没有运行动态测试；selected源码包含其他Session/用户的在途改动，实施前必须重取物理指纹。

当前最危险的不是控件数量不足，而是完整外观和绿色route测试掩盖了“没有领域”的事实。M0应先删除误导性成功路径并建立Unavailable真相；随后必须从Runtime151的typed source/build/runtime contract向上生长Editor，而不是继续给三份ZUI追加固定字段、按钮和日志文本。
