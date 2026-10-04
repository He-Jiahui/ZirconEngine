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

### 2026-09-08 Parser Test Consumer Repair

Stable fixing Session `failure-roll-01a07160-text07`, baseline 601, retains
this lifecycle. Registration request `e3e92d8a7a494eee8e098ea7653df0a4`
resolved the existing lifecycle. Managed Windows text-only job
`9e3f8ac2a6564bd48fbeab45874342e3` reported a missing compiled-cache owner
import and `usize` arguments passed to the current `u32` range constructor
in the legacy performance comparator. The complete result had 22 compiler
errors, zero tests; it is not parser acceptance.

Both pre-edit files matched HEAD and are preserved in snapshot `3087`.
Transfer `2db4a177403248aba77ee8674cad88cb` binds the exact paths to Text07.
Source `3090`, request `88e756e337dc42838b7df04dd9e6e8cb`, freezes:

- `zircon_runtime/src/text/rich/parser_registry.rs`, hash
  `2d19eb339791c342027e91d977c8ee560ce248623559b69a92e3113a6db622f0`:
  explicitly import `CompiledRichTextCacheOwner` into the test module, keeping
  identity/generation exhaustion, retirement and last-use artifact assertions.
- `zircon_runtime/src/text/rich/parser/performance_tests.rs`, hash
  `bc282443dd83976beab9c816cfe0958f231bb2ac5c40f71af8f1d9aa2b87c581`:
  convert grapheme start/end with checked `u32::try_from` before the legacy
  `styled_run` call. Exact optimized/legacy equality, workload sizes, sampling
  and the original 75% P95 reduction threshold remain unchanged.

No production parser, cache or trust policy changes. Scoped repository-edition
formatting and whitespace checks pass. Managed execution and independent
review of this increment are pending. The original command/run/300-frame
matrix, Text09 counters, product traces and WGPU evidence remain required.

Managed job `1f242fb1789e44ce87d7a5cacc569d5f` subsequently compiled both
`3090` paths without diagnostics on immutable input
`runtime-text-owner-test-support-3095-20260908`, digest
`4e5f800cd19e5748464ac759b4e1efbfae073ecf1c05d419327a6d6739cbe9e2`.
The same text-only static locked library configuration now has three errors,
previously 22. All 14 new overlay paths are diagnostic-free; zero tests ran.
The exact receipt is `results/text-library-recheck.json` under the approved
benchmark input. Source synchronization and all 354 dependency packages were
verified, followed by the complete 10,955-file post-run manifest. This result
does not establish the optimized/legacy timing threshold or parser behavior;
both still require actual execution and independent source review.

### 2026-09-19 compiled-artifact static contract receipt

Corrected coordinator-managed ticket `a3c74161e23c4282b6f67f6abcb018b1`
passed with job/run `28a7a402214241569edd27abf42eee57` /
`a3c74161e23c4282b6f67f6abcb018b1` (exit 0,
`TEXT07_COMPILED_RICH_ARTIFACT_SOURCE_CONTRACT_PASS`, `CHECKED_PATHS=6`).
The sealed source-manifest hash is
`304828504d9182fa4dee197447f4c4236cd6fb525d4344b7b1919de86012b4a5`.
The rerun uses `rustfmt --config skip_children=true --check` after the
earlier coordinator-only nested `compiled/dependency.rs` resolution failure;
no source edits were made. It confirms one generation-owned
`CompiledRichText` artifact is projected through UI, renderer, and resource
consumers, with decorator/emoji generations in the parser owner and no direct
renderer reparse. `zircon_runtime/src/ui/text/layout_engine.rs` remains
explicitly deferred to the active Text03 owner. Managed Text07/Text09 Cargo,
scale/cache counters, WGPU/product evidence, independent C/I/M review,
canonical failure return, and closeout remain pending; the failure stays
`open`.

### 2026-09-20 independent review

- Reviewer Session `review-text07-rich-artifact-r2` inspected the source-sealed failure record,
  parser registry, compiled artifact, UI projection, texture dependency consumer, and rich-text
  renderer. The five production hashes are `parser_registry.rs`
  `af98f803b3b1090d74cff894249e22a8f1136664a681c01931fb8c53e18fda2e`, `compiled.rs`
  `6d52c237ef16a72a2dbc0b30385459b3d943e8e29c1fd4fa356fd454d374bfa6`, `ui/text/rich_text.rs`
  `a755e30b76d5cda8aec354d8fb40bd799a6220ddb9b37677c803093802da4253`, `ui_texture.rs`
  `06557e7a6419327f1e28636d60b7b5b5c9c612337216c7c3f89e9851d404f984`, and renderer
  `rich_text.rs` `1ad8a6ede730d729b5e22bcc84a0220c2b26a9a4984886e53a38ee5050f40e37`.
  The reviewer held the failure-document lease during the audit.
- Review result: `Critical=0`, `Important=0`, `Moderate=0`. `CompiledRichText` owns one shared
  `Arc<str>` source and generation metadata; parser registry owns the bounded compiled-cache
  owner plus decorator/emoji generations. `UiParsedText::from_compiled` projects ranges and
  indices, while texture preparation and renderer lookup resolve the layout’s artifact handle
  instead of reparsing markup. The renderer has no direct parser-registry compile call; cache
  eviction and link-hit identity regressions remain represented by the existing tests.
- The source-contract probe emitted `TEXT07_COMPILED_RICH_ARTIFACT_REVIEW_CONTRACT_PASS`;
  scoped Rustfmt (edition 2021, root-only) and `git diff --check` passed. `layout_engine.rs` is
  explicitly deferred to Text03, and Text09 bounded-cache telemetry, run-scale/300-frame
  performance evidence, managed Cargo, WGPU/product artifacts, fixed return, and closeout remain
  pending. No static receipt is promoted to those dynamic gates.

## 2026-09-25 current-source rolling reconciliation (compiled rich artifact r3)

Successor Session `failure-roll-01a084c8-text07-rich-artifact-r3` claimed the failure record,
Text07 plan, and the five non-overlapping parser/artifact/texture/renderer owners. The active
Text03 Session still owns `zircon_runtime/src/ui/text/layout_engine` and
`zircon_runtime/src/ui/text/measure_cache`; those paths remain explicitly deferred and were not
claimed or modified here. No Rust source was edited in this continuation. The current source
probe completed with marker `TEXT07_COMPILED_RICH_ARTIFACT_CURRENT_SOURCE_PASS 8 of 8`:
`CompiledRichText` and its shared source/generation metadata remain canonical, parser registry
owns the bounded cache/decorator generations, UI/texture/renderer consumers use the artifact
handle, and the renderer has no direct parser call.

The claimed current hashes are:

| Path | SHA-256 |
|---|---|
| `zircon_runtime/src/text/rich/parser_registry.rs` | `af98f803b3b1090d74cff894249e22a8f1136664a681c01931fb8c53e18fda2e` |
| `zircon_runtime/src/text/rich/compiled.rs` | `6d52c237ef16a72a2dbc0b30385459b3d943e8e29c1fd4fa356fd454d374bfa6` |
| `zircon_runtime/src/ui/text/rich_text.rs` | `a755e30b76d5cda8aec354d8fb40bd799a6220ddb9b37677c803093802da4253` |
| `zircon_runtime/src/graphics/scene/resources/ui_texture.rs` | `06557e7a6419327f1e28636d60b7b5b5c9c612337216c7c3f89e9851d404f984` |
| `zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs` | `1ad8a6ede730d729b5e22bcc84a0220c2b26a9a4984886e53a38ee5050f40e37` |

All five claimed source paths were already dirty current-source overlays before r3 and retain
their existing attribution; r3 did not absorb or rewrite them. The failure document also had
prior receipt edits. Scoped `git diff --check` and exact-file Rustfmt (edition 2021,
`skip_children=true`) are clean. The authoritative current-source manifest is coordinator
snapshot `3845`.

This is static handoff evidence only. Text03-owned layout/measure paths, Text09 bounded-cache
telemetry and 300-frame scale evidence, managed Windows Text07/Text09 Cargo, WGPU/RenderDoc and
product PNG evidence, independent review, canonical fixed return, closeout SHA, and WeCom result
remain pending; historical static tickets are not reused as dynamic acceptance.

Independent review receipt (2026-09-25): the reviewer rechecked coordinator snapshot `3845`
and confirmed all five non-overlapping source hashes, the
`TEXT07_COMPILED_RICH_ARTIFACT_CURRENT_SOURCE_PASS 8 of 8` marker, shared artifact/generation
ownership, bounded parser cache, artifact-handle consumers, absence of a renderer direct parser
call, Text03 deferral, dirty attribution, clean scoped diff/rustfmt, and pending Text09/Cargo/
WGPU/RenderDoc/PNG gates. Review result is Critical `0`, Important `0`, Moderate `0`. This
receipt is carried in post-review coordinator snapshot `3846`.
