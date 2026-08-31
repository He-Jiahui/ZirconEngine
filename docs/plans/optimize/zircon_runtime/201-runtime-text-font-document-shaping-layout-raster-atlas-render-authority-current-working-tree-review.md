---
title: Runtime Text、Font、Document、Shaping、Layout、Raster、Atlas 与 Render Authority 当前工作树复审
category: zircon_runtime
report_id: Runtime201
review_date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/84-runtime-rich-text-markup-parser-token-style-span-inline-object-link-image-table-list-layout-selection-accessibility-security-product-integration-current-source-review.md
canonical_parent_owners:
  - docs/plans/optimize/zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/84-runtime-rich-text-markup-parser-token-style-span-inline-object-link-image-table-list-layout-selection-accessibility-security-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/196-runtime-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-product-integration-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
related_code:
  - zircon_runtime/src/text
  - zircon_runtime/src/ui/text
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/ui/surface/input/editable_text
  - zircon_runtime/src/ui/dispatch/input_manager/text_document_session
  - zircon_runtime/src/graphics/scene/scene_renderer/ui
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/font_source
  - zircon_runtime/src/asset/importer/ingest/import_font_asset
  - zircon_runtime_interface/src/ui/text
  - zircon_runtime_interface/src/ui/surface/render
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/godot/servers/text/text_server.h
  - dev/godot/modules/text_server_adv/text_server_adv.h
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/parley_context.rs
  - dev/bevy/crates/bevy_text/src/font_atlas_set.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/Texture2DAtlas.cs
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime201 · Text / Font / Document / Shaping / Layout / Raster / Atlas / Render Authority 当前工作树差距

## 1. 结论

当前文本栈不是临时 demo，也不应推倒重写。它已经有 per-Core `FontCollectionService`、不可变字体数据库快照和 generation fence、项目字体 cooked blob/claim 生命周期、Unicode 数据快照、脚本/双向/Joining/Emoji/行断分析、piece-table 文档事务、带容量和字节预算的 shaped/rich cache、富文本信任与解析预算、异步 raster/SDF worker、位图图集 shadow commit/recovery，以及保留式 UI 文本 frame product。这些是应继续保留并提升为统一产品 authority 的真实基础。

但产品入口仍是分裂的。`TextModule` 只注册字体服务；Dynamic Runtime 和标准 Graphics module host 会显式注入该 Core 的字体集合，这是重要进展。然而 `shared_text_layout_service()`、`SharedTextLayoutSession::new()`、`UiSurface::new`、部分 UiV2 builder、单次 layout/measure/viewport/rich API 和兼容 renderer constructor 仍可静默绑定进程全局字体集合。一个进程内的多 Runtime、Editor preview、游戏会话和离线操作可以各自看见不同字体 generation，却使用同名 API 和无 owner 的全局 metrics。当前还没有统一的 `TextRuntimeContext` 把字体、Unicode、解析器、缓存、worker、预算、诊断和关闭状态绑定到 Core/session/surface/frame。

最明确的产品功能断层在 typography contract。底层 `TextStyle`、字体实例、HarfBuzz/Swash 路径已经能表达 italic、OpenType feature、variation coordinate 和 stretch；富文本 `StyleOverride` 也能解析 italic、letter spacing 和 feature。但是 `UiResolvedStyle` 没有 italic/slant、stretch、variation/optical sizing、letter/word spacing、OpenType feature 或 fallback family list，`text_style()` 直接写死 `italic: false` 和空 features，renderer fallback 又写死 `stretch: 100`、`italic: false`。这不是“后续样式增强”，而是 authoring -> cascade -> layout -> cache -> glyph artifact -> raster identity 的合同不完整。

本轮没有发现新的唯一 P0；Runtime80/81/82/84、Runtime196 和 Runtime200 的 canonical owner 继续追踪其原问题。本报告登记 40 项当前 P1、12 项 P2 和 30 个工程资格门：

| 等级 | Open/Fail | Partial | Closed/Pass | 合计 |
|---|---:|---:|---:|---:|
| P0（继承 owner，不重复计数） | 0 | 0 | 0 | 0 |
| P1 | 32 | 8 | 0 | 40 |
| P2 | 10 | 2 | 0 | 12 |
| Gate | 20 | 10 | 0 | 30 |

目标结构应收敛为：

```text
TextRuntimeContext (Core/session owned)
  -> FontCollectionAuthority (policy + immutable revision)
  -> UnicodeDataAuthority (versioned provider snapshot)
  -> TextDocumentRegistry (revision + leases + compaction/history)
  -> TextLayoutPipeline (parse -> shape -> break -> layout -> artifact)
  -> TextWorkScheduler (deadline + cancel + priority + receipt)
  -> TextCacheResidency (profile/memory-pressure aware)
  -> GlyphRenderPipeline (instance -> raster/SDF -> atlas -> upload/present)
  -> TextHealthSnapshot (owner/session/surface/frame correlation)
```

进程全局入口只能保留为测试、工具或显式 compatibility adapter；Runtime 产品路径不得从 convenience constructor 隐式取得 global collection。

## 2. 扫描范围与冻结指标

### 2.1 当前工作树选择集

本轮按当前磁盘内容逐文件复核以下闭包，包含已有测试文件和未提交修改；Tooling 按用户要求排除：

- `zircon_runtime/src/text/**`：module/service、font/database/claim、document、Unicode、shaping、layout、rich、cache、raster、SDF、bitmap atlas、glyph artifact、render state 和并行 worker。
- `zircon_runtime/src/ui/text/**`、`ui/surface/render/**`、editable text、text document session、bound model update、inline widget、surface/builder 入口。
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/**` 及 font collection 注入所需的 renderer/Core construction 文件。
- `zircon_runtime/src/asset` 的 font/font source/artifact/importer/test 闭包。
- `zircon_runtime_interface/src/ui/text/**` 与 `ui/surface/render/**` 的文档、样式、布局、shaped glyph 和 frame transport DTO。
- Dynamic Runtime construction 的 per-Core 字体服务接入点。

指标按规范化 lowercase 相对路径排序；fingerprint 使用 `path + NUL + raw bytes + NUL` 的 SHA-256：

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Runtime text 核心 | 381 | 96,110 | 87,760 | 3,354,926 | 1,262 | 0 | `24ab9a76bc1fab8275abc53735680186e536cabf9e4baec4017265c8a31a30bd` |
| Runtime UI text/edit/render closure | 183 | 47,603 | 44,243 | 1,634,731 | 397 | 0 | `8ee703c4be03bea5912e97784cd95893af9052ed3d9cf41e95417b6aa6d3e204` |
| Scene UI text renderer/Core wiring | 124 | 34,941 | 32,511 | 1,271,767 | 412 | 0 | `e4176491388a816aed60cf8a255e02311b9a0029a972ad02e3e05200f72ed239` |
| Font asset/source/import closure | 10 | 3,241 | 2,923 | 109,563 | 32 | 0 | `1e004fdfd155f97d5e4ecacea1d6741b80307791101aadaf7a125ff5d53fef02` |
| Runtime Interface text/render DTO | 38 | 7,998 | 7,296 | 260,330 | 53 | 0 | `f4f8a1aab0c8a593f82a98d2d9e3f627382af17603d3183a6d64be8ccb38edd6` |
| Dynamic Core integration | 1 | 445 | 424 | 18,287 | 4 | 0 | `6c5bb82d6dd9308012a0fa7c80426209ccb8f5c30fafddafb1442f966d692dbd` |
| 去重总选择集 | **737** | **190,338** | **175,157** | **6,649,604** | **2,160** | **0** | `3c97437fc594e20c1609b4d6d4d29c0afad1db3a9c809fd40e51f59edb0893b5` |

冻结时该闭包有 573 条 git status 记录，因此旧 Runtime80/81/82/84 只能作为 canonical owner 和历史基线，不能直接当作当前源码结论。本报告以当前工作树为准。

### 2.2 验证限制

本轮是 review-only：没有运行 Cargo、真实 Runtime/Editor、真实 GPU、系统字体枚举、IME、screen reader、不同 DPI/locale、device loss、fault injection、fuzz、scale、long-soak、视觉 golden 或 benchmark。因此 2,160 个测试声明只证明源码中存在测试意图，不能据此宣称动态正确性、性能或表现优于 Unreal。

### 2.3 与历史 owner 的边界

| 主题 | 本报告处理 | canonical owner |
|---|---|---|
| 字体 asset/source/cook/face/fallback/variation/color | 记录当前接入、policy 和 authority 断点 | Runtime80 |
| Unicode/Bidi/script/cluster/line break/layout | 刷新已修复项，登记剩余 provider/normalization/scheduling 断点 | Runtime81 |
| document/selection/caret/IME/clipboard/secure text | 只复核 document store、snapshot 和 text pipeline 边界 | Runtime82、Runtime200 |
| rich markup/style/inline object/security/a11y | 复核 parser/cache/render contract，不重复完整语法矩阵 | Runtime84 |
| 统一内存域、pressure、OOM、cache residency | 文本侧只登记未接入点 | Runtime196 |

## 3. 当前真实基础与 owner 拓扑

### 3.1 应保留的真实基础

1. `TextModule` 创建独立 `FontCollectionService`；`font_collection_service_for_core` 可解析每个 Core 的服务，且测试覆盖不同 Core 隔离。
2. Dynamic session construction 与 builtin Graphics module host 会把同一 collection 注入 UI extract cache、Wgpu framework、SceneRenderer 和 `ScreenSpaceUiTextSystem`。
3. `FontCollectionSnapshot` 以 `Arc<FontDatabase>` 固定 collection id/generation/database；变更发布新不可变快照，shape 在 generation 变化时有限重试并返回 typed deferred failure。
4. `res://` 项目字体通过 ProjectAssetManager/cooked blob 进入数据库；runtime font asset claim 在最后一个 owner 释放时撤销并推进 generation。
5. Unicode snapshot schema 已记录 locale、normalization、bidi/mirroring、script、grapheme、word、line break、emoji、general category、joining 和 vertical orientation 的 provider/data version/fingerprint。
6. 当前 script run 已使用 Script_Extensions、paired-bracket context 和 language preference；Emoji、Joining、BCP 47 和 line break 也已改用 Unicode/ICU provider，不应继续沿用 Runtime81 早期“手写范围”结论。
7. `TextDocumentStore` 有显式无 `Default` 的容量/字节/lease limits、revision 校验、prepare/commit receipt、piece-table storage 和 snapshot lease accounting，并已接入 Runtime UI text session。
8. shaped cache key 包含原文 hash/range、字体、weight/italic/size、direction/orientation、kerning、features、language、font collection/generation 和 Unicode snapshot；还会比较原文以拒绝 hash collision。
9. rich parser 有 trust、source/output/token/attribute/depth/run/table/diagnostic/bidi budget、structured diagnostics、decorator panic containment 和 bounded compiled cache。
10. raster worker、SDF scheduler、native bitmap atlas 均有请求/完成队列与字节预算、duplicate/in-flight 状态、cancel、generation/face epoch、bounded drain 和诊断；atlas upload 有 pending commit/recovery。
11. rich inline image/icon/widget 不是占位符；widget 由独立 UI child node 拥有，文本 renderer 不重复绘制它。

### 3.2 当前分裂的数据流

```text
Core Runtime path
  TextModule -> FontCollectionService(core)
       -> Dynamic UI extract session
       -> WgpuRenderFramework -> SceneRenderer -> ScreenSpaceUiTextSystem

Compatibility/public path
  shared_text_layout_service()
       -> shared_font_collection_service() [process global]
  SharedTextLayoutSession::new()
       -> process global collection + session-local parser/caches
  UiSurface/UiV2/one-shot layout/measure/rich constructors
       -> compatibility session/global collection

UI text contract
  authored style map
       -> UiResolvedStyle [typography fields incomplete]
       -> TextStyle [italic=false, features=[]]
       -> layout/glyph artifact
       -> UiTextPaint/UiShapedText [lineage/provenance reduced]
       -> renderer fallback [stretch=100, italic=false]
```

这两条路径可以在一个进程内同时存在，且 DTO/metrics 没有 runtime/session identity；因此“主要 Runtime 路径已经注入”不能推出“文本 authority 已经统一”。

## 4. P1：工程级阻断与重构要求

### 4.1 Runtime authority、字体策略与生命周期

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| RT-TXT-P1-001 | Partial | `TextModule` 只注册 `TextRuntimeServices { font_collection }`；Core 注入真实存在，但 parser/cache/worker/budget/Unicode/health 不属于该服务。 | 建立 Core/session-owned `TextRuntimeContext`，把 collection、Unicode、layout session factory、worker、residency、diagnostics 和 shutdown 统一为一个 capability。 |
| RT-TXT-P1-002 | Open | `SharedTextLayoutService::shape` 总是调用 `shared_font_collection_service()`；另有 global metrics/OnceLock。 | 产品接口必须要求 context/handle；global service 降为明确的 standalone/test adapter，并在产品构建中 lint 禁用。 |
| RT-TXT-P1-003 | Open | `SharedTextLayoutSession::new()`、多处 one-shot layout/measure/viewport/rich、`UiSurface::new`/UiV2 builder 和兼容 renderer constructor 可隐式绑定 global collection。 | 所有 retained/product constructor 注入 `TextContextHandle`；命名区分 `new_in_runtime` 与显式 `standalone_for_tool`，禁止静默 fallback。 |
| RT-TXT-P1-004 | Open | shaping/cache/atlas reports 和 profile counters 没有 runtime/session/surface/document/frame owner。 | 统一 `TextRequestIdentity` 与 `TextHealthSnapshot`，携带 runtime、session、surface、document revision、frame token、trace/correlation id。 |
| RT-TXT-P1-005 | Open | 未见 Text context 的 freeze/drain/cancel/trim/close 状态机；服务销毁主要依赖字段 Drop。 | 定义 `Created -> Active -> Draining -> Closed/Faulted`，关闭时停止接单、取消 worker、drain terminal receipt、撤销 claim 并报告遗留 lease。 |
| RT-TXT-P1-006 | Open | `ScreenSpaceUiTextSystem::new_with_font_collection` 无条件执行 `SystemFontPolicy::Discover`。 | system font discovery 必须由 RuntimeProfile/ProjectPolicy/target capability 决定，不得由 renderer constructor 改变字体事实源。 |
| RT-TXT-P1-007 | Open | `SystemFontPolicy` 只有 Disabled/Discover；没有 packaged-only、allowlist、locale pack、shipping determinism、memory/startup budget 或 discovery receipt。 | 定义 versioned font policy artifact，记录平台、目录、face order、fallback tier、允许来源、预算和可重放 fingerprint。 |
| RT-TXT-P1-008 | Partial | publication 使用不可变 `Arc` 快照，但每次 mutation 在 write lock 内 clone 整个 `FontDatabase`；`mutate()` 还再次 owned clone。 | 将 catalog/source/face/cache 分层为结构共享快照或事务 builder；量化 mutation latency/peak memory，避免大字体库热更新全量复制。 |
| RT-TXT-P1-009 | Partial | `res://` 已走 cooked blob；legacy/direct 非项目路径仍可读 manifest/font source 文件系统。 | shipping Runtime 只允许 artifact/packaged/system-policy 来源；legacy path 进入显式 Tool/Dev capability，并记录 provenance 与 sandbox decision。 |
| RT-TXT-P1-010 | Open | 当前可注册字体和 fallback，但未见产品级 coverage/license/embedding/locale-pack/last-resort 验证制品与 admission receipt。 | cook 阶段生成 font coverage manifest、license/embedding policy、variation/color support、fallback closure 和 target-specific footprint；Runtime 只消费已验证制品。 |

### 4.2 Typography、Unicode、layout 与 shaping scheduling

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| RT-TXT-P1-011 | Open | `UiResolvedStyle` 无 italic/slant；`text_style()` 写死 `italic: false`。 | authoring/schema/cascade/resolved style/layout key/shape request/raster key 全链路加入 slant/italic，区分真实 face 与 synthetic oblique。 |
| RT-TXT-P1-012 | Open | UI contract 无 stretch、variation axes、optical sizing；renderer fallback 写死 `stretch: 100`，底层实例却支持 variation coordinates。 | 引入规范化 axis set、stretch、optical sizing policy 和 instance identity；缓存/atlas 必须按有效实例而非仅 face 分桶。 |
| RT-TXT-P1-013 | Open | UI style invalidation识别 `letter_spacing`，但 `UiResolvedStyle`/`TextStyle` 无该字段；word spacing 也不存在。 | 完成 schema -> cascade -> shaping/layout -> hit/caret -> artifact 的 spacing contract，并定义 complex-script 安全策略。 |
| RT-TXT-P1-014 | Open | UI resolved contract 无 OpenType feature list、generic/fallback family stack；底层 shape request/cache 已支持 features。 | 提供 typed/tag-validated feature set、ordered family list、generic family mapping和明确 fallback policy；全部进入 key 和 receipt。 |
| RT-TXT-P1-015 | Partial | rich `StyleOverride` 有 italic/letter_spacing/features；`resolve_rich_run_style` 应用 italic/features，却完全不消费 letter_spacing。 | 不得只“解析成功”；编译制品、layout 和 renderer 必须保留每个 override 的实际应用/拒绝 receipt，letter spacing 要么生效要么 typed reject。 |
| RT-TXT-P1-016 | Partial | canonical rich glyph artifact 能保留已 shaped 的 italic/features；但 `UiTextRunPaintStyle` 只剩 strong/emphasis/code，`RichTextRunPresentation` 不带 feature/spacing/instance，fallback 会降级。 | artifact 和 fallback 必须共享同一 run typography descriptor；stale/missing artifact 时不能静默改变 glyph choice 或 advance。 |
| RT-TXT-P1-017 | Open | `LineBreakTailoringProfile` 只有 `UnicodeDefault`；有 kinsoku、soft hyphen、Arabic/CJK justify 和 ellipsis，但未见 locale tailoring/dictionary hyphenation。 | 引入 locale/project line-break provider、hyphenation dictionary artifact、language fallback 和版本化 decision receipt。 |
| RT-TXT-P1-018 | Open | `ShapingTextView` 明确保持原 UTF-8，不做 NFC/NFD，因为没有双向 source map；canonically equivalent input 仍是不同文本/cache identity。 | 建立 versioned normalization/source map，覆盖 selection、IME、a11y、cache、glyph projection；在该合同完成前继续禁止无映射 normalization。 |
| RT-TXT-P1-019 | Open | Unicode snapshot 能描述 12 类 provider/version，但仅有 compiled constant，未见 profile/project provider negotiation、data package update 或 replay admission。 | 建立 `UnicodeDataAuthority`：provider set、schema compatibility、asset/update provenance、locale policy、snapshot lease 和 replay/save fingerprint。 |
| RT-TXT-P1-020 | Open | `TextShapingWorkBudget` 对超过 64 KiB 的请求只计数，注释明确仍同步处理。 | 提供不破坏语义边界的 deferred work item、future/ticket、priority、deadline、cancel 和 terminal receipt；UI 线程不得同步吞大段文本。 |
| RT-TXT-P1-021 | Partial | paragraph batch 可在 TaskPool 并行，但调用方同步 `parallel_for` join，并记录 caller wait nanos。 | 将批处理提升为调度服务；支持增量 publication、cooperative cancellation、deadline 和 frame budget，而非只把 CPU 分散后阻塞 caller。 |
| RT-TXT-P1-022 | Open | `TextLayoutService::shape` request 没有 owner/deadline/cancel/priority；stable generation retry 之外没有调度语义。 | 定义 `TextWorkRequest`/`TextWorkTicket`/`TextWorkReceipt`，明确 queued/running/deferred/cancelled/stale/failed/ready 终态。 |
| RT-TXT-P1-023 | Open | shaping failure/report 有类型和计数，但无法关联 surface/document/revision/frame/font/Unicode generation。 | 所有 failure/degrade/backpressure 进入有界、可游标读取的 per-context receipt store，产品 UI 可定位具体文本对象。 |

### 4.3 Cache、rich parser 与 document 长生命周期

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| RT-TXT-P1-024 | Open | `TextLayoutCache` 只有 2,048 entry capacity；虽然统计 estimated bytes，却没有 max bytes/admission bypass。 | 同时限制 entry/bytes，准确计入 value/index capacity；超大单项必须 bypass 或降级，接入 Runtime196 memory pressure。 |
| RT-TXT-P1-025 | Open | 多个 public/compatibility one-shot API 每次创建 `SharedTextLayoutSession`，继而创建 parser 和若干 cache。 | 将 one-shot API 路由到调用方 context 的 session pool/scratch lease；明确 retained 与 ephemeral residency，避免无意义 cache 冷启动。 |
| RT-TXT-P1-026 | Open | 每个 `SharedTextLayoutSession` 拥有自己的 `RichTextParser` 和 compiled cache；parser/decorator/emoji generation 没有 Runtime catalog owner。 | 建立 context-owned parser registry/cache partition，按 trust/plugin generation 隔离并支持 revoke、warmup、trim 和 health snapshot。 |
| RT-TXT-P1-027 | Open | compiled rich cache 用 `OnceLock::get_or_init` 做 single-flight；等待者无限阻塞，仅统计 wait nanos。 | single-flight cell 必须有 deadline/cancel/owner、panic/fault terminal state 和 waiter fan-out budget；不能让 UI caller无限等待另一个解析者。 |
| RT-TXT-P1-028 | Open | `RichParseBudget` 限制 bytes/tokens/depth/run/table/diagnostics，但 32 MiB 同步解析仍无 CPU time、yield、cancel。 | 增加 instruction/time slice、cooperative checkpoint、deadline/cancel，长文档以增量 artifact 发布，不把 byte budget 等同于 latency budget。 |
| RT-TXT-P1-029 | Open | decorator/emoji 注册属于可变 parser 实例，没有 module/plugin catalog、capability、generation lease、reload/revoke transaction。 | 使用 Runtime plugin catalog 发布 immutable parser extension snapshot；compile artifact pin generation，reload 后旧 artifact 可判 stale 并安全退役。 |
| RT-TXT-P1-030 | Open | shaped/rich/hard-line/fallback/instance/source/atlas/SDF budgets 大量写死为局部常量。 | 建立 `TextResidencyProfile` 和按产品/平台/quality tier 的预算快照；响应 memory pressure，输出 trim/degrade/admission receipt。 |
| RT-TXT-P1-031 | Open | document piece table持续追加 addition source/piece；未找到 compaction/rebase。 | 实现 snapshot-safe transactional compaction：保留 document identity/revision、活跃 lease 和 source map，发布 compaction generation/receipt。 |
| RT-TXT-P1-032 | Open | store 能计数 active snapshot lease，但 lease 没有独立 id/owner/deadline/revocation或泄漏诊断。 | `TextSnapshotLeaseId` 绑定 session/document/revision/consumer；关闭或超预算时可定位 owner，支持 terminal revoke/retire receipt。 |
| RT-TXT-P1-033 | Open | document store 没有持久 undo/redo/history/checkpoint artifact；长期编辑只能由上层另行拼装。 | 按 Runtime82 owner 建立 operation log/checkpoint/undo group 与 selection/composition snapshot，避免 UI session 私有状态成为唯一历史。 |

### 4.4 Transport、raster/SDF、atlas 与产品资格

| ID | 状态 | 当前证据 | 工程级差异 / 重构要求 |
|---|---|---|---|
| RT-TXT-P1-034 | Open | `UiTextShapeArtifact::Canonical(UiShapedText)` 没有 schema/version/provider identity；“Canonical”只是当前进程 DTO 名称。 | 分开 in-process artifact handle 与 versioned transport artifact；明确兼容版本、producer、font/Unicode provenance 和 validation contract。 |
| RT-TXT-P1-035 | Open | `UiTextPaint`/`UiShapedText` 没有 document revision、font collection/generation、Unicode snapshot、runtime/surface/frame token。 | publication 必须携带完整 lineage；renderer 对 stale/mismatched artifact typed reject，不用字符串/resource key 猜 generation。 |
| RT-TXT-P1-036 | Partial | 内部 `ShapedGlyphRun`/glyph artifact有 font instance、script/break/vertical receipt；公开 `UiShapedGlyph` 只保留 glyph/font resource、frame、advance、简化 flags/rotation/atlas。 | 定义最小但无损的 render artifact schema；若字段只在 sidecar，sidecar identity/lease必须进入 transport，跨进程路径需可重建或明确不支持。 |
| RT-TXT-P1-037 | Open | font generation shape retry固定最多两轮，deferred 后没有 owner 自动重排、退避或 deadline。 | 将 retry policy移入 scheduler，按 context generation event 重唤醒；receipt记录尝试、观察到的 revisions 和最终 disposition。 |
| RT-TXT-P1-038 | Partial | raster/SDF 有 `cancel`/`cancel_all`，但 worker body不协作检查；取消主要在完成发布时丢弃结果。 | source decode、outline、SDF generation支持 cooperative checkpoint/deadline；释放大 source lease和CPU时间不能等到整个 batch完成。 |
| RT-TXT-P1-039 | Open | SDF completion queue满时删除 active id并增加计数；consumer随后只得到泛化 `Retryable`，丢失 backpressure reason。 | 保留 terminal `Backpressured { queue, bytes, age, retry_after }` receipt；重试必须有预算、优先级和防抖，不得把panic之外所有失活合并。 |
| RT-TXT-P1-040 | Open | 未见多 Runtime/Editor preview共享与隔离、locale/DPI/backend矩阵、device loss、超长编辑、复杂脚本视觉 golden 和可比 benchmark 资格。 | 建立 deterministic harness + real GPU/platform suite，报告 correctness corpus、p50/p95/p99、allocation/frame、cache hit、atlas churn、queue latency、fallback/degrade和long-soak。 |

## 5. P2：精度、可维护性与性能债务

| ID | 状态 | 当前证据 | 重构要求 |
|---|---|---|---|
| RT-TXT-P2-001 | Open | collection id 使用进程全局 `AtomicU64`，耗尽时 `expect` panic。 | 返回 typed allocation failure或使用不可复用generation-qualified id；禁止库层 panic。 |
| RT-TXT-P2-002 | Open | direction-alias fingerprint 包含 collection/Unicode，但 `matches_lookup_except_base_direction` 精确比较漏掉这两项。 | 精确比较所有 fingerprint 字段并加入强制 collision test；不能依赖 64-bit process hash 不碰撞。 |
| RT-TXT-P2-003 | Open | `TextShapeRunProvider` 默认 vertical method 委托 horizontal，custom provider可在无声明时产生水平结果。 | vertical capability必须显式实现或返回 `Unsupported`；artifact记录 provider capability/version。 |
| RT-TXT-P2-004 | Open | Swash color outline路径固定 `palette_index: 0`。 | 将 palette selection纳入 font style/instance/raster key，支持 CPAL authoring、fallback和cache invalidation。 |
| RT-TXT-P2-005 | Partial | layout cache estimated bytes计入 entry/text/caller additional bytes，但未像 shaped cache一样保守计入多重 index/control/load-factor capacity。 | 统一 cache residency estimator；用 allocator/heap profile校准并给出误差区间。 |
| RT-TXT-P2-006 | Open | `UiTextPaint`、line、cluster、paint run会重复拥有 `String` 文本。 | transport/frame内改用 Arc slice、source range或interned document snapshot；跨进程序列化再显式 materialize。 |
| RT-TXT-P2-007 | Open | font/family/color在高频 render DTO中仍是 `Option<String>`，renderer重复 clone/parse。 | cascade阶段生成 interned typed handles和linear color；字符串只保留在资产/序列化边界。 |
| RT-TXT-P2-008 | Open | 多类 frame/generation/report counter使用 saturating arithmetic，达到上限后永久粘住且无 epoch rollover receipt。 | 定义 generation exhaustion/epoch rollover策略；健康快照区分真实上限与telemetry saturation。 |
| RT-TXT-P2-009 | Open | system font枚举和fallback结果依赖平台数据库顺序，未见规范化排序/冲突 receipt。 | 对发现结果按稳定 descriptor/source identity排序，记录冲突、同名字体和平台差异。 |
| RT-TXT-P2-010 | Open | compatibility global constructor没有统一 deprecation/feature gate/architecture test。 | 建立 public API lint/source gate，新增产品调用点不得引用 `shared_*` 或无 context constructor。 |
| RT-TXT-P2-011 | Open | cache/worker/atlas reports各自 take/reset/interval语义不同，产品无法原子采样同一 frame。 | 使用 immutable health snapshot和cursor，统一interval/cumulative/high-water语义。 |
| RT-TXT-P2-012 | Partial | 当前有大量单元/source/performance evidence tests，但未见系统性的 adversarial hash、Unicode differential、worker interleaving和cache/atlas state-machine property suite。 | 加入 differential/fuzz/model checking和seeded replay，failure corpus进入版本库并绑定 provider snapshot。 |

## 6. 参考引擎对比

| 能力 | 参考源码提供的形态 | Zircon 当前状态 | 应吸收的工程原则 |
|---|---|---|---|
| 字体/塑形 owner | Unreal `FSlateTextShaper` 显式持有 FreeType cache、composite font cache、renderer/font cache；有 `FlushCache`、双向/单向、Full/KerningOnly策略 | Core collection注入真实，但 parser/cache/worker/global convenience仍分裂 | 一个 context拥有完整依赖、策略、缓存和生命周期；快路径选择必须有正确性条件和统计 |
| shaped artifact/source map | Unreal `FShapedGlyphSequence` 保留 source range/index map、face data、bitmap/SDF atlas weak handles | 内部 artifact较丰富，Runtime Interface transport缩减且无provider lineage | 内部/跨边界artifact必须显式分层、versioned、可判stale，不以“Canonical”掩盖丢失字段 |
| 字体能力面 | Godot TextServer接口覆盖data/style/weight/stretch/AA/MSDF/variation/OpenType/language/script/cache/glyph，advanced backend用RID owner管理font/variation/shaped text | 底层能力存在，但 UI contract/owner/lifecycle未全部暴露 | 借鉴capability和resource lifecycle，不采用本仓库禁止的非网络“server”命名 |
| Rust资源化布局 | Bevy `TextPipeline` 使用显式 `FontCx`/`LayoutCx`，传递 letter spacing、font features/variations；atlas key绑定font特征 | Zircon Core路径显式collection较好，但公共路径和UI style合同落后 | Context资源显式注入；typography字段必须端到端进入layout和atlas identity |
| Rust UI接入 | Fyrox `FormattedText` 把 FontResource、wrap/alignment、资源等待/错误接到UI对象 | Zircon shaping/Unicode更强，产品owner和错误呈现仍不足 | 保留Zircon复杂塑形能力，同时补齐资源等待、错误状态和Editor/Runtime一致入口 |
| GPU residency | Unity Graphics `Texture2DAtlas` 有allocator、texture identity/cache、GPU valid/invalid、update/release/clear target | Zircon native/SDF atlas已有shadow commit/recovery和generation，但产品health/统一policy不足 | 分离CPU source、GPU allocation、validity和publication；device loss/backpressure必须可观察并可恢复 |

对照结论不是照搬 C++、ECS 或 RID API。Zircon已有更强的 Rust ownership、typed failure和部分缓存预算，应保留；缺的是把这些局部能力组成一个可配置、可关闭、可重放、可诊断的产品级 context。

## 7. 目标重构路线

### Phase A：统一 authority 与禁止隐式 global

1. 定义 `TextRuntimeContextId`、`TextSessionId`、`TextRequestId`、`TextSnapshotLeaseId`、`TextArtifactId` 和 `TextFrameToken`。
2. 扩展 `TextModule`，注册 context factory/registry、font/Unicode authority、work scheduler、cache residency和health snapshot。
3. 将 Dynamic UI、Graphics/SceneRenderer、Editor preview和standalone tool显式绑定 context；为 compatibility path增加标识、feature gate和调用点 lint。
4. 定义 active/draining/closed/faulted 生命周期，关闭时drain worker、cache lease、font claim和artifact sidecar。

### Phase B：补齐 typography 与 Unicode 合同

1. 扩展UI asset schema、cascade、`UiResolvedStyle`、`TextStyle`：slant、stretch、letter/word spacing、family stack、OpenType features、variation axes、optical sizing、hyphenation/locale policy。
2. 生成规范化 `TextTypographyDescriptor`，作为 layout/cache/glyph instance/raster/atlas共同输入。
3. 完成 rich override实际应用矩阵；artifact route和fallback route必须用同一descriptor。
4. 设计 normalization双向source map和Unicode provider artifact；在 selection/IME/a11y/caret测试闭合前不启用破坏offset的normalize。

### Phase C：调度、缓存与长文档

1. 建立 `TextWorkScheduler`，承载shape/parse/raster/SDF的priority、deadline、cancel、frame budget、generation event和terminal receipt。
2. one-shot API租用context scratch/session pool；rich parser/cache按trust/plugin generation分区。
3. 所有cache同时限制entries/bytes并接入MemoryPressure；预算从profile artifact读取，不再散落常量。
4. 为TextDocument实现snapshot-safe compaction、history/checkpoint、lease owner和long-session soak。

### Phase D：artifact、render 与恢复

1. 分离in-process retained sidecar与versioned transport artifact，定义font/Unicode/document/frame lineage和stale validation。
2. renderer只消费有效glyph instance/artifact；fallback需要同一typography descriptor并发布degradation receipt。
3. raster/SDF加入cooperative cancel和具体backpressure outcome；atlas把CPU source、GPU residency、upload transaction、present generation串成一个receipt链。
4. system font policy、color palette、variation instance和device-loss recovery进入project/profile和健康面板。

### Phase E：资格与性能闭环

1. 建立 Unicode differential corpus：Latin/Arabic/Indic/SE Asian/CJK/Emoji/variation selector/vertical/bidi/isolate/combining/invalid-control。
2. 建立真实Runtime + Editor preview + multi-context隔离测试，验证font generation、claim/revoke、artifact stale、IME/document revision一致。
3. 建立GPU backend/DPI/scale/device-loss视觉golden和atlas residency测试。
4. 以相同corpus和硬件记录p50/p95/p99、allocation、cache/atlas命中、worker queue、fallback率、first-frame/warm-frame和long-soak；在数据前不得宣称优于Unreal。

## 8. 工程资格门

| Gate | 当前 | 通过条件 |
|---|---|---|
| TXT-G01 | Partial | 每个 Core有独立font collection，且完整TextRuntimeContext也绑定同一owner |
| TXT-G02 | Fail | 所有产品constructor必须注入context；无隐式process-global文本路径 |
| TXT-G03 | Partial | 项目字体只消费cooked artifact；legacy filesystem被显式Dev/Tool capability隔离 |
| TXT-G04 | Fail | font/system/locale/fallback policy由profile/project artifact决定并有receipt |
| TXT-G05 | Fail | shipping system font发现、排序和fallback在目标平台可重放并有fingerprint |
| TXT-G06 | Fail | UI typography contract覆盖slant/stretch/spacing/features/variations/family stack |
| TXT-G07 | Fail | rich canonical和fallback route产生相同glyph/advance/style语义或typed reject |
| TXT-G08 | Partial | Unicode provider版本已进入snapshot，进一步由context pin并支持兼容/admission |
| TXT-G09 | Partial | bidi/script/emoji/joining/line-break基础真实，需通过differential corpus |
| TXT-G10 | Fail | normalization有selection/IME/a11y/cache/glyph双向source map |
| TXT-G11 | Fail | locale line break和dictionary hyphenation有versioned provider/receipt |
| TXT-G12 | Fail | 超预算shape/parse可defer/yield，不阻塞UI caller |
| TXT-G13 | Fail | shape/parse/raster/SDF统一支持deadline/priority/cooperative cancel/terminal receipt |
| TXT-G14 | Partial | rich byte/token/depth/table/diagnostic预算真实，需增加CPU slice/deadline |
| TXT-G15 | Partial | document prepare/commit/revision/snapshot lease真实，需补compaction/history |
| TXT-G16 | Fail | 长期编辑可compaction/rebase且不破坏revision、source map和active lease |
| TXT-G17 | Fail | undo/redo/checkpoint/persistence属于稳定document operation authority |
| TXT-G18 | Fail | 所有文本cache都有entry+byte+single-item admission和准确residency |
| TXT-G19 | Fail | parser/decorator/emoji/cache由runtime context/catalog拥有并支持revoke/reload |
| TXT-G20 | Fail | rich single-flight等待有deadline/cancel/panic终态和waiter budget |
| TXT-G21 | Partial | raster/SDF/native atlas队列与字节预算真实，需接入统一scheduler/profile |
| TXT-G22 | Partial | generation invalidation、shadow commit和recovery真实，需形成统一render receipt |
| TXT-G23 | Fail | transport artifact有schema、producer、font/Unicode/document/frame lineage |
| TXT-G24 | Partial | 内部glyph artifact保留丰富provenance，跨边界路径也必须无损或明确受限 |
| TXT-G25 | Fail | cancel真正停止昂贵工作；completion backpressure保留具体terminal reason |
| TXT-G26 | Fail | TextResidencyProfile接入Runtime196 memory domain/pressure并可动态trim/degrade |
| TXT-G27 | Fail | 字体缺失、fallback、stale、queue、atlas/device-loss进入产品health/diagnostics |
| TXT-G28 | Fail | multi-Core、Runtime+Editor preview、reload/revoke隔离集成测试通过 |
| TXT-G29 | Partial | 现有2,160个test markers经实际执行、分类和flaky/ignored审计后形成静态基线 |
| TXT-G30 | Fail | Unicode/locale/DPI/backend视觉golden、fault、scale、soak和benchmark资格通过 |

## 9. 完成判据与非目标

Runtime201 的实施不能以“能显示文本”“测试数量多”“有HarfBuzz/Swash”“有一个atlas”作为完成判据。完成要求是：同一 Runtime context 从字体/Unicode/document revision开始，经过parse/shape/layout/cache/artifact/raster/atlas/upload/present，始终携带可验证identity、generation、预算、失败阶段和terminal receipt；多context不能串数据；fallback不能悄悄改变排版语义；关闭后没有活跃claim、worker、lease或sidecar。

本计划不要求复制Unreal的C++对象图，不采用Godot非网络“server”命名，也不把Bevy ECS资源模型机械移植到Core。目标是用Zircon现有Rust ownership、immutable snapshot和typed error构建同等强度、可证明且可持续演进的工程合同。
