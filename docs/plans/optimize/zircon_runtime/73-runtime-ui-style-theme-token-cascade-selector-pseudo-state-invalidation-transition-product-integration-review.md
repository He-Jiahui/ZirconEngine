---
title: Runtime UI Style、Theme、Token、Cascade、Selector、Pseudo-state、Invalidation、Transition 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime73
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/asset/prototype.rs
  - zircon_runtime_interface/src/ui/template/asset/style.rs
  - zircon_runtime_interface/src/ui/v2
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/theme
  - zircon_runtime/src/ui/v2
  - zircon_runtime/src/ui/template/asset/style.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply
  - zircon_runtime/src/ui/template/asset/component_contract
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/property_mutation/metadata_dirty.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/layout_transitions.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/shared.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/asset_editor/node_projection.rs
  - zircon_editor/src/ui/layouts/views/view_projection
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/transition_metadata
  - examples/woc/assets/ui
  - zircon_editor/assets/ui
tests:
  - zircon_runtime/src/ui/tests/asset_component_contract.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/asset_prototype_store.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/layout.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_x_runtime.rs
  - zircon_runtime/src/ui/tests/v2_asset/composite_components.rs
  - zircon_runtime/src/ui/tests/v2_asset/demo_and_builder.rs
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/ISlateStyle.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/SlateStyleRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/SlateStyleRegistry.cpp
  - dev/godot/scene/resources/theme.h
  - dev/godot/scene/resources/theme.cpp
  - dev/godot/scene/gui/control.cpp
  - dev/bevy/crates/bevy_feathers/src/theme.rs
  - dev/Fyrox/fyrox-ui/src/style/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/CoreEditorStyles.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 73 · Runtime UI Style、Theme、Token、Cascade、Selector、Pseudo-state、Invalidation、Transition 与 Product Integration 工程化差距

## 1. 结论

当前 Zircon 已经拥有可保留的样式底座：公共 selector parser 使用 tuple specificity，V2 能区分 static 与 pseudo-state rule，runtime state 变化能局部或按 subtree 重算，token 可以递归指向 document token 或 theme palette，Editor file cache 能加载样式依赖，surface metadata 也保留部分 token provenance。不能把这些实现删除后换成另一套临时 CSS 字符串解释器。

但当前并不存在一个统一、工程级、产品语义一致的 UI style system，而是至少三套并行权威：legacy typed `StyleProperty/StyleField/StyleSheetScope` 基本只停留在局部 material button/test；V1 template compiler 以静态属性生成约4,865行 MUI web class mirror；V2 再以 `BTreeMap<String, toml::Value>` 实现另一套 static/runtime cascade。它们对 component scope、imports、theme、pseudo-state、slot declaration、inline origin、dirty classification 和 transition 的解释不同，且最终 renderer 继续按字符串 key 二次解析。

本轮登记 **5项 Runtime73 独有 P0、48项 P1、12项 P2 与44项资格门**。五项 P0 均是产品真值问题：默认 `Closed` component scope、`:part()` 与 `:host` 从未进入真实匹配；gameplay runtime prototype store 不合并 `imports.styles` 或组件文档的 token/stylesheet，WOC 132个非空 style import 中的真实主题链会被跳过；gameplay surface 没有 theme owner，而 Editor 使用 active theme；hot reload 返回 theme changed 并只标 dirty，却不重建 resolved style/runtime index；公开的 Collapse/Fade/Grow/Slide/Zoom 没有 transition clock/runner，progress 从未被 runtime 推进。

Runtime11A 继续拥有 UI tree、layout、input、navigation、accessibility、总体 hot-reload/rebuild lifecycle；Runtime11B/11C 继续拥有 text 与 GPU painter。Runtime73 只拥有 style schema、cascade、component encapsulation、theme/token generation、selector dependency、style invalidation和transition execution的当前源码差距。完成 typed schema、统一 compiler、generation-qualified theme/style binding、真实 gameplay/Editor parity、pixel/fault/scale/benchmark 证据前，不得宣称该系统达到、更不能宣称性能或表现超过当前 Unreal。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime73 责任 | 不重复登记 |
|---|---|---|---|
| UI tree/layout/input/a11y/publication | Runtime11A | style delta 产生何种 typed dirty impact 与 style generation handoff | tree owner、layout算法、hit-test、focus/nav、a11y总体P0 |
| Text/font/shaping 与 GPU painter | Runtime11B/11C | style property 到 typed text/paint input 的合同 | shaping、glyph atlas、GPU submit、pixel alpha/clip父问题 |
| Dynamic gameplay UI session | Runtime43 | gameplay surface 的 style/theme/import binding parity | FFI/session/world/event/shader prewarm总体边界 |
| Resource/import/reload authority | Runtime64 | UI style/theme dependency generation 与 rebind receipt | 通用asset handle/cache/reload/cancellation父问题 |
| Editor retained host与UI authoring | Editor01/23 | Editor消费同一 compiled style generation，不保留第二语义 | Dock/host、UI document/transaction/save/preview完整工作流 |
| Hot-path与证据治理 | Tooling32、O07/O10/O11/O14 | selector/cascade/invalidation workload 和预算 | 通用benchmark/evidence控制面 |

本篇不重开 Runtime11A 已登记的“整个 UI asset hot reload 不完整”父项；本篇第四项P0只拥有当前 `execute_runtime_reload` 对 theme reload 返回成功 receipt、但不重新求值任何已存在 surface 的精确合同缺陷。类似地，Runtime43拥有 gameplay session 生命周期，本篇只拥有同一 `.zui` 在 gameplay 与 Editor 中产生不同 style/theme 结果的语义分叉。

### 2.2 Zircon 物理冻结

本轮核心冻结57个 production/product 源码、18,962行、655,639 bytes，manifest fingerprint 为 `f6549fa8040e551eaf0c7e453e2347ce57a0e7c8f7abb69b809d27c76f47779b`；聚焦13个测试文件、5,394行、162,808 bytes，fingerprint 为 `fa2f8748cb3b502f1cc21f3d08c77fdd1ac43c6985e54558ac1d77fbf299cb8e`。算法为对排序后的 `path=per-file SHA-256` 以 LF 连接、末尾不附加 LF，再做 SHA-256。结论绑定当前共享 working copy，不把 baseline HEAD 单独当作源码事实。

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public style/theme/V1/V2 schema | 6 / 1,705 / 56,378 | color/theme DTO、selector grammar、specificity、raw declaration/resolved maps、component scope |
| Compiler/cascade/component/surface integration | 36 / 12,938 / 450,195 | V1/MUI、V2 instancing/import/cache/static/runtime cascade、slot、token、surface metadata |
| Theme/reload/product/transition/Editor projection | 15 / 4,319 / 149,066 | gameplay/Editor builder差异、theme registry、reload receipt、transition descriptor与静态投影 |
| 聚焦测试 | 13 / 5,394 / 162,808 | scope parser/compile、prototype、file cache、theme、pseudo-state、inline override与transition metadata |
| 非测试 `.zui` 资产语料 | 297 / 49,773 / 3,012,860 | 1,017 selectors、149 pseudo selectors、2,376 token引用、132个非空style imports、214个component header |

297份产品/样例/插件 `.zui` 的独立 fingerprint 为 `35811f294fddfa16d75401b4b4c58ed63d0a1483e5f7048b766c2075a776d511`。其中没有一处显式 `style_scope`、`:part()` 或 `:host`，但 `UiStyleScope::default()` 为 `Closed`；这说明组件封装是隐式公共承诺，不是“产品没有写该字段所以无需实现”。语料另有5处 inline style token，transition相关7行全部只是 `editor_material.zui` duration token，未发现产品 transition component 实例。

本轮只做 review，不修改 production/tests/assets，不运行 Cargo、Editor、WOC、真实窗口、pixel、fault、soak 或 benchmark。源码处于多Session共享工作树，实施前必须重新取指纹并取得对应 owner 的写租约。

### 2.3 参考物理冻结

参考侧9个实际读取文件、8,948行、326,125 bytes，manifest fingerprint 为 `d81c430cc54118b8d2d7acfefe9c19e804c24a80edfe5a58334a535a1988286d`。

| 参考 | 本轮采用的工程事实 | 对 Zircon 的约束 | 不外推的内容 |
|---|---|---|---|
| Unreal Slate | `ISlateStyle`提供typed float/vector/color/margin/brush/sound/font/widget style lookup、key/resource enumeration与missing resource记录；registry显式register/unregister/find并通知renderer加载资源 | style key必须连接typed schema、resource lifetime、registry generation与diagnostics；注册/替换不是裸map覆盖 | 不复制C++宏、global singleton或Slate widget模型 |
| Godot Theme/Control | Theme按typed item/type map存储，校验名称、发出changed、支持type variation与dependency chain；Control传播theme owner并按类型依赖查询 | theme owner、inheritance/type dependency、change propagation和cycle稳定性必须是显式模型 | 不复制Godot Object/ClassDB或其具体property naming |
| Bevy Feathers | `ThemeToken`/`UiTheme`由change detection把token投影到background/border/text；缺token发warning并返回明显错误色 | 缺token不能静默保留原字符串；theme generation变化必须触发明确consumer重投影 | Feathers当前主要覆盖颜色，不能冒充完整engine UI baseline |
| Fyrox UI | 有限typed `StyleProperty`、`StyledProperty<T>`、parent style与message/build-time style应用 | 即使功能较小，也证明typed property和运行时style owner优于任意string/value map | 不把Fyrox当前selector/继承细节当作唯一目标 |
| Unity Graphics | `CoreEditorStyles`延迟创建GUIStyle/texture，区分light/dark并在assembly reload清理资源 | Editor style资源必须有生命周期和theme variant owner | 本地Graphics仓不是完整Unity UI Toolkit/runtime cascade源码，不据此外推Unity整体 |

## 3. 当前真实底座与应保留内容

| 已存在底座 | 当前价值 | 重构要求 |
|---|---|---|
| tuple `UiSelectorSpecificity` | 不用加权整数决定cascade，避免class数量与ID错误打平 | 保留语义，改为有预算的compiled selector program |
| V2 static/runtime rule分离 | pseudo-state变化无需总是重新解析全部asset | 收敛为同一compiled stylesheet generation，不保留两套rule clone |
| runtime ancestor pseudo检测 | 已识别ancestor state可能影响descendant | 编译成dependency index和bounded invalidation set |
| token source metadata | 能记录部分`token.a -> theme.x`链 | 改为typed provenance DAG、cycle/missing diagnostics和generation |
| Editor file cache依赖加载 | 能解析`res://`与asset id依赖并缓存compiled document | 修正import precedence、scope和Runtime parity，不再flat extend所有依赖 |
| surface dirty transaction | style变化能进入layout/text/render dirty链 | dirty impact由property schema生成，不由字符串allowlist猜测 |
| component descriptor schema | transition/property已有类型、default/range基础 | descriptor必须连接真实behavior provider，未实现的transition不得报告可用 |

## 4. 五项新增 P0

### RST-P0-001：默认 Closed component scope、`:part()` 与 `:host` 是不可执行的封装合同

`UiStyleScope`默认值为 `Closed`，V1/V2 component definition都公开该字段；selector parser也接受`:part(name)`与`:host`。但 production matcher在V1和V2对`Part`都无条件返回false，`Host`只认整个展开文档的root，不认每个component instance host；`style_scope`在runtime只被复制到flat/editor DTO，没有参与cascade。V1 validation最多检查声明的public part和private selector target，测试也只断言“能编译/会拒绝”，没有验证最终styled metadata或像素。V2 compiler/instancer连等价contract validation都没有。

结果同时违反封装的两边：外部type/class/descendant selector可穿透默认Closed组件内部，而唯一宣称可开放的`:part()`永远匹配不到；`:host`也无法针对嵌套组件实例。214个产品component header都隐式接受Closed默认值，因此不是未使用API。必须让component expansion保留instance boundary、host identity、scope generation和public part map；selector compiler在跨boundary时fail-close，只有Open或显式part export允许穿透。完成前要么删除/拒绝这些声明，要么把组件样式能力标为Unavailable，不能继续报告Closed/part支持。

### RST-P0-002：gameplay runtime不消费`imports.styles`或组件文档样式，Editor与Runtime编译同一资产得到不同级联

Editor `UiV2PrototypeStoreFileCache`把root与所有依赖source收集后，通过`root_document_with_imported_styles`把依赖tokens/stylesheets扁平extend进root；gameplay `project_ui_prototype_store`只把文档插入store并验证reachable imports。`build_surface_with_prototype_store`虽用store展开component node，却仍把原始root document传给static resolver、runtime style index和surface tree。instancer返回的compiled document又只保留arena，不返回合并后的style context。

因此 gameplay 中`imports.styles`文档完全不进入rule/token collection，组件文档自带的stylesheet/token也不随展开节点进入cascade。WOC的13个主要view明确依赖shell/hud theme imports；297份非测试ZUI共有132个非空style import。这是已启用产品资产链，不是理论路径。必须建立单一 `CompiledUiStyleBundle`，由依赖图按显式layer/order解析root/style/component文档并同时交给arena、static resolver、runtime index和reload dependency index；Editor与gameplay只能消费同一个bundle fingerprint。

### RST-P0-003：gameplay surface没有theme owner，同一theme token资产只在Editor解析

Editor runtime host保存`active_theme`并调用themed surface builder；gameplay `RuntimeUiSurfaceSet::load`只调用无theme的`build_surface_with_prototype_store`，project/session也没有theme registry、theme asset选择、generation或rebind owner。token resolver在没有registry时会把`$theme.*`/`var(theme.*)`原样留下，unknown role也静默保留为`UiStyleColor::Role`。

同一资产在Editor preview可被解析成颜色，在实际gameplay却可能把theme字符串送到renderer ad-hoc parser；这破坏preview/runtime parity和可重复产品构建。必须让project manifest/compiled UI root选择qualified theme asset，Runtime session持有`ThemeGeneration`，所有surface绑定明确theme id/fingerprint，缺失/unknown token在build admission时报typed error或显式fallback receipt。Editor preview必须使用同一artifact和generation，而不是私有active theme覆盖。

### RST-P0-004：theme hot reload返回成功receipt但不重新求值任何已存在surface

`UiAssetHotReloadPlan::execute_runtime_reload`会调用`UiThemeRegistry::apply_document`并返回`UiThemeReloadOutcome { changed: true }`，随后`UiAssetSurfaceIndex::mark_target_surfaces_dirty`只设置node/root dirty flags。surface里保存的attributes/style_overrides和`UiV2RuntimeStyleIndex`已经是旧theme解析结果；dirty rebuild不会重新运行selector/token resolver，也没有theme rebind API。template rebuild targets同样只是复制到report，没有执行rebuild。该executor在production没有caller，只有re-export和测试。

现有测试只断言registry fingerprint变化、target surface和dirty flag，不检查metadata、layout、render command或像素变化。必须把reload实现为prepare new dependency/style/theme generation、resolve affected nodes、原子publish surface binding、retire old generation的事务；失败保留last-good，receipt报告rebound/unchanged/failed/missing surfaces和old/new generation。单纯“标dirty”不得称为theme restyle成功。

### RST-P0-005：Collapse/Fade/Grow/Slide/Zoom公开为组件，但没有transition执行器

catalog公开五个transition component，descriptor包含`in`、status、progress、duration、easing、mount/unmount以及end listener形态。全production搜索没有transition runner、clock、timeline、scheduler或progress writer；runtime只把这些key分类为dirty。Editor projection读取authored metadata，`entering/exiting`且缺progress时直接回退固定0.5，paint仅用静态progress调整opacity；没有代码随时间推进、取消、反转或完成transition。

`mount_on_enter`、`unmount_on_exit`、`addEndListener`因此也是不可执行合同，Collapse/Slide几何变化也没有runtime owner。必须在component availability中区分descriptor与behavior provider；建立per-surface transition scheduler、clock domain、interrupt/reverse policy、reduced-motion、mount lifetime、end/cancel receipt和dirty impact。未接入runner前五个组件必须fail-close或明确Unavailable，不能继续用静态默认值冒充动画完成。

## 5. P1 重构清单

### 5.1 Schema、selector 与 cascade contract

| ID | 当前差距 | 重构要求 |
|---|---|---|
| RST-P1-001 | V2 rule/property最终只是`BTreeMap<String, toml::Value>` | 建立stable `StylePropertyId`与typed value enum，serialization只负责authoring边界 |
| RST-P1-002 | 没有单位、范围、百分比/auto/color/resource/font/brush等统一grammar | property schema声明parser、validator、canonical form与错误span |
| RST-P1-003 | 没有inheritance、initial/unset/explicit inherit与inherited-property表 | 在schema中声明inheritance/default，并生成computed style |
| RST-P1-004 | renderer/painter按别名字符串重复解析相同属性 | compiler一次canonicalize alias，consumer只读typed slot |
| RST-P1-005 | theme/color/typography/shape/spacing/control/elevation反序列化无有限值和唯一键校验 | theme admission执行finite/range/name/duplicate/resource validation |
| RST-P1-006 | registry只解析13个palette角色，theme其余公开字段不可寻址 | 提供typed token namespace覆盖全部theme item和custom extension |
| RST-P1-007 | token递归深度硬编码8，缺失、环、fallback与空白grammar无诊断 | 编译token DAG，检测cycle/missing，支持typed fallback和budget |
| RST-P1-008 | selector无长度、segment/token/ancestor-walk预算和identifier grammar | asset admission实施bounded parser与合法identifier规则 |
| RST-P1-009 | selector能力只有type/class/id/state/part/host与两种combinator，也无显式capability/version | 冻结支持子集与schema version；新增语义走feature gate/migration |
| RST-P1-010 | stylesheet/rule id在`ResolvedRule`中被丢弃 | 保留source asset/sheet/rule/span/layer/order与winner provenance |
| RST-P1-011 | legacy typed style、V1 static MUI和V2 runtime cascade三套权威并存 | 硬切到单一compiler/runtime representation，删除compat facade |
| RST-P1-012 | V1生成MUI状态class，V2不生成，同资产两入口语义不同 | MUI import只作为authoring adapter，输出canonical native property/state |

### 5.2 Import、component、inline 与 slot

| ID | 当前差距 | 重构要求 |
|---|---|---|
| RST-P1-013 | Editor flat merge先放root再append imports，equal specificity时import反而后胜 | 定义显式import layer和local-last source order，加入冲突测试 |
| RST-P1-014 | file cache把widget依赖的tokens/stylesheets也全局extend，Closed component局部样式泄漏 | component stylesheet保留scope，不把widget dependency当global style import |
| RST-P1-015 | direct builder、prototype-store builder与file-cache builder编译上下文不同 | 三入口只接收同一种compiled bundle，结果fingerprint必须相同 |
| RST-P1-016 | static surface先resolve attributes，却把inline `node.style.self_values`原样复制到style_overrides | inline value也走token/theme/type resolver并记录origin |
| RST-P1-017 | runtime用“inline值是否不同于base值”猜inline优先级；值恰好相等时pseudo可覆盖inline | 每个computed property保存origin/layer/specificity/order，不比较value猜来源 |
| RST-P1-018 | resolver计算`resolved.slot`，surface tree没有消费者 | slot declaration必须进入child mount computed slot style或删除公开字段 |
| RST-P1-019 | runtime pseudo apply只处理`self_values`，动态slot rule永远不更新child layout | dependency index把slot owner映射到children并产生typed layout invalidation |
| RST-P1-020 | component instancing只展平node，component-local style/token origin不进入compiled artifact | compiled component prototype携带style scope、token namespace与dependency generation |
| RST-P1-021 | 没有cascade layer、importance、author/user/runtime override策略 | 定义有限native layer enum和稳定precedence，不复制无限CSS surface |
| RST-P1-022 | public part只是一段字符串，节点没有typed part export/instance boundary | component contract生成`PartId`、host/part map与compile-time visibility diagnostics |

### 5.3 Runtime pseudo-state、invalidation 与成本

| ID | 当前差距 | 重构要求 |
|---|---|---|
| RST-P1-023 | static resolve对每个node扫描每条rule并逐祖先匹配，成本约`O(N*R*D)` | selector program按terminal key索引，预编译ancestor predicates |
| RST-P1-024 | pseudo update对affected node再次扫描全部runtime rules；ancestor state可全subtree重算 | 编译state-to-rule与ancestor dependency set，按实际受影响node/property增量更新 |
| RST-P1-025 | V1/V2重复selector matcher且`Part/Host`语义分别漂移 | 只有一个compiled matcher和conformance corpus |
| RST-P1-026 | runtime index为每个node克隆attributes、style_overrides、token map，apply再次克隆比较 | 使用immutable computed-style arena、structural sharing与dirty property bitset |
| RST-P1-027 | 没有selector terminal index、class/id/state dependency或rule invalidation graph | artifact生成rule index和mutation dependency plan |
| RST-P1-028 | runtime保留`enabled`属性但只发出`disabled`state，`:enabled`永远不匹配 | canonical state registry生成互补状态和alias |
| RST-P1-029 | `focus-visible/focus_visible/focusVisible`、drop/popup等alias集合在static/runtime间不一致 | authoring阶段canonicalize为`PseudoStateId`，runtime不再比较字符串别名 |
| RST-P1-030 | 任意bool property名可被当custom pseudo-state，schema与selector依赖隐式耦合 | component schema显式声明state export与mutation权限 |
| RST-P1-031 | `visual_state_for_family`忽略family参数，family mapping只是硬编码component字符串表 | painter family由component descriptor注册并拥有typed state recipe |
| RST-P1-032 | legacy scalar state会把focus压成单一primary，组合状态信息丢失 | 全consumer迁移`UiPainterVisualState`/typed state layers后删除scalar路径 |
| RST-P1-033 | dirty impact由字符串allowlist分类，新增属性容易误分layout/text/render | property schema生成impact mask、dependent resource与cache invalidation |
| RST-P1-034 | public mutation主要改attribute；class/control-id/selector identity没有受管mutation与重算合同 | 提供style identity transaction，更新index、control map、cascade和dirty receipt |

### 5.4 Theme、reload 与 product diagnostics

| ID | 当前差距 | 重构要求 |
|---|---|---|
| RST-P1-035 | theme fingerprint用JSON+`DefaultHasher`，序列化失败退回id | 使用versioned canonical bytes和stable content hash |
| RST-P1-036 | unknown theme role静默保留原role/string | build admission报missing token；允许fallback时receipt必须记录 |
| RST-P1-037 | hot reload executor没有production caller | 接入asset watcher/project operation或删除“runtime executor”公开可用表述 |
| RST-P1-038 | template rebuild target只返回字符串，没有执行、排队或terminal结果 | 交给bounded rebuild operation并返回per-surface terminal receipt |
| RST-P1-039 | surface index只标dirty，不更新compiled/style/theme generation | index维护asset-to-style dependency与generation-qualified rebind plan |
| RST-P1-040 | surface metadata的token provenance不是theme dependency owner | 保存typed token DAG edge、source generation与resolved value origin |

### 5.5 Transition、MUI mirror、测试与可观测性

| ID | 当前差距 | 重构要求 |
|---|---|---|
| RST-P1-041 | transition status/progress完全由caller authored，无runtime状态机 | runner拥有idle/entering/entered/exiting/exited/cancelled状态和时间推进 |
| RST-P1-042 | mount/unmount/end listener字段没有行为consumer | 生命周期与listener只在transition terminal commit后执行且可取消/重入 |
| RST-P1-043 | duration/easing是未解析字符串，没有reduced motion、time scale或clock domain | 编译typed curve/timing policy并绑定UI clock/reduced-motion设置 |
| RST-P1-044 | 10个MUI class模块约4,865行，手工镜像数百web class分支 | 迁移为数据驱动authoring adapter；runtime不携带web class taxonomy |
| RST-P1-045 | arbitrary component/variant/color/size字符串会合成`Mui*`class | descriptor enum/extension registry验证，unknown值typed reject |
| RST-P1-046 | 多个测试只检查source字符串、compile成功或metadata字段，没有真实import/theme/pixel变化 | 增加behavior、gameplay/Editor parity、golden pixel与negative corpus |
| RST-P1-047 | 没有node/rule/depth/state-churn规模benchmark、内存预算或dirty amplification门 | 建立固定workload与p50/p95/p99、allocation、changed-node指标 |
| RST-P1-048 | selector错误只有原字符串，runtime report没有winner/conflict/span/layer | 提供cascade inspector、source span、winning/overridden rule和fallback diagnostics |

## 6. P2 清理项

| ID | 清理项 | 处理 |
|---|---|---|
| RST-P2-001 | snake/camel/hyphen state/property alias散落 | 迁移时一次canonicalize并发布deprecation diagnostics |
| RST-P2-002 | `legacy_display_score`仍暴露历史加权显示概念 | inspector改显示tuple specificity，删除易误用score |
| RST-P2-003 | stylesheet/rule id允许空字符串 | loader生成/要求稳定authoring id |
| RST-P2-004 | token provenance用自由文本`a -> b` | 改为structured edge array，UI再格式化 |
| RST-P2-005 | token depth 8是裸常量 | 在移除递归解释器前提升为schema budget并记录超限 |
| RST-P2-006 | 默认theme硬编码`Inter`但不绑定font asset/fallback | theme typography引用qualified font resource |
| RST-P2-007 | `UiRgbaColor::new` clamp而derived Deserialize绕过，API语义不一致 | custom Deserialize走validated constructor或拒绝非法值 |
| RST-P2-008 | theme registry每个palette role手写match | schema/codegen生成token table和枚举文档 |
| RST-P2-009 | Editor transition缺progress时固定0.5 | runner落地前不渲染伪中间状态，改为Unavailable/terminal snapshot |
| RST-P2-010 | 注释宣称“canonical token registry”，实际inline/import/theme路径分叉 | 实现统一后再恢复声明，当前文档明确限制 |
| RST-P2-011 | MUI class文件按web产品族平铺且单文件数百行 | adapter迁移期间按schema domain拆分生成表与测试fixture |
| RST-P2-012 | style map多处使用字符串path拼接表示nested token | typed property path/field id替代format字符串 |

## 7. 目标架构

| 组件 | 所属 | 责任 |
|---|---|---|
| `UiStyleSchemaRegistry` | runtime_interface/runtime | stable property/state/token/part id、typed value grammar、inheritance、dirty impact、animatability |
| `UiStyleDependencyGraph` | asset/compiler | root/style/component/theme import DAG、layer/order/scope、cycle和budget validation |
| `CompiledSelectorProgram` | compiler artifact | bounded selector bytecode、terminal index、specificity、ancestor/state dependency与source span |
| `CompiledUiStyleBundle` | asset artifact | arena与style context同代发布，含schema/theme/dependency/build fingerprint |
| `ComponentStyleBoundary` | component compiler | instance host、Closed/Open scope、public part map、slot/style namespace |
| `ComputedStyleArena` | runtime UI | typed property slots、origin/provenance、structural sharing、parent inheritance和generation |
| `UiStyleDependencyIndex` | surface | state/class/id/theme/token/resource变化到rule/node/property的精确影响集 |
| `ThemeGeneration` | project/runtime | qualified theme asset、canonical hash、resolved token DAG、last-good与consumer binding |
| `UiSurfaceStyleBinding` | surface | compiled bundle/theme/schema generation和atomic rebind/retire receipt |
| `UiStyleMutationTransaction` | surface | property/state/class/identity变化的preflight、cascade commit、dirty impact与rollback |
| `UiTransitionScheduler` | runtime UI | clock、curve、interrupt/reverse、mount lifetime、reduced motion、terminal event与budget |
| `UiCascadeDiagnostics` | runtime/editor | winner/overridden origin、missing token、scope rejection、fallback与cost counters |

关键数据流必须收敛为：

`source documents -> validated dependency graph -> compiled selector/property/token bundle -> theme-bound computed style generation -> surface dependency index -> typed layout/text/paint delta -> atomic publication`。

禁止继续让component instancer只返回arena、让caller另传原始document；禁止renderer自行解释新属性；禁止以`mark_dirty`替代style/theme重新求值；禁止在V1/V2之间增加新的compat re-export。

## 8. 实施里程碑

### M0 · Product truth与parity冻结

- 对三种builder入口、WOC roots、Editor roots生成style/import/theme parity manifest；
- 未实现的scope/part/host/transition fail-close，hot reload不再返回虚假成功；
- 冻结property/state/token/selector现有语料与unknown-value diagnostics。

### M1 · Typed schema与compiled artifact

- 建立`UiStyleSchemaRegistry`、canonical value grammar、impact/animatability/inheritance；
- 编译selector、token DAG、stylesheet origin和dependency graph；
- 让compiled document同时拥有arena与style bundle，删除原始document旁路。

### M2 · Component boundary与import hard cutover

- 实现instance host、Closed/Open、public part、slot style与component-local token namespace；
- 修正import layer/order，Editor/gameplay共享bundle fingerprint；
- 删除flat extend widget style与不可执行`:part/:host` matcher分支。

### M3 · Computed style与增量invalidation

- computed arena保存typed origin，不再按value猜inline precedence；
- 编译state/class/id/theme/token dependency index；
- mutation transaction只更新受影响property/node并产生typed dirty impact。

### M4 · Theme generation与reload transaction

- project/runtime拥有qualified theme generation，覆盖全部公开theme item；
- missing/cycle/fallback有typed diagnostics，theme hash稳定；
- reload prepare/resolve/publish/retire并保留last-good，Editor与gameplay同代切换。

### M5 · Transition runtime

- 建立per-surface scheduler、clock、typed easing、reduced motion、interrupt/reverse；
- mount/unmount/end/cancel有terminal receipt；
- Collapse/Slide驱动几何，Fade/Grow/Zoom驱动typed paint/layout delta，而非静态metadata。

### M6 · V1/MUI/legacy authority硬切

- MUI class generator降为离线/authoring adapter；
- 所有产品asset迁移canonical schema，删除V1/V2双matcher与dead typed style shell；
- 不保留pub use、compat module或双写computed metadata。

### M7 · Product qualification与性能

- 真实WOC、Editor、plugin view完成load/reload/theme switch/state churn/pixel parity；
- fault、cycle、missing token、large selector、rapid mutation、reduced motion和shutdown通过；
- 同硬件固定workload记录CPU/frame、allocation/RSS、dirty amplification与pixel correctness后才做竞争性比较。

## 9. 资格门

| Gate | 验收内容 |
|---|---|
| RST-G01 | 所有公开style property有stable id、typed value、default、inheritance与dirty impact |
| RST-G02 | unknown property/value/unit在asset admission给出source span和typed error |
| RST-G03 | selector长度、segment、token、depth与compile/runtime work均有预算 |
| RST-G04 | specificity/order/layer由compiled artifact决定且有conformance corpus |
| RST-G05 | stylesheet/rule/source span与winning origin保留到runtime diagnostics |
| RST-G06 | direct/store/file-cache三入口对同输入生成相同bundle fingerprint |
| RST-G07 | local rule在定义的layer中按合同覆盖import，import顺序确定性 |
| RST-G08 | widget import不把Closed component样式泄漏为global rule |
| RST-G09 | component instance拥有独立host identity和scope generation |
| RST-G10 | Closed阻断外部穿透，Open与public part只开放明确目标 |
| RST-G11 | `:host`对每个实例而非文档root匹配 |
| RST-G12 | `:part`命中typed exported part，private/unknown part编译失败 |
| RST-G13 | slot rule静态与pseudo变化都更新正确child mount并触发layout impact |
| RST-G14 | inline token/theme value被解析并保存Inline origin |
| RST-G15 | inline值即使等于lower layer也不会被pseudo rule覆盖 |
| RST-G16 | inherit/initial/unset/default在nested component和theme切换下确定性 |
| RST-G17 | token DAG检测missing/cycle/depth，fallback有显式receipt |
| RST-G18 | theme公开palette/typography/shape/spacing/control/elevation全部可typed寻址 |
| RST-G19 | theme数值、颜色、字体、elevation通过finite/range/resource校验 |
| RST-G20 | theme/content fingerprint跨进程/构建稳定，不使用`DefaultHasher` |
| RST-G21 | gameplay project显式选择theme asset并绑定surface generation |
| RST-G22 | Editor preview与gameplay消费相同compiled UI/theme artifact |
| RST-G23 | WOC全部非空style import在gameplay生效并有pixel/metadata oracle |
| RST-G24 | component-local stylesheet/token在实例化后仍按scope生效 |
| RST-G25 | theme reload改变已存在surface的computed metadata与预期像素 |
| RST-G26 | reload失败保留last-good，receipt列出per-surface terminal outcome |
| RST-G27 | template/style/theme/resource dependency变化只rebuild实际受影响surface/node |
| RST-G28 | class/id/state/property mutation通过统一transaction更新selector index |
| RST-G29 | `:enabled`、`:disabled`、focus/drop/popup aliases经过canonical state corpus |
| RST-G30 | ancestor pseudo-state只访问编译依赖集且dirty amplification受限 |
| RST-G31 | static cascade不再对每node全扫全部rules |
| RST-G32 | computed style使用sharing/arena，规模测试记录allocation与RSS |
| RST-G33 | dirty impact由schema生成，layout/text/render/cache测试覆盖新增属性 |
| RST-G34 | painter只读取typed computed style，不再解析任意字符串map |
| RST-G35 | Collapse/Fade/Grow/Slide/Zoom只有behavior provider存在时才Available |
| RST-G36 | transition progress由clock推进，支持interrupt/reverse/cancel与time scale |
| RST-G37 | mount/unmount和end listener在唯一terminal commit执行 |
| RST-G38 | reduced-motion可禁用/缩短transition且不破坏mount/state语义 |
| RST-G39 | rapid state churn、surface close和theme reload不会留下transition callback/lease |
| RST-G40 | MUI adapter unknown component/variant/color/size typed reject，不合成任意class |
| RST-G41 | V1/V2/legacy style authority完成hard cutover且无compat facade/双写 |
| RST-G42 | behavior测试覆盖真实gameplay import、Editor parity、reload和negative corpus |
| RST-G43 | GPU/窗口pixel golden覆盖theme、pseudo、scope、transition、clip/opacity组合 |
| RST-G44 | 同硬件scale benchmark报告node/rule/depth/churn、p50/p95/p99、allocation/RSS和正确性 |

## 10. Finding owner 与实施状态

| Finding组 | Owner module | 依赖 | 状态 |
|---|---|---|---|
| Component scope、selector、part/host | runtime_interface UI schema + runtime UI compiler | M1、M2；Runtime11A component tree boundary | pending |
| Import、component-local cascade与compiled bundle | runtime asset UI compiler/cache | M1、M2；Runtime64 dependency generation | pending |
| Theme schema、token DAG与gameplay owner | runtime UI theme/project session | M1、M4；Runtime43 product owner | pending |
| Reload与surface style generation binding | runtime UI reload/surface binding | M3、M4；Runtime11A reload lifecycle | pending |
| Transition behavior与lifetime | runtime UI component behavior/transition scheduler | M5；Editor01 product consumer | pending |
| Typed property/cascade/computed arena | typed style compiler/computed arena | M1-M3、M6 | pending |
| Runtime selector/invalidation/diagnostics/performance | runtime selector/invalidation/diagnostics | M3、M7；Tooling32 evidence | pending |
| MUI adapter、tests与migration cleanup | MUI adapter/tests/migration cleanup | M6、M7 | pending |

## 11. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Current-source schema/cascade/theme/transition纵切面 | review_complete | 2026-08-21 | 57 production/product文件、18,962行、655,639 bytes |
| 聚焦测试与产品资产语料 | review_complete | 2026-08-21 | 13测试文件；297非测试ZUI、1,017 selectors、149 pseudo、132非空style imports |
| 五参考工程事实复核 | review_complete | 2026-08-21 | 9文件、8,948行、326,125 bytes；明确Unity Graphics参考边界 |
| Production重构 | pending | - | 本篇不修改production/tests/assets，不运行Cargo或产品资格 |
