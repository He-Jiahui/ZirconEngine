---
title: Editor Localization、String Table、Culture、Translation Import/Export、Fallback、Pseudo-localization 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor107
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor33
refreshes:
  - docs/plans/optimize/zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md
related_code:
  - zircon_editor/src/core/i18n
  - zircon_editor/assets/i18n/en.toml
  - zircon_editor/assets/i18n/zh-CN.toml
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/notifications/presentation.rs
  - zircon_editor/src/ui/asset_editor/session/resolver_state.rs
  - zircon_editor/src/ui/asset_editor/session/runtime_report_state.rs
  - zircon_editor/src/ui/asset_editor/session/preview_compile.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/locale.rs
  - zircon_runtime_interface/src/ui/template/asset/localization
  - zircon_runtime/src/ui/template/asset/localization
  - zircon_runtime/src/ui/template/asset/compiler/component_props.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/manifest.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/text/language.rs
  - zircon_runtime/src/text/font/composite_resolve.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - tools/editor-workbench-preview/design.js
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime/src/text/language.rs
  - zircon_editor/src/core/i18n/tests.rs
  - zircon_runtime/src/ui/template/asset/localization/collect/performance_tests.rs
  - zircon_editor/src/ui/asset_editor/session/runtime_report_state.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/Localization/Public/LocalizationTargetTypes.h
  - dev/UnrealEngine/Engine/Source/Developer/Localization/Public/TextLocalizationResourceGenerator.h
  - dev/UnrealEngine/Engine/Source/Developer/Localization/Public/LocalizationChunkDataGenerator.h
  - dev/UnrealEngine/Engine/Source/Editor/LocalizationCommandletExecution/Public/LocalizationCommandletTasks.h
  - dev/UnrealEngine/Engine/Source/Editor/LocalizationDashboard/Private/SLocalizationTargetEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/StringTableEditor/Private/StringTableEditor.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Commandlets/GatherTextCommandlet.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Commandlets/GenerateTextLocalizationResourceCommandlet.h
  - dev/godot/core/string/translation.h
  - dev/godot/core/string/translation_domain.h
  - dev/godot/core/string/translation_server.h
  - dev/godot/core/io/translation_loader_po.cpp
  - dev/godot/editor/import/resource_importer_csv_translation.cpp
  - dev/godot/editor/translations/localization_editor.h
  - dev/godot/editor/translations/editor_translation_preview_menu.cpp
  - dev/bevy/crates/bevy_text/Cargo.toml
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Utilities/LocalizationHelper.cs
doc_type: current_source_refresh
review_status: complete
implementation_status: pending
source_recheck_required: true
finding_status:
  p0: 5 open
  p1: 60 open
  p2: 12 open
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor33/107 · Localization、String Table、Culture、Translation Import/Export、Fallback、Pseudo-localization 与 Preview 当前源码复核

## 1. 结论

Zircon Editor shell 有真实 i18n 基础：内嵌 TOML bundle、`EditorLocale`、线程安全 `EditorI18nService`、设置热切换、locale change/resync、bounded delivery、English fallback，以及英文/简中各 54 个完全对齐 key。Notification、后台任务、Play pending decision 在 presentation 边界捕获同一 locale 后解析，这个快照一致性应保留。

Runtime 文本也有可用底座：system locale、BCP-like text language、composite font 的 culture selector、Cosmic font-system locale cache 和 render language tag；UI Asset compiler 有 `UiLocalizedTextRef`、LTR/RTL、递归 dependency collector、missing table/key diagnostic 与 package manifest。问题是它们没有汇合为内容 Localization 产品。

最危险的正确性断点在 UI Runtime：String schema 遇到 localized table 时，compiler 用 fabricated 空 `UiValue::String` 通过类型校验，却把 TOML table 原样留在 template attributes；renderer `resolve_string_attribute()` 只接受 `Value::as_str()`，没有 Localization resolver。`text_key` 因而不能解析 translation value 或 fallback。`localization_dependencies` 只生成/序列化/测试，全仓没有 cook/package/runtime consumer。

Locale Preview 也只是报告面。菜单硬编码 `authoring-fallback`、`en-US`、`zh-CN`；切换只重算 diagnostics/list，不把 locale 传给 `compile_preview()`，session catalog 只存 locale/table/key set，不存 translation values，production registration 通常没有 caller。它能报告 key 缺失，不能预览 plural、RTL、font fallback、text expansion 或 translated glyph。

项目/游戏内容层没有正式 Localization domain。`ResourceKind` 没有 String Table、Localization Target、Translation Catalog、Compiled Localization Resource；project/cook 没有 native/supported cultures、fallback graph、PO/CSV/XLIFF、archive/manifest/chunk；Runtime 没有 culture authority、localized handle、plural/select/number/date formatting、script API 或 generation。

目标必须分离 Editor shell 与 game/plugin/DLC content：`LocalizedTextIdentity(namespace,key,source,context) -> GatherManifest -> Target/StringTable -> per-culture archive -> validated compiled catalog -> culture/package/chunk cook -> generation-qualified LocalizationService -> UI/Text/Accessibility/Script`。不能在 `UiLocalizationTableCatalog` 里继续堆字符串，也不能把 shell 的 54 key 改名成项目本地化。

## 2. 当前物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与说明 |
|---|---:|---|
| Zircon Editor/Runtime/UI/text selected | **34 / 14,184 / 13,421 / 703,024 / 20 / 0** | shell i18n、UI localized ref/compiler/renderer、font/culture、preview session 与 design evidence；fingerprint `7eabd1292af54250b7e63fe8e5571ccdd8a7cf2bb0a892ffdbd952d23d410d6a` |
| Unreal/Godot/Bevy/Unity reference | **17 / 2,460 / 1,992 / 101,262 / 0 / 0** | Localization target/resource/chunk/dashboard/string table/gather、Godot Translation/PO/CSV/editor preview、Bevy text dependency、Unity L10n helper；fingerprint `f7738fd8d56e10468c919b3333c7ade831569d7e0d1fef10c89c4e4dfca2bb33` |
| Zircon selected union | **51 / 16,644 / 15,413 / 804,286 / 20 / 0** | 两组路径不重叠；fingerprint `ef2626ccb0edadc0e88f755597652a1dad86eef03f767bb9cf2d85484a875eb6` |

当前 Editor bundle 仍是 `en`/`zh-CN` 各 54 key；这是 shell 证据，不是游戏内容覆盖率。`ResourceKind` 与 project/cook 搜索没有 StringTable/LocalizationTarget/TranslationCatalog/native culture/cultures-to-cook。实施前需重算当前 34 文件 manifest，并把 preview compile、catalog registration、font locale cache 的共享在途状态重新冻结。

逐层事实：

1. Bundle 只校验 key 非空与英文 fallback；没有 plural/select AST、placeholder schema、rich-text balance、format safety 或 culture statistics。
2. `EditorLocale::parse` 只是近似 BCP 47 的长度/ASCII/case normalization；没有 canonical parent fallback、script/region resolution 或 domain policy。
3. 参数插值是循环 `String::replace("{name}")`；没有 typed formatter、escaping、缺参数/多余参数诊断。
4. 252 个 production `.zui` 的 `text_key` 扫描为 0；动态 host presentation 文案也没有统一 LocalizedText identity/gather manifest。
5. `UiLocalizedTextRef` 只有 key/table/fallback/direction，没有 namespace、source identity、context、comment、format signature、argument schema、revision 或 owner。
6. collector 只识别 path 末尾 `.text/.label/.title` 和结构化 `text_key`；placeholder、tooltip、a11y、option/column/row、rich text、validation/custom component 文案不完整覆盖。
7. `UiLocalizationTableCatalog` 只存 key set；lookup 没有 language/script/region parent、project fallback、domain priority、entry state、value 或 revision；register 会覆盖旧 entry，无 lease/merge/conflict/unregister receipt。
8. TOML loader flatten leaf scalar，忽略 array，不验证 value 类型、format pattern、placeholder 或 rich-text tag。
9. localized table 的 compiler type check 依赖 fabricated empty String；compiled node 仍保留 TOML table，renderer 只读 literal string；fallback 不进入 visible text。
10. localization dependency 只有 manifest serialization/tests，没有 production cook/package/runtime consumer；没有 compiled catalog artifact。
11. Locale Preview 固定三项 action，unknown locale 静默回 default；`compile_preview` 没有 locale/catalog/LocalizationService 参数，projection 与 selected locale 无关。
12. session 注册 key 通常只有 tests caller；preview list 显示 locale/key 摘要与 candidate 数量，不显示 translation value、glyph、plural、RTL 或 expansion。
13. Runtime system locale只服务字体/shaping；render style language tag没有 localized handle、culture authority、translation generation、plural/number/date API 或 script host API。
14. UI asset compiler 与 shell i18n 各自拥有 locale/diagnostic概念，没有统一 domain/owner；tools Workbench 只是静态 design fixture。

## 3. 参考引擎对照

- Unreal LocalizationTarget、GatherText/GenerateTextLocalizationResource、Localization Dashboard、StringTableEditor 与 ChunkDataGenerator 分离 target/source/archive/compile/chunk，具备 namespace/key/source/notes、stale/format/rich-text validation 和 cultures-to-cook。
- Godot `Translation`/`TranslationDomain`/`TranslationServer` 提供 typed resource、context/plural、domain fallback、locale canonicalization、number formatting 与 pseudo/preview；CSV/PO importer 是真实 source pipeline。
- Bevy 本地只足以证明文本生态与 locale dependency，不提供 first-party Localization Editor；Unity Graphics 仅有 temporary L10n helper，不是 Unity Localization 产品证据。

## 4. Owner 边界与目标链

| 领域 | 唯一 owner | Editor107 必须消费/提供 |
|---|---|---|
| shell locale/preferences | Editor12 | user locale、shell bundle、fallback/resync |
| content document/asset | Editor02/04 | StringTable/Target source、stable entry、dirty/save/reimport |
| gather/import/export | Tooling03 + Localization domain | deterministic manifest、PO/CSV/XLIFF、archive/revision |
| validation/compile/cook | Editor09 + Tooling03 | placeholder/plural/markup/coverage、compiled catalog/chunk receipt |
| runtime culture | Runtime text owner | culture/fallback domain、generation、localized handle/value resolver |
| UI/text/a11y/script | Runtime UI/Text/Script owners | resolver output、format/plural/RTL/font fallback、missing/fallback telemetry |
| preview/diagnostics | Editor UI asset owner | actual catalog/culture compile/render，不另建 fixture authority |

## 5. P0：先关闭的正确性与权限边界

| ID | 当前差异 | 必须重构 |
|---|---|---|
| P0-1 | localized table 通过 fabricated String，renderer 不解析 table | compiler 输出 typed localization handle；renderer 只消费 generation-qualified resolver |
| P0-2 | Locale Preview 不影响 compile/render，只改变报告 | preview 接入真实 catalog/culture/Runtime text pipeline，返回 render receipt |
| P0-3 | 没有 content Localization domain/asset/cook/package | StringTable/Target/Archive/CompiledCatalog/Chunk source 与 factory/toolkit |
| P0-4 | shell、UI asset、game content 各自定义 locale/catalog | 明确 domain owner、fallback graph、revision、load/permission boundary |
| P0-5 | 没有 Runtime culture authority/handle/generation | LocalizationService、per-domain fallback、hot switch、missing/fallback receipt |

## 6. P1：Source、Catalog、Runtime、Preview 与资格

| ID | 差异 | ID | 差异 |
|---|---|---|---|
| P1-01 | LocalizedText identity 缺 namespace | P1-02 | key/source/context/notes 缺 schema |
| P1-03 | StringTable/Target resource 缺失 | P1-04 | entry stable id/revision 缺失 |
| P1-05 | source gather manifest 缺失 | P1-06 | literal/placeholder extraction 不完整 |
| P1-07 | PO/CSV/XLIFF importer 缺失 | P1-08 | export/archive roundtrip 缺失 |
| P1-09 | native/supported culture 配置缺失 | P1-10 | language/script/region canonicalization 缺失 |
| P1-11 | parent/domain fallback 缺失 | P1-12 | source/native fallback policy 缺失 |
| P1-13 | translation value/metadata 未存 catalog | P1-14 | context/comment/translator note 缺失 |
| P1-15 | plural/select rules 缺失 | P1-16 | number/date/list format 缺失 |
| P1-17 | placeholder type/signature validation 缺失 | P1-18 | rich-text/markup safety 缺失 |
| P1-19 | stale/obsolete/needs-review state 缺失 | P1-20 | conflict/merge/lock owner 缺失 |
| P1-21 | catalog generation/receipt 缺失 | P1-22 | content-addressed compiled resource 缺失 |
| P1-23 | cultures-to-cook/chunk manifest 缺失 | P1-24 | platform/DLC/domain load policy 缺失 |
| P1-25 | Runtime culture authority 缺失 | P1-26 | localized handle/value resolver 缺失 |
| P1-27 | resolver parent fallback 缺失 | P1-28 | runtime generation/hot-switch 缺失 |
| P1-29 | missing/fallback telemetry 缺失 | P1-30 | script translation/plural API 缺失 |
| P1-31 | UI compiler table value contract 错误 | P1-32 | fallback 未进入 visible text |
| P1-33 | dependency manifest 无 production reader | P1-34 | UI asset catalog 只存 key set |
| P1-35 | unregister/lease/merge receipt 缺失 | P1-36 | custom component text gather 缺失 |
| P1-37 | tooltip/placeholder/a11y extraction 缺失 | P1-38 | diagnostics range/source revision 缺失 |
| P1-39 | direction unknown 静默 Auto | P1-40 | RTL/bidi shaping qualification 缺失 |
| P1-41 | font culture/script fallback 不绑定 catalog generation | P1-42 | text expansion/overflow test 缺失 |
| P1-43 | pseudo-localization 缺失 | P1-44 | fake bidi/accent/markup stress 缺失 |
| P1-45 | Locale Preview 固定三项 | P1-46 | preview 未传 locale/catalog |
| P1-47 | preview 不渲染 translated glyph | P1-48 | preview 不显示 plural/RTL/fallback |
| P1-49 | supported cultures 不动态生成菜单 | P1-50 | preview stale/unsupported 状态缺失 |
| P1-51 | no StringTable toolkit | P1-52 | no Localization Target dashboard |
| P1-53 | no translation diff/review UI | P1-54 | no gather/import/export jobs |
| P1-55 | no cook/package validation gate | P1-56 | no asset reference/factory/thumbnail |
| P1-57 | no missing-key health projection | P1-58 | no scale/IO/lock/merge budgets |
| P1-59 | no cross-platform culture qualification | P1-60 | no fault/cancel/restart/migration matrix |

## 7. P2 与 32 Gate

P2 全部 Open：voice/subtitle/dialogue localization、machine translation provider、translation memory、linguistic QA dashboard、remote localization service、live content patch、collaborative semantic merge、scripted locale tests、automated screenshot diff、font atlas prewarm、regional compliance与跨引擎 archive exchange。

32 个 Gate 当前为 **32 Fail / 0 Partial / 0 Pass**。必须证明：Editor shell 与 content domain 不互相污染；gather/import/compile/cook/runtime 使用同一 entry/placeholder/plural/markup schema；missing/fallback/stale/unsupported culture 均有 receipt；Locale Preview 真正改变 UI layout/glyph/RTL/number/plural；compiled catalogs按 culture/domain/chunk 原子加载且 generation 可追溯；font fallback、text expansion、pseudo/bidi、LSP/script/a11y 与 Client/Server/Cook 矩阵均有基准。

## 8. 分层重构顺序与禁止修补

1. **M0 owner/truthfulness**：冻结 shell 与 content locale owner，补 project supported/native/fallback schema；删除 static Workbench numbers 作为功能证据。
2. **M1 source/assets**：建立 LocalizedTextIdentity、StringTable/Target source、entry revision、gather manifest、Editor02 transaction 与 Editor04 catalog/factory/toolkit。
3. **M2 archive/compile/cook**：接 Tooling03/Editor09，加入 PO/CSV/XLIFF、placeholder/plural/markup validation、per-culture archive、compiled catalog/chunk receipt。
4. **M3 runtime**：建立 generation-qualified LocalizationService、domain/parent fallback、localized handle、format/plural/number/date、script/a11y API、missing/fallback telemetry。
5. **M4 UI integration/preview**：compiler 不再 fabricated String；renderer 消费 resolver；Locale Preview 传实际 catalog/culture，输出 render/layout/diagnostic receipt。
6. **M5 quality**：pseudo/RTL/expansion/font fallback、hot switch、DLC/platform/late catalog、fault/cancel/restart、large catalog/lock/merge/cook benchmark。

禁止在 catalog 中添加几个字符串、在 preview 列表切 locale、在 renderer 做 `unwrap_or(key)` 或在 shell bundle 里硬编码游戏文案来冒充 Localization。禁止把 key count、missing-key report、font selector、test fixture 或静态 Workbench coverage 当 translation value、cook artifact、runtime culture 或 shipping 资格。

本轮只完成当前工作树逐文件静态复核、Unreal/Godot/Bevy/Unity Graphics 参考对照与差异文档，没有修改生产代码，没有运行 Cargo、PO/CSV import、gather/cook/package、locale hot-switch、translated render、pseudo/RTL、font qualification 或跨平台动态验证；实施前需重算 34 文件 manifest 与 culture/catalog 组合。
