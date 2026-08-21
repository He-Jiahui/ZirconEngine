---
title: Runtime Localization、Internationalization、Locale、Culture、Message Format、Plural、Number/Date、String Table、Resource Fallback 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime83
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/ui/template/asset/localization
  - zircon_runtime_interface/src/ui/template/asset/localization
  - zircon_runtime/src/ui/template/asset/compiler/component_props.rs
  - zircon_runtime/src/ui/template/asset/compiler/package
  - zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs
  - zircon_runtime/src/ui/template/build
  - zircon_runtime/src/ui/template/pipeline.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/accessibility/name.rs
  - zircon_runtime/src/text/language.rs
  - zircon_runtime_interface/src/ui/surface/render/text_language.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/src/core/i18n
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/ui/asset_editor/session/runtime_report_state.rs
  - zircon_editor/src/ui/asset_editor/session/resolver_state.rs
  - zircon_editor/src/ui/asset_editor/session/preview_compile.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor/actions/mode_preview/locale.rs
  - examples/woc/native/apps/woc_client/src/preferences/settings/options.rs
  - examples/woc/native/apps/woc_client/src/shell/realm_directory.rs
tests:
  - zircon_runtime/src/ui/tests/asset_localization.rs
  - zircon_runtime/src/ui/tests/asset_package_validation.rs
  - zircon_editor/src/core/i18n/tests.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/action_localization_reports.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_reports.rs
  - zircon_editor/src/tests/editing/ui_asset/runtime_report_productization.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
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
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/Internationalization.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/Text.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/TextFormatter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/StringTableRegistry.cpp
  - dev/godot/core/string/translation.h
  - dev/godot/core/string/translation.cpp
  - dev/godot/core/string/translation_domain.h
  - dev/godot/core/string/translation_domain.cpp
  - dev/godot/core/string/translation_server.h
  - dev/godot/core/string/translation_server.cpp
  - dev/godot/core/string/optimized_translation.h
  - dev/godot/core/string/optimized_translation.cpp
  - dev/bevy/crates/bevy_text/Cargo.toml
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Utilities/LocalizationHelper.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Localization、Internationalization、Locale、Culture、Message Format、Plural、Number/Date、String Table、Resource Fallback 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前并非完全没有本地化相关代码。UI Asset层已经有`UiLocalizedTextRef { key, table, fallback }`、方向枚举、递归依赖收集、缺表/缺键诊断和package dependency manifest；Editor shell已有English/zh-CN各54个key的内嵌catalog、用户locale设置热同步、同一locale快照翻译和有界事件投递。这些局部底座应保留。

但是运行时产品链在最关键的值解析处断开。String类型属性遇到localized table时，compiler通过伪造空`UiValue::String`绕过类型检查，同时把原TOML table保留到template attributes；tree builder继续原样复制，renderer和accessibility只接受scalar string，因此既不显示翻译值，也不消费`fallback`。依赖清单虽被序列化进artifact report和persistent cache，运行时实例却只有template tree，仓库中没有任何consumer读取`localization_dependencies`并装载catalog。

`UiLocalizationTableCatalog`的名字也高估了能力：它只保存`locale -> table -> source_uri + key set`，不保存translation value、pattern、source hash、revision、owner或generation；所谓resolver仅做存在性验证。注册同locale/table会直接覆盖旧entry，lookup严格匹配原始locale字符串，没有canonicalization、parent fallback、native culture、domain priority、load/unload或last-good generation。

产品证据与此一致：本轮扫描296个产品`.zui/.toml`，其中272个`.zui`共有3,239处text-like赋值，但`text_key`出现0次。WOC原生客户端另有32处`label_key/text_key`声明，却没有运行时翻译consumer。UI Asset Locale Preview仍只有`authoring-fallback/en-US/zh-CN`三个硬编码选项，切换只重算报告，`compile_preview()`不接收locale或catalog，production也没有调用`register_locale_table_keys()`。

Editor33已经唯一登记了5项架构P0：无Runtime resolver、无项目Localization资产/culture authority/shipping cook、Locale Preview不真实、无Gather/Import/Export/Compile闭环、Editor shell/project content/Runtime text locale语义分裂。本报告不重复计数，新增P0为0；它把这5项阻断在Runtime侧展开为48项P1、12项P2和48项资格门。目标不是给现有key-set catalog补几个字符串，而是建立generation-qualified、可装卸、可追踪、可格式化、可被UI/A11y/Script/App共同消费的Runtime LocalizationService。

## 2. 审查边界与可复现冻结

### 2.1 Zircon当前源码冻结

统计规则：路径去重；物理行按`Get-Content`行数计；test attributes统计`#[test]`与`#[tokio::test]`；fingerprint为按仓库相对路径排序后，对`path<TAB>file_sha256<LF>`再次做SHA-256。冻结包含当前工作树内容，不把dirty文件回退到HEAD。

| 范围 | 文件 / 行 / bytes / test attributes | fingerprint | 本轮证据 |
|---|---:|---|---|
| Runtime identity / catalog contracts | 12 / 749 / 22,361 / 3 | `48c57ae0768d583eef5fc2c51124076c24422335b76f06c4c8cc96c688432aea` | localized ref/report、key-set catalog、locale helper、resource kind |
| Compiler / artifact / render / reload | 20 / 2,532 / 90,445 / 0 | `b8c12ad84fdbaf2c4ea2ca9151485413b6eca1255bc9f06432eb72fe78364f7e` | validation、package/cache、tree/surface、renderer/a11y、hot reload |
| Editor bridge / preview | 22 / 5,007 / 190,297 / 23 | `c8a9bdb8e67a6ad838d3f274f18223c85dadcd2b8bb94a9f152657cbc7622636` | shell i18n、settings/bus、UI Asset report/catalog/preview |
| Product assets / focused tests | 303 / 56,230 / 3,193,729 / 22 | `ad9b1e3ca1ebea23e9646dbb62fdbd10a60732042f75aac05cd0e74db340b287` | 296份ZUI/TOML、WOC key模型、focused contract tests |
| 去重合计 | **357 / 64,518 / 3,496,832 / 48** | `98d7f826947e54dd59e7ef9e9604002ff4e9be6d7a7f98ffd66b4c1010db6af0` | 当前专题完整冻结 |
| Production子集 | **351 / 62,939 / 3,447,624 / 16** | `5dc3d110d72c6ff45ca4fef22dfb301457b149be0bbb66b8743112430c190bb0` | 排除focused test文件，保留production内联测试 |

冻结时6个入选路径dirty且均非本轮产生：`zircon_editor/src/core/i18n/mod.rs`、`tests.rs`、UI Asset的`preview_compile.rs`、`resolver_state.rs`及两份runtime-report测试。本报告按读取时工作树事实编写；实施前必须重算fingerprint，并复核这些路径的最终状态。

### 2.2 参考引擎冻结

| 引擎 | 文件 / 物理行 / 非空行 / bytes | fingerprint | 采用的工程合同 |
|---|---:|---|---|---|
| Unreal | 14 / 10,449 / 8,775 / 401,199 | `02c90d99dd32510f9d7f7204560e8c843b5426581b0fddf65f176c62ead172ca` | FText identity/history、language/locale/asset-group culture、priority source、locres、revision、format/string table |
| Godot | 8 / 2,496 / 2,105 / 96,226 | `67417e964107f2c0c467fe97e723c1636fca33f60ff96379eb8fc84fae784593` | domain、locale score/fallback、plural/context、pseudo、translation remap、optimized artifact |
| Bevy | 1 / 56 / 51 / 1,978 | `7115edc9acddcba6f0492ada10375619917fd00c1bbce53bf09cec0212217d6b` | `sys-locale`仅服务text生态，证明本地树无完整Localization authority |
| Unity Graphics | 1 / 47 / 41 / 1,697 | `f5e0812e975a6942392ac9405f33351a36b3294de52920f871442a5676ccde5b` | 临时Editor VisualElement helper边界，不当作Unity Localization产品实现 |
| 去重合计 | **24 / 13,048 / 10,972 / 501,100** | `706ea1234c802e678a33fb0b277ef75809609f998fd79e61202e27f2e570effa` | 只引用本专题直接相关源文件 |

Fyrox全树关键词扫描没有找到专用Localization/Internationalization运行时系统，因此本专题不从无关UI文本代码虚构能力，也不把它作为本地化工程上限。参考选择以Unreal为主、Godot为轻量对照；Bevy/Fyrox和Unity Graphics只用于确认owner边界。

### 2.3 未执行的验证

本轮是current-source静态review，没有修改Rust/TOML/ZUI production或tests，没有运行Cargo、App、Editor、真实locale switch、translated render、RTL/pseudo、screen reader、catalog load/unload、plugin/DLC、fault、soak或benchmark。48个test attributes只表示入选源码存在测试，不表示Runtime translation产品已通过；更不构成性能优于Unreal的证据。

## 3. 当前值得保留的底座

1. `UiLocalizedTextRef`、`UiLocalizationDependency`、diagnostic与direction已经是可序列化的中立DTO雏形，可以迁移为完整LocalizedText合同，而不是删除后重做。
2. collector会递归遍历props/layout/params和stylesheet值，package report也会稳定排序并序列化依赖，适合继续承担source usage与cook closure输入。
3. compiled artifact有magic、schema version、payload length、fingerprint和persistent cache，说明compiled localization catalog可复用现有artifact/resource基础，而不应直接读取authoring TOML。
4. UI hot reload已经有plan/executor、cache eviction、surface dirty和last-known registry方向，可扩展为catalog generation reload，而不应在renderer热路径读文件。
5. Editor i18n会捕获同一locale后解析compound projection，settings generation能拒绝late snapshot，event queue有32 events/64 bytes上限并在背压时合并resync；这些并发语义应保留。
6. `EditorContextBuilder`当前确实配置了i18n event sink和settings change subscriber，并向`EditorTopic::i18n()`发布changed/resync。这纠正了“完全无producer”的过时判断，但product host仍没有保留的subscriber去重建所有locale-sensitive projection。
7. Runtime80/81已有font culture、language hint、shaping/layout generation基础；Localization resolver应向该链输出language/script/direction和resolved text，不另造字体或BiDi引擎。
8. WOC settings和realm模型已经使用locale-neutral key字段，后续可以接Runtime service；现状只是model substrate，不是display translation完成。

## 4. P0状态与唯一归属

本报告新增P0为0。以下阻断仍由Editor33唯一计数，Runtime83只提供当前源码复核与Runtime实施分解：

| Editor33唯一P0 | Runtime83当前证据 |
|---|---|
| `text_key`可编译但没有Runtime resolver | compiler伪造空String，tree保留table，render/a11y只读scalar string |
| 无项目Localization资产、culture authority与shipping cook | `ResourceKind`/`UiResourceKind`无catalog类型，App/project/ABI/script全无相关合同 |
| Locale Preview不是真实预览 | 3个硬编码选项；preview compile签名无locale/catalog；切换只刷新报告 |
| 无Gather/Import/Export/Compile闭环 | Runtime只有dependency/key存在性报告，artifact没有compiled translation value |
| Editor shell、project content与Runtime text locale语义分裂 | Editor近似parser、Runtime lowercase shaping tag、UI exact raw locale lookup各自独立 |

任何实现不得在Runtime83再次登记这5项P0，也不得通过读取`fallback`或把Editor 54-key bundle复制到Runtime来宣称关闭。

## 5. 当前源码值路径

### 5.1 Authoring reference到compiler

`collect_document_localization_report()`能识别`{ text_key, table?, fallback?, direction? }`，但reference只有key/table/fallback；空key是唯一结构验证，未知direction静默变成`Auto`。literal extraction只认路径尾部`.text/.label/.title`，没有覆盖tooltip、placeholder、options、a11y label/description或独立语义节点字段。

`component_prop_value()`遇到String schema和localized table时返回空`UiValue::String`作为类型检查替身。测试明确要求compiled attributes继续保存原table，却没有测试要求surface实际显示指定翻译。这一设计把“schema通过”与“可渲染值”分裂为两个互不验证的事实。

### 5.2 Package、cache与dependency

package validation把localization report写入`UiCompiledAssetPackageValidationReport`，dependency manifest也保存`localization_dependencies`。`UiRuntimeCompiledAssetArtifact`则只有`report + UiTemplateInstance`；surface builder只消费instance，仓库中`localization_dependencies`的production使用点只有manifest builder，其他命中均是测试或interface定义。

persistent cache key包含root/import/resource revision，但没有外部catalog generation；translation table改变不会因dependency identity自动驱逐已编译/已构建surface。artifact report保存依赖不等于runtime loader会装载依赖，也没有required culture closure、source hash或catalog schema兼容检查。

### 5.3 Catalog、lookup与reload

`UiLocalizationTableCatalog`只保存key集合，`validate_localization_report_against_catalog()`严格使用输入locale和table查找。空locale直接返回无诊断；表注册直接替换；TOML helper只收集scalar leaf path，数组被忽略，translation value本身不验证也不保留。

UI hot reload classifier只区分Template/Theme/Icon/Font/Texture/Other；没有LocalizationCatalog/StringTable类型、owner、generation或locale-specific damage。Other虽然会触发粗粒度render damage，却不会更新任何translation registry，也不能保证旧generation reader安全退出。

### 5.4 Render、accessibility与product

tree builder把raw attributes复制到`UiTemplateNodeMetadata`。`resolve_text()`、placeholder、option和editable value路径最终依赖`Value::as_str()`或scalar conversion，localized table被忽略；a11y name只扫描`text/label/value` scalar，tooltip/alt也只接受scalar。因此display与screen reader会同时失去翻译，而不是只存在视觉问题。

296个产品ZUI/TOML中没有一个`text_key`；两个出现“Locale/Localization”的ZUI仍是literal UI。WOC原生模型的32处key只在模型文件出现，仓库没有对应translator调用。没有Runtime/App/project/dynamic ABI/script API命名为`LocalizationService`、`translate_for_locale`、`resolve_localized`、supported/native/fallback culture或translation catalog。

### 5.5 Editor shell与UI Asset Preview

Editor shell bundle只有plain key/value，missing key退回raw key；没有namespace/source/context、plural/select、typed argument、pattern signature或catalog generation。`EditorLocale::parse()`是近似2-3字符language加2-8字符qualifier规则，不是完整BCP 47 canonicalization。

Locale changed/resync producer和settings subscriber已经接入message bus，但本轮除测试外没有发现product subscriber消费`EditorTopic::i18n()`并重建retained host。UI Asset session的catalog只保存key，唯一注册方法没有production caller；locale selector传不到`compile_preview()`，所以“Preview”只是依赖报告筛选器。

## 6. 参考引擎差异

| 工程轴 | Unreal | Godot | Zircon当前 |
|---|---|---|---|
| 文本身份 | FText history、namespace/key/source、string table引用 | source message + context | key/table/fallback，无namespace/source/context/history |
| Culture authority | language、locale、asset-group culture分离并广播 | singleton locale、domain override、fallback | shaping tag、Editor locale、UI lookup字符串三套语义 |
| Fallback | prioritized parent culture与source priority | locale score + explicit fallback | exact locale/table lookup |
| Resource | versioned locres、priority/conflict、target mount/unmount | Translation Resource、project load、domain add/remove | key-set内存表，无ResourceKind/compiled value artifact |
| Lookup generation | global/local text revision与change event | translation changed notification、resource remap reload | 无catalog generation；UI surface不绑定locale |
| Formatting | compiled pattern、named/ordered typed args、plural/gender、number/currency/date/time | plural/context、number digit formatting | 无message formatting；fallback string也不显示 |
| Pseudo/RTL | culture/text系统协同 | accent/double-vowel/fake-bidi/padding/placeholder policy | direction仅报告字段，unknown静默Auto，无pseudo |
| Runtime product | async resource refresh、mounted target、live display table | script-visible server/domain API | App/ABI/script均无Localization API |
| Size/performance | compact locres、shared display strings、revision cache | optimized perfect-hash-like table + compressed strings | BTreeMap key existence检查，未承载translation value |

Unreal和Godot并非所有设计都应照搬；例如Godot optimized translation明确不支持context/plural，不能作为Zircon最终artifact上限。Zircon要优于Unreal，必须先以相同catalog规模、fallback深度、format workload和switch频率建立可复现实测，不能从Rust、BTreeMap或“更简单”推导性能优势。

## 7. 目标架构与Owner边界

### 7.1 Runtime中立合同

```text
CultureTag + CultureFallbackGraph
    -> LocalizationDomainId + LocalizedTextIdentity(namespace/key/source/context)
    -> LocalizedTextPattern + LocalizedTextArgumentSignature
    -> LocalizationCatalogArtifact(domain/culture/schema/source-hash/entries)
    -> LocalizationCatalogGeneration + LocalizationSnapshot
    -> RuntimeLocalizationService::resolve(identity, args, snapshot)
    -> LocalizationOutcome(value/language/direction/provenance/status)
    -> UI Render + Accessibility + Script + Product
```

`zircon_runtime_interface`只拥有稳定DTO、error/status和generation identity；`zircon_runtime`拥有compiled catalog loader、registry、fallback、format、cache、snapshot lease与diagnostics；App拥有启动culture/profile/platform override；Editor33拥有gather/import/export/compile/cook authoring；UI只消费resolved outcome；Runtime80/81继续拥有font/shaping/layout；plugin/DLC通过owner lease提供domain，不能直接改全局map。

### 7.2 必需类型

| 类型 | 最小职责 |
|---|---|
| `CultureTag` | 完整解析、canonical form、alias/likely-subtag policy、稳定serialization |
| `CultureFallbackGraph` | requested/native/default/parent链、循环拒绝、bounded depth |
| `LocalizedTextIdentity` | domain、namespace、key、source hash、optional context，值与身份分离 |
| `LocalizedTextPattern` | 已验证AST、escaping、plural/select、argument signature与schema version |
| `LocalizationCatalogArtifact` | culture/domain/owner、entry索引、source provenance、checksum与compatibility |
| `LocalizationCatalogGeneration` | 原子发布代际、旧reader lease、load/unload receipt |
| `LocalizationSnapshot` | 一次复合投影使用同一culture/catalog generation |
| `LocalizationOutcome` | resolved text、resolved culture、fallback path、language/direction、missing/stale状态 |
| `RuntimeLocalizationService` | load、publish、resolve、format、invalidate、diagnostics、budget与shutdown |

## 8. P1工程化差距

### 8.1 Culture、身份与作用域

| ID | 当前差距 | 工程级要求 |
|---|---|---|
| RLI-P1-001 | 无Runtime content culture authority | App启动后发布requested/resolved culture snapshot，Runtime服务为唯一lookup authority |
| RLI-P1-002 | language、locale、font culture混为字符串 | 明确language/format locale/asset group culture职责与转换 |
| RLI-P1-003 | 三套近似locale normalization | 单一完整`CultureTag` parser/canonicalizer，invalid与unsupported分开诊断 |
| RLI-P1-004 | 无parent/native/default fallback graph | 有向无环、深度有界、可解释的fallback计划 |
| RLI-P1-005 | 无project/profile/CLI/platform/session优先级 | 冻结override precedence与receipt，不让任意子系统改全局locale |
| RLI-P1-006 | direction只是authoring hint | resolved outcome携带语言、script与direction，override冲突可诊断 |
| RLI-P1-007 | culture/catalog切换无共同generation | snapshot同时绑定culture与每个domain catalog generation |
| RLI-P1-008 | 无game/plugin/DLC/editor domain ownership | domain注册使用owner lease、priority、version和revoke协议 |

### 8.2 LocalizedText与catalog合同

| ID | 当前差距 | 工程级要求 |
|---|---|---|
| RLI-P1-009 | ref只有key/table/fallback | 加domain/namespace/source/context和稳定identity equality |
| RLI-P1-010 | 无source hash与stale translation判定 | catalog记录source hash，source改变时按policy拒绝或标记stale |
| RLI-P1-011 | catalog只保存key，不保存value | compiled entry持有validated pattern/value及provenance |
| RLI-P1-012 | 同locale/table注册静默覆盖 | owner-aware merge/conflict、lease/unregister和typed receipt |
| RLI-P1-013 | TOML loader丢弃value并忽略数组 | source importer与compiled reader分离，所有schema shape显式验证 |
| RLI-P1-014 | `ResourceKind`没有catalog/string table | 增加typed source/compiled resource及loader/importer边界 |
| RLI-P1-015 | 无catalog schema/version/checksum | artifact magic、version、entry/index checksum、compatibility和size cap |
| RLI-P1-016 | 无argument signature | pattern编译产出typed parameter schema，跨culture保持完全一致 |

### 8.3 Artifact、load与lifecycle

| ID | 当前差距 | 工程级要求 |
|---|---|---|
| RLI-P1-017 | localization dependency仅在report sidecar | runtime artifact引用compiled catalog dependency与required culture closure |
| RLI-P1-018 | `UiCompiledDocument`不保留resolver binding | 编译产出stable localized slot/identity，不把TOML table留给renderer猜 |
| RLI-P1-019 | package/cache无external catalog generation | dependency revision进入invalidation与package manifest |
| RLI-P1-020 | 无runtime product loader consumer | service按project/package/domain加载并验证catalog后才发布 |
| RLI-P1-021 | hot reload无Localization kind | watch -> prepare -> validate -> atomic publish -> targeted invalidation |
| RLI-P1-022 | 无last-good generation | corrupt/partial reload继续服务旧generation并给出失败receipt |
| RLI-P1-023 | 无plugin/DLC unload reader fence | revoke新lookup、等待snapshot lease、再释放mapping和blob |
| RLI-P1-024 | 无内存、entry、pattern、cache预算 | 每domain/culture预算、LRU/eviction policy、bounded diagnostics和admission |

### 8.4 Compiler、UI、render与accessibility

| ID | 当前差距 | 工程级要求 |
|---|---|---|
| RLI-P1-025 | compiler用空String绕过类型检查 | schema接受typed LocalizedText值并产出validated slot，失败fail-closed |
| RLI-P1-026 | tree复制raw localized table | metadata保存identity/slot handle，不保存运行时未解释TOML |
| RLI-P1-027 | renderer只读`Value::as_str()` | build/extract前以snapshot解析，render command只接收qualified resolved text |
| RLI-P1-028 | `fallback`只影响诊断severity | source/fallback/missing policy进入`LocalizationOutcome`并记录provenance |
| RLI-P1-029 | extraction仅覆盖text/label/title | component schema声明localizable fields，覆盖tooltip/placeholder/options/rich/a11y |
| RLI-P1-030 | unknown direction静默Auto | invalid enum有path/code；Auto由resolved culture/script决定 |
| RLI-P1-031 | accessibility只读scalar text | semantic name/description/value与视觉文本共享同一snapshot/outcome |
| RLI-P1-032 | locale切换无UI invalidation合同 | 按localized slot索引定向触发text/layout/hit/render/a11y/font generation重建 |

### 8.5 Message format、国际化与API

| ID | 当前差距 | 工程级要求 |
|---|---|---|
| RLI-P1-033 | 无plural/select/gender | 使用成熟CLDR/ICU级规则或经验证library，pattern AST有bounded evaluation |
| RLI-P1-034 | 无typed arguments | bool/int/float/string/text/date/time/enum等参数类型与required/optional policy |
| RLI-P1-035 | 无number/percent/currency/unit/list格式化 | deterministic locale data、rounding/grouping/numbering system与typed options |
| RLI-P1-036 | 无date/time/timezone/duration格式化 | 时区数据库版本、calendar/hour-cycle和invalid timezone语义 |
| RLI-P1-037 | 无escaping/rich text安全边界 | pattern literal、argument、markup分别转义，禁止translation注入action/resource |
| RLI-P1-038 | 无runtime pseudo locale | accent、expansion、fake BiDi、placeholder保护和release exclusion |
| RLI-P1-039 | 无locale data版本与determinism | artifact记录数据版本；server/replay与client display职责分离 |
| RLI-P1-040 | 无Script/plugin/dynamic ABI API | versioned resolve/format/snapshot合同，typed unavailable/stale/unsupported结果 |

### 8.6 App、Editor、产品与资格

| ID | 当前差距 | 工程级要求 |
|---|---|---|
| RLI-P1-041 | App无culture bootstrap/profile/CLI | 启动前解析override与supported cultures，发布可观测bootstrap receipt |
| RLI-P1-042 | Editor 54-key service与Runtime孤立 | 共享底层Culture/Pattern/Catalog合同，保持shell和project domain隔离 |
| RLI-P1-043 | bus有producer但无product rebuild consumer | host持有subscriber，locale resync后按generation重建全部locale-sensitive projection |
| RLI-P1-044 | Locale Preview选项硬编码 | 从project target/cooked catalog snapshot生成locale列表和availability状态 |
| RLI-P1-045 | preview compile不接locale/catalog | preview host使用与Runtime相同snapshot/resolver并显示fallback/provenance |
| RLI-P1-046 | 产品ZUI零`text_key`，WOC key模型无consumer | 迁移真实产品flow并以translated framebuffer/a11y/script证据验收 |
| RLI-P1-047 | diagnostics无统一code/provenance/redaction | structured code、identity hash、culture/path/generation、去重/限流且不泄露参数 |
| RLI-P1-048 | 无并发、fault、scale与同负载性能资格 | cold/warm lookup、switch、load/unload、fallback、format和memory全套基准及故障矩阵 |

## 9. P2扩展差距

| ID | 扩展能力 | 前置条件 |
|---|---|---|
| RLI-P2-001 | locale-specific asset remap | culture snapshot、asset dependency与atomic remap generation |
| RLI-P2-002 | dialogue/subtitle/caption translation domain | timed text identity、speaker/context与audio sync |
| RLI-P2-003 | localized voice/audio pack与lip-sync variant | chunk/owner lease、fallback和streaming budget |
| RLI-P2-004 | grammatical inflection与morphology | typed entity features、locale rule provider和authoring validation |
| RLI-P2-005 | locale-aware collation/search/sort | deterministic collation data与index generation |
| RLI-P2-006 | locale-aware editable number/date parsing | Runtime82 document transaction与typed intermediate-invalid state |
| RLI-P2-007 | live localization session与remote preview | authenticated source、generation isolation、disconnect recovery |
| RLI-P2-008 | mod/UGC translation overlay | signed package、domain priority、sandbox和revoke |
| RLI-P2-009 | per-user/network locale policy | gameplay determinism与display-only localization边界 |
| RLI-P2-010 | TTS/speech locale和voice selection | accessibility authority、privacy与platform capability |
| RLI-P2-011 | per-culture screenshot/a11y golden orchestration | stable font/render environment和coverage manifest |
| RLI-P2-012 | signed live-ops translation hotfix | trust、rollback、expiry、audit和offline last-good |

## 10. 分层重构路线

### M0：Truthfulness与断路止血

新增失败测试证明localized ref当前不会显示translation/fallback；UI Asset的Locale Preview在真实resolver接入前标为Report；冻结Editor33五项P0唯一owner与Runtime83 48项P1映射。禁止继续给key-set catalog增加产品宣称。

### M1：Culture与LocalizedText公共合同

在Runtime Interface落地`CultureTag`、fallback graph、domain/identity、argument signature、outcome和generation；统一Editor/Runtime/UI的locale parser，先迁移调用点再删除旧helper。

### M2：Compiled catalog artifact

建立String Table/Translation Archive source与compiled catalog的硬边界；artifact包含schema、culture/domain/owner、source hash、pattern AST、index、checksum和budgets。Editor33负责source workflow，Runtime只读compiled artifact。

### M3：RuntimeLocalizationService

实现prepare/validate/atomic publish、snapshot lease、fallback、cache、diagnostics、last-good与shutdown；先单project/domain，再扩展plugin/DLC owner lease。无全局裸map或renderer文件IO。

### M4：Message format与internationalization

接入经验证的plural/select、typed args、number/date/time/currency/unit/list格式化与deterministic locale data；建立pattern fuzz、argument mismatch、escaping和work budget。

### M5：UI、A11y与hot reload

compiler产出localized slot，surface绑定snapshot generation；resolve发生在render command之前，visual/a11y共享outcome；locale/catalog变化按slot index定向触发text/layout/font/a11y重建。

### M6：App、Script、Plugin与Editor bridge

App启动culture authority、dynamic ABI/script API、plugin domain lease、Editor shell shared substrate和真实UI Asset Preview接入。message-bus resync必须有product subscriber和rebuild receipt。

### M7：产品迁移

先迁移一个Editor flow和一个WOC native flow，再扩大到272个ZUI；每批保留source key coverage、fallback、RTL/pseudo、glyph coverage、framebuffer和screen-reader证据，不做一次性字符串替换。

### M8：Fault、scale与性能资格

覆盖corrupt/too-new/missing catalog、disk/IO failure、reload race、plugin unload、culture storm、cache pressure、malicious pattern与shutdown；以同一机器、同一catalog和同一workload对照Unreal/Godot。达到预算前不得声称性能优于Unreal。

## 11. 48项资格门

| Gate | 验收条件 |
|---|---|
| G01 | 所有locale入口使用同一`CultureTag` canonicalizer，旧helper无production caller |
| G02 | invalid、unsupported、unavailable culture有不同typed status |
| G03 | fallback graph拒绝循环、重复和超深链，路径可导出 |
| G04 | language、format locale与asset-group culture职责有contract test |
| G05 | project/profile/CLI/platform/session override precedence可复现 |
| G06 | culture switch发布单一generation receipt且复合projection不混代 |
| G07 | domain注册必须有owner/version/priority/lease |
| G08 | plugin/DLC revoke后新lookup失败，旧snapshot安全结束 |
| G09 | identity包含domain/namespace/key/source/context并稳定序列化 |
| G10 | source改变能标记stale translation且不会静默沿用 |
| G11 | 各culture的argument signature完全一致 |
| G12 | duplicate/conflicting entry在compile或load阶段fail-closed |
| G13 | compiled catalog有magic/schema/checksum/size cap |
| G14 | too-new、truncated、checksum错误返回typed error并保留last-good |
| G15 | catalog index与payload有明确bytes/entry/pattern预算 |
| G16 | source TOML/PO/CSV不能直接成为shipping runtime authority |
| G17 | UI package manifest携带catalog dependency和required culture closure |
| G18 | external catalog revision进入compile/cache invalidation |
| G19 | runtime artifact loader实际消费localization dependency |
| G20 | load/reload只有prepare成功后才原子发布新generation |
| G21 | hot reload按catalog/domain/culture定向失效，不把未知文件当完成 |
| G22 | unload等待reader lease且无dangling resolved string |
| G23 | cold load、warm load与failure receipt均含bytes/time/generation |
| G24 | shutdown后无后台load、watch、lease或callback残留 |
| G25 | compiler不再伪造空String绕过localized prop schema |
| G26 | compiled template不再把raw localized TOML交给renderer解释 |
| G27 | translated value与fallback都能进入真实paint command |
| G28 | visual text与accessibility name使用同一snapshot/outcome |
| G29 | tooltip/placeholder/options/rich/a11y字段由component schema声明并收集 |
| G30 | invalid direction fail-closed，Auto由culture/script解析 |
| G31 | locale change触发text/layout/hit/render/a11y/font所需dirty域 |
| G32 | stale surface/render artifact不能混用新catalog generation |
| G33 | plural/select覆盖目标culture的所有CLDR类别 |
| G34 | missing/wrong/extra argument在compile或resolve边界可诊断 |
| G35 | number/percent/currency/unit/list golden固定locale data版本 |
| G36 | date/time/timezone/duration golden覆盖DST与invalid timezone |
| G37 | translation和arguments不能注入rich action/resource或破坏markup |
| G38 | pattern parser/evaluator通过fuzz、depth、output bytes和deadline预算 |
| G39 | pseudo覆盖accent/expansion/fake BiDi/placeholder并禁止release泄漏 |
| G40 | Script/dynamic ABI保留identity/args/generation/status且版本化 |
| G41 | App从supported cultures选择启动culture并发布bootstrap receipt |
| G42 | Editor shell与project domain共享底层合同但加载/owner隔离 |
| G43 | `EditorTopic::i18n()`有product subscriber和full-resync重建证据 |
| G44 | UI Asset Preview locale列表来自真实catalog，不再硬编码三项 |
| G45 | Preview与shipping Runtime使用同一resolver和compiled artifact |
| G46 | 至少一个Editor和一个WOC flow有translated framebuffer+a11y+script证据 |
| G47 | diagnostics有界去重、参数脱敏并带culture/domain/generation/provenance |
| G48 | 同负载benchmark覆盖p50/p95/p99、alloc、RSS、switch/load；优于Unreal声明附原始证据 |

## 12. 禁止的临时实现

1. 禁止让renderer直接读取`fallback`或TOML文件并称为LocalizationService。
2. 禁止继续用空String让localized table通过schema。
3. 禁止把Editor 54-key内嵌bundle直接改名为game/project catalog。
4. 禁止用process-global`RwLock<BTreeMap<String, String>>`替代owner、generation和snapshot。
5. 禁止用locale字符串逐段截断冒充完整fallback/canonicalization。
6. 禁止手写plural、date/time、currency或CLDR表来追求短期完成。
7. 禁止只更新视觉文字而不更新accessibility、layout、font与hit-test。
8. 禁止用hardcoded `en-US/zh-CN`菜单、静态coverage或测试fixture冒充产品支持文化。
9. 禁止在translation热路径执行文件IO、TOML解析、provider回调或无界format。
10. 禁止在没有同负载benchmark和原始数据时声称性能优于Unreal。

## 13. 本轮完成边界

本轮完成357个Zircon入选文件、24个参考文件的静态current-source审查，复核了reference -> compiler -> package/cache -> tree/render/a11y -> Editor preview -> product的完整值路径，登记0项新增P0、48项P1、12项P2、M0-M8路线和48项资格门。没有修改production/test源码，没有新增catalog、resolver、culture service、format engine或产品翻译，也没有运行动态验证；tooling按用户要求暂不纳入本专题。下一实施入口必须是M0 truthfulness测试与M1公共合同，不得从renderer fallback补丁开始。
