---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_effect_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_tags_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_lifecycle.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/document_module.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_inspector_property_edit.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/text_and_module_input.rs
  - zircon_editor/src/tests/workbench/reference_surface.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilitiesEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilityGraph.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilityGraphSchema.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilitiesBlueprintFactory.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilityAudit.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayEffectDetails.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayEffectModifierMagnitudeDetails.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayEffectExecutionDefinitionDetails.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayEffectCreationMenu.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/SGameplayTagWidget.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/SAddNewGameplayTagWidget.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/SRenameGameplayTagDialog.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/SCleanupUnusedGameplayTagsWidget.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/GameplayTagsSettingsCustomization.cpp
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 21 · Gameplay Ability / Effect / Attribute / Tag / Cue / Debug Authoring 工程化差距

## 1. 结论

Zircon Editor当前已经画出三份看似完整的Gameplay工作区：Effect有资产列表、duration、period、stacking、modifier、tag和simulation output；Ability有graph、task、cost、cooldown、network policy、timeline与playtest；Tags有registry、source、validation、owner、redirect、add和rename。三份ZUI合计771行、90个control、68个event/route，顶部toolbar还能在Effect/Ability/Tags之间切换；Effect默认处于checked/selected状态。这些surface不是隐藏fixture，而是Workbench首屏直接展示的产品模块。

但它们没有Gameplay Editor或runtime owner。`first_party_editor_catalog`只装配Navigation和Neural，不存在Gameplay/Ability provider；runtime审查08G也确认没有Ability/Effect/Attribute/Tag/Cue domain、asset、scene component、plugin、prediction或replication。三份workspace中的 `GE_HealthRegen`、`GE_DamageFire`、`GA_DashAttack`、`GE_DashAttack_Cost`、`DefaultGameplayTags.ini`、`Server Initiated`、4秒cooldown、`+50 health`和tag错误计数均为静态文本，不是项目内容或runtime状态。

现有路由证明的是模板控件可交互，不是作者工具可用。字段Change/Submit只把输入字符串写回retained control的 `value`与`value_text`并刷新surface，没有document、source revision、transaction、validation、dirty、undo、save或runtime apply。Save/Compile/Diff/Simulate、Effect Apply、Ability Playtest、Tags Add/Rename全部由 `module_command_feedback.rs`直接返回固定成功/queued/pending字符串；reference action最终只改selected/checked/visible/popup等UI状态。focused tests又明确断言这些固定文案，因此当前绿色测试会固化假业务authority。

底部面板同样没有真实数据。Effect Attribute Delta/Validation/Compile Log、Ability Compile Log/Gameplay Event Log/Simulation Console、Tags Reference Scan/Migration Preview/Compile Log九类row和route只切换静态模式、selection和label，不订阅compiler、asset operation、job、PIE、network或runtime trace。它们甚至无法证明当前source revision与显示结果属于同一generation；用户改字段后立刻点击Compile/Apply会收到成功语气，但没有任何内容真正变化。

Unreal参考并非只提供漂亮面板。GameplayAbilitiesEditor有asset factory、Ability graph/schema、audit、Effect details/magnitude/execution customization和创建菜单；GameplayTagsEditor有picker、query、settings、source、add、rename、cleanup和引用迁移。这些Editor能力建立在runtime AbilitySystem、GameplayEffect和GameplayTags registry真实合同上。Zircon不必复制Blueprint/UObject UI，但必须实现同等级的transactional document、shared semantic compiler、prepared artifact、preview world、authority-aware playtest和reader-gated trace，且所有UI结果带source/artifact/world generation。

本轮登记5项P0、60项P1、12项P2。M0先把默认首屏的虚假成功硬切为Unavailable并建立真实provider/catalog边界；M1-M5依次完成Tag、Attribute/Effect、Ability/Cue资产和语义作者链；M6接入sandbox/PIE/network debug；M7收敛job、diagnostic、性能和故障恢复；M8删除静态第二authority并建立产品资格。Runtime08G全部P0与核心P1是本篇真实Apply/Playtest/Prediction可启用的前置，Editor不得继续用样例文案代替runtime实现。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes | 证据等级 |
|---|---:|---:|---|
| Gameplay Workbench surfaces | 7 / 1,913 / 117,321 | 0 | E3：三份业务ZUI、toolbar、workspace composition和generated bottom panel逐项 |
| route、binding、field edit、feedback与bottom panel | 11 / 3,288 / 137,034 | 1 | E3：所有Gameplay action从event到最终UI mutation逐分支 |
| focused Workbench tests | 5 / 2,373 / 85,949 | 24 | E3静态阅读：模块选择、字段输入、固定反馈、template/reference投影 |
| runtime通用gameplay host | 16 / 2,875 / 105,895 | 14 | E3交叉复核08G：证明通用host存在，也证明Ability domain不存在 |
| runtime asset与first-party Editor catalog absence anchors | 7 / 768 / 27,493 | 6 | E3完整导出/分支表：无Gameplay资产、scene owner或Editor provider |
| selected combined scope | 46 / 11,217 / 473,692 | 45 | 当前工作树fingerprint `8fb4d5c922b1f59d7f17c03b106e08e2b0bc11d39b8c120d92f0c5c674f9c156`；0 ignored，1个纯import排序在途source |

行数为物理文本行；fingerprint按相对路径排序，对每个当前工作树文件计算SHA-256，再对`path<TAB>hash<LF>`清单计算SHA-256。`zircon_editor/src/ui/retained_host/workbench_preview_actions.rs`存在非本轮产生的纯import排序修改，本轮保持原样，因此实施前需要重新计算fingerprint并复核action inventory。整个仓库另有用户与其他Session修改，本轮不吸收、不回退。

三份Gameplay ZUI物理统计为：Ability 283行/16,696 bytes/34 controls/25 event-or-routes/3 fields-or-dropdowns/9 buttons；Effect 303/17,393/35/28/6/1；Tags 185/10,237/21/15/3/2。数字只证明surface复杂度和审查覆盖，不能证明domain完成。generated bottom panel另有534行/36,336 bytes，其中九类Gameplay row/route均已追到静态feedback/lifecycle代码。

### 2.2 动态证据边界

本轮没有运行动态测试。此前 `zircon_editor --lib`测试编译在617.2秒后被239个既有错误、122个warning阻断，本轮没有重复无代码变化且无法到达Gameplay domain的同一lane。selected scope中的45个test attributes只作为静态inventory；它们主要证明route、selected状态和固定字符串能被写入control，反而不能证明asset、transaction、compiler、runtime、prediction或debug成立。

### 2.3 参考边界

- Unreal GameplayAbilitiesEditor将factory、graph/schema、details customization、audit、diff/compile和runtime debug建在真实Ability/Effect类型之上。Zircon可以采用data-oriented authoring document，但必须有同一schema驱动Editor、cook和runtime，不能让ZUI字符串自行定义业务字段。
- Unreal GameplayEffect details对duration、period、stacking、modifier magnitude、attribute capture、execution和tag requirement按类型呈现，并依据上下文隐藏非法组合。Zircon首版可以减少表达式种类，但必须提供constraint-driven property editor和shared validation，不能让任意文本框保存非法数值。
- Unreal GameplayTagsEditor提供source-aware tag picker、add、rename、cleanup、settings、query与reference migration。Zircon必须让runtime cooked dictionary和Editor registry共享stable ID、redirect和generation，不能只有 `DefaultGameplayTags.ini` 样例行。
- Runtime08G已经从Unreal AbilitySystemComponent、GameplayAbility、GameplayEffect、GameplayPrediction和GameplayTags runtime定义中提取目标合同；本篇只拥有authoring、operation、preview和debug projection，不重复发明另一套runtime语义。
- Fyrox、Bevy、Godot主仓和本地Unity Graphics没有同级first-party Gameplay Ability Editor。它们不能用来降低Unreal级目标，也不作为这些缺失产品能力的反证。

## 3. 必须保留的基础

1. 保留Workbench稳定control/action/binding identity和集中route inventory，迁移时用adapter逐个替换业务handler，避免隐式字符串散落更多位置。
2. 保留template projection、selected/checked/popup/visible等纯presentation状态机；它们应继续负责视觉导航，但不再承担业务成功与数据持久化。
3. 保留Change/Submit事件区分，但把提交路由到typed document command；control只投影accepted revision或显示validation error。
4. 保留generated bottom panel的统一布局与分类入口，替换其数据源为operation/job/compiler/runtime delta provider。
5. 保留focused tests对control identity、路由唯一性和键盘输入的价值，同时删除“固定成功字符串即业务正确”的断言。
6. 复用共享Editor document transaction、autosave/recovery、background job、diagnostic journal、asset toolkit、runtime event和viewport provider，不为Gameplay另造简易栈。
7. 保留runtime通用host的typed descriptor/error基础作为脚本适配层，但Editor不直接编辑 `script.bindings.hp`或把它包装成Effect。

## 4. 目标架构

```text
ProjectPluginManifest(gameplay-ability)
  -> first_party_editor_catalog provider
  -> GameplayEditorRegistration
       -> Tag / AttributeSet / Effect / Ability / Cue asset definitions
       -> toolkit + operation factories + property customization
       -> graph / query / picker / timeline / preview providers

AuthoringDocument(revision, stable IDs, dependencies)
  -> Transaction / Undo / Autosave / Recovery
  -> Shared Gameplay Semantic Compiler
  -> Diagnostics + Prepared Artifact + LKG generation
  -> Runtime08G world owner / sandbox / PIE

Operation and trace
  -> operation_id + source/artifact/world generation
  -> cancellable job + progress + bounded diagnostics
  -> reader-gated Effect/Ability/Tag/Prediction deltas
  -> Workbench + generated bottom panel projections
```

核心owner划分：

1. Runtime08G拥有Tag/Attribute/Effect/Ability/Cue schema、semantic compiler中立合同、prepared artifact和world execution truth。
2. Asset系统拥有source revision、import/reimport、dependency、cook/DDC、atomic publish和LKG；Editor不直接写产物。
3. Gameplay Editor plugin拥有document/session/selection/transaction、property/graph projection、operation controller和preview/debug presentation。
4. Shared Editor host拥有job、notification、diagnostic、runtime event和viewport provider lifecycle；Workbench不能绕过它直接改runtime。
5. Simulation host拥有sandbox/PIE/client-server实例与权限；Playtest结果必须绑定session/world/owner/ability/prediction generation。
6. 静态ZUI仅定义layout和presentation defaults，不保存项目业务数据，不产生compile/apply成功，也不成为runtime snapshot。

## 5. P0 阻断项

### P0-1：默认首屏公开Effect/Ability/Tags，但产品没有Gameplay Editor provider或runtime domain

Effect在top toolbar默认checked/selected，用户启动Workbench即进入Gameplay Effect界面；first-party Editor catalog却没有Gameplay feature/dependency/registration，runtime也没有对应资产和服务。M0必须在provider不存在时显示Unavailable并禁用业务命令；只有真实项目插件、asset toolkit和runtime operation装配完成后才能作为默认产品模块。

### P0-2：Save/Compile/Diff/Simulate/Apply/Playtest全部用固定字符串伪造成功或排队

`module_command_feedback.rs`直接写 `GE_HealthRegen sample persisted`、`compile queued`、`applied +50 health preview`和 `predicted activation GA_DashAttack`，没有operation、job、artifact或runtime调用。M0必须删除业务成功分支；factory/provider缺失返回typed Unavailable/MissingProvider，操作开始、进度、取消和终态只由真实operation result驱动。

### P0-3：字段Change/Submit只改control属性，没有document、transaction、dirty、undo或验证

`edit_workbench_module_field`确认binding后无条件写 `value/value_text`并刷新surface。Duration、Period、Stack、Cooldown、Net Policy、Tag Source和Owner都因此只是瞬时UI字符串；重启或换模块无法保证一致。必须先建立versioned AuthoringDocument与typed command，accepted revision投影回control，invalid input保留草稿并显示诊断，所有业务提交接入undo/redo和dirty/save lifecycle。

### P0-4：Ability界面宣称Server Initiated和predicted activation，却没有network authority或prediction trace

Network policy是静态dropdown值，Playtest固定输出predicted；没有client/server session、owner connection、Prediction Key、request/receipt、rollback、reconciliation或cue/task side effect。Runtime08G网络门禁通过前必须禁用这些选项和成功语气；启用后Playtest必须展示真实key、server result、correction和最终收敛状态。

### P0-5：Tags Add/Rename/Reference Scan/Migration Preview只是静态反馈，可能诱导用户相信跨资产迁移已完成

Add/Rename只写pending文案，底部Reference Scan/Migration Preview/Compile Log只有静态row和mode切换；没有tag registry、source、redirect、reference index、transaction或文件写入。标签重命名属于跨资产破坏性操作，必须先完成全引用扫描、staging migration、冲突/循环校验、用户确认、原子提交和可回滚记录；当前按钮必须不可用而不是声称prepared。

## 6. P1 核心重构差距

### P1-1：first-party Editor catalog没有Gameplay/Ability插件入口

新增显式feature、Cargo dependency、registration branch、capability和真实App projection；project未选择、provider加载失败、disable与reload必须反映到surface状态。

### P1-2：没有Tag、Attribute Set、Effect、Ability和Cue asset type

为五类内容定义stable type ID、extension、schema version、icon/category、thumbnail和runtime/cook owner；禁止用ZUI中的显示名代替资产身份。

### P1-3：没有Create factory、template和命名事务

Content Browser应能创建最小合法资产，处理目录权限、重复名、取消、失败回滚和初始source落盘；template必须来自版本化schema而非复制样例字符串。

### P1-4：没有Open toolkit与document session

实现locator到session的去重、focus、read-only、missing source、external modification和close decision；三个Workbench tab不能继续代表打开的真实资产。

### P1-5：没有import/reimport、provenance和settings合同

Tag source、Ability/Effect文本或外部数据导入必须记录source hash、importer/version/settings、dependency和diagnostic；reimport不得覆盖未保存Editor revision。

### P1-6：AuthoringDocument没有stable field/node identity和source revision

每个属性、graph node/pin、modifier、task和tag entry需要stable ID；selection、undo、diff、diagnostic和runtime debug都绑定revision/generation，不能只靠control ID。

### P1-7：没有transactional command与undo/redo

字段、列表、graph、rename、bulk edit均用typed command生成before/after或inverse；多字段约束作为一个transaction提交，失败不留下半修改。

### P1-8：没有dirty、autosave、recovery和close decision

accepted document mutation设置dirty并进入autosave/recovery；Save失败保持dirty并报告原因，关闭/切项目/重载插件时走统一decision flow。

### P1-9：Save没有durability acknowledgement和artifact状态

Save必须区分source persisted、directory sync policy、compile pending、artifact stale和publish generation；不能把control文本变化称为persisted。

### P1-10：没有外部变更、source control、multi-user和merge策略

检测磁盘revision变化并提供reload/diff/merge/stash；锁定、checkout、只读和冲突状态进入toolkit，不允许silent last-writer-wins。

### P1-11：Effect没有共享typed schema和context-sensitive property visibility

Duration Policy、Period、Stack、Modifier、Cue、Tag requirement和Execution应由runtime schema驱动；非法组合隐藏或诊断，ZUI不能独自决定字段集合。

### P1-12：Magnitude只会显示常量字符串，缺少Scalable/Attribute/SetByCaller模型

实现typed magnitude variant、curve/data source、coefficient、capture和caller key picker；每种variant有专属validation与preview解释。

### P1-13：Attribute capture没有source/target、snapshot/live和dependency检查

picker只允许当前registry中的Attribute，显示capture origin与snapshot policy；schema/tag/attribute变化使Effect artifact失效并给出定位诊断。

### P1-14：Modifier列表没有稳定ID、排序、duplicate、multi-edit和provenance

提供add/remove/reorder/duplicate、operation/channel、attribute、magnitude和source说明；列表操作进入transaction且保留selection。

### P1-15：Duration/Period字段没有单位、range、time policy和互斥验证

明确秒/帧/simulation time、infinite/instant禁用规则、period execute-on-apply与missed tick policy；负值、NaN/Inf和超限在提交前拒绝。

### P1-16：Stacking设置没有key、overflow、refresh/reset和expiration语义

Editor需完整投影runtime stacking policy，并用constraint matrix禁止不相容组合；preview显示每次apply的typed outcome而非固定数值。

### P1-17：Effect的application/removal requirements与immunity不可编辑

集成Tag Query editor、source/target条件、ongoing inhibition、remove query和immunity policy；引用失效时编译失败并提供修复入口。

### P1-18：Execution Calculation没有registry、schema、预算和owner信息

从runtime extension catalog列出可用calculation及typed参数、thread/determinism/budget metadata；provider缺失或generation stale时不可发布。

### P1-19：Gameplay Cue只是一行文本，没有asset picker和生命周期预览

支持Executed/Added/WhileActive/Removed mapping、资源依赖、quality/server policy与池预算；preview缺provider时显式降级且不影响Effect truth。

### P1-20：Effect Details没有property customization或批量编辑

不同duration/magnitude/stack/execution variant需要专用控件、条件说明和mixed-value处理；多选只修改用户触碰字段，禁止整对象覆盖。

### P1-21：Effect创建菜单没有模板、继承/组合和项目策略

提供Instant/Damage/Heal/Buff/Debuff/Cooldown等可配置模板，但模板生成真实source并受项目命名/tag/attribute policy约束，不把样例当builtin truth。

### P1-22：Effect依赖与引用扫描缺失

显示Attribute Set、Tag Dictionary、Cue、Ability、calculation和linked Effect依赖；删除/rename前查询资产索引，结果绑定source revision。

### P1-23：Compile没有shared semantic compiler、artifact或LKG

Editor调用Runtime08G同一compiler，产生diagnostic code/location、artifact hash和dependency generation；失败保留LKG并明确stale，不能只写queued。

### P1-24：Effect preview没有隔离sandbox和可复现实例

preview world包含明确source/target owner、initial attributes/tags/level/time和seed；Apply返回真实spec/effect handle与delta，Reset销毁整个sandbox generation。

### P1-25：Attribute Delta/Validation/Compile Log底部面板没有provider

三类view订阅document/compiler/sandbox operation delta，支持filter、copy/export、定位source和清空；slow consumer有entry/bytes/age限制。

### P1-26：Ability没有Definition/Spec/activation policy的typed asset模型

编辑instancing、activation group、required/blocked/owned tags、grant policy和net policy；UI字段直接来自runtime schema与项目profile。

### P1-27：Ability graph只是静态文本流程，没有graph document

建立stable node/pin/link ID、viewport metadata、selection、copy/paste、comment、find和transaction；source模型与render projection分离。

### P1-28：Graph schema没有pin type、cardinality、cycle和context validation

连接前检查execution/data/event/target/attribute/tag类型、单/多连接、循环、owner与phase；invalid link不给source留下残片。

### P1-29：Ability Task palette没有runtime extension catalog

palette由task descriptor/schema/owner generation驱动，支持搜索、category、favorites、provider unavailable和reload；不能把五段静态流程称为task graph。

### P1-30：Activate/Commit/End/Cancel阶段没有可视语义与编译规则

Graph/compiler明确入口、terminal、commit和cancel路径，检测无终点、重复commit、不可取消latent task和未处理失败分支。

### P1-31：Cost/Cooldown只显示资产名和4秒，没有真实引用与preview

使用Effect asset picker和resolved generation，展示affordability、commit timing、cooldown tag与剩余时间；缺失/循环引用阻止发布。

### P1-32：Net Execution/Security policy没有项目网络能力约束

只有runtime network/prediction provider可用且ability满足rollback要求时才允许Predictive；Server Only/Initiated/Local策略显示实际authority与安全影响。

### P1-33：Activation/owned/cancel/block tags没有统一picker/query

所有tag字段消费同一registry generation，支持exact/container/query语义和引用跳转；文本自由输入只作为受验证草稿。

### P1-34：Target Data没有类型、provider、filter或server validation authoring

编辑Actor/Entity/Location/Hit等target schema、range/LOS/team规则和authority policy；preview展示被server拒绝的原因。

### P1-35：Gameplay Event没有schema、payload和订阅关系视图

事件tag、payload fields、sender/target与listener有typed definition和引用图；compile检测无producer、无consumer、payload mismatch和循环触发预算。

### P1-36：Ability timeline没有latent task真实时间与simulation clock

timeline消费sandbox/PIE trace，展示task start/wait/event/finish/cancel、effect/cue和prediction阶段；静态 `1.22s Ability Activated OK`必须删除。

### P1-37：Cancel/interrupt/activation-group关系不可编辑和验证

提供cancel/block tag、group、priority、replace policy和terminal cleanup审计；compiler检测互相取消循环与不可达终止。

### P1-38：Animation/Navigation/Physics/Audio task没有跨系统asset与provider检查

picker解析真实资源和broker capability，preview结果绑定task handle/provider generation；provider unavailable不能返回Ability成功。

### P1-39：没有PIE owner/spec/activation实例选择器

debugger按play session、world、owner/avatar、ability spec和activation generation筛选，entity despawn或world replacement自动retire selection。

### P1-40：Prediction debug没有key、receipt、rollback和reconciliation可视化

显示local key、dependent operation、server receive/accept/reject/catch-up、correction delta和最终收敛；所有事件受reader和privacy budget约束。

### P1-41：Ability Compile Log与Diff没有source/artifact revision

compile diagnostics定位node/pin/property，Diff比较明确base/local/remote revision并区分authoring与compiled变化；不能输出固定 `changes compared`。

### P1-42：Playtest没有启动、取消、超时、终态和sandbox teardown

Playtest是cancellable operation，返回session/world/owner/activation ID；失败、用户停止、provider reload与Editor关闭都清理task/effect/cue/prediction。

### P1-43：Tags registry没有真实数据provider和dictionary generation

Tree/Table按runtime tag dictionary source投影name、parent、source、owner、references和status；project或plugin切换原子替换generation。

### P1-44：Tag source管理缺少project/plugin/native/generated优先级和只读状态

显示source路径、format、owner、priority、writability和load error；写操作路由到对应source adapter，不能假定一个INI文件拥有全部标签。

### P1-45：Tag hierarchy没有lazy tree、parent closure和large-registry导航

支持展开、breadcrumb、search result context、exact/implicit parent区分和stable selection；大字典按需加载/virtualize，不为每次输入重建全树。

### P1-46：Tag search/filter没有source、owner、status和usage维度

提供可组合filter与query preview，结果与dictionary/reference-index revision绑定；异步查询可取消并报告stale。

### P1-47：Add Tag没有规范化、重复、父级、source权限和命名策略验证

dialog预览canonical name与parent chain，拒绝大小写冲突、非法段、只读source、超限深度和并发重复；commit形成transaction。

### P1-48：Rename没有redirect lifecycle、冲突/循环检查和原子提交

先验证目标、redirect chain、source writable和引用影响，再staging；失败不修改registry或任一资产，成功产生migration receipt。

### P1-49：Delete/Cleanup Unused能力缺失

基于完整reference index区分unused、native、indirect query和runtime-generated tag，支持dry run、selection、confirmation、backup与恢复。

### P1-50：Reference Scan没有资产索引、进度、取消和结果定位

扫描source/assets/scenes/config/code-generated references，记录index generation和unknown providers；结果可跳转到具体asset/property并导出。

### P1-51：Migration Preview/Apply没有staging workspace与rollback

展示每个文件的before/after、redirect vs hard rewrite、冲突和预计cook影响；apply使用跨文档transaction或durable journal，崩溃可恢复。

### P1-52：缺少Gameplay Tag Query编辑器

提供Any/All/None/Exact嵌套AST、drag/reorder、depth/node budget、文本/图形双视图和runtime preview；序列化使用stable ID/generation。

### P1-53：Tags Compile Log没有真实cook/dictionary diagnostics

展示source parse、redirect resolve、parent closure、dense index、network dictionary和依赖recook结果；diagnostic绑定source line/tag ID。

### P1-54：没有network index/hash与client-server dictionary compatibility视图

显示dictionary hash、index count、bit width、redirect generation和与目标server/build的compatibility；mismatch阻止预测playtest并提供修复路径。

### P1-55：九类generated bottom panel没有统一operation/trace provider

建立typed view model和subscription lifecycle，row只投影真实delta；panel隐藏/关闭释放reader，模式切换不重置业务state。

### P1-56：长操作没有Background Job、取消、进度、deadline和shutdown barrier

compile、reference scan、migration、audit、cook和playtest都进入共享job scheduler，使用source revision compare-and-publish；Editor关闭等待或安全隔离late result。

### P1-57：模块、资产、document、selection和bottom panel状态没有统一路由

切换Effect/Ability/Tags时应保存各自document/selection/view state并绑定active toolkit；不能由selected checkbox猜业务owner或默认回退Effect。

### P1-58：可访问性、键盘、localization和数值输入语义未形成业务资格

复杂graph/tree/table/dialog提供焦点顺序、screen-reader name/state、键盘操作、错误关联和非颜色状态；display text进入locale bundle，数值遵循locale显示但canonical存储。

### P1-59：大资产没有virtualization、增量projection和entries/bytes/time预算

Tag tree、reference result、modifier/task list、diagnostic和trace按delta/分页/virtualization更新；consumer lag丢旧中间态但保留终态/错误，不反压runtime。

### P1-60：测试架构把固定文案当成功，缺少真实产品、故障和性能门禁

替换为schema/transaction/compiler/operation/provider合同测试，增加default catalog/App启动、asset round-trip、undo/recovery、sandbox/PIE、network chaos、provider reload、large registry和consumer stall。测试结果绑定source/build generation。

## 7. P2 高阶能力

### P2-1：缺少复杂Magnitude/Execution表达式图与静态分析

在P1 typed model上增加曲线、数据注册表、捕获链、条件modifier和execution graph，并进行常量折叠、范围/单位、复杂度和循环分析。

### P2-2：缺少高级Ability组合、combo/input trigger和可复用graph fragment

支持ability set、combo/window、hold/release/chord、subgraph/template与版本化fragment引用，所有组合仍落到同一runtime activation/prediction合同。

### P2-3：缺少多客户端实时prediction对比和网络时间轴

同屏对齐client predicted、server authoritative与simulated proxy trace，展示packet/receipt/correction和收敛时间；数据采集需权限与隐私控制。

### P2-4：缺少跨项目/插件Tag迁移、CI审计和自动修复

建立可脚本化audit/migration plan、headless dry-run、policy gate和签名receipt，支持大型仓库分批提交与失败恢复。

### P2-5：缺少三方revision语义Diff/Merge与graph冲突解决

按stable node/field/tag ID比较base/local/remote，提供结构化冲突、可视graph merge和transactional apply，而非纯文本diff。

### P2-6：缺少Attribute/Effect曲线与平衡数据联动

集成Curve/Data Registry编辑、level sweep、敏感性图、范围告警和引用更新；preview结果绑定同一artifact hash。

### P2-7：缺少Cue Sequencer、VFX/Audio事件轨和资源预算预览

把Cue生命周期投影到Sequencer或等价时间工具，预览并发、池、距离/质量LOD和dedicated-server边界。

### P2-8：缺少批量表格、数据驱动Ability/Effect生成和schema migration工具

支持table/grid authoring、模板实例化、bulk validation和版本迁移，但生成结果仍是可审计typed asset而非隐藏代码生成。

### P2-9：缺少远程团队权限、review/approval和敏感安全策略

Tag migration、net/security policy和跨资产重写可要求review/approval，记录actor、scope、artifact与rollback receipt。

### P2-10：缺少内容级CPU/内存/带宽/rollback成本profiler

按Ability/Effect/Tag Query/Task/Cue归因runtime开销，支持预算阈值、历史趋势与同场景对照，不以静态复杂度估算代替测量。

### P2-11：缺少离线平衡、不可达、循环和数值爆炸审计

扫描activation requirements、effect cycles、infinite stack/period、unreachable task、tag contradiction和极值，输出可定位、可抑制且绑定artifact的诊断。

### P2-12：缺少第三方property/graph/task/query/cue authoring扩展生态

建立versioned schema、owner lease、sandbox、reload、compat和capability admission；插件UI只能贡献typed extension，不能注入任意业务成功反馈或直接改runtime容器。

## 8. 分层实施顺序

### M0：产品真相硬切

provider/runtime不存在时三模块显示Unavailable，删除固定Save/Compile/Apply/Playtest/Add/Rename成功路径；建立first-party catalog、capability truth table和业务route审计。

### M1：共享资产与document基础

交付五类asset type/factory/toolkit、versioned AuthoringDocument、transaction/undo、dirty/save/autosave/recovery、source control和operation identity。

### M2：Gameplay Tags作者链

接入runtime dictionary/source/redirect/query compiler，实现registry、picker、add/rename/delete/cleanup、reference scan和migration staging。

### M3：Attribute Set与Gameplay Effect作者链

实现typed details、modifier/capture/execution、duration/period/stack/requirements/cue、semantic compiler、artifact和sandbox Effect preview。

### M4：Gameplay Ability与Task graph

实现Ability Definition/Spec字段、typed graph schema/palette、cost/cooldown、event/target/task、cancel/terminal验证和真实compile/diff。

### M5：Cue与跨系统资源作者链

接入Animation/Navigation/Physics/Audio/VFX provider、Cue生命周期与资源预算，provider missing/reload返回明确结果。

### M6：Playtest、PIE与network prediction debug

建立sandbox/PIE/client-server session、instance selector、reader-gated trace、Prediction Key/receipt/rollback/reconciliation和可取消Playtest。

### M7：底部面板、job、诊断与规模资格

九类bottom view全部切到真实provider；完成job shutdown、large registry/graph、consumer stall、accessibility/localization和性能曲线。

### M8：静态Workbench收敛与产品验收

删除或隔离样例第二authority，默认入口只打开真实toolkit；Windows产品、dedicated server、目标平台、network chaos和current-build evidence全部通过后再接受完成声明。

## 9. 验收门禁

- **G-01 默认真相**：Gameplay provider/runtime缺失时Effect/Ability/Tags明确Unavailable，业务按钮不输出成功或queued。
- **G-02 产品装配**：真实App按project plugin selection装配/禁用/reload Gameplay Editor，catalog snapshot覆盖完整。
- **G-03 Asset类型**：Tag/AttributeSet/Effect/Ability/Cue均可Create/Open/Save/Reopen且stable identity不变。
- **G-04 Document事务**：所有业务编辑经typed command，undo/redo、dirty、autosave、recovery和close decision通过。
- **G-05 持久化终态**：Save acknowledgement、失败、外部变更、只读/source-control和crash recovery测试通过。
- **G-06 Shared compiler**：Editor与runtime/cook使用同一schema/diagnostic/artifact hash，无第二套验证规则。
- **G-07 Tag字典**：source/hierarchy/redirect/container/query/dense index和dictionary hash与Runtime08G一致。
- **G-08 Tag操作**：Add/Rename/Delete/Cleanup/Reference Scan/Migration dry-run与rollback覆盖冲突、循环和并发变更。
- **G-09 Tag规模**：大型registry搜索、tree、picker和reference结果virtualized且受entries/bytes/time预算。
- **G-10 Attribute编辑**：schema、default/clamp/metadata、stable ID和reload migration可视且编译通过。
- **G-11 Effect Details**：duration/period/stack/modifier/capture/execution/requirements/cue非法组合均被同源schema拒绝。
- **G-12 Effect artifact**：Compile产生真实diagnostic、dependency、hash、generation与LKG，不存在固定queued反馈。
- **G-13 Effect preview**：sandbox Apply返回真实spec/effect handle、attribute/tag/cue delta，Reset完整retire generation。
- **G-14 Effect bottom views**：Attribute Delta/Validation/Compile Log只展示真实revision-bound数据并能定位source。
- **G-15 Ability graph**：stable node/pin/link、typed连接、cycle/cardinality、copy/paste、undo和compiler source map通过。
- **G-16 Task catalog**：palette来自runtime extension catalog，provider missing/revoke/reload和schema migration可处理。
- **G-17 Activation语义**：CanActivate/Try/Commit/End/Cancel、cost/cooldown、tags和failure path可编辑且与runtime一致。
- **G-18 Target/Event**：typed target/payload、server validation policy和引用关系有compile与preview证据。
- **G-19 Cross-system task**：Animation/Navigation/Physics/Audio/VFX broker handle、cancel和provider generation正确显示。
- **G-20 Ability playtest**：operation有session/world/owner/spec/activation ID、取消/超时/失败/终态和teardown。
- **G-21 Prediction debug**：显示真实key、request/receipt/reject/catch-up/rollback/reconcile，最终与server truth收敛。
- **G-22 Net policy约束**：runtime/network能力缺失或ability不可回滚时，Predicted/Server选项被禁用并解释原因。
- **G-23 Cue authoring**：四类生命周期、资源引用、quality/server policy和预测去重preview通过。
- **G-24 Generated bottom provider**：九类Gameplay view均有typed data source、subscription lifecycle、filter/export和slow-consumer策略。
- **G-25 Job lifecycle**：compile/scan/migration/audit/playtest可取消、限时、报告进度并在Editor shutdown前终结或隔离。
- **G-26 Revision fence**：operation晚到结果不能覆盖较新document/artifact/world generation，stale状态可见。
- **G-27 插件热重载**：Editor/runtime extension owner revoke后toolkit、task、provider和trace安全retire，LKG/恢复可验证。
- **G-28 无固定业务结果**：production搜索不到 `sample persisted`、`applied +50 health preview`、固定predicted activation等成功实现。
- **G-29 Accessibility/I18n**：graph/tree/table/dialog完成键盘与screen reader路线，业务文案进入locale bundle。
- **G-30 产品故障矩阵**：missing source/provider、compile failure、disk full、runtime offline、network reject和project close均诚实终结。
- **G-31 Current-build证据**：Windows Editor、client/dedicated server和目标平台端到端结果绑定source/build/artifact generation。
- **G-32 性能声明**：只有同内容、质量、硬件、网络与采样窗口的可复现实验达到阈值，才允许声称优于Unreal。

## 10. 硬切与兼容政策

- 不保留静态Workbench与真实Gameplay toolkit两个可写authority；M8后默认产品只使用真实document/provider，样例若保留必须显式标为non-persistent showcase。
- 不保留“先改control，未来再同步document”的双轨。业务字段唯一truth是versioned AuthoringDocument，control只投影草稿或accepted revision。
- 不保留固定Save/Compile/Diff/Apply/Playtest/Add/Rename成功反馈。operation缺失、runtime offline或provider failure必须成为typed terminal error。
- 不允许Editor私有实现Tag/Attribute/Effect/Ability/Cue语义。schema、compiler、artifact和diagnostic code必须与Runtime08G共享。
- 不在Prediction/Replication未通过网络门禁前启用对应UI；隐藏或Unavailable比伪造成功更符合工程产品要求。
- 不以现有route tests或首屏视觉完整度作为业务完成证据；验收绑定current source、artifact、session/world generation和真实runtime outcome。

## 11. 当前验证结论

本轮完成46个selected files、11,217行、473,692 bytes的静态审查，登记45个test attributes、0 ignored。没有修改production code，也没有运行动态测试；已知Editor编译lane仍被既有239个错误/122个warning阻断，且重复同一未变化lane不能证明Gameplay产品行为。

实施前必须重算fingerprint并复核 `workbench_preview_actions.rs` 的在途import排序终态。Runtime08G的Tag/Attribute/Effect/Ability/Cue、asset/scene/world owner、authority/prediction/replication门禁未完成时，本篇的Apply/Playtest/Prediction只能保持Unavailable；任何静态字符串、截图、路由成功或旧测试绿色都不能覆盖这一前置关系。
