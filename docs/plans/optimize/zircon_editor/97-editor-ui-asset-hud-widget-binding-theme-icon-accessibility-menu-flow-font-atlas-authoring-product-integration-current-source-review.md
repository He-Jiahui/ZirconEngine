---
title: Editor UI Asset / HUD / Widget / Binding / Theme / Icon / Accessibility / Menu Flow / Font Atlas Authoring 与 Product Integration 当前源码复审
category: zircon_editor
report_id: Editor97
review_date: 2026-08-26
baseline_head: a8eca85cc83008aeb200dce2d2b01e2ae3c157c9
verification_head: a8eca85cc83008aeb200dce2d2b01e2ae3c157c9
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
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

# Editor97 · UI Authoring 与 Product Integration 当前源码复审

## 1. 结论

当前Zircon UI Asset Editor不是空壳。它已经有typed session、V1/V2解析、真实`UiSurface`预览、component descriptor palette、native slot schema、slot-aware drop、binding CRUD、style/theme inspection、source outline、undo/redo/replay、watcher、autosave/recovery、dependency generation、后台refresh、BLAKE3磁盘摘要与原子文件基础。这些都应保留，不能在重构时退回成一组表单和字符串回调。

但核心文档合同仍是P0。V2视觉编辑继续经`v2_projection.rs`投影到legacy model再写回：`repeat`被写成`None`、node-level `slots`被清空、`ThemeTokens`降级成`Style`，只重建root/component root可达节点，并由`toml::to_string_pretty`整篇重排。更严重的是legacy `UiNodeDefinition`已有`focus/navigation/picking/a11y/widget`，V2 node schema没有这些字段，投影还明确写成`None`。因此当前Designer无法承诺无损编辑、可访问性/导航可创作或future-schema保留。

保存链已有实质进展：普通Save现在先做compare-and-swap式磁盘检查，再通过staging、flush、sync、atomic replace与parent sync提交，成功后才推进disk baseline和clean状态；外部修改测试也要求普通Save拒绝覆盖。但是reimport结果仍被丢弃，clean已在import/hydration确认前推进；`keep local and save`与CAS合同及现有测试存在矛盾。Promote与外部effect Undo/Redo虽增加局部回滚和原子单文件写入，仍没有跨文件、registry、import、session、journal的一体化transaction。

产品表面仍有第二authority。HUD加六份extension ZUI共1,665行、196个node、140条route，固定显示`Gameplay_HUD`、`WBP_Inventory`、`Health.Value`、`icon-warning`、`Screen_Start`、`Inter UI`及固定计数；callback继续回预制成功文本或只改retained control字符串。真实UI Asset editor自身有823行、94个node，却只有6条inline route；Designer只有Select、ResizeSlot、PreviewInteract三种模式，PreviewInteract只生成metadata DTO，没有把输入送入真实surface hit-test/focus/state/action链。

插件链也未闭合。`ui_asset_authoring`已进入workspace并正确标成experimental，但四个descriptor URI指向不存在的ZUI，Create Layout/Widget/Style只发`OpenView`，没有operation factory，默认minimal host还显式禁用该插件。`ui_document_importer`能导入`.zui`并生成V2 View/Style/Component，这是应保留的真实基础；它尚未形成包含component/style/font/icon依赖与generation provenance的closed cooked artifact。

本轮重判Editor23的 **5项P0为2 Open/3 Partial，60项P1为41 Open/19 Partial，12项P2为11 Open/1 Partial；32项资格门为29 Fail/3 Partial**。Editor97只刷新currentness，不重复增加canonical finding总数。没有动态、规模、故障或跨引擎同内容benchmark，因而不能声称功能、性能或表现优于Unreal。

## 2. 审查边界、统计与currentness

### 2.1 冻结范围

统计对象为当前working tree物理文件。行与非空行按文本物理行统计，bytes取文件长度；tests/ignored只计Rust test/ignore属性。fingerprint按repository-relative lowercase path排序，为每个文件拼接path、NUL、lowercase文件SHA-256与LF后再取SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| UI Asset editor core | **95 / 25,748 / 24,025 / 901,839 / 31 / 0** | a5f6d3617b1e891f9ad809871605eaee08b1f80bb2fd2492ff51bef0456d426f |
| Host session与retained app | **70 / 7,242 / 6,709 / 273,558 / 34 / 0** | 2dee428f7ddf8344616dbdcd3cca0fec372b3a35da1266fe8759ca6f51595ceb |
| Authoring与importer plugins | **16 / 1,308 / 1,187 / 48,159 / 16 / 0** | 80d787f25d0eba4760e1f26b59fb41573f7cd30be3500209caf4a9aa19ae12d3 |
| Product surfaces与Workbench callbacks | **19 / 5,505 / 5,069 / 277,762 / 2 / 0** | cbbfd36dc7eb360328f93e9f964faf3f794268b12e11d0677982340d8716ef63 |
| Focused Editor tests | **102 / 33,919 / 31,198 / 1,191,429 / 448 / 0** | cb54e7272039c9da1c68f6a200569a7e58c97a26de8084c3cea31e4b5e599df5 |
| Runtime/interface boundary selected | **145 / 21,065 / 19,229 / 702,177 / 166 / 0** | edf1003b157cd3e17ba61cf737a33eaf87aa32f57c75f27deb3b2d942bf3b622 |
| Zircon selected union | **447 / 94,787 / 87,417 / 3,394,924 / 697 / 0** | bf2be08280f381773a1dc9235c4a745646e49a8c8b221bba9627ce680b91c38f |
| Unreal selected | **9 / 13,278 / 11,242 / 469,264 / 0 / 0** | 1f8ef705b2de7f13336a170f74fbef3676c74306d6739186ff2c3a5d5d9af5d8 |
| Unity Graphics selected | **2 / 1,467 / 1,257 / 58,478 / 0 / 0** | 4548b37a12c104221f3025fc9b46c7da5cabadd98fb3408da3dc87929eaf81bd |
| Godot selected | **3 / 6,070 / 5,003 / 229,232 / 0 / 0** | 0e201912203ab83efedb734f84d18dc428f7470ff00d74d119e6d8bff0a49ea5 |
| Fyrox selected | **7 / 1,400 / 1,249 / 48,396 / 0 / 0** | 12994d6646022b3d517803140d647bf9333ee6d1f80b0f742803dad8d0713b2c |
| Bevy selected | **5 / 2,772 / 2,521 / 102,757 / 16 / 0** | deb957d999c8855cc9c949b7b5659e350cae7533a1924b0855f412370ee5f48d |
| Slint selected | **6 / 6,081 / 5,432 / 215,053 / 85 / 0** | 905dc5a48fb4d9e56187e845dbcec8a64518c4fe85b821f7c64796cbe4a3f8c0 |
| Reference selected union | **32 / 31,068 / 26,704 / 1,123,180 / 101 / 0** | 723b2a8c9091e71cb1e0c911e96d38de40689958e34b932b22252bc79ec46c10 |

### 2.2 currentness与限制

- baseline与初始verification HEAD均为`a8eca85cc83008aeb200dce2d2b01e2ae3c157c9`；最终校验若HEAD变化，以本表物理fingerprint而不是提交假定为准。
- UI authoring直接选择集有30个用户或其他Session在途文件。本轮读取并冻结当前物理状态，不回退、不覆盖，也不把在途实现当成已集成资格。
- 参考revision：Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Slint `a47a293e5289c4c795a44cca089ba13b841e3c2b`；Unreal没有独立nested Git边界，以所选文件fingerprint冻结。
- 按用户要求未查询、轮询或等待协调器；Tooling不在本轮范围。
- 本轮仅静态review，没有运行Cargo、Editor、asset import/cook、save/reopen、PIE、screen reader、IME、GPU/font atlas、fault、scale、soak、profile或竞争benchmark。

### 2.3 Owner边界

- Editor97唯一负责lossless authoring document、Designer command/transaction、预览控制面、UI asset创建/保存/迁移，以及HUD/Binding/Theme/Icon/A11y/Menu/Font产品投影。
- Runtime73-78唯一负责style/theme、template/binding、widget、layout、input/focus/navigation与accessibility执行；Editor只消费versioned schema、snapshot和receipt。
- Runtime79-82及84唯一负责UI GPU提交、font/glyph、shaping/BiDi、text editing/IME与rich text。Editor不得再建私有font atlas、focus graph或a11y tree。
- Runtime85及Asset owners唯一负责cook、artifact、dependency与runtime package；Editor负责请求、诊断和generation-safe publication workflow。
- Editor02/08/09/11分别拥有通用document transaction、command、job和diagnostic基础；本域不得复制简化版本。

## 3. 当前产品链事实

| 层 | 当前事实 | 判定 |
|---|---|---|
| Document | V1/V2解析与typed session真实存在，但V2 mutation走legacy projection | 有能力底座，无无损合同 |
| V2 schema | 有repeat、style、slot/events；缺legacy已有focus/navigation/picking/a11y/widget | schema owner未统一 |
| Save | CAS + staging/flush/sync/replace/parent sync，成功后mark clean | 单文件耐久Partial；reimport receipt缺失 |
| Conflict | 外改拒绝测试、reload/diff/local copy存在 | keep-local合同矛盾，无three-way merge |
| Import refresh | watcher走job/cancel/generation/retry/bounded ingress | initial open/save/undo hydration仍同步递归且无预算 |
| Palette/drop | Runtime component registry、native slot schema、候选target、clone-validate | 无版本化schema receipt与commit revision |
| Designer | Select/ResizeSlot/PreviewInteract，支持insert/move/reparent/wrap/extract/promote | 无delete/clipboard/multi-select/完整canvas tools |
| Preview | 真实UiSurface与compile/mock基础 | preset固定，PreviewInteract只产metadata |
| Binding | CRUD、payload projection、当前值嵌套建议 | endpoint schema非权威，仍含硬编码关键词 |
| Theme | cascade/compare/promotion/refactor基础 | token仍弱类型，跨项目usage/transaction缺失 |
| Workbench | 七份surface 1,665行、196 nodes、140 routes | 固定业务事实与伪成功第二authority |
| UI Asset ZUI | 823行、94 nodes、6 routes；另有action bar 15 nodes/8 routes | 大量能力靠Rust旁路，route/schema未统一 |
| A11y/Icon/Menu/Font | Runtime owners分别有真实底层基础 | Editor workspace没有接入真实snapshot/index/artifact |
| Authoring plugin | workspace member、experimental、descriptor完整 | 4个URI缺文件、create无factory、default禁用 |
| Importer | `.zui`到V2 View/Style/Component真实可执行 | 未发布closed cooked UI artifact |

## 4. 必须保留的工程基础

1. 保留`UiAssetEditorSession`、source revision、replay artifact、dependency generation及typed presentation，而不是以Workbench control state替代document。
2. 保留atomic file primitive与BLAKE3 digest，将其提升为document repository + reimport receipt + crash recovery协议。
3. 保留watcher background job、cancel、generation拒旧、retry/backoff和bounded ingress，扩展到open/save/undo/import全部入口。
4. 保留Runtime component descriptor、slot schema和clone-validate drop，将schema version/capability/owner generation纳入receipt。
5. 保留真实`UiSurface` preview、binding/style/theme inspection及runtime artifact snapshot consumer，删除固定产品数据。
6. 保留`.zui` V2 importer，但把source、dependency、compiler、cook、publication和runtime install generation连成单链。

## 5. P0：数据安全与产品真实性

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| UIED-P0-01 | Open | V2经legacy投影丢repeat/node slots/ThemeTokens/不可达节点/unknown/trivia；focus/navigation/picking/a11y/widget还不存在于V2 node | 建立单一lossless CST + typed semantic model，未知字段与未触及span稳定；不支持语义只能read-only，禁止降级保存 |
| UIED-P0-02 | Partial | Save已原子CAS并延后clean，但reimport失败被丢弃，clean早于import/hydration receipt | 统一write、replace、reimport、hydrate、baseline与journal；任一阶段失败保持dirty并可重试/恢复 |
| UIED-P0-03 | Partial | 普通Save可拒外改，但keep-local语义与测试/CAS矛盾，无base/ours/theirs merge | 建共享document owner或revision广播、three-way merge、显式force权限及source-control aware journal |
| UIED-P0-04 | Partial | promote有发布前回滚；发布后import/refresh失败及external-effect undo/redo仍可部分提交 | 接Editor02 Cross-Asset Transaction，声明read/write set、stage、commit、rollback、receipt与restart recovery |
| UIED-P0-05 | Open | authoring plugin四个资源URI缺失，Create只OpenView、无factory，default minimal host禁用 | 确定builtin/plugin唯一owner，补真实resource、operation factory、versioned asset、import/open/save/reopen与qualification |

## 6. P1：工程化完整性

### 6.1 Document、持久化、导入与身份

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| UIED-P1-01 | Open | 无lossless source document；以CST/token owner保存comments、unknown fields、顺序、span及semantic identity |
| UIED-P1-02 | Open | visual mutation整篇pretty-print；改为source-range edit，Format必须是独立显式command |
| UIED-P1-03 | Open | V1/V2没有versioned migration/dry-run/backup/idempotence；future version只能read-only |
| UIED-P1-04 | Partial | save/local-copy已有atomic primitive，但save/autosave/recovery仍未由同一repository contract驱动 |
| UIED-P1-05 | Partial | watcher refresh异步，initial open/save/undo hydration同步；全部入口统一走bounded job与generation commit |
| UIED-P1-06 | Open | import traversal递归且无depth/node/edge/byte/time预算；改显式栈、cycle path与typed budget outcome |
| UIED-P1-07 | Open | source update触发全量validate/hydrate/presentation；建立dirty-range、dependency impact与incremental compile |
| UIED-P1-08 | Open | undo/replay保留整份source/document副本；改结构共享、delta、checkpoint与bounded retention |
| UIED-P1-09 | Partial | 已用BLAKE3内容摘要，但持久identity仍缺file id/revision/self-write token/source-control identity |
| UIED-P1-10 | Partial | watcher path identity、disk baseline与asset identity局部存在；收敛canonical physical/logical identity与generation |

### 6.2 Designer、Hierarchy、Palette与Preview

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| UIED-P1-11 | Open | 没有node delete command；补selection policy、child/resource引用检查与single undo group |
| UIED-P1-12 | Open | 无duplicate/cut/copy/paste；实现typed clipboard、stable remap、slot validation与跨document import |
| UIED-P1-13 | Open | model可表达多个selected id但interaction不建立multi-select；补range/toggle/marquee与primary selection |
| UIED-P1-14 | Open | 无zoom/pan/fit/ruler/guide/grid/snap；建立独立viewport state与project/user policy |
| UIED-P1-15 | Open | 只有ResizeSlot，无anchor/pivot/rotate/container/slot完整handle与applicability诊断 |
| UIED-P1-16 | Open | 无align/distribute/match-size；所有批量操作需preview、constraint检查和单事务 |
| UIED-P1-17 | Open | hierarchy/palette search无可审计query authority、索引generation、virtualized result与匹配解释 |
| UIED-P1-18 | Partial | drop使用真实catalog/slot schema和clone validation；仍缺schema version/owner generation/accepted revision receipt |
| UIED-P1-19 | Partial | 可打开component/reference，但无breadcrumb/back-forward/cycle/cross-asset viewport/selection state |
| UIED-P1-20 | Open | designer tools不可插件化；建立tool descriptor、capability、input capture、overlay、transaction与lifecycle |
| UIED-P1-21 | Open | preview preset硬编码1280x720、1100x780、1920x1080、640x480；改项目device profile authority |
| UIED-P1-22 | Open | 无breakpoint与多设备矩阵、safe zone/cutout/orientation/user scaling对照 |
| UIED-P1-23 | Open | locale selector不加载真实localization generation；接Editor33/Runtime text owner |
| UIED-P1-24 | Open | 无RTL/vertical/long text/pseudo/glyph coverage矩阵与source diagnostic |
| UIED-P1-25 | Open | PreviewInteract只生成metadata；输入必须经过真实hit-test/focus/state/action并输出trace |
| UIED-P1-26 | Open | 无deterministic clock/animation/async/seed；capture不能复现，旧结果可能污染新preview |
| UIED-P1-27 | Partial | mock expression/value resolution真实存在；仍缺typed scenario source、schema version和secret boundary |
| UIED-P1-28 | Partial | preview compile/runtime report已有generation字段；未形成source/import/compiler/runtime/frame统一receipt |
| UIED-P1-29 | Open | 无pointer capture、focus path、navigation/device/IME状态可视化 |
| UIED-P1-30 | Open | 无同内容golden geometry/visual/input evidence与稳定阈值 |

### 6.3 Inspector、Binding、Theme、Menu与运行时产品

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| UIED-P1-31 | Partial | inspector有typed字段语义基础，但大量值仍按literal/TOML编辑；全面接prop/slot schema与resource picker |
| UIED-P1-32 | Partial | 有default/inherited projection片段；缺统一overridden/reset/applicability/validation source |
| UIED-P1-33 | Open | 无工程级resource/color/font/icon/localization picker、引用预览和依赖变更transaction |
| UIED-P1-34 | Open | binding suggestion仍含`SelectedNode`等硬编码关键词；只允许schema/registry驱动 |
| UIED-P1-35 | Partial | 能从当前target/value投影payload字段，但不是versioned endpoint schema；补type/required/default/enum/capability |
| UIED-P1-36 | Open | binding/action identity仍是字符串，rename不跨资产；建立stable ID、usage index与refactor transaction |
| UIED-P1-37 | Open | authoring不验证runtime service/context/capability generation；compile与preview都需fail-close |
| UIED-P1-38 | Open | Menu Flow workspace是固定`Screen_Start`；建立typed screen graph、entry/back/modal/transition资产 |
| UIED-P1-39 | Open | navigation authoring未消费Runtime77 focus graph/snapshot；source graph与runtime trace必须可对照 |
| UIED-P1-40 | Open | action/binding/menu edit没有统一journal与refactor owner；接Editor08/02 |
| UIED-P1-41 | Open | ThemeTokens仍是弱类型TOML且会降级；建立typed token kind、alias、cycle与variant schema |
| UIED-P1-42 | Partial | theme compare/pseudo state基础存在；缺design-system variant、density、contrast、platform矩阵 |
| UIED-P1-43 | Partial | cascade inspection有selector/specificity基础；缺完整source span、origin、layer、why-won解释和currentness |
| UIED-P1-44 | Partial | local/imported theme promotion/refactor存在；缺project usage index与跨资产atomic rename |
| UIED-P1-45 | Open | Accessibility Audit固定`Gameplay_HUD`/9 issues；改投影Runtime78 generation-qualified semantic snapshot |
| UIED-P1-46 | Open | V2 node没有a11y/focus/navigation字段，无法提供name/role/state/order/fix command |
| UIED-P1-47 | Open | 无键盘-only、high contrast、screen reader受控链及平台artifact |
| UIED-P1-48 | Open | Icon Library固定312/4/14；建立icon asset/cook catalog、usage index、missing诊断与theme/DPI preview |
| UIED-P1-49 | Open | icon atlas/render completeness没有Editor consumer；只消费Runtime79/asset artifact，禁止私有atlas |
| UIED-P1-50 | Open | Font Atlas固定Inter UI/4096/4/12；接Runtime80/79实际page、glyph、UV、residency和missing snapshot |
| UIED-P1-51 | Partial | importer能生成typed V2 asset，但不是依赖闭合、platform-qualified、immutable cooked artifact |
| UIED-P1-52 | Partial | runtime compiler有structured report基础，Editor未形成可消费的build receipt/diagnostic/artifact link |
| UIED-P1-53 | Partial | source/dependency/import generation分别存在；必须合并成source-to-runtime单一generation chain |
| UIED-P1-54 | Open | 无node/import/rule/binding/profile/glyph规模预算与machine-readable regression threshold |
| UIED-P1-55 | Open | 无100k hierarchy/palette/inspector/source/diagnostic统一virtualization证据 |
| UIED-P1-56 | Partial | plugin贡献有descriptor/capability基础；缺schema version、resource existence、owner lease与reload资格 |
| UIED-P1-57 | Partial | Runtime style/binding/a11y/font owners已存在，Editor也有局部artifact consumer；产品面仍未连接，必须禁止重复authority |
| UIED-P1-58 | Open | 真实UI Asset Editor与Workbench UI Asset页是两个产品入口；后者必须嵌入同一session/provider或降为test fixture |
| UIED-P1-59 | Open | 七份Workbench仍固定资产名、计数、warnings、DPI/locale；生产surface只能投影provider或Unavailable |
| UIED-P1-60 | Open | action/field handler只改control字符串却报告Saved/Validated/Applied；只有accepted domain receipt可回写成功 |

## 7. P2：完整性、诊断与维护性

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| UIED-P2-01 | Open | `style_state.rs`982行、drop resolution 951行、undo stack 932行等继续耦合；按repository/schema/command/service/projection拆owner |
| UIED-P2-02 | Open | action/control/endpoint id散落裸字符串；生成typed IDs并启动时检查重复/悬空route/resource |
| UIED-P2-03 | Open | profile/locale/budget常量没有project policy与effective source |
| UIED-P2-04 | Open | diagnostic code/severity/source mapping未统一到Editor11 journal schema |
| UIED-P2-05 | Open | 缺可关闭finding的受控视觉/输入/平台artifact，ignored microbenchmark不能替代资格 |
| UIED-P2-06 | Open | 448个focused test多数证明结构/字符串/小fixture，缺roundtrip、fault、multi-process、真实输入/平台链 |
| UIED-P2-07 | Open | bool/Option表达unsupported，丢owner/generation/reason/recovery action |
| UIED-P2-08 | Open | outline/projection/preview/import caches没有统一entries/bytes/hit/miss/evict/peak accounting |
| UIED-P2-09 | Partial | plugin maturity已从错误稳定承诺收敛为experimental；仍不是由qualification gate自动生成 |
| UIED-P2-10 | Open | authoring telemetry缺隐私/content边界，不能采集source、payload、localized text、secret和用户路径 |
| UIED-P2-11 | Open | scope fingerprint、route/resource inventory与P0 assertion未进入自动staleness检查 |
| UIED-P2-12 | Open | Layout/Widget/Style/ThemeTokens/HUD/Surface术语与owner混用；建立schema glossary |

## 8. 参考引擎差异与采用路由

| 参考 | 当前源码证据 | Zircon应采用的合同 |
|---|---|---|
| Unreal UMG | Editor映射Delete/Copy/Cut/Paste/Duplicate；Widget compiler生成class/tree并验证binding；Factory真实创建；修改走`FScopedTransaction`并标记structural change；Navigation自定义支持方向、Wrap/Explicit/Custom等规则 | 作为产品完整性、compiler、factory、transaction与navigation authoring主参考 |
| Godot | Control editor以UndoRedo分组anchors/offset/grow/size变更并提供pivot/snap/drag/preset；Theme editor支持typed import tree、filter、partial/full import、progress与单undo snapshot merge | 作为Canvas工具、theme import和可逆交互主参考 |
| Fyrox | UI Scene有deep-clone clipboard、Delete/Copy/Paste As Child、selection command；move交互先暂存，mouse-up生成`CommandGroup` | 作为Rust原生command/interaction/clipboard结构参考 |
| Bevy | Accessibility由真实UI component同步semantic node；directional map是显式可block/set图；tab group/modal有typed error；font atlas debug读取实际atlas pages | 作为runtime truth、导航图和真实诊断可视化参考，不作为完整Editor产品参考 |
| Unity Graphics | `SerializedObject` Update/Apply、Undo callback、created-object undo与redo-safe销毁顺序；DebugUI分Panel/Widget/container/query path | 本地Graphics不含完整UI Builder/TextCore源码，只采用serialized transaction/provider pattern，不推测其UI authoring细节 |
| Slint | DocumentCache保存source version/CST/dependency；property edit生成versioned WorkspaceEdit并拒绝版本错配；drop位置结合几何/布局/source version；undo以file hash防外改；catalog来自compiler type registry | 作为source-preserving、versioned edit、schema property和LSP式增量authoring稳定器 |

## 9. 目标架构与分层重构

目标链固定为：

`lossless UiAuthoringDocument -> validated UiCommand/Transaction -> source revision CAS -> import/compiler/cook receipt -> immutable runtime artifact generation -> real UiSurface preview/runtime snapshot -> truthful Editor projection`

| 里程碑 | 交付物 | 关闭范围 |
|---|---|---|
| M0 | V2 golden corpus、unknown/trivia/repeat/slots/ThemeTokens/focus/a11y roundtrip；unsupported edit fail-close | P0-01前置 |
| M1 | Lossless CST + typed semantic document、range edit、explicit format、migration | P0-01、P1-01~03 |
| M2 | Unified repository、atomic CAS save、reimport receipt、crash recovery、three-way merge | P0-02/03、P1-04/09/10 |
| M3 | Cross-Asset Transaction接管promote/refactor/undo/redo，带fault injection与restart recovery | P0-04 |
| M4 | 真实factory/resources/catalog，Create->Import->Open->Save->Reopen；closed cooked artifact | P0-05、P1-51~53/56 |
| M5 | Schema-driven inspector/palette与完整designer command/canvas/navigation | P1-11~20、31~37 |
| M6 | 多设备/locale/RTL/deterministic preview与真实input trace | P1-21~30 |
| M7 | Typed binding/menu/theme与cross-asset usage/refactor | P1-34~44 |
| M8 | Runtime-backed A11y/Icon/Font产品面，删除固定Workbench authority | P1-45~50、57~60 |
| M9 | 100k/fault/platform/package/benchmark资格、统一诊断/telemetry/memory accounting | P1-54/55、全部P2与Gate |

## 10. 验收门禁

| Gate | 状态 | 当前证据与通过条件 |
|---|---|---|
| G01 V2 repeat | Fail | 投影明确写`repeat: None`；所有mutation/save/undo后须语义与source span稳定 |
| G02 V2 slots | Fail | node slots被清空；named slot/component mount必须typed roundtrip |
| G03 ThemeTokens | Fail | 当前会降级成Style；kind/token/unknown/import必须保留 |
| G04 Unknown/trivia | Fail | pretty-print整篇重写；未触及区域需字节稳定 |
| G05 Save failure | Partial | atomic primitive和部分fault基础存在；write/flush/replace/reimport全阶段失败仍须dirty且磁盘旧版本完整 |
| G06 Crash save | Fail | 无进程终止/restart recovery证据 |
| G07 Multi-instance | Partial | CAS基础存在；两个实例同base编辑仍需merge/conflict产品证据 |
| G08 External edit | Partial | 普通Save拒外改、reload/diff/copy存在；keep-local合同矛盾且无merge/authorized force |
| G09 Cross-asset promote | Fail | import/refresh失败可留下部分提交 |
| G10 Undo/redo | Fail | external effects无transaction receipt/restart recovery |
| G11 Factory | Fail | create无operation factory且资源缺失 |
| G12 Plugin resources | Fail | 四个descriptor URI不存在 |
| G13 Migration | Fail | 无V1->V2 dry-run/backup/idempotence/future-version read-only链 |
| G14 Designer basics | Fail | delete/clipboard/multi-select/align/zoom缺失 |
| G15 Layout tools | Fail | anchor/pivot/container handle与golden geometry缺失 |
| G16 Hierarchy scale | Fail | 无100k virtualization/budget artifact |
| G17 Device matrix | Fail | preset固定，无safe-zone/orientation/user scaling矩阵 |
| G18 Localization | Fail | locale/RTL/pseudo/glyph coverage未接真实runtime generation |
| G19 Preview input | Fail | PreviewInteract未进入真实hit-test/focus/state/action |
| G20 Preview determinism | Fail | 无clock/seed/async generation资格 |
| G21 Inspector schema | Fail | builtin/plugin字段未完整schema/reset/unknown preservation |
| G22 Binding schema | Fail | endpoint非versioned authority，硬编码关键词仍存在 |
| G23 Menu flow | Fail | 固定Screen_Start，无真实graph/runtime trace |
| G24 Theme | Fail | typed token/variant/cross-asset transaction未闭环 |
| G25 Accessibility | Fail | V2缺a11y/focus字段，Audit未接Runtime78 |
| G26 Icon | Fail | 无真实catalog/cook/atlas/usage闭环 |
| G27 Font | Fail | Font Atlas静态，未验证package fallback与实际page/UV/residency |
| G28 Cook | Fail | importer不是依赖闭合runtime artifact，package source independence未证 |
| G29 Performance | Fail | 无1k/10k/100k统一内容与阈值趋势 |
| G30 Failure outcomes | Fail | provider缺失时仍可显示固定成功，状态语义未统一 |
| G31 Workbench convergence | Fail | 七份surface仍是固定业务第二authority |
| G32 Current-source evidence | Fail | 无自动stale gate与dynamic qualification artifact |

## 11. 禁止的临时修补

- 不得继续给legacy projection零散补字段后宣称V2无损；必须收敛单一document owner与unknown-field合同。
- 不得把单文件atomic write当完整Save；reimport、hydration、baseline、journal与crash recovery必须同一协议。
- 不得把keep-local默认解释为force overwrite；merge、copy和force权限必须显式。
- 不得以多个inverse closure模拟跨资产事务；promotion/refactor/undo/redo必须由transaction coordinator提交。
- 不得只补四个空ZUI或继续发`OpenView`；factory必须创建可解析、导入、打开、保存、重启再开的真实资产。
- 不得新建Editor私有font atlas、icon atlas、focus graph或a11y tree；只消费Runtime owner的generation-qualified artifact/snapshot。
- 不得用固定资产名、计数、截图、字符串feedback或retained control value模拟domain成功。
- 不得只补按钮而缺typed command、selection、transaction、undo、failure和scale合同。
- 不得在同内容、同硬件、同backend/profile和统计方法门禁通过前宣称优于Unreal。

## 12. 本轮产出边界

本文是current-source静态review与重构计划，不包含生产代码修改，也不把任何finding标记为implemented。Editor23继续是canonical finding owner，Editor97只刷新当前状态、证据和优先级。实施应先从M0 lossless contract与M1 document owner开始，不能先美化七份Workbench页面。Runtime执行边界继续由Runtime73-85等专项持有；Tooling按用户要求排除，且本轮没有查询或实时跟踪协调器。
