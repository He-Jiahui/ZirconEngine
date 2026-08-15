---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: rich-text-reparsed-across-frame-consumers
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/rich
  - zircon_runtime/src/text/rich/parser_registry.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/graphics/scene/resources/ui_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
---

# Rich text跨阶段重复解析与多份文本所有权

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/rich/**`当前源14/14 Rust文件及其UI/graphics调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md`
- 联动责任：artifact cache/generation预算联动Text09。
- 交接原因：parse artifact、decorator generation、run/inline/link/resource metadata归Text07；缓存容量、LRU与跨帧命中预算归Text09，不能由性能计划复制owner。

## 失败现象与复现证据

PERF-MVP-238已直接修复每parse重建builtins、无替换片段临时String和grapheme×run全表find，并加入源码/行为门禁。

PERF-MVP-239仍存在：同一command在prewarm、measure、layout、resource collection、render paint分别parse；inline frame fallback按inline run再次parse，link hit按事件再次parse。`UiParsedText`又在完整`RichParseResult`之外clone stripped text、paragraphs并为每run分配substring String。稳定帧没有command-generation artifact，resource streamer和renderer继续从markup重建metadata。静态证据见`docs/plans/performance/01/2026-07-18-text-rich-static-review.md`。

PERF-MVP-301/302补充UI adapter证据：BiDi visual order为每grapheme依次物化owned token/cluster/cloned cluster/fragment/final run；rich table每cell执行preferred+actual双layout，每次都全量扫描并切片clone runs/paragraphs/tables，local parse DTO继续双持text/paragraphs。证据见`docs/plans/performance/01/2026-07-18-runtime-ui-text-static-review.md`。

## 最低共享层根因

neutral `UiRenderCommand`只携markup、paint/layout DTO与byte ranges，没有指向canonical compiled rich document的generation-owned handle。UI parse owner把方便消费的所有权DTO复制给局部阶段，却没有把一次解析结果贯穿到graphics resource、paint、inline和interaction消费者。

## 架构修复验收

- Text07定义`CompiledRichText`：唯一stripped source、排序run/paragraph/table ranges、inline objects、links、resource ids及decorator/format generation；以`Arc`或generation handle跨UI/graphics共享。
- `UiParsedText`不再同时拥有`text clone + run substring Strings + paragraphs clone + RichParseResult`；run保存range并借用/shared source，style/inline/link只保留一份canonical metadata。
- shape prewarm、measure、full layout、paint、inline frame、link hit与resource streamer消费同一artifact；graphics层禁止从markup再次调用parser。
- compiled artifact提供visual/source cluster index与table cell→run/paragraph/nested-table range索引；UI visual projection不得创建per-grapheme owned String，cell intrinsic/final layout不得重复切片全文或shape同一cell。
- artifact key覆盖exact markup、format及custom decorator/emoji registry generation；内容/registry变化只失效相关command，不允许stable frame重parse。
- Text09设置entry/byte上限、O(1)命中与LRU、frame hit/miss/parse/evict counters；不得建立无界markup cache。
- 1/100/1k commands×1/100/1k runs及stable 300 frames记录parse calls/bytes、source ownership、command/run visits、cache bytes与p50/p95；每command generation parse≤1，stable parse=0，per-run substring owned bytes=0。
- BBCode/HTML/Markdown/plain、自定义decorator/emoji、cluster-first style、paragraph/table、inline image/icon/widget、link hit、resource readiness、horizontal/VerticalRl与产品像素等价。

## 禁止临时方案

- 不得只在renderer旁新增第二个parse cache；canonical artifact必须从UI owner贯穿全部消费者。
- 不得以hash命中后仍clone完整`RichParseResult`或每run substring String。
- 不得让cache key遗漏decorator/emoji registry generation，或禁止custom parser来换取共享。
- 不得跳过resource/link/inline metadata解析；应共享已编译索引，不是删除功能。

## 修复结果与回传

2026-08-01 implementation state: `open / resolving_failure / non_validation_implementation_complete / post_fix_review_complete / managed_validation_pending`。

- `CompiledRichText` 是唯一 generation-owned source/run/paragraph/table/link/inline/resource index；`UiParsedText` 使用 range/index projection，cell 复用 parent `Arc<CompiledRichText>`，不保留 stripped-text、run substring、style/link/inline metadata 或二次 compiled artifact clone。
- `UiResolvedTextLayout.rich_text_artifact` 直接持有 type-erased `Arc`。extract 仅在 layout 没有可解析 artifact 时编译；renderer、texture preparation 与 link hit 全部从该 layout handle 解析，不再按 markup lookup/reparse。registry 已硬删除，因此空闲 frame 不会保留离开布局生命周期的强引用。
- cache-eviction regression 先布局 A1、驱逐 parser cache、再执行 extract preparation，并锁定 A1 指针 identity 与 link hit；rich projection regression 同时锁定 local run range 有序、不重叠以及 stable parent run index。
- post-fix review reports P0=0/P1=0. failure 保持 `open`，因为 Text09 的 run-scale cache/parse counters、managed Cargo、真实 WGPU/RenderDoc 和新的产品 PNG 仍未形成验收证据；本轮未生成策略文字截图。
- 2026-08-11 structural follow-up: parser-local registry, built-in singleton, parser identity, decorator/emoji generation and compiled-cache handoff now live in `text/rich/parser_registry.rs`. `text/rich/mod.rs` is again a declaration/re-export boundary, and all crate-internal parser/cache-test callers use the leaf module directly. This closes the root-owner structure item without adding a compatibility parser or a second cache; the failure remains `open` pending Text09 bounded-cache telemetry and managed runtime evidence.
- 2026-08-11 second review followed the public parser through generation-keyed single-flight admission and the UI compiled-artifact consumers. The root remains a 32-line declaration/curated-export boundary, every touched production owner remains below the 800-line warning, and no root helper re-export, compatibility path, duplicate parser, or actionable P0/P1/P2 remains. Repository-edition leaf rustfmt and scoped whitespace checks pass; the failure remains `open / non_validation_implementation_complete / second_review_complete / managed_validation_pending` until coordinator-owned Cargo/WGPU/product-frame evidence exists.
- 2026-08-11 M0 observability forward repair: the existing `CompiledRichTextCacheFrameSampler` is now owned by `UiTextMeasureCache` only under `profiling`, and caller-thread extract publishes one fixed delta/residency counter set after rich artifact preparation. No cache policy, parser path, worker, or layout algorithm changed. The rich/vertical prewarm regression uses a unique Markdown key and requires one frame-owned counter sample with a real parse; the profiling-owner regression independently locks every hit/miss/parse/evict/admission/probe/residency counter name and value. Exact-file Rustfmt and scoped whitespace checks pass; Cargo, WGPU profiling, power, and product PNG remain coordinator-managed pending evidence.
