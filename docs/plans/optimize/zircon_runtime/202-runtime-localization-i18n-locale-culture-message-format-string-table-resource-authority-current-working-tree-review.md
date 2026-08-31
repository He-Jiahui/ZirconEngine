---
title: Runtime Localization、I18N、Locale、Culture、Message Format、String Table、Resource Authority 当前工作树复核
category: zircon_runtime
report_id: Runtime202
review_date: 2026-08-31
snapshot_time: 2026-08-31T07:48:00+08:00
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Runtime83
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/83-runtime-localization-internationalization-locale-culture-message-format-plural-number-date-string-table-resource-fallback-product-integration-current-source-review.md
editor_owner:
  - docs/plans/optimize/zircon_editor/210-editor-localization-string-table-culture-translation-import-export-fallback-pseudo-preview-current-source-review.md
related_code:
  - zircon_runtime/src/ui/template/asset/localization
  - zircon_runtime_interface/src/ui/template/asset/localization
  - zircon_runtime/src/ui/template/asset/compiler/component_props.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/src/ui/template/asset/compiler/package
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/accessibility/name.rs
  - zircon_runtime/src/ui/accessibility/semantic_text.rs
  - zircon_runtime/src/ui/accessibility/extract/resolution.rs
  - zircon_runtime/src/text
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/plugin
  - zircon_runtime/src/engine_module
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime_interface/src/resource
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_app/src/entry/product_host_config
  - zircon_app/src/entry/runtime_entry_app/config
  - zircon_editor/assets/ui
  - zircon_plugins
  - examples
tests:
  - zircon_runtime/src/ui/tests/asset_localization.rs
  - zircon_runtime/src/ui/tests/asset_package_validation.rs
  - zircon_runtime/src/ui/tests/asset_resource_refs.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/Text.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/TextLocalizationManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/TextLocalizationResource.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/Internationalization.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/Culture.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/TextFormatter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/StringTableCore.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/StringTableRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/TextLocalizationManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/TextLocalizationResource.cpp
  - dev/godot/core/string/translation.h
  - dev/godot/core/string/translation.cpp
  - dev/godot/core/string/translation_domain.h
  - dev/godot/core/string/translation_domain.cpp
  - dev/godot/core/string/translation_server.h
  - dev/godot/core/string/translation_server.cpp
  - dev/godot/core/string/optimized_translation.h
  - dev/godot/core/string/optimized_translation.cpp
  - dev/bevy/crates/bevy_text/Cargo.toml
  - dev/Fyrox/Cargo.toml
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Utilities/LocalizationHelper.cs
finding_status:
  p0_open: 4
  p0_partial: 1
  p1_open: 29
  p1_partial: 19
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 36
  partial: 12
  pass: 0
---

# Runtime202 · Localization / I18N / Locale / Culture / Message Format / String Table / Resource Authority 当前工作树复核

## 1. 结论

Runtime83之后出现了三类真实进展。第一，`zircon_runtime/src/text/language.rs`已经用`icu_locale_core::Locale`统一文本系统内部的BCP 47解析、大小写规范化、language/script/region fallback key，并通过`sys-locale`取得系统文本Locale。第二，复合字体、shaping cache、glyph/atlas identity已经能携带规范化language/culture信息。第三，UI Localization collector和key-presence resolver做了path buffer、locale map lookup hoist等微优化，通用UI hot reload也具备prepare/publish式执行底座。这些代码应保留并成为未来Culture/Catalog snapshot的消费层。

但Zircon仍没有Runtime Localization产品。专用实现仍只有`UiLocalizedTextRef { key, table, fallback }`、依赖收集和`locale -> table -> key set/source URI`验证表。它不保存translation value、message pattern、source revision、owner、generation或fallback graph。`register_table_keys()`的production caller仍为0；App、project manifest、`ResourceKind`、`ImportedAsset`、Script、Dynamic API、plugin/runtime module均没有Localization catalog或culture bootstrap合同。

最严重的truthfulness断点没有变化：String属性遇到`{ text_key = ... }`时，compiler仍制造空`UiValue::String`通过schema验证；compiled tree继续保存raw TOML table；renderer、editable text和Accessibility继续只读取scalar string。因此翻译值和authoring `fallback`都不会进入真实paint、layout、shaping或screen reader。UI package虽然序列化`localization_dependencies`，但runtime artifact loader没有consumer，也没有required culture closure或catalog generation进入cache invalidation。

产品语料同样没有接入。当前扫描`zircon_editor/assets/ui`、`zircon_plugins`和`examples`共516个`.zui/.toml`文件，其中298个ZUI；`text_key`与`label_key`均为0。相同语料有`text=2937`、`label=169`、`title=12`、`placeholder=64`、`tooltip=15`、`description=197`处字面值赋值。现有ignored release microbenchmark只测collector path allocation和key存在性查找，不测catalog decode、fallback、message format、culture switch、UI rebuild、RSS或与Unreal同负载性能，不能支持“性能优于Unreal”的结论。

本报告不复制Runtime83/Editor33的唯一P0编号。当前继承状态为`4 Open / 1 Partial`；Runtime侧48项P1重判为`29 Open / 19 Partial / 0 Closed`，12项P2均Open，48门为`36 Fail / 12 Partial / 0 Pass`。正确目标不是继续扩展key-set catalog，而是建立`CultureSnapshot -> LocalizedTextIdentity -> compiled catalog generation -> bounded formatter -> LocalizationOutcome -> UI/A11y/Script/App`的唯一Runtime authority。

## 2. 冻结范围与证据等级

统计规则：路径去重；物理行与非空行按磁盘当前内容统计；tests统计`#[test]`与`#[tokio::test]`；fingerprint为相对路径排序后，对`path<TAB>file_sha256<LF>`再次计算SHA-256。选择集包含当前工作树，不回退到HEAD。

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Localization值路径、UI compiler/artifact/render/a11y/hot reload | **34** | **5,494** | **4,909** | **187,252** | **44** | **5** | `c1ad071ec6c81be876747244634ea748228df2b5b38b9c293f15c5eba6d05e85` |
| Text/Culture/Font消费底座 | **391** | **97,970** | **89,474** | **3,421,329** | **1,276** | **41** | `f62505f360ad12fb6df1fea49b3330d03e2747f0fa402ae075b4164580fb2c7d` |
| Project/App/Resource/Plugin/Module/Script边界 | **805** | **89,644** | **81,602** | **3,230,131** | **804** | **104** | `b59aed49c97fdfc5b74740d629e01dca7f5f795ce58def304df4d72d6725d3da` |
| Product `.zui/.toml` corpus | **516** | **66,403** | **57,855** | **3,549,609** | **0** | **0** | `f89d5404e33fadc9f64b2a72c0cd4cb475511e862a1e267a9203661c6f0f11cc` |
| 去重总选择集 | **1,745** | **259,294** | **233,677** | **10,382,601** | **2,124** | **150** | `cb98aad7d912f9377dbd6965d421fe633a9af1c8b068ea0d40ffe5b33ee7aef2` |

冻结时全仓`git status --short --untracked-files=all`为15,161项；总选择集有875项status record，其中source选择集727项。本报告记录的是`2026-08-31T07:48:00+08:00`读取到的共享工作树事实，不能把HEAD当成当前源码，也不能在实施时沿用本指纹而不重算。

参考选择集如下：

| 参考 | files | lines | non-empty | bytes | fingerprint | 采用合同 |
|---|---:|---:|---:|---:|---|---|
| Unreal Runtime Internationalization | **10** | **6,105** | **5,080** | **257,417** | `47b361e0ea4ac9d93eed1c0935bd5a27e87b5b38e537e37b1d3a08e0a916d72b` | FText identity/history、LocRes priority/source hash、revision、culture authority、String Table、formatter |
| Godot Translation Runtime | **8** | **2,496** | **2,105** | **96,226** | `67417e964107f2c0c467fe97e723c1636fca33f60ff96379eb8fc84fae784593` | Translation Resource、domain、locale fallback、context/plural、pseudo、optimized artifact |
| Bevy/Fyrox/Unity Graphics边界证据 | **3** | **151** | **136** | **4,489** | `bbc72650b4225c02b459216f85af13bbc23a355be2a9e5b8bc73e4d86ea39f8d` | 无first-party authority或临时helper，不作为能力上限 |
| 参考去重总计 | **21** | **8,752** | **7,321** | **358,132** | `3b1e1afc98907bf0209f741b261e63c4d4dc2cdef8d8fe2e08c2325e8aeb6a50` | 本专题直接相关切片 |

本轮是review-only，没有修改Runtime/App/Interface/plugin/Cargo/ABI/test/UI资产，没有运行Cargo、Runtime、Editor、locale switch、translated framebuffer、A11y、fault、scale、soak或benchmark。Tooling按用户要求排除；也没有查询、轮询、等待或实时跟踪协调器。

## 3. 当前存在且应保留的底座

1. `UiLocalizedTextRef`、dependency、candidate、diagnostic和direction是可序列化中立DTO雏形，可迁移为完整identity/usage合同。
2. collector递归props/layout/params/stylesheet/table/array并稳定排序；path buffer减少临时字符串分配，适合作为deterministic gather emitter的局部基础。
3. UI package已有magic/schema/payload-length验证、dependency manifest和persistent cache，可承载compiled catalog dependency，但不能继续把report sidecar当runtime resource。
4. `canonical_text_language()`使用ICU4X解析并区分empty/invalid syntax，规范化underscore和BCP 47大小写；这比旧的lowercase字符串路径更可靠。
5. `TextLanguageFallbackKey`与`TextCultureSelector`已形成language/script/region选择基础，复合字体和shaping cache能消费规范化identity。
6. Runtime font asset、project font cook、font generation、text layout/shaping/raster/atlas链已经存在，Localization只应提供resolved text/language/script/direction，不另造文本引擎。
7. 通用Resource registry/lease/generation、plugin owner生命周期与UI hot reload prepare/publish可复用于catalog generation、reader fence和last-good发布。
8. Editor contribution的LocalizationBundle具备typed contribution variant和owner校验方向，但它是Editor extension bundle，不是Game/Runtime catalog，也没有pattern/signature/generation。
9. `TextLayoutError`和UI binding diagnostic已经使用稳定localization key；这些是未来diagnostic catalog consumer，不代表当前已有翻译服务。
10. 两个ignored release benchmark保留了raw sample与alternating order思路，但只能作为collector/key-presence局部基线。

## 4. 当前真实值路径与断路

### 4.1 Authoring、collector与schema

`collect_document_localization_report()`只在TOML table包含string `text_key`时形成引用，只验证key非空。`direction`未知值静默降为`Auto`。literal extraction只按路径尾部`.text/.label/.title`判断，不由component schema声明，也遗漏placeholder、tooltip、description、options、rich text与显式A11y字段。

`component_prop_value()`先尝试普通`UiValue`转换；失败后只要String schema和合法`text_key`，就返回空`UiValue::String`。这不是解析，而是绕过类型验证。compiler产物没有localized slot、identity、argument signature或catalog binding。

### 4.2 Artifact、dependency与cache

package report和dependency manifest会保存`localization_dependencies`，但`UiCompiledDocument`只有template instance与普通resource dependencies；`UiRuntimeCompiledAssetArtifact`最终只有report和template tree。生产代码中`localization_dependencies`没有loader consumer。

cache key不包含catalog domain/culture/source hash/generation；translation更新不能基于依赖identity驱逐compiled document或已构建surface。Runtime package profile的retained section也没有Localization catalog/index/pattern段。

### 4.3 Catalog与lookup

`UiLocalizationTableCatalog`实际是key存在性索引：`BTreeMap<locale, BTreeMap<table, {source_uri, key_set}>>`。注册同locale/table会静默覆盖；lookup严格匹配trim后的原始locale；空locale直接返回空诊断；没有canonicalization、parent/native/default fallback、priority、owner lease、load/unload或generation。

TOML helper只收集scalar leaf path，数组被忽略，translation value本身被丢弃。missing key的`fallback`只把diagnostic从Error降为Warning，不返回fallback文本。

### 4.4 Tree、render与Accessibility

tree builder复制`node.attributes`，因此localized TOML table原样进入metadata。`resolve_text()`、placeholder、editable state、collection row label等路径只接受scalar string；Accessibility name/description/tooltip preflight和resolution也只接受string。当前没有一个snapshot/resolver在surface build前把identity解析为value。

结果不是“仅缺高级plural”：最基础的translation value和fallback都无法显示，visual text与screen reader也没有共同generation。

### 4.5 Culture、Text与App边界

文本系统的ICU Locale类型为`pub(crate)`且职责是font/shaping hint。它没有requested/resolved content culture、format locale、asset group culture、supported/native culture、override precedence或switch receipt。`system_text_locale()`直接选择系统Locale或`en-US`，没有project policy。

`zircon_app` product host/runtime config、project manifest、`ResourceKind`、`ImportedAsset`和asset importer都没有Localization/StringTable/Catalog类型。Script host只有几何`translate`，Dynamic API/Runtime plugin没有resolve/format/snapshot API。Editor shell/extension bundle和Runtime UI key ref仍是分裂authority。

### 4.6 性能与真实性

当前微基准的optimized path只证明：collector可复用path buffer，resolver可把locale table lookup从每dependency一次提升到一次。它们的catalog只有一个key set，50,000个dependency重复查同一个key，不包含真实value、UTF-8 payload、pattern AST、fallback chain、plural rule、format argument、generation lease或UI rebuild。

工程性能声明必须在同catalog规模、locale数、fallback深度、pattern复杂度、线程数和switch频率下，与选定Unreal基线比较p50/p95/p99、alloc、RSS、load/switch pause和failure recovery。否则“Rust/BTreeMap/更简单”都不是优于Unreal的证据。

## 5. 参考实现差异

| 工程轴 | Unreal | Godot | Zircon当前 |
|---|---|---|---|
| 文本身份 | namespace/key/source、FText history、display string identity | source message + context | key/table/fallback，无domain/namespace/source/context/history |
| Culture authority | language、locale、asset-group culture分离，prioritized culture names | 全局locale、fallback及domain override | crate-private text language hint；无content culture snapshot |
| Resource | LocRes native culture、source hash、localized value、priority/target path | Translation Resource、project load、domain add/remove | key set + optional source URI；无typed Runtime resource |
| Publication | global/local text revision与change event | translation changed/reload | 无catalog generation、lease或change receipt |
| Fallback | prioritized culture/source layer | locale comparison、fallback locale、domain | exact raw locale/table lookup |
| Formatting | compiled format pattern、named/ordered typed arguments与modifier | plural/context及language rules | 无message AST、plural/select、typed args |
| String Table | source/dev note/metadata、registry、display string | Translation资源可作为table/domain | TOML leaf key collector丢弃value |
| Pseudo | 与culture/text system协同 | accent/double vowel/fake bidi/expansion/placeholder policy | 无Runtime pseudo；direction仅authoring hint |
| Compact runtime | versioned LocRes加载与priority merge | OptimizedTranslation压缩/索引 | UI TOML report，不是shipping catalog |
| 性能证据 | 成熟revision/cache/load路径，可定义同负载基线 | optimized resource可作轻量对照 | 仅key存在性微基准 |

Bevy/Fyrox当前本地树未发现first-party Localization/I18N authority，只能说明它们没有提供本专题目标合同，不能降低Zircon标准。Unity Graphics的`LocalizationHelper`源码明确写明它是等待更好UXML支持的temporary helper，且只遍历tooltip/label调用`L10n.Tr`；它是应避免的临时实现反例，而不是Unity完整Localization产品证据。

## 6. 目标Architecture与Owner

```text
Platform/Profile/Project locale inputs
    -> CulturePolicy + CultureSnapshot(requested/resolved/format/asset-group)
    -> LocalizationDomainRegistry(owner/version/priority/lease)
    -> LocalizationCatalogArtifact(domain/culture/schema/source-hash/index/patterns)
    -> LocalizationCatalogGeneration + reader snapshot
    -> LocalizationService::resolve(identity, typed_args, snapshot)
    -> LocalizationOutcome(value/resolved-culture/fallback/provenance/status)
    -> UI + Accessibility + Script + App + Diagnostics
    -> TextRuntimeContext(language/script/direction/font/unicode/layout)
```

Owner边界：

| Owner | 必须拥有 | 禁止拥有 |
|---|---|---|
| `zircon_runtime_interface` | Culture/identity/argument/outcome/status/generation稳定DTO | loader、全局map、Editor bundle实现 |
| `zircon_runtime` LocalizationService | catalog load/validate/publish、fallback、format、cache、budget、lease、diagnostic | Editor gather UI、renderer文件读取 |
| Asset/Resource | typed source/archive/compiled catalog、dependency/cook/chunk、revision | locale选择与消息格式业务 |
| App | platform/project/profile/CLI override precedence与bootstrap receipt | 私有翻译表或直接修改子系统locale |
| UI/A11y | pin snapshot、消费resolved outcome、定向dirty | 解析TOML、直接读catalog或自行fallback |
| Text | shaping/layout/font/raster，消费language/script/direction | 翻译identity、catalog priority、message format |
| Script/Plugin | bounded versioned resolve/format API、domain owner lease | 裸全局map、无owner注册、绕过generation |
| Editor | gather/import/export/authoring/preview/cook，消费同一Runtime artifact | 另造shipping resolver |

## 7. P0继承状态

| Canonical P0 | 状态 | 当前Runtime证据 |
|---|---|---|
| localized ref没有Runtime resolver | `Open` | compiler空String，render/a11y仍只读scalar |
| 无项目Localization资产/culture authority/shipping cook | `Open` | ResourceKind/ImportedAsset/project/App均无typed owner |
| Locale Preview不是真实Runtime预览 | `Open` | Editor210确认固定选项且preview compile无snapshot/service |
| 无Gather/Import/Export/Compile闭环 | `Open` | Runtime只有dependency/key presence report |
| Editor shell/project/Runtime text locale语义分裂 | `Partial` | text已有ICU规范化，Editor/plugin有typed key；仍无公共Culture/Catalog合同 |

## 8. P1工程化差距

### 8.1 Culture、身份与domain

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| RLI-P1-001 | `Open` | 无Runtime content culture authority | App发布requested/resolved snapshot，Service为唯一lookup authority |
| RLI-P1-002 | `Partial` | text language已规范化，format/content/asset culture未分离 | 定义职责、转换和序列化合同 |
| RLI-P1-003 | `Partial` | Runtime text用ICU；UI catalog和Editor仍各自解析 | 公共完整`CultureTag`，invalid/unsupported/unavailable分开 |
| RLI-P1-004 | `Open` | 无parent/native/default fallback graph | 有向无环、深度有界、可解释fallback plan |
| RLI-P1-005 | `Open` | 无project/profile/CLI/platform/session优先级 | 冻结precedence并输出typed bootstrap receipt |
| RLI-P1-006 | `Partial` | Text可推script，localized direction仍只是hint | outcome携带language/script/direction并诊断override冲突 |
| RLI-P1-007 | `Partial` | Text/font有generation，culture/catalog无共同代际 | snapshot原子绑定culture/catalog/font/locale-data generation |
| RLI-P1-008 | `Partial` | Editor plugin有bundle owner；Game/DLC domain没有 | Runtime domain owner/version/priority/lease/revoke |

### 8.2 LocalizedText、catalog与artifact

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| RLI-P1-009 | `Open` | ref只有key/table/fallback | domain/namespace/key/source/context及稳定identity |
| RLI-P1-010 | `Open` | 无source hash/stale判定 | source revision进入entry并驱动stale policy |
| RLI-P1-011 | `Open` | catalog只保存key | compiled entry保存validated value/pattern/provenance |
| RLI-P1-012 | `Partial` | source URI存在，但同locale/table静默覆盖 | owner-aware merge/conflict、lease、unregister receipt |
| RLI-P1-013 | `Open` | TOML loader丢value且忽略数组 | source importer与shipping reader分离，schema fail-closed |
| RLI-P1-014 | `Open` | ResourceKind/ImportedAsset无catalog/string table | typed source/archive/catalog资源、loader/importer/facade |
| RLI-P1-015 | `Partial` | UI artifact有envelope，Localization无独立artifact | catalog magic/schema/checksum/index/size cap/compatibility |
| RLI-P1-016 | `Open` | 无pattern argument signature | 跨culture typed signature与compile validation |

### 8.3 Load、generation、reload与budget

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| RLI-P1-017 | `Partial` | dependency写入manifest/report，未被loader消费 | artifact引用catalog ID/revision/required culture closure |
| RLI-P1-018 | `Open` | UiCompiledDocument无localized slot/binding | compiler产出stable identity/slot，不保留raw TOML |
| RLI-P1-019 | `Open` | cache key无external catalog generation | catalog revision进入dependency与invalidation |
| RLI-P1-020 | `Open` | 无runtime loader consumer | load/validate/admit后才原子publish generation |
| RLI-P1-021 | `Partial` | 通用hot reload有plan，Localization落入Other | typed catalog/domain/culture prepare-publish-targeted dirty |
| RLI-P1-022 | `Partial` | 通用UI publication有可复用准备阶段，无catalog last-good | corrupt/partial reload保留旧generation并给失败receipt |
| RLI-P1-023 | `Partial` | generic plugin/resource lease存在，无catalog reader fence | revoke新lookup、等待snapshot reader、再释放blob |
| RLI-P1-024 | `Open` | 无entry/pattern/cache/locale-data预算 | per-domain/culture admission、LRU与bounded diagnostics |

### 8.4 Compiler、UI、A11y与invalidation

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| RLI-P1-025 | `Open` | 空String绕过String schema | schema接受typed LocalizedText并fail-closed |
| RLI-P1-026 | `Open` | tree复制raw localized table | metadata只保存identity/slot/resolved handle |
| RLI-P1-027 | `Open` | renderer只读scalar string | build/extract前以pinned snapshot解析 |
| RLI-P1-028 | `Open` | fallback只改变diagnostic severity | outcome显式返回value/path/status/provenance |
| RLI-P1-029 | `Partial` | collector递归，但只识别text/label/title | component schema声明所有localizable property |
| RLI-P1-030 | `Partial` | direction枚举存在，unknown静默Auto | invalid fail-closed，Auto来自resolved culture/script |
| RLI-P1-031 | `Open` | Accessibility只消费scalar | visual/semantic text共享同一outcome/snapshot |
| RLI-P1-032 | `Partial` | Text dirty/font generation存在，无catalog slot index | culture change定向触发text/layout/hit/render/a11y/font更新 |

### 8.5 Message format、国际化数据与API

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| RLI-P1-033 | `Open` | 无plural/select/gender | 采用成熟CLDR/ICU级provider，bounded AST evaluation |
| RLI-P1-034 | `Open` | 无typed arguments | bool/int/float/string/text/date/time/enum及required policy |
| RLI-P1-035 | `Open` | 无number/percent/currency/unit/list formatter | locale-data、rounding/grouping/numbering system合同 |
| RLI-P1-036 | `Open` | 无date/time/timezone/duration formatter | calendar/hour-cycle/TZDB版本与invalid timezone语义 |
| RLI-P1-037 | `Open` | 无pattern/rich escaping边界 | literal/argument/markup分层，禁止action/resource注入 |
| RLI-P1-038 | `Open` | 无Runtime pseudo locale | accent/expansion/double-vowel/fake bidi/placeholder保护 |
| RLI-P1-039 | `Partial` | ICU依赖版本固定，artifact不记录locale-data lineage | catalog记录provider/data版本并保证replay determinism |
| RLI-P1-040 | `Open` | Script/plugin/dynamic ABI无Localization API | versioned resolve/format/snapshot/status/limit合同 |

### 8.6 App、Editor、产品与性能资格

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| RLI-P1-041 | `Open` | App只可取得system text locale，无supported/native policy | startup culture negotiation与bootstrap receipt |
| RLI-P1-042 | `Partial` | Editor bundle与Runtime Interface有局部typed key，底层仍分裂 | 共享Culture/Identity/Pattern/Catalog substrate，domain隔离 |
| RLI-P1-043 | `Partial` | Editor locale event producer存在，Runtime无共同consumer | generation resync驱动所有locale-sensitive projection |
| RLI-P1-044 | `Open` | Preview locale不来自project catalog | project/cooked snapshot生成可用culture列表 |
| RLI-P1-045 | `Open` | Preview compile无locale/catalog/service | Preview与shipping共用resolver/artifact |
| RLI-P1-046 | `Open` | 516份产品语料零text_key | 迁移真实Editor/WOC flow并验证visual+a11y+script |
| RLI-P1-047 | `Partial` | key/missing diagnostics有code/path，无generation/provenance/redaction | structured bounded diagnostic与identity hash |
| RLI-P1-048 | `Partial` | 只有collector/key-presence ignored微基准 | 同负载cold/warm/switch/load/unload/format/RSS/fault基准 |

## 9. P2扩展差距

| ID | 状态 | 后续能力 |
|---|---|---|
| RLI-P2-001 | `Open` | locale-specific asset remap与atomic dependency generation |
| RLI-P2-002 | `Open` | dialogue/subtitle/caption domain、speaker/context/audio sync |
| RLI-P2-003 | `Open` | localized voice/audio pack、lip-sync variant、streaming budget |
| RLI-P2-004 | `Open` | grammatical inflection/morphology与typed entity features |
| RLI-P2-005 | `Open` | locale-aware collation/search/sort及index generation |
| RLI-P2-006 | `Open` | locale-aware editable number/date parsing与intermediate-invalid state |
| RLI-P2-007 | `Open` | live localization session、authenticated remote preview、recovery |
| RLI-P2-008 | `Open` | mod/UGC translation overlay、签名、priority、sandbox、revoke |
| RLI-P2-009 | `Open` | per-user/network locale与gameplay determinism边界 |
| RLI-P2-010 | `Open` | TTS/speech locale、voice selection、privacy/platform capability |
| RLI-P2-011 | `Open` | per-culture screenshot/a11y golden与coverage manifest |
| RLI-P2-012 | `Open` | signed live-ops translation hotfix、rollback、expiry、offline last-good |

## 10. 分层重构顺序

1. **M0 Truthfulness**：删除空String绕过；在Runtime resolver完成前，localized prop必须以typed unsupported error fail-closed，不能静默渲染空文本。
2. **M1 Public Contract**：在Interface定义`CultureTag`、`LocalizedTextIdentity`、typed args、generation、outcome/status；Text内部ICU helper收敛为其消费实现。
3. **M2 Asset/Artifact**：建立Localization Target/String Table/Archive/Compiled Catalog资源、schema、checksum、source hash、required culture/chunk closure。
4. **M3 Runtime Service**：domain registry、fallback DAG、prepare/publish、snapshot lease、last-good、cache/budget/diagnostic与shutdown。
5. **M4 Formatter**：引入成熟locale-data/provider，完成plural/select、number/date/time/unit/list和escaping；禁止手写简化复数规则。
6. **M5 UI/A11y/Text**：compiler产slot，surface pin snapshot，resolve outcome进入visual与semantic text，culture change按slot定向dirty。
7. **M6 App/Script/Plugin/Editor Bridge**：启动culture policy、versioned host API、owner lease、真实Preview和Editor shell底层收敛。
8. **M7 Product Migration**：迁移至少一个Editor完整flow和一个WOC完整flow；保存、重开、cook、shipping、fallback、pseudo、RTL均留证。
9. **M8 Qualification**：fault/scale/soak/security/locale-data determinism和与Unreal同负载性能对照，通过后才允许优于Unreal声明。

M0-M3是MVP产品真实性前置；M4-M8不得通过复制Editor bundle、renderer读文件或hard-coded locale绕过。

## 11. 48项资格门

| Gate | 状态 | 验收条件 |
|---|---|---|
| G01 | `Partial` | 所有locale入口使用同一CultureTag canonicalizer，旧helper无production caller |
| G02 | `Partial` | invalid/unsupported/unavailable culture返回不同typed status |
| G03 | `Fail` | fallback graph拒绝循环/重复/超深链且路径可导出 |
| G04 | `Partial` | language/format locale/asset culture职责有contract test |
| G05 | `Fail` | project/profile/CLI/platform/session precedence可复现 |
| G06 | `Fail` | culture switch发布单一generation且复合projection不混代 |
| G07 | `Fail` | domain注册必须有owner/version/priority/lease |
| G08 | `Fail` | plugin/DLC revoke后新lookup失败、旧snapshot安全结束 |
| G09 | `Fail` | identity含domain/namespace/key/source/context并稳定序列化 |
| G10 | `Fail` | source改变标记stale且不静默沿用 |
| G11 | `Fail` | 各culture argument signature完全一致 |
| G12 | `Fail` | duplicate/conflicting entry在compile/load fail-closed |
| G13 | `Partial` | compiled catalog有magic/schema/checksum/size cap |
| G14 | `Fail` | too-new/truncated/checksum错误保留last-good |
| G15 | `Fail` | index/payload有bytes/entry/pattern预算 |
| G16 | `Fail` | authoring TOML/PO/CSV不能成为shipping authority |
| G17 | `Partial` | UI/package manifest携带catalog dependency与culture closure |
| G18 | `Fail` | external catalog revision进入compile/cache invalidation |
| G19 | `Fail` | runtime loader实际消费localization dependency |
| G20 | `Partial` | load/reload仅在prepare成功后原子publish |
| G21 | `Fail` | hot reload按catalog/domain/culture定向失效 |
| G22 | `Fail` | unload等待reader lease且无dangling resolved string |
| G23 | `Partial` | cold/warm/failure receipt包含bytes/time/generation |
| G24 | `Fail` | shutdown后无后台load/watch/lease/callback |
| G25 | `Fail` | compiler不再伪造空String |
| G26 | `Fail` | compiled template不再交付raw localized TOML |
| G27 | `Fail` | translation value和fallback进入真实paint command |
| G28 | `Fail` | visual/A11y使用同一snapshot/outcome |
| G29 | `Partial` | component schema声明全部localizable property |
| G30 | `Fail` | invalid direction fail-closed，Auto来自culture/script |
| G31 | `Partial` | locale change触发正确dirty域 |
| G32 | `Fail` | stale surface不能混用新catalog generation |
| G33 | `Fail` | plural/select覆盖目标culture全部CLDR类别 |
| G34 | `Fail` | missing/wrong/extra argument可诊断 |
| G35 | `Fail` | number/percent/currency/unit/list golden固定data版本 |
| G36 | `Fail` | date/time/timezone/duration覆盖DST和invalid timezone |
| G37 | `Fail` | translation/args不能注入rich action/resource |
| G38 | `Fail` | pattern parser/evaluator有fuzz/depth/bytes/deadline预算 |
| G39 | `Fail` | pseudo覆盖accent/expansion/fake bidi/placeholder且release排除 |
| G40 | `Fail` | Script/dynamic ABI保留identity/args/generation/status |
| G41 | `Partial` | App按supported cultures选择并发布bootstrap receipt |
| G42 | `Fail` | Editor shell与project domain共享substrate且owner隔离 |
| G43 | `Fail` | locale topic有product subscriber/full-resync证据 |
| G44 | `Fail` | Preview locale来自真实catalog，不硬编码 |
| G45 | `Fail` | Preview与shipping使用同一resolver/artifact |
| G46 | `Fail` | Editor和WOC各一条translated visual+a11y+script证据 |
| G47 | `Partial` | diagnostics有界去重、脱敏并含culture/domain/generation/provenance |
| G48 | `Partial` | 同负载benchmark覆盖p50/p95/p99/alloc/RSS/switch/load并附原始证据 |

## 12. 禁止的临时实现

1. 禁止让renderer或Accessibility直接读取`fallback`、TOML、CSV、PO并称为Runtime resolver。
2. 禁止继续制造空String让localized table通过schema。
3. 禁止把`UiLocalizationTableCatalog`改名为LocalizationService但仍只保存key set。
4. 禁止复制Editor内嵌bundle到Runtime或把Editor contribution bundle当shipping catalog。
5. 禁止手写`if locale == ...`、两类plural规则、字符串替换式message formatter或无限fallback递归。
6. 禁止process-global可变locale/catalog、无owner注册、静默覆盖和无reader fence卸载。
7. 禁止在render/layout热路径做文件I/O、TOML解析、catalog merge或锁住全局map。
8. 禁止把系统Locale当project支持Locale，也禁止无策略固定`en-US`。
9. 禁止以collector/key-presence微基准、测试数量、Rust语言或容器选择推导优于Unreal。
10. 禁止只迁移测试fixture而不迁移真实Editor/WOC产品flow。

## 13. 本轮完成边界

本轮完成了Runtime Localization当前工作树、Text/Font邻接层、Asset/Project/App/Plugin/Module/Script边界、516份产品语料和本地参考源码的静态复核，登记了当前状态、目标owner、分层路线与资格门。没有实施代码修正，也没有关闭任何Runtime Localization P1或工程门。后续实施必须从M0 truthfulness和M1公共合同开始，并在共享工作树稳定后重新计算所有指纹与调用点。
