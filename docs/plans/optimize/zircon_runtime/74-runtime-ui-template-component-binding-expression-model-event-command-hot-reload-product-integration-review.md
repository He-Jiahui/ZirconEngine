---
title: Runtime UI Template、Component、Binding Expression、Model、Event、Command、Hot Reload 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime74
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime_interface/src/ui/binding
  - zircon_runtime_interface/src/ui/component
  - zircon_runtime_interface/src/ui/event_ui
  - zircon_runtime_interface/src/ui/template
  - zircon_runtime/src/ui/binding
  - zircon_runtime/src/ui/component
  - zircon_runtime/src/ui/event_ui
  - zircon_runtime/src/ui/template
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_editor/src/ui/template_runtime/component_adapter
  - zircon_editor/src/ui/template_runtime/runtime
  - zircon_editor/src/ui/host/editor_event_runtime_access/component_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/component_adapter.rs
  - zircon_editor/assets/ui
  - examples/woc/assets/ui
tests:
  - zircon_runtime/src/ui/tests/asset_action_policy.rs
  - zircon_runtime/src/ui/tests/asset_binding.rs
  - zircon_runtime/src/ui/tests/asset_component_contract.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_plan.rs
  - zircon_runtime/src/ui/tests/asset_package_validation.rs
  - zircon_runtime/src/ui/tests/binding.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/data_binding.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/retained_events.rs
  - zircon_runtime/src/ui/tests/event_manager.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes.rs
  - zircon_runtime/src/ui/tests/template/interaction_bindings.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/ModelViewViewModel/Source/ModelViewViewModel/Public/Bindings/MVVMCompiledBindingLibrary.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/ModelViewViewModel/Source/ModelViewViewModel/Private/Bindings/MVVMCompiledBindingLibrary.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/ModelViewViewModel/Source/ModelViewViewModelEditor/Private/Tests/MVVMBindingExecuteTest.cpp
  - dev/godot/core/object/object.cpp
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/godot/scene/gui/control.cpp
  - dev/godot/tests/core/object/test_object.cpp
  - dev/godot/tests/scene/test_packed_scene.cpp
  - dev/Fyrox/fyrox-ui/src/message.rs
  - dev/Fyrox/fyrox-ui/src/lib.rs
  - dev/Fyrox/fyrox-ui/src/control.rs
  - dev/Fyrox/fyrox-ui/src/button.rs
  - dev/Fyrox/fyrox-ui/src/check_box.rs
  - dev/bevy/crates/bevy_ui/src/focus.rs
  - dev/bevy/crates/bevy_ui/src/interaction_states.rs
  - dev/bevy/crates/bevy_ui_widgets/src/lib.rs
  - dev/bevy/crates/bevy_ui_widgets/src/checkbox.rs
  - dev/bevy/crates/bevy_ui_widgets/src/slider.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/SerializedDataParameter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/Volumes/VolumeCollectionTests.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 74 · Runtime UI Template、Component、Binding Expression、Model、Event、Command、Hot Reload 与 Product Integration 工程化差距

## 1. 结论

当前 Zircon 已经拥有可保留的 UI template/component 基础：`.zui` document、component contract、prototype expansion、package envelope、component reducer、surface event dispatch、dirty report、Editor component adapter 和 reload plan 都不是空目录；binding expression 也有 literal、param、prop、control-prop、比较和布尔运算 AST。问题不在于“完全没有代码”，而在于 compiler、package、runtime evaluator、component event、model adapter 与 hot reload 没有形成同一份可执行合同。

最严重的差距是公开资产格式与真实产品能力不一致。`UiBindingTargetAssignment` 可以声明 property/class/visibility/enabled/action-payload target，validation 会接受、compiler 会携带、package report 还会声明 `RuntimeBindings`，但 runtime 没有任何 target executor。`ParamRef` 会被 validator 类型检查并原样穿过 component expansion，surface 与 Editor evaluator 却都返回 `None`。component event 又不是由 typed event descriptor 决定，而是从 binding id、route、action 中用区分大小写的 CamelCase 子串猜事件类型；产品资产实际大量使用 lower_snake route。重复 component instance 的 `control.X.prop` 还会在扁平化后退化为全 surface 最小 node id。hot reload executor 最后只清 cache、换 theme、标 dirty并回报 rebuild targets，不重编译、不替换树、不迁移 state、不撤销旧 binding。

本轮登记 **5项 Runtime74 独有 P0、48项 P1、12项 P2 与48项资格门**。Runtime11A继续拥有 UI tree/layout/input/focus/accessibility、总体 surface publication和总体 asset rebuild lifecycle；Runtime64拥有通用资源版本、依赖、reload与lease；Runtime73拥有 style/theme/cascade/transition；Editor23拥有UI资产编辑、suggestion、保存/cook与预览工作流。Runtime74只拥有 template/component binding 的 compile-execute-model-command vertical contract、component instance scope、typed event/action、binding generation与reload state migration。

在 compiled binding artifact、typed endpoint、实例作用域、真实 target mutation、双向 model transaction、generation-qualified subscription、两阶段 reload、产品 parity、fault/scale/benchmark 证据全部通过前，不得把当前系统称为工程级 MVVM/模板绑定系统，也不得宣称性能或表现达到或超过当前 Unreal。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime74 责任 | 不重复登记 |
|---|---|---|---|
| UI tree/layout/input/focus/a11y | Runtime11A | compiled binding 如何定位同代 surface/node 并产生 typed mutation/command | tree、layout、hit-test、focus、navigation、a11y总体问题 |
| Dynamic gameplay UI session | Runtime43 | gameplay session 消费同一 binding artifact、model/action provider generation | FFI/session/world sync/frame/event总体合同 |
| Resource/reload authority | Runtime64 | template/component/model/action dependency如何进入binding generation与reload transaction | 通用asset handle、load、cache、lease、dependency、cancellation |
| Style/theme/transition | Runtime73 | Class target和style dirty impact的binding入口 | selector/cascade/theme/token/transition runner |
| Editor retained host与UI authoring | Editor01/23 | Editor必须消费同一 compiled binding语义，不保留第二evaluator | authoring document、suggestion、undo/save/cook/preview完整工作流 |
| Operation/security/benchmark | Runtime41/42与Tooling既有报告 | command admission、queue budget、binding workload与receipt | 通用operation、安全、evidence基础设施 |

本篇不重开 Runtime11A 的“gameplay没有权威UI action/data-binding interface”父项；本篇 P0/P1 只登记现有 template binding compiler/executor 即使接上 host 也会失败的具体语义。hot reload 同理：Runtime11A/64拥有总体 rebuild/resource transaction，Runtime74只拥有 binding generation、component state snapshot/migration、subscription teardown/rebind 和旧代际 quiescence。

### 2.2 Zircon 物理冻结

本轮核心冻结290个 production/cross-product Rust文件、50,717行、1,742,740 bytes，manifest fingerprint 为 `2a7ae05badf0f39629823862fdf39916dbfd4f54d9c99e3a122dadca64064774`；聚焦16个测试文件、5,730行、181,320 bytes，fingerprint 为 `29c1cfff384020a2a9ed868a132cb0fb7797ee96b89f4c0727c3c0da5474e516`。算法为对排序后的 `path=per-file SHA-256` 以 LF 连接、末尾不附加 LF，再做 SHA-256。结论绑定当前共享 working copy，而不是只绑定 baseline HEAD。

| 范围 | 文件 / 行 / bytes | fingerprint / 本轮证据 |
|---|---:|---|
| Public template/component/binding/event schema | 95 / 6,339 / 196,551 | `73ce57a0a5018564c0c9c864819d127f3e3d88b68f3c63b04ac77852b0db71e5`；AST、target、document、component contract、binding update/event DTO |
| Runtime compiler/component/router/reload | 154 / 29,953 / 1,029,287 | `0ef5de8ed4bf3995c37af7d0aee145a25dc285f28d03f314edb4efd1686a7a9c`；validation、prototype expansion、package/cache、reducers、event manager、reload |
| Surface execution与Editor交叉消费者 | 41 / 14,425 / 516,902 | `b6a1d7fff54039c3267d154522cfa288577dc30ef1f7dd5ff0195eb6393edd46`；action evaluator、component event matcher、control index、Editor adapter/projection |
| 聚焦测试 | 16 / 5,730 / 181,320 | validation/package/reload/router/component data binding/event dispatch/Editor adapter |
| Editor与WOC产品 `.zui` | 267 / 48,298 / 2,951,961 | `4c1b8679da2f9734512b723c96f065efd831a593aadc3f58bda8183ad7f0e9b4`；真实route/action命名与未使用能力 |

267份产品语料中有150份包含 `events =`，共1,429个 event 声明行；这些行包含1,132个 route entry、36个 action entry、0个 payload entry。全语料没有 `targets =` 或 `expression =`。`value_changed/select_option/toggle_expanded/set_visible_range/begin_drag` 分别出现16/13/4/1/1次，而对应 `ValueChanged/SelectOption/ToggleExpanded/SetVisibleRange/BeginDrag` 均为0。这不是说产品永远不能用 target，而是证明当前产品通过回避 advertised binding surface 才没有立即暴露 executor 缺失。

本轮只做 review，不修改 production/tests/assets，不运行 Cargo、Editor、WOC、真实窗口、fault、soak 或 benchmark。共享工作树存在其他 Session 修改，实施前必须重取上述三个指纹并取得对应 source owner 写租约。

### 2.3 参考物理冻结

参考侧实际冻结21个文件、23,464行、841,107 bytes，manifest fingerprint 为 `904502e5486169afba563f73e027893747f1b4d1e6d1985c5a62a9975c74aa04`。

| 参考 | 本轮采用的工程事实 | 对 Zircon 的约束 | 不外推内容 |
|---|---|---|---|
| Unreal MVVM | compiled library保存source/destination/conversion field path；显式Load/Unload后Execute/Evaluate，测试覆盖赋值、conversion与资源释放 | binding必须是可加载、可卸载、可执行的compiled artifact；endpoint、conversion与lifetime必须可验证 | 不复制UObject reflection、Blueprint compiler或C++宏 |
| Godot Object/PackedScene/Control | signal显式connect/disconnect/emit；对象销毁清连接；PackedScene保存并在instantiate恢复persistent connection，测试验证connection count与清理 | template实例化必须恢复同一代连接，销毁/reload必须可证明无旧订阅；signal identity不能靠名称子串猜类型 | 不复制Object/ClassDB或scene tree全部语义 |
| Fyrox UI | `UiMessage`显式destination、direction、handled；preview与bubble routing分层，widget处理后可set_handled | component event需要typed message、目标、方向和handled ownership；route不是裸字符串callback map | 不把Fyrox当前控件集合或单线程模型当成最终规模目标 |
| Bevy UI Widgets | widget state是typed component；`On<Pointer<T>>`、`On<ValueChange<T>>`、`Changed<T>`与observer registration显式；checkbox/slider测试覆盖disabled、focus、child bubbling与值写回 | 事件种类与value change必须由类型和change detection决定；view-model写回应是显式observer/transaction | 不复制ECS entity布局或要求所有binding都用Bevy observer |
| Unity Graphics | `SerializedDataParameter`和Volume editor以`SerializedProperty`/property path投影编辑资产并通过测试约束集合行为 | 可借鉴Editor serialized property handle、update/apply与资产级测试分层 | 本地Graphics不是Unity UI Toolkit/runtime data binding源码，不能据此声称Unity运行时UI parity |

## 3. 当前真实底座与应保留内容

| 已存在底座 | 当前价值 | 重构要求 |
|---|---|---|
| `UiBindingExpression` AST/parser | 已避免把所有条件都塞进任意脚本字符串 | 编译为有span、type、dependency、budget的IR；runtime不再逐事件parse |
| component contract/validation | 已有param、event、slot和control property kind检查入口 | contract必须生成实例作用域和typed endpoint，不只做字符串报告 |
| prototype expansion/cache/package | 已有确定性展开和依赖缓存入口 | 产出真实binding section、schema generation和可加载执行表 |
| component reducers | keyboard/menu/tree/toast等已集中状态变换 | reducer事件必须来自typed descriptor，不靠binding name token触发 |
| surface mutation/dirty report | 已有局部mutation与Layout/Render等dirty flag | target executor以事务方式复用该入口并完整映射所有dirty domain |
| action policy报告 | 已有副作用分类意识 | 分类来自注册能力和principal，不靠action id子串；error必须fail close |
| hot reload plan/executor | 已有依赖目标、cache/theme invalidation框架 | 扩展为prepare/commit/rollback、tree replace、state migration与rebind |
| Editor component adapter | 已证明复杂数据源可投影成component patch | 下沉为runtime provider contract，Editor只注册provider，不拥有第二语义 |

## 4. 五项新增 P0

### RTB-P0-001 · Target assignment 被验证、编译和打包，却没有任何 Runtime executor

`UiBindingTargetAssignment`公开支持 `Prop/Class/Visibility/Enabled/ActionPayload`。`asset/binding/validation.rs`会检查assignment expression，compiler把原始bindings克隆进node，package report声明保留 `RuntimeBindings`；但生产消费者搜索只找到validation与测试，surface dispatch只尝试生成action，从不求值或应用target。所谓package binary也只是envelope内的TOML `UiTemplateInstance`，没有compiled target program。

影响：合法资产可以通过validation/cook并在产品中静默不生效；Class/Visibility/Enabled可能进一步造成layout、hit-test、a11y与render事实分裂。必须建立compiled source/target endpoint、typed mutation instruction、事务化apply与dirty-impact receipt；在此之前target schema不得标记为runtime supported。

关闭门：四类UI target和action-payload都必须经过cook artifact在gameplay与Editor执行；无效target fail close，部分apply可回滚，receipt记录binding id、surface generation、old/new value和dirty impact。

### RTB-P0-002 · `ParamRef` 被类型检查并穿过编译，但 Runtime 与 Editor 都返回 `None`

validator把声明过的component param视为合法并推断类型；component expander和prototype instancer又把binding原样扩到实例树，没有把param替换成实例值。`pointer_component_events.rs`与Editor `runtime/projection.rs`的evaluator遇到 `UiBindingExpression::ParamRef(_)` 都返回 `None`。当前测试名为“resolves component param refs”却只断言diagnostics为空，没有build/dispatch/result断言。

影响：复用组件一旦用param构造command payload或target expression，事件会被 `.ok().and_then(...)` 静默丢弃；Editor与gameplay都可能显示控件却不执行操作。必须在compile/instantiate阶段把param绑定为typed slot或常量，并让缺值、类型错、过期generation成为显式错误。

关闭门：literal/default/override/nested component param在artifact round-trip后保持类型和值；缺失param阻断compile或instantiate，不能降级为`None`；Editor与gameplay对同一资产输出一致。

### RTB-P0-003 · Component event 类型由区分大小写的名称子串猜测，真实产品 route 与测试语义分叉

`binding_targets_component_event`在binding id、route、action里执行 `contains(token)`，token为 `ValueChanged/SelectOption/ToggleExpanded/...`。keyboard、popup、radio、timer、toast等default interaction在多数非Click事件上强制通过该猜测。测试使用 `MenuList/ValueChanged`、`SceneTree/SelectOption` 等CamelCase id；产品语料则大量使用 `value_changed/select_option/...` lower_snake route，对应CamelCase为0。任意业务action只要碰巧包含token也可能被误触发。

影响：同一component reducer在测试与产品中走不同路径；事件可能漏发、错发或因重命名改变行为。必须让component descriptor声明typed event id/payload/routing policy，compiler把asset event解析到event handle，runtime只按handle dispatch。

关闭门：事件行为与route/action文字、大小写和命名风格无关；未知event compile失败；相似子串不触发；产品lower_snake fixture与typed test共享同一执行路径。

### RTB-P0-004 · `control.X.prop` 在组件扁平化后丢失实例作用域，重复实例读取最小 node id

binding validation只在component definition内部收集control id/property kind，允许 `control.X.prop`。component expansion没有为内部control id建立instance-qualified identity，runtime evaluator再通过全surface `UiControlIndex::node_id`查找；该函数对重复control id返回最小node id，并明确称为compatibility行为。两个同类型component实例因此都可能读取第一个实例的state/metadata。

影响：列表、Inspector row、菜单项、虚拟化cell等重复组件会跨实例取值和发command，属于数据完整性与错误操作风险。必须建立 `ComponentInstanceId + LocalControlId -> NodeHandle(generation)`，compiled binding直接持有实例相对endpoint，禁止全局最小值兜底。

关闭门：至少两个和1,000个重复实例分别更新自身control并产生自身payload；销毁/复用node后旧handle拒绝；嵌套component不穿透private control scope。

### RTB-P0-005 · Hot reload 回报 rebuild target/执行成功，但不重建tree、迁移state或重绑binding

`hot_reload_plan.rs`承认在没有asset-node ownership前只能把root标dirty。executor只evict compile cache、invalidate resource cache、apply theme document并mark surfaces dirty；`template_rebuild_targets`仅复制进execution report。生产搜索也没有真实caller完成recompile/tree replace/state migration/route teardown/rebind。测试只断言cache为空、theme变化和dirty flag，不验证已实例化surface行为。

影响：修改template、component param、event、model/action schema后，旧树和旧subscription继续运行，而receipt可能被上层解释为成功；失败reload也没有last-known-good和回滚证据。必须由Runtime11A/64总体owner协调两阶段reload，Runtime74提供compiled binding generation、state snapshot/migrator、subscription lease retirement和新代际publish。

关闭门：活动Editor与gameplay surface在修改节点、param、binding、event后原子切换；可迁移state保留，不兼容state有显式reset/migration receipt；失败保持旧代可用；旧callback、model subscription和node handle全部quiesce。

## 5. P1 工程化重构清单

| ID | 差距 | 必须收敛到的合同 |
|---|---|---|
| RTB-P1-001 | `UiTemplateRuntimePipeline`与asset `UiDocumentCompiler`是两套authority，前者几乎只有测试调用 | 单一compiler facade与单一artifact schema |
| RTB-P1-002 | binding id、property、path、route、action均为裸`String` | interned/versioned `BindingId/PropertyId/RouteId/ActionId` |
| RTB-P1-003 | source/target只携node id和字符串path | generation-qualified compiled endpoint handle |
| RTB-P1-004 | 没有OneTime/OneWay/TwoWay/Event/Command模式 | 显式binding mode、触发时机与写回权限 |
| RTB-P1-005 | 没有model/provider schema registry | typed model schema、field id、provider id与版本 |
| RTB-P1-006 | 没有surface/component/row/item model context | 分层data context、inheritance与override规则 |
| RTB-P1-007 | cache key不含model/action/provider generation | dependency-complete compiled binding cache key |
| RTB-P1-008 | conversion没有typed signature和注册生命周期 | source/destination type校验、conversion handle与generation |
| RTB-P1-009 | null/optional/missing只有`Option`静默丢弃 | required/optional/default/fallback/error policy |
| RTB-P1-010 | `UiBindingValue`只有scalar/array | typed record/map/enum/asset/entity/optional与受控collection view |
| RTB-P1-011 | expression没有dependency graph | field/control/param依赖DAG与reverse invalidation index |
| RTB-P1-012 | 没有cycle检测 | compile-time cycle path diagnostic与运行时loop guard |
| RTB-P1-013 | parser/evaluator没有深度、节点、字符串、指令预算 | per-asset/per-event compile与execute budget |
| RTB-P1-014 | diagnostic缺少稳定source span和实例路径 | asset/component/node/binding/span/generation定位 |
| RTB-P1-015 | event时重新parse payload expression | canonical typed bytecode/IR，cook后不再解析文本 |
| RTB-P1-016 | map/vector遍历顺序没有成为artifact合同 | deterministic endpoint/instruction/update ordering |
| RTB-P1-017 | duplicate control id被BTreeMap覆盖 | 定义域、展开实例与最终tree三阶段唯一性检查 |
| RTB-P1-018 | component binding可直接捕获private internal control | public param/property/event/slot端点与封装边界 |
| RTB-P1-019 | action/route存在性和payload多依赖字符串 | typed action registry与compile-time capability resolution |
| RTB-P1-020 | payload kind靠`checked/committed/...`名字硬编码，否则Any | versioned payload schema与field-level validation |
| RTB-P1-021 | dirty impact只覆盖部分domain，遗漏Accessibility/Interaction/Schedule | property schema生成完整typed impact set |
| RTB-P1-022 | target写入没有prepare/apply/rollback | `UiBindingMutationTransaction`与atomic receipt |
| RTB-P1-023 | 多target、payload与action执行顺序未定义 | canonical evaluation/apply/command publication order |
| RTB-P1-024 | `RuntimeBindings`只是report enum，不是真实artifact section | 独立compiled binding section、offset与dependency table |
| RTB-P1-025 | 所谓binary envelope内部仍是TOML，无section checksum/schema matrix | versioned binary artifact、bounds/checksum/compat reader |
| RTB-P1-026 | component registry revision之外缺action/model/conversion依赖 | 全依赖generation进入fingerprint与invalidation |
| RTB-P1-027 | 默认compiler使用`editor_showcase()` registry | product/profile注入明确component catalog，不允许Editor默认泄漏 |
| RTB-P1-028 | action side-effect class由id子串推断，policy error不阻断package | registered capability/principal/admission且compile/cook fail close |
| RTB-P1-029 | Runtime与Editor各自实现expression evaluator | 共享一个consumer-neutral evaluator/executor authority |
| RTB-P1-030 | binding在输入回调中即时求值/写入 | frame safe-point batch、phase owner与snapshot semantics |
| RTB-P1-031 | model change没有subscription/change detection | provider subscription lease、field delta与poll fallback budget |
| RTB-P1-032 | view-to-model没有transaction/revision/conflict语义 | two-way write request、expected revision、validation与receipt |
| RTB-P1-033 | 同帧重复变化不去重不合并 | binding-level dedupe/coalesce与last-write policy |
| RTB-P1-034 | endpoint不带surface/tree/model generation | stale handle rejection与ABA-safe identity |
| RTB-P1-035 | command/event/model queue没有统一bytes/items/time budget | bounded queue、backpressure、drop/reject/coalesce policy |
| RTB-P1-036 | parse/eval/lookup失败被`.ok().and_then`或`None`吞掉 | typed outcome、diagnostic journal与operator-visible counters |
| RTB-P1-037 | component event由token猜测 | descriptor-generated `ComponentEventHandle`与typed payload |
| RTB-P1-038 | route注册缺owner、subscription token、generation与RAII撤销 | `UiRouteSubscriptionLease`与terminal quiescence |
| RTB-P1-039 | `UiEventRouter`只有测试/guard调用且与`UiEventManager`并行 | productize为唯一router或硬删除，不留第二authority |
| RTB-P1-040 | component data source adapter实际只有Editor registry | runtime provider contract与gameplay/App注册入口 |
| RTB-P1-041 | adapter mutation没有principal/capability/field权限 | per-endpoint read/write/command capability admission |
| RTB-P1-042 | command没有async result/cancel/progress/timeout模型 | operation handle、completion event与surface lifetime policy |
| RTB-P1-043 | two-way/command reentrancy没有transaction id和loop suppression | causality id、origin、depth guard与echo suppression |
| RTB-P1-044 | control lookup依赖全局字符串index和最小值兼容 | component-instance relative dense slot lookup |
| RTB-P1-045 | hot path会clone String/BTreeMap并逐事件parse/eval | interned IDs、SoA instruction/value arena与allocation budget |
| RTB-P1-046 | 没有按asset/binding/generation的执行、miss、error、cost telemetry | bounded binding diagnostics/profiling receipt |
| RTB-P1-047 | `UiBindingUpdateReport`不是实际engine apply结果且dirty映射不全 | executor生成old/new/revision/impact/outcome report |
| RTB-P1-048 | reload缺asset-to-node/binding ownership、两阶段publish、state migration和旧代清退 | `UiBindingReloadTransaction`、stable state key、migrator与quiescence receipt |

## 6. P2 清理项

| ID | 清理内容 |
|---|---|
| RTB-P2-001 | 移除`M18`等历史里程碑命名对当前runtime expression的语义污染，改为能力名或schema version |
| RTB-P2-002 | 在真正生成binary section前，不再把TOML envelope描述成compiled binary binding |
| RTB-P2-003 | 把payload字段名、component event名与route/action命名规范集中到typed schema，不保留散落字符串表 |
| RTB-P2-004 | 删除default interaction中的CamelCase token数组和component-name fallback |
| RTB-P2-005 | 移除`UiControlIndex`“重复id取最小值”兼容行为，重复必须在compile/instantiate时报错 |
| RTB-P2-006 | 统一route、action、event、command、binding、target术语，文档与诊断不得互换含义 |
| RTB-P2-007 | 对无生产caller的simple template pipeline与router做硬切，不用re-export维持假表面 |
| RTB-P2-008 | source-string boundary guard只证明路径/词存在，不能再命名为runtime capability test |
| RTB-P2-009 | 大型generated component catalog与手写executor分目录，避免声明、样例和行为混在同一文件 |
| RTB-P2-010 | binding error code、diagnostic id与localization key由单一模块拥有并保持稳定 |
| RTB-P2-011 | package report明确区分declared、compiled、loaded、bound、executed、applied六个状态 |
| RTB-P2-012 | 产品fixture加入param、target、repeated instance、reload和negative schema样例，停止只用route回避binding系统 |

## 7. 目标架构

```text
.zui / component contracts / model-action schemas
                    |
                    v
          UiBindingCompilerAuthority
      parse -> type -> resolve -> DAG -> IR
                    |
                    v
             CompiledUiPackage
  tree | component slots | binding program | dependencies
                    |
                    v
             UiBindingGeneration
 surface/tree/model/action/provider qualified handles
                    |
          +---------+----------+
          |                    |
          v                    v
 UiModelSubscriptionHub   UiTypedEventRouter
 field delta/change set   typed event/command/admission
          |                    |
          +---------+----------+
                    v
          UiBindingBatchExecutor
 evaluate -> prepare -> validate -> commit -> publish
                    |
                    v
 UiBindingMutationReceipt + UiBindingDiagnostics
                    |
                    v
       UiBindingReloadTransaction
 prepare new generation -> migrate -> swap -> quiesce old
```

| 目标组件 | 最小责任 |
|---|---|
| `UiBindingSchemaRegistry` | model field、component endpoint、action、event、conversion、payload type与版本 |
| `UiBindingCompilerAuthority` | 唯一parse/type/resolve/dependency/IR/artifact入口 |
| `CompiledUiBindingProgram` | 无字符串查找的source/target/event/command instruction与budget |
| `UiComponentInstanceMap` | instance id、local control slot、node handle、public/private scope |
| `UiBindingGeneration` | surface/tree/model/action/provider/catalog generation一致性 |
| `UiModelSubscriptionHub` | bounded subscription、delta、coalesce、lease与resync |
| `UiTypedEventRouter` | typed payload、routing/handled、owner generation、RAII subscription |
| `UiBindingBatchExecutor` | safe-point snapshot、deterministic evaluation、transactional target apply |
| `UiCommandGateway` | principal/capability/admission、async operation与completion |
| `UiBindingReloadTransaction` | prepare/validate/migrate/commit/rollback/quiesce |
| `UiBindingDiagnostics` | asset/binding/instance/generation/span/outcome/cost与bounded history |

## 8. 分层重构顺序

| Milestone | 内容 | 前置/退出条件 |
|---|---|---|
| Runtime74-M0 | 冻结真实能力表，target与ParamRef在未实现前fail close；加入五项P0回归fixture | 不再有“validation green但runtime silent no-op” |
| Runtime74-M1 | 合并compiler authority，建立typed schema、endpoint、event/action/payload与source span | 同一资产只产生一份语义和artifact |
| Runtime74-M2 | 生成compiled binding section、dependency DAG、cache key与version/checksum reader | cook/load/execute不再parse TOML expression |
| Runtime74-M3 | 建立component instance map和typed event router，删除substring与smallest-id fallback | repeated/nested component隔离通过 |
| Runtime74-M4 | 实现batch executor、target transaction、dirty report、model subscription和two-way write | 四类target、model-view/view-model、command通过 |
| Runtime74-M5 | runtime provider、Editor/gameplay adapter与command admission收敛 | Editor/WOC消费同一executor和schema generation |
| Runtime74-M6 | 接入Runtime11A/64 reload transaction，完成state migration、rebind与old-generation quiescence | 成功/失败reload均有原子receipt |
| Runtime74-M7 | fault/scale/soak/benchmark与产品资产迁移，关闭compat surface | 48项资格门全绿后才允许能力/性能声明 |

实施顺序不能倒置。先给现有字符串router增加更多token、在evaluator里补一个`ParamRef`分支、或让hot reload直接重建root，都只会扩大第二authority与状态丢失；必须先冻结typed schema、artifact和generation owner。

## 9. 资格门

### 9.1 Schema、compiler 与 artifact

- [ ] RTB-G001：单一compiler authority；simple pipeline与asset compiler不再产生不同语义。
- [ ] RTB-G002：duplicate component/control/binding/endpoint在definition、expansion、tree三个阶段均fail close。
- [ ] RTB-G003：binding/property/model/action/event identity为stable typed id并带schema version。
- [ ] RTB-G004：OneTime/OneWay/TwoWay/Event/Command模式及触发时机可序列化、可检查。
- [ ] RTB-G005：source/target endpoint在cook时解析为generation-qualified handle。
- [ ] RTB-G006：conversion的输入、输出、错误与provider generation通过正负测试。
- [ ] RTB-G007：required/optional/default/fallback语义覆盖missing/null/type mismatch。
- [ ] RTB-G008：record/map/enum/optional/collection view在预算内round-trip并拒绝越界。
- [ ] RTB-G009：直接和间接dependency cycle输出完整路径并阻断compile。
- [ ] RTB-G010：parser、IR、instruction、string、collection和执行深度预算可配置且fail bounded。
- [ ] RTB-G011：所有diagnostic包含asset/component/node/binding/source span与generation。
- [ ] RTB-G012：相同输入跨运行、跨机器、Editor/gameplay生成字节与instruction order一致。
- [ ] RTB-G013：package含真实binding section、endpoint table、dependency table和section offsets。
- [ ] RTB-G014：截断、篡改、未知version、checksum错误与超大section被bounded拒绝。
- [ ] RTB-G015：model/action/conversion/component/provider generation变化准确使cache失效。
- [ ] RTB-G016：未授权或未知side-effect action在compile/cook/load/execute各层fail close。

### 9.2 Runtime execution、event 与 model

- [ ] RTB-G017：Prop target通过artifact加载后真实写入并生成正确dirty impact。
- [ ] RTB-G018：Class target通过Runtime73统一style owner重算，不直接改任意字符串map。
- [ ] RTB-G019：Visibility target同步layout/hit-test/a11y/render事实。
- [ ] RTB-G020：Enabled target同步interaction/focus/a11y事实。
- [ ] RTB-G021：ActionPayload多target求值、合并、顺序和类型错误均有确定性测试。
- [ ] RTB-G022：literal/default/override/nested `ParamRef`在Runtime与Editor输出一致。
- [ ] RTB-G023：`control.X.prop`在重复和嵌套component中只读本实例并拒绝private穿透。
- [ ] RTB-G024：model-to-view field delta只重算依赖binding并按同帧策略coalesce。
- [ ] RTB-G025：view-to-model携expected revision、validation、conflict与apply receipt。
- [ ] RTB-G026：command有principal、capability、operation id、cancel/progress/timeout/completion。
- [ ] RTB-G027：missing/stale endpoint产生typed failure，不静默`None`或保留旧值。
- [ ] RTB-G028：component event由typed handle决定，route/action文字和大小写不改变事件类型。
- [ ] RTB-G029：相似名称、恶意子串、未知event与错误payload不会误触发reducer/action。
- [ ] RTB-G030：route/model/action registration返回owner-qualified RAII lease。
- [ ] RTB-G031：surface/component/provider销毁后callback、subscription和handle全部quiesce。
- [ ] RTB-G032：event/command/model queue按items/bytes/time有界，overflow有明确policy与receipt。
- [ ] RTB-G033：two-way echo、command回写和nested dispatch受transaction id/depth guard约束。
- [ ] RTB-G034：safe-point batch对同一snapshot求值，apply顺序和冲突策略确定。
- [ ] RTB-G035：Layout/HitTest/Render/Style/Text/Input/VisibleRange/Accessibility/Interaction/Schedule全量映射。
- [ ] RTB-G036：gameplay/App可注册真实model/component/action provider，不依赖Editor crate。
- [ ] RTB-G037：Editor与gameplay对同一artifact、model snapshot、input trace输出相同receipt。
- [ ] RTB-G038：shipping/cooked package无需source `.zui`或Editor registry即可load/bind/execute。

### 9.3 Reload、产品与非功能资格

- [ ] RTB-G039：修改node/param/target/event/action后活动surface原子切到新generation。
- [ ] RTB-G040：compile/load/migration/rebind任一失败均保持last-known-good并给出rollback receipt。
- [ ] RTB-G041：focus、selection、text edit、scroll、expanded、validation等state按稳定key迁移或显式reset。
- [ ] RTB-G042：旧generation route/model callback/node handle不能在publish后继续产生effect。
- [ ] RTB-G043：2、1,000和虚拟化复用component实例通过隔离、stale-handle与ABA测试。
- [ ] RTB-G044：Editor与WOC产品资产真实使用param、target、typed event、command和reload，不只测validation。
- [ ] RTB-G045：missing provider、slow command、panic/error callback、malformed artifact、reload storm均fault-contained。
- [ ] RTB-G046：大model、大列表、高频delta、长会话与反复open/close/reload通过scale/soak且内存稳定。
- [ ] RTB-G047：compile、bind、steady frame、dirty frame、event、reload有可复现benchmark及allocation/cache指标。
- [ ] RTB-G048：只有真实Unreal/Fyrox/Bevy/Godot/Unity适用工作负载同硬件对比后，才允许性能或表现宣称。

## 10. 不得作为完成证据

- parser能构造AST、validator没有diagnostic，不等于expression在runtime执行。
- package report出现`RuntimeBindings`，不等于artifact含compiled binding program。
- target assignment被serde round-trip，不等于property/class/visibility/enabled发生mutation。
- 测试资产使用CamelCase binding id，不等于产品lower_snake route能触发同一component event。
- dirty flag或rebuild target被回报，不等于tree、state、subscription和binding已切换generation。
- Editor adapter能生成projection patch，不等于gameplay/runtime有model provider与two-way transaction。
- `UiEventRouter`存在且单测通过，不等于产品把它作为唯一router。
- source guard找到某个符号、字段或路径，不等于有production caller和behavior receipt。
- 当前产品没有使用target/param，不等于advertised schema是安全的；它只说明产品绕开了缺失能力。
- 文件数量、测试数量、参考引擎数量和静态审查完成，都不能替代运行时产品资格。

## 11. Owner、状态与实施前复核

Runtime74是本纵切面的review owner，不是现阶段production implementation owner。实现必须拆入Runtime11A/43/64/73、Editor23及实际compiler/component/surface owner计划，并通过Coordinator取得源码写租约；不得在本报告里直接“顺手”改Rust。

当前状态：`review_complete / implementation_pending / source_recheck_required`。MVP总计划仍处于基础阻断期，本轮没有授权实现高级UI绑定系统。开始Runtime74-M0前必须重新检查：五项P0路径是否漂移、Runtime11A/73或Editor23是否已改变owner、两个compiler/router是否已有新production caller、产品ZUI语料是否新增target/param，以及开放failure是否出现新的跨计划移交。

本报告只证明当前源码差距已形成可执行重构规格，不证明任何一项修复完成。
