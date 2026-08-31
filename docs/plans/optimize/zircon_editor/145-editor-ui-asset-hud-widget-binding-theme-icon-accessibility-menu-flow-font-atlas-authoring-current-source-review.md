---
title: Editor UI Asset / HUD / Widget / Binding / Theme / Icon / Accessibility / Menu Flow / Font Atlas Authoring 当前源码复审
category: zircon_editor
report_id: Editor145
review_date: 2026-08-26
baseline_head: d4ca9a802ecd19976c653caa58614af0c2fb15f7
verification_head: d4ca9a802ecd19976c653caa58614af0c2fb15f7
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/97-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-product-integration-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/84-runtime-rich-text-markup-parser-token-style-span-inline-object-link-image-table-list-layout-selection-accessibility-security-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md
related_code:
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor
  - zircon_editor/assets/ui/editor/ui_asset_editor.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui
  - zircon_plugins/ui_asset_authoring
  - zircon_plugins/ui_document_importer
  - zircon_runtime_interface/src/ui/v2
  - zircon_runtime_interface/src/ui/template/asset
  - zircon_runtime/src/ui/binding
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging
  - dev/godot/editor/scene/gui/control_editor_plugin.cpp
  - dev/godot/editor/scene/gui/theme_editor_plugin.cpp
  - dev/Fyrox/editor/src/ui_scene
  - dev/bevy/crates/bevy_ui/src/accessibility.rs
  - dev/bevy/crates/bevy_input_focus
  - dev/bevy/examples/ui/text/font_atlas_debug.rs
  - dev/slint/tools/lsp
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor145 - UI Authoring 当前源码复审

## 1. 最终结论

当前 Zircon UI Asset Editor 已经不是表单原型。它拥有 typed session、V1/V2 parser、真实 `UiSurface` preview、component descriptor palette、native slot schema、slot-aware drop、binding CRUD、style/theme inspection、source outline、undo/redo/replay、watcher、autosave/recovery、dependency generation、后台 refresh、BLAKE3 digest 和耐久单文件写入基础。当前 working tree 还增加了 hash membership 优化、borrowed lookup、generation-aware refresh，以及 Runtime `UiModelSchemaRegistry` / `UiBindingConversionRegistry`。这些实现应保留并收敛到统一 owner。

但五个 P0 没有关闭。V2 视觉编辑仍经 legacy projection：`repeat`写为 `None`，node `slots`清空，`ThemeTokens`降级成 `Style`，legacy 已有的 `focus/navigation/picking/a11y/widget`被显式写成 `None`，最终用 `toml::to_string_pretty`整篇重排。因此它不能承诺无损编辑、future-schema preservation 或可访问性/导航 authoring。

保存与编辑事务也只完成局部。普通 Save 有 CAS、staging、flush、sync、atomic replace 和 parent sync，但在 reimport/hydration receipt 之前已经推进 clean baseline，并丢弃 `import_asset`结果。`keep local and save`仍直接调用普通 Save。Promote、Undo、Redo 的单文件 effect 可耐久写入，却会先移动 session/undo 状态，再逐个执行跨文件 effect；中途失败仍可能留下部分提交。

产品入口仍是两套 authority。真实 UI Asset editor 有 823 行、94 nodes，但 HUD 与六份 extension workspace 共 1,665 行、196 nodes、140 routes、0 provider，继续固定显示 `Gameplay_HUD`、`WBP_Inventory`、`Health.Value`、`icon-warning`、`Screen_Start`、`Inter UI`及静态计数。Designer 只有 Select、ResizeSlot、PreviewInteract；PreviewInteract 只生成 route/action/payload metadata DTO，没有经过真实 hit-test/focus/state/action 链。

本轮重判 Editor23/97 的 **5 项 P0 为 2 Open/3 Partial，60 项 P1 为 41 Open/19 Partial，12 项 P2 为 11 Open/1 Partial；32 项资格门为 29 Fail/3 Partial**。Editor145 只刷新 currentness，不重复增加 canonical finding 总数。没有动态 correctness、save/restart、平台 accessibility、font atlas、scale、fault、soak 或同内容跨引擎 benchmark，不能声称该域达到 Unreal 级，更不能声称性能或表现优于 Unreal。

## 2. 审查边界与 currentness

### 2.1 当前物理选择集

以下统计来自本轮读取到的 working tree 物理文件。各范围用于说明 owner 边界，存在有意重叠，不得直接相加为 union。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮用途 |
|---|---:|---|
| UI Asset editor core | **101 / 27,684 / 25,760 / 970,738 / 67 / 13** | session、document、designer、binding、theme、preview、replay |
| Host session 与 retained product integration | **70 / 7,414 / 6,869 / 281,002 / 37 / 2** | open/save/import/refresh/watcher/editing 与真实 surface host |
| Authoring 与 document importer plugins | **16 / 1,306 / 1,185 / 48,136 / 16 / 1** | capability、descriptor、create 入口与 `.zui` importer |
| UI Asset/HUD/六份 extension product surfaces | **10 / 2,681 / 2,341 / 149,600 / 0 / 0** | 真实 UI Asset surface 与固定 Workbench surface |
| Selected Workbench callback boundary | **7 / 4,502 / 4,443 / 201,921 / 0 / 0** | route feedback、field mutation、navigation、module command |
| Selected Runtime/interface UI boundary | **235 / 37,458 / 34,378 / 1,283,219 / 143 / 12** | V2/template/binding/accessibility/font 合同，不重复审查算法 owner |
| Selected reference union | **32 / 31,068 / 26,704 / 1,123,180 / 101 / 0** | Unreal/Unity/Godot/Fyrox/Bevy/Slint 对照 |

规模本身已经暴露维护边界：`style_state.rs` 982 行、palette drop resolution 956 行、`undo_stack.rs` 932 行、session lifecycle 851 行、binding payload editing 843 行、theme state 833 行、source sync 802 行。核心编辑路径仍频繁克隆整份 `last_valid_document`，preview mock 也克隆整份 document；13 个 core ignored test 主要是 release performance evidence，不能替代产品资格。

### 2.2 冻结点与限制

- baseline HEAD 为 `d4ca9a802ecd19976c653caa58614af0c2fb15f7`；本轮以 dirty working tree 的物理内容为准，最终 HEAD 如移动，以 `verification_head`和本报告列出的源码断点为准。
- UI authoring 范围含大量用户或其他 Session 在途修改。本轮只读取、归纳并写 review，不回退、不覆盖，也不把未集成代码自动视为已通过资格。
- 参考 revision：Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Slint `a47a293e5289c4c795a44cca089ba13b841e3c2b`；Unreal 没有独立 nested Git 边界，以所选文件内容为准。
- 按用户要求未查询、轮询、等待或实时跟踪协调器。
- 本轮仅静态 review，没有运行 Cargo、Editor、asset cook、save/reopen、PIE、screen reader、IME、GPU/font atlas、fault、scale、soak、profile 或竞争 benchmark。

### 2.3 Owner 边界

- Editor145 负责 lossless authoring document、Designer command/transaction、preview control plane、UI asset 创建/保存/迁移，以及 HUD/Binding/Theme/Icon/A11y/Menu/Font 的真实产品投影。
- Runtime73-78 负责 style/theme、template/binding、widget、layout、input/focus/navigation 与 accessibility 执行；Editor 只能消费 versioned schema、snapshot 和 receipt。
- Runtime79-82 与 84 负责 UI GPU submit、font/glyph、shaping/BiDi、text editing/IME 与 rich text。Editor 不得建立私有 font atlas、focus graph 或 a11y tree。
- Runtime85 与 Asset owners 负责 cook、artifact、dependency 和 runtime package；Editor 负责请求、diagnostic 与 generation-safe publication workflow。
- Editor02/08/09/11 分别拥有通用 document transaction、command、job 和 diagnostic 基础；本域不得复制弱化版本。

## 3. 当前源码事实与断点

| 子链 | 当前真实基础 | 仍然断开的工程合同 |
|---|---|---|
| V2 document | V1/V2 parser、typed session、source revision、validation 均真实存在 | `v2_projection.rs` 写 `repeat: None`、`slots: BTreeMap::new()`、五类 legacy node semantic 为 `None`，ThemeTokens 合并到 Style，整篇 pretty-print |
| Save | CAS、staging、flush、sync、replace、parent sync 和 local copy 基础 | `mark_canonical_source_persisted`早于 reimport/hydration receipt；`let _ = import_asset(...)`丢弃失败；keep-local 仍调用普通 Save |
| Promote/Undo/Redo | 单文件 `atomic_write_new`、外部 effect 描述与 replay 基础 | session/stack 先变更，再逐 effect 写文件/import；无跨文件 prepare/commit/rollback/restart receipt |
| Import/refresh | watcher background job、cancellation、generation reject、retry 与 dependency commit | initial open/save/undo hydration 仍同步；`collect_ui_asset_import_document`递归遍历 widget/style import，无 depth/node/edge/byte/time budget |
| Importer | `.zui`可解析为 `ImportedAsset::UiV2View/Style/Component` | artifact 只包 parsed document，不是 dependency-closed、platform-qualified、immutable cooked artifact |
| Designer | typed selection、palette/drop、wrap/extract/promote、真实 preview surface | tool mode 仅 Select/ResizeSlot/PreviewInteract；无 node delete/clipboard/multi-select/canvas tools；PreviewInteract 只记录 metadata |
| Binding | CRUD、payload projection、nested mock resolution；Runtime 新增 model/conversion registry | Editor 只消费名称语法，未消费 registry；`SelectedNode`仍硬编码；route/action 仍是字符串，无 endpoint generation/refactor owner |
| Theme | selector/specificity、compare、promotion、merge 与 pseudo state 基础 | ThemeTokens 弱类型且可被降级；无 design-system variant、project usage index、跨资产 atomic rename |
| A11y/Icon/Menu/Font | Runtime 分别已有 semantic snapshot、catalog/render、focus/input、font/glyph 基础 | 真实 UI Asset core/host/product 对这些 snapshot/registry 没有消费；六份 Workbench 只展示固定样例 |
| Product surfaces | 真实 UI Asset surface 823 行/94 nodes，action bar 有真实 command routes | HUD 与六份 extension 形成第二 authority；固定资产名、计数和成功文案不来自 provider |
| Authoring plugin | workspace member、descriptor、contribution、experimental maturity 均存在 | 四个 `plugins://ui_asset_authoring/...zui` 资源实际不存在；三个 Create 只发 `OpenView`，无 operation factory；默认 App/catalog 不装配 |

当前新增的 `UiModelSchemaRegistry`与 `UiBindingConversionRegistry`属于正确 Runtime 底座：它们已有 schema/provider key、revision、field resolution、context validation、provider generation、stale handle 与 typed signature。但 Editor 搜索只命中 `UiBindingSchemaNameKind`和硬编码 payload 模板，没有 registry consumer，因此只能把 P1-35 维持 Partial，不能关闭 P1-34/36/37。

Workbench 七份 UI surface 共 1,665 行、196 nodes、140 routes、0 provider；其 callback 仍回报 42 widgets/3 issues、18 bindings/2 invalid、312 icons/4 missing/14 refs、9 accessibility issues、64 focus rules/2 issues、4096 glyphs/4 pages，并把 Save/Validate/Apply/Compile 表达成既成成功。生产 UI 文本必须来自 accepted domain receipt；否则只能显示明确 Unavailable。

## 4. 必须保留的工程基础

1. 保留 `UiAssetEditorSession`、source revision、replay artifact、dependency generation 与 typed presentation，不以 Workbench control state 替代 document。
2. 保留 atomic file primitive 与 BLAKE3 digest，将它们提升为 document repository + reimport receipt + crash recovery 协议。
3. 保留 watcher background job、cancel、generation reject、retry/backoff 和 bounded ingress，并扩展到 open/save/undo/import 全入口。
4. 保留 Runtime component descriptor、native slot schema 与 clone-validate drop，把 schema version、capability 和 owner generation 纳入 receipt。
5. 保留真实 `UiSurface` preview、binding/style/theme inspection 和 Runtime registry/snapshot consumer，删除固定产品数据。
6. 保留 `.zui` V2 importer，但把 source、dependency、compiler、cook、publication 和 runtime install generation 连为单链。
7. 保留当前哈希索引与 borrowed lookup 优化，但必须用统一规模基准验证端到端收益，不能用 ignored microbenchmark 代替产品资格。

## 5. P0：数据安全与产品真实性

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| UIED-P0-01 | Open | V2 经 legacy projection 丢 repeat/node slots/ThemeTokens/不可达节点/unknown/trivia；focus/navigation/picking/a11y/widget 不在 V2 node 合同且投影显式清空 | 建单一 lossless CST + typed semantic model；未知字段与未触及 span 稳定，不支持语义只能 read-only，禁止降级保存 |
| UIED-P0-02 | Partial | Save 已原子 CAS，但 clean baseline 早于 import/hydration receipt，reimport 结果被丢弃 | 统一 write、replace、reimport、hydrate、baseline 与 journal；任一阶段失败保持 dirty 并可重试/恢复 |
| UIED-P0-03 | Partial | 普通 Save 可拒外改，但 keep-local 直接调用同一 Save，无 base/ours/theirs merge 或显式 force authority | 建共享 document owner/revision broadcast、three-way merge、authorized force 与 source-control-aware journal |
| UIED-P0-04 | Partial | promote 有发布前局部回滚，external-effect undo/redo 仍可能在 stack 已移动后部分提交 | 接 Editor02 Cross-Asset Transaction，声明 read/write set、stage、commit、rollback、receipt 与 restart recovery |
| UIED-P0-05 | Open | authoring plugin 四个资源 URI 缺失，Create 只 OpenView、无 factory，默认 host/catalog 未装配 | 确定 builtin/plugin 唯一 owner，补真实 resource、operation factory、versioned asset、import/open/save/reopen 与 qualification |

## 6. P1：工程化完整性

### 6.1 Document、持久化、导入与身份

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| UIED-P1-01 | Open | 无 lossless source document；以 CST/token owner 保存 comments、unknown fields、顺序、span 与 semantic identity |
| UIED-P1-02 | Open | visual mutation 整篇 pretty-print；改为 source-range edit，Format 必须是独立显式 command |
| UIED-P1-03 | Open | V1/V2 无 versioned migration/dry-run/backup/idempotence；future version 只能 read-only |
| UIED-P1-04 | Partial | save/local-copy 有 atomic primitive，但 save/autosave/recovery 未由同一 repository contract 驱动 |
| UIED-P1-05 | Partial | watcher refresh 异步，initial open/save/undo hydration 同步；全部入口统一走 bounded job 与 generation commit |
| UIED-P1-06 | Open | import traversal 递归且无 depth/node/edge/byte/time budget；改显式栈、cycle path 与 typed budget outcome |
| UIED-P1-07 | Open | source update 触发全量 validate/hydrate/presentation；建立 dirty-range、dependency impact 与 incremental compile |
| UIED-P1-08 | Open | undo/replay 保留整份 source/document 副本；改结构共享、delta、checkpoint 与 bounded retention |
| UIED-P1-09 | Partial | 已用 BLAKE3 摘要，但持久 identity 仍缺 file id/revision/self-write token/source-control identity |
| UIED-P1-10 | Partial | watcher path identity、disk baseline 与 asset identity 局部存在；收敛 canonical physical/logical identity 与 generation |

### 6.2 Designer、Hierarchy、Palette 与 Preview

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| UIED-P1-11 | Open | 没有 node delete command；补 selection policy、child/resource 引用检查与 single undo group |
| UIED-P1-12 | Open | 无 duplicate/cut/copy/paste；实现 typed clipboard、stable remap、slot validation 与跨 document import |
| UIED-P1-13 | Open | model 可表达多个 selected id，但 interaction 不建立 multi-select；补 range/toggle/marquee 与 primary selection |
| UIED-P1-14 | Open | 无 zoom/pan/fit/ruler/guide/grid/snap；建立独立 viewport state 与 project/user policy |
| UIED-P1-15 | Open | 只有 ResizeSlot，无 anchor/pivot/rotate/container/slot 完整 handle 与 applicability diagnostic |
| UIED-P1-16 | Open | 无 align/distribute/match-size；所有批量操作需 preview、constraint check 和单事务 |
| UIED-P1-17 | Open | hierarchy/palette search 无可审计 query authority、index generation、virtualized result 与匹配解释 |
| UIED-P1-18 | Partial | drop 使用真实 catalog/slot schema 和 clone validation；仍缺 schema version/owner generation/accepted revision receipt |
| UIED-P1-19 | Partial | 可打开 component/reference，但无 breadcrumb/back-forward/cycle/cross-asset viewport/selection state |
| UIED-P1-20 | Open | designer tools 不可插件化；建立 tool descriptor、capability、input capture、overlay、transaction 与 lifecycle |
| UIED-P1-21 | Open | preview preset 硬编码 1280x720、1100x780、1920x1080、640x480；改用项目 device profile authority |
| UIED-P1-22 | Open | 无 breakpoint、多设备矩阵、safe zone/cutout/orientation/user scaling 对照 |
| UIED-P1-23 | Open | locale selector 不加载真实 localization generation；接 Editor33/Runtime text owner |
| UIED-P1-24 | Open | 无 RTL/vertical/long text/pseudo/glyph coverage 矩阵与 source diagnostic |
| UIED-P1-25 | Open | PreviewInteract 只生成 metadata；输入必须经过真实 hit-test/focus/state/action 并输出 trace |
| UIED-P1-26 | Open | 无 deterministic clock/animation/async/seed；capture 不可复现，旧结果可污染新 preview |
| UIED-P1-27 | Partial | mock expression/value resolution 真实存在；仍缺 typed scenario source、schema version 与 secret boundary |
| UIED-P1-28 | Partial | preview compile/runtime report 有 generation 字段；未形成 source/import/compiler/runtime/frame 统一 receipt |
| UIED-P1-29 | Open | 无 pointer capture、focus path、navigation/device/IME 状态可视化 |
| UIED-P1-30 | Open | 无同内容 golden geometry/visual/input evidence 与稳定阈值 |

### 6.3 Inspector、Binding、Theme、Menu 与运行时产品

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| UIED-P1-31 | Partial | inspector 有 typed 字段语义基础，但大量值仍按 literal/TOML 编辑；全面接 prop/slot schema 与 resource picker |
| UIED-P1-32 | Partial | 有 default/inherited projection 片段；缺统一 overridden/reset/applicability/validation source |
| UIED-P1-33 | Open | 无工程级 resource/color/font/icon/localization picker、引用 preview 与 dependency change transaction |
| UIED-P1-34 | Open | binding suggestion 仍含 `SelectedNode`等硬编码关键词；只允许 schema/registry 驱动 |
| UIED-P1-35 | Partial | Runtime registry 与当前 target/value projection 存在，但 Editor 未消费 versioned endpoint schema/type/default/enum/capability |
| UIED-P1-36 | Open | binding/action identity 仍是字符串，rename 不跨资产；建立 stable ID、usage index 与 refactor transaction |
| UIED-P1-37 | Open | authoring 不验证 Runtime service/context/capability generation；compile 与 preview 都需 fail-close |
| UIED-P1-38 | Open | Menu Flow workspace 是固定 `Screen_Start`；建立 typed screen graph、entry/back/modal/transition asset |
| UIED-P1-39 | Open | navigation authoring 未消费 Runtime77 focus graph/snapshot；source graph 与 runtime trace 必须可对照 |
| UIED-P1-40 | Open | action/binding/menu edit 无统一 journal 与 refactor owner；接 Editor08/02 |
| UIED-P1-41 | Open | ThemeTokens 仍是弱类型 TOML 且会降级；建立 typed token kind、alias、cycle 与 variant schema |
| UIED-P1-42 | Partial | theme compare/pseudo state 基础存在；缺 design-system variant、density、contrast、platform matrix |
| UIED-P1-43 | Partial | cascade inspection 有 selector/specificity 基础；缺完整 source span、origin、layer、why-won 与 currentness |
| UIED-P1-44 | Partial | local/imported theme promotion/refactor 存在；缺 project usage index 与跨资产 atomic rename |
| UIED-P1-45 | Open | Accessibility Audit 固定 `Gameplay_HUD`/9 issues；改投影 Runtime78 generation-qualified semantic snapshot |
| UIED-P1-46 | Open | V2 node 无 a11y/focus/navigation 字段，不能提供 name/role/state/order/fix command |
| UIED-P1-47 | Open | 无 keyboard-only、high contrast、screen reader 受控链及平台 artifact |
| UIED-P1-48 | Open | Icon Library 固定 312/4/14；建立 icon asset/cook catalog、usage index、missing diagnostic 与 theme/DPI preview |
| UIED-P1-49 | Open | icon atlas/render completeness 无 Editor consumer；只消费 Runtime79/asset artifact，禁止私有 atlas |
| UIED-P1-50 | Open | Font Atlas 固定 Inter UI/4096/4/12；接 Runtime80/79 实际 page、glyph、UV、residency 与 missing snapshot |
| UIED-P1-51 | Partial | importer 可生成 typed V2 asset，但不是依赖闭合、platform-qualified、immutable cooked artifact |
| UIED-P1-52 | Partial | Runtime compiler 有 structured report 基础，Editor 未形成可消费的 build receipt/diagnostic/artifact link |
| UIED-P1-53 | Partial | source/dependency/import generation 分别存在；必须合并为 source-to-runtime 单一 generation chain |
| UIED-P1-54 | Open | 无 node/import/rule/binding/profile/glyph 规模 budget 与 machine-readable regression threshold |
| UIED-P1-55 | Open | 无 100k hierarchy/palette/inspector/source/diagnostic 统一 virtualization 证据 |
| UIED-P1-56 | Partial | plugin contribution 有 descriptor/capability 基础；缺 schema version、resource existence、owner lease 与 reload qualification |
| UIED-P1-57 | Partial | Runtime style/binding/a11y/font owners 已存在，Editor 也有局部 consumer；产品面仍未连接，必须禁止重复 authority |
| UIED-P1-58 | Open | 真实 UI Asset Editor 与 Workbench UI Asset 页是两个产品入口；后者必须嵌入同一 session/provider 或降为 fixture |
| UIED-P1-59 | Open | 七份 Workbench 固定 asset name、count、warning、DPI/locale；production surface 只能投影 provider 或 Unavailable |
| UIED-P1-60 | Open | action/field handler 只改 control string 却报告 Saved/Validated/Applied；只有 accepted domain receipt 可回写成功 |

## 7. P2：完整性、诊断与维护性

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| UIED-P2-01 | Open | 多个 700-982 行文件继续混合 state/schema/command/projection/cache；按 repository/schema/command/service/projection 拆 owner |
| UIED-P2-02 | Open | action/control/endpoint id 散落裸字符串；生成 typed IDs 并启动时检查重复、悬空 route/resource |
| UIED-P2-03 | Open | profile/locale/budget 常量无 project policy 与 effective source |
| UIED-P2-04 | Open | diagnostic code/severity/source mapping 未统一到 Editor11 journal schema |
| UIED-P2-05 | Open | 缺可关闭 finding 的视觉/输入/平台 artifact；ignored release microbenchmark 不能替代 qualification |
| UIED-P2-06 | Open | 当前 tests 主要证明结构、小 fixture 和局部性能；缺 roundtrip、fault、multi-process、真实输入/平台链 |
| UIED-P2-07 | Open | bool/Option 表达 unsupported，丢 owner/generation/reason/recovery action |
| UIED-P2-08 | Open | outline/projection/preview/import cache 无统一 entries/bytes/hit/miss/evict/peak accounting |
| UIED-P2-09 | Partial | plugin maturity 已收敛为 experimental；仍不是 qualification gate 自动生成 |
| UIED-P2-10 | Open | authoring telemetry 缺 privacy/content boundary，不能采集 source、payload、localized text、secret 与用户 path |
| UIED-P2-11 | Open | scope fingerprint、route/resource inventory 与 P0 assertion 未进入自动 staleness check |
| UIED-P2-12 | Open | Layout/Widget/Style/ThemeTokens/HUD/Surface 术语与 owner 混用；建立 schema glossary |

## 8. 参考引擎差异与采用路由

| 参考 | 本轮源码证据 | Zircon 应采用的合同 |
|---|---|---|
| Unreal UMG | UMG Editor 有 Delete/Copy/Cut/Paste/Duplicate commands；compiler 生成 class/tree 并验证 binding；factory 真实创建；Designer 使用 transaction、preview、navigation 与 safe-zone 工具 | 产品完整性、compiler/factory、transaction、navigation authoring 主参考 |
| Godot | `control_editor_plugin.cpp` 以 `EditorUndoRedoManager`分组 anchors/offset/grow/size 变更；`theme_editor_plugin.cpp`有 typed import tree、filter、partial/full import 与 undo snapshot merge | Canvas tools、theme import 与可逆交互主参考 |
| Fyrox | UI Scene 有 deep-clone clipboard、Delete/Copy/Paste As Child、selection command；move interaction 在 mouse-up 生成 `CommandGroup` | Rust command/interaction/clipboard 结构参考 |
| Bevy | Accessibility 从真实 UI component 同步 semantic node；directional navigation 是显式 typed graph；tab group/modal 有 typed error；font atlas debug 读取实际 atlas pages | Runtime truth、navigation graph 和真实 diagnostic projection 参考，不作为完整 Editor 产品参考 |
| Unity Graphics | `SerializedObject` Update/Apply、Undo callback、created-object undo 与 redo-safe destruction；DebugUI 分 Panel/Widget/container/query path | 本地 Graphics 不含完整 UI Builder/TextCore，只采用 serialized transaction/provider pattern，不推测缺失源码 |
| Slint | LSP `DocumentCache`保存 source version/CST/dependency；property edit 生成 versioned WorkspaceEdit 并拒绝版本错配；catalog 来自 compiler type registry | source-preserving、versioned edit、schema property 与增量 authoring 参考 |

这些参考不是照搬对象。Zircon 需要先统一自己的 ownership、version、transaction、artifact 和 product truth 合同，再选择实现结构。任何“比 Unreal 更快/更好”的结论都必须使用相同内容、功能、质量、硬件、配置和冻结 revision 的动态证据。

## 9. 目标架构

```text
LosslessUiAuthoringDocument(source_version, CST, semantic_ids, unknown_fields)
  -> UiAuthoringCommand(read_set, write_set, capability_generation)
  -> EditorTransaction(prepare, validate, commit, rollback, journal)
  -> SourceRevisionCAS + ReimportReceipt
  -> UiCompileCookReceipt(source, dependencies, target, schema_versions)
  -> ImmutableUiArtifactGeneration
  -> RealUiSurfacePreview(input/focus/state/action trace)
  -> RuntimeSnapshot(a11y, navigation, icon, glyph, diagnostics)
  -> Provider-backedEditorProjection | ExplicitUnavailable
```

Document、transaction、artifact、runtime surface 和 Editor projection 必须各有单一 authority。所有 UI asset、binding、theme、icon、a11y、menu 与 font 产品文本都从 typed state/receipt 派生；control 字符串不得保存业务真值。

## 10. 依赖顺序与重构里程碑

| 阶段 | Owner | 必须交付 | 关闭范围 |
|---|---|---|---|
| R0 Corpus/truth hard cut | Editor145 + Runtime schemas | V2 golden corpus；unknown/trivia/repeat/slots/ThemeTokens/focus/a11y roundtrip；固定 Workbench 成功文本改 Unavailable | P0-01 前置、P1-58~60 |
| R1 Lossless document | Editor145 + Runtime interface | CST + typed semantic model、range edit、explicit format、migration/future read-only | P0-01、P1-01~03 |
| R2 Repository/save | Editor02 + Editor145 | unified repository、CAS save、reimport/hydration receipt、crash recovery、three-way merge | P0-02/03、P1-04/09/10 |
| R3 Cross-asset transaction | Editor02/08 | promote/refactor/external-effect undo/redo 的 read/write set、stage/commit/rollback/restart | P0-04、P1-36/40/44 |
| R4 Factory/cook | Plugin + Runtime85 | 真实 resource/factory/catalog，Create->Import->Open->Save->Reopen；closed cooked artifact | P0-05、P1-51~53/56 |
| R5 Designer/schema | Editor145 + Runtime73-77 | 完整 command/canvas/clipboard、schema inspector/palette、typed binding/menu/theme | P1-11~20、31~44 |
| R6 Preview/products | Editor145 + Runtime77-82 | device/locale/RTL/deterministic preview、真实 input trace、A11y/Icon/Font snapshot consumer | P1-21~30、45~50、57~60 |
| R7 Qualification | Runtime + Editor + App | 100k、fault、restart、platform、package、visual/input golden、same-content benchmark | P1-54/55、全部 P2/Gate |

MVP `00` 与 F0-F5 未通过前，不应继续扩张高级 UI authoring surface。可以先实施 R0 的 fail-close、golden corpus 与 schema 设计，但不得把静态 Workbench、ignored microbenchmark 或 capability descriptor 重新标成已交付产品。

## 11. G01-G32 资格门

| Gate | 状态 | 当前证据与通过条件 |
|---|---|---|
| G01 V2 repeat | Fail | projection 明确写 `repeat: None`；所有 mutation/save/undo 后语义与 source span 必须稳定 |
| G02 V2 slots | Fail | node slots 被清空；named slot/component mount 必须 typed roundtrip |
| G03 ThemeTokens | Fail | 当前降级为 Style；kind/token/unknown/import 必须保留 |
| G04 Unknown/trivia | Fail | pretty-print 整篇重写；未触及区域必须字节稳定 |
| G05 Save failure | Partial | atomic primitive 存在；write/flush/replace/reimport 任一失败仍须 dirty 且磁盘旧版本完整 |
| G06 Crash save | Fail | 无进程终止/restart recovery 证据 |
| G07 Multi-instance | Partial | CAS 基础存在；两个实例同 base 编辑仍需 merge/conflict 产品证据 |
| G08 External edit | Partial | 普通 Save 拒外改基础存在；keep-local 合同矛盾且无 merge/authorized force |
| G09 Cross-asset promote | Fail | import/refresh 失败仍可留下部分提交 |
| G10 Undo/redo | Fail | external effects 无 transaction receipt/restart recovery |
| G11 Factory | Fail | create 无 operation factory 且资源缺失 |
| G12 Plugin resources | Fail | 四个 descriptor URI 实际不存在 |
| G13 Migration | Fail | 无 V1->V2 dry-run/backup/idempotence/future-version read-only 链 |
| G14 Designer basics | Fail | delete/clipboard/multi-select/align/zoom 缺失 |
| G15 Layout tools | Fail | anchor/pivot/container handle 与 golden geometry 缺失 |
| G16 Hierarchy scale | Fail | 无 100k virtualization/budget artifact |
| G17 Device matrix | Fail | preset 固定，无 safe-zone/orientation/user scaling matrix |
| G18 Localization | Fail | locale/RTL/pseudo/glyph coverage 未接真实 Runtime generation |
| G19 Preview input | Fail | PreviewInteract 未进入真实 hit-test/focus/state/action |
| G20 Preview determinism | Fail | 无 clock/seed/async generation qualification |
| G21 Inspector schema | Fail | builtin/plugin field 未完整 schema/reset/unknown preservation |
| G22 Binding schema | Fail | Editor 未接 versioned registry authority，硬编码关键词仍存在 |
| G23 Menu flow | Fail | 固定 Screen_Start，无真实 graph/runtime trace |
| G24 Theme | Fail | typed token/variant/cross-asset transaction 未闭环 |
| G25 Accessibility | Fail | V2 缺 a11y/focus 字段，Audit 未接 Runtime78 |
| G26 Icon | Fail | 无真实 catalog/cook/atlas/usage 闭环 |
| G27 Font | Fail | Font Atlas 静态，未验证 package fallback 与实际 page/UV/residency |
| G28 Cook | Fail | importer 不是依赖闭合 runtime artifact，package source independence 未证 |
| G29 Performance | Fail | ignored microbenchmark 无法替代 1k/10k/100k 同内容阈值趋势 |
| G30 Failure outcomes | Fail | provider 缺失时仍显示固定成功，状态语义未统一 |
| G31 Workbench convergence | Fail | 七份 surface 仍是固定业务第二 authority |
| G32 Current-source evidence | Fail | 无自动 stale gate 与 dynamic qualification artifact |

## 12. 禁止的临时修补

- 不得继续给 legacy projection 零散补字段后宣称 V2 无损；必须建立单一 document owner 与 unknown-field contract。
- 不得把单文件 atomic write 当完整 Save；reimport、hydration、baseline、journal 与 crash recovery 必须是同一协议。
- 不得在 Undo/Redo 已移动 stack 后逐个写外部文件；跨资产 effect 必须 prepare 后原子发布并可重启恢复。
- 不得为 A11y/Icon/Menu/Font 再造 Editor 私有 runtime model；只能消费 versioned Runtime snapshot/artifact。
- 不得继续用固定 asset name、count、warning 或 queued/saved/applied 文本填充 production Workbench。
- 不得只补缺失 ZUI 文件而保留无 factory 的 Create；资源、operation、import、open/save/reopen 必须一起 qualification。
- 不得用 release ignored microbenchmark、低规模 unit test 或 capability descriptor 证明工程级性能和产品完整性。

## 13. 验证边界与裁决

| Canonical 范围 | 当前状态 | 本轮裁决 |
|---|---:|---|
| 5 项 P0 | **2 Open / 3 Partial** | lossless document 与产品 factory/resource 未建立；save/conflict/cross-asset transaction 只有局部基础 |
| 60 项 P1 | **41 Open / 19 Partial** | Runtime registry、watcher、importer 与局部 schema 是真实底座，但产品/receipt/generation 链未闭合 |
| 12 项 P2 | **11 Open / 1 Partial** | experimental maturity 已诚实化，其余维护、diagnostic、telemetry、evidence 仍缺 |
| 32 项 Gate | **29 Fail / 3 Partial** | 只有 save failure、multi-instance、external edit 具备局部证据 |

当前应把该域定义为“真实 authoring 底层已形成，但 source fidelity、事务、factory/cook 和 Runtime-backed 产品面仍未闭环”。第一实施优先级是 R0/R1：冻结 lossless corpus，禁止 lossy save，并把固定 Workbench authority 改为 provider-backed 或明确 Unavailable。随后按 repository/transaction -> factory/cook -> designer/schema -> runtime-backed products -> qualification 的顺序推进。

只有 32 个 Gate 全部通过，并完成真实平台、fault/restart、规模、soak、visual/input golden 与同质量跨引擎 benchmark，才可声称该子系统达到工程级；在此之前不得声称性能或表现优于 Unreal。
