---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: native-bitmap-source-cache-idle-flush-and-linear-maintenance
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache/lru.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache/residency.rs
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/storage.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/atlas/page_residency.rs
  - zircon_runtime/src/text/atlas/slot_cache.rs
  - zircon_runtime/src/text/atlas/bitmap_run/types.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs
---

# Native bitmap source cache空帧清空与线性维护

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/native_bitmap_atlas/**`当前源10/10 Rust文件及root/renderer调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 联动责任：worker cancel/backpressure联动Text09，bytes/slot ownership回链PERF-MVP-231。
- 交接原因：source cache residency、atlas slot与raster key属于Text04；pending worker总预算属于Text09，不能在性能切片中单独放宽cache而留下无界内存或失效竞态。

## 失败现象与复现证据

PERF-MVP-242：任意空native-text帧都会清空最多2048张cached glyph image和pending work映射，但没有取消已提交worker；迟到结果变unknown并浪费raster。文本恢复后重新miss、placeholder/raster/upload。exact miss的approximate lookup扫描整个HashMap，满容量每次insert又全表找LRU；横向bin已经归零，近似纵向候选实际最多3个。

PERF-MVP-231已覆盖同目录cache-hit bytes clone、retry嵌套匹配/多轮clone、无persistent slot及mixed-storage report/render重复建submission；不得在本任务重复造第二套slot/cache owner。静态证据见`docs/plans/performance/01/2026-07-18-text-native-bitmap-atlas-submodules-static-review.md`。

## 最低共享层根因

当前把“本帧没有文本”误当成cache invalidation和worker cancellation信号；cache只按entry数保存image，没有与persistent atlas slot、byte/page预算、last-visible frame和face generation统一的residency owner。LRU tick只存在HashMap value中，近似key也没有利用有限subpixel domain，因此维护操作退化为全表扫描。

## 架构修复验收

- 空文本帧只更新idle age/report，不清空预算内source cache、persistent slot或pending request；显式face generation失效、device loss、budget pressure和shutdown分别处理。
- cache/atlas按entry+CPU bytes+GPU page bytes设置watermark与hard cap；逐出统一从persistent slot owner发起，source、slot、page generation和upload state原子失效。
- approximate lookup直接构造另外最多3个`y_bin` key做HashMap probe；禁止遍历全部entries。
- LRU使用stable slot/generational index加intrusive/indexed queue，hit/update/evict amortized O(1)；不得用每次`min_by_key`扫全表。
- pending raster携face/cache generation与cancel token；idle不得孤儿化work-id。明确取消、失效或饱和结果有独立counter，迟到completion不得混入unknown。
- 与PERF231合并后，draw occurrence只引用persistent glyph slot，cached bytes用shared ownership且只在miss/invalidated时上传。
- stable text→1/300 empty frames→same text记录cache/slot resident、raster submitted/completed/canceled/unknown、placeholder、upload/page rebuild：预算内后五项均为0。
- 2048 resident下1/100/1k新glyph记录approx probes、LRU touched slots、caller CPU/alloc；每miss probes≤3，逐出amortized O(1)，无capacity倍数斜率。
- face hot reload、font removal、worker saturation、shutdown、Alpha/Color/Subpixel、Softbuffer/WGPU/RenderDoc resource upload与像素等价。

## 禁止临时方案

- 不得仅删除idle clear而不增加byte/page预算、generation invalidation和shutdown策略。
- 不得扩大2048容量掩盖全表扫描，或降低近似质量来绕过candidate lookup。
- 不得在空帧只清pending map而让worker继续无owner运行；取消与迟到结果必须闭环。
- 不得按format重排Alpha/Color/Alpha submission破坏painter order；storage pass合并必须由renderer支持有序texture binding/plan后再做。

## 修复结果与回传

2026-07-31 非验收实现收敛：空文本帧已只刷新 report 并保留 source image/pending worker；source cache 使用 2048 entry + 8 MiB CPU byte hard cap，共享 `Arc<[u8]>` pixels，近似命中直接探测最多 3 个 vertical-bin key。O(1) intrusive LRU 已拆为 `source_cache/lru.rs` leaf；正常 hit/insert/evict 不扫描全表，链接异常不再由生产 `expect` panic，而是一次性重建并通过 `lru_repair_count` 显式报告。face invalidation 会取消可取消 worker、推进 face epoch 并清理 shared font identity/bytes；饱和、取消、unknown/invalid completion 均有独立 counter。新增 dangling-tail 回归锁定 fail-closed 修复、recent ordering 与后续逐出；二次静态审查 P0/P1/P2=0，未运行 Cargo 或产品 WGPU。

2026-08-01 统一预算压力前向实现：exact source hit 现在以 O(1) `CacheKey <-> GlyphRasterKey` 反向索引绑定 persistent slot identity。source entry/byte hard-cap 的 LRU 淘汰只发出已绑定 key；`GlyphAtlasSet` 以该 key 定位 owner page，一次提升 generation 并原子清 allocator、全部 page slots 与 CPU shadow，再把同页全部 raster keys 定点回传 source cache。反向 atlas page eviction 通过显式 `GlyphAtlasBitmapRunPlan.invalidated_raster_keys` 回传，renderer upload failure 也去重 page keys、仅推进一次 generation，并把失效 source counter 延迟到下一帧报告。正常 hit/touch/evict 仍为 O(1)，跨层压力路径只按受影响 key/page 工作；没有全 cache 扫描。

产品 prepare report 已新增 source resident/max bytes、LRU touches、budget-linked eviction、linked raster invalidation 与 atlas resident page bytes。300 个 empty frames 回归要求 resident sources 保持、submitted/unknown/slot miss/upload copy 均为 0；2048 resident + 1/100/1k new glyph 回归严格锁定每 miss 最多 3 probes、0 LRU touch/eviction，ignored 31-sample exporter输出 p50/p95 而不设机器时间阈值。预算联动与规模测试已拆到 216 行 `tests/source_cache/residency.rs`，父 owner 为 844 行且低于 1000 行测试阈值；production source cache/LRU/page/render-state 为 722/243/527/396 行。

2026-08-01 二次静态审查先后修复：native frame/storage 仍读取已删除 `gpu_draw.vertices` 的 current-source 编译缺口、atlas hidden invalidation keys 跨帧积累风险、同页多个 failed upload copy 重复推进 generation、旧 exact report 漏写既有 capacity/max/resident 字段，以及 LRU guard 搜索不存在 `detach(`。修复后旧 GPU vertex symbol、production panic/unwrap/expect/dead-code allow 扫描为 0，scoped rustfmt/diff check 通过，未留下新的 actionable P0/P1/P2。

Open state: `source/slot/page budget-pressure implementation complete / resolving_failure / managed_validation_pending`。仍需 coordinator 执行 current-source focused/upward Cargo、ignored p50/p95 与真实 WGPU/RenderDoc 像素；成功回执前保持 open，不写成 blocked，不生成或登记占位截图。Text04 Plan registration 本轮在 coordinator health 阶段超时且无 receipt，按 cross-session 规则未重试/轮询，也没有 validation ticket 进入 queued/running；coordinator wakeup 后前向提交上述精确测试与 exact product framebuffer。

2026-08-01 产品捕获状态前向修复：`ScreenSpaceUiTextRasterUploadReport::worker_pending_count` 现在投影 source cache 的持久 `pending_worker_count`，而不是只统计本帧再次遇到的 pending glyph；`worker_failed_count` 同时汇聚 worker request failure、completion error 与 rejected completion bitmap；`missing_raster_image_count` 与 `visible_placeholder_count` 也通过 `RenderStats` 进入 gate。前者表示没有 source raster，后者覆盖 source raster 已存在却因 atlas 页分配受限而输出透明 glyph 的独立路径。prepare-report 回归明确构造本帧 pending 事件为 1、实际 in-flight work 为 3、request/completion/rejected failure 为 2/3/4、visible placeholder 为 1 的情况，并要求产品报告分别为 3/9/1；产品 gate 回归要求 `pending=0`、`failed=0`、`missing=0` 但 `visible_placeholder=1` 仍不可稳定。因此 framebuffer harness 不能因一次可见 glyph 遍历的零/低计数、未上报的 completion/rejection failure、无 source image 或透明 native-atlas placeholder 而提前 capture。此修复不替代受管 Cargo、真实 WGPU/RenderDoc 或新 PNG，failure 继续保持 `open`。

2026-08-01 本轮实现后二次独立静态审查：P0=0、P1=0。复核确认 durable in-flight、三类 raster failure、缺失 native-atlas image 和可见透明 placeholder 均已完整投影到连续两帧稳定门禁；未运行 Cargo、未修改代码、未生成 PNG。实现阶段完成，待 coordinator wakeup 后提交一次受管 Windows WGPU ignored product test；在该回执和实际截图到位前，failure 继续保持 `open`。

2026-08-01 capture gate 前向复核修复：审查发现 `submission.visible_placeholder_count` 原先只按 viewport 统计计划 placeholder，既可能包含 text bounds 外 glyph，也可能在 byte-budget/queue-overflow 选择 Glyphon fallback 后并未实际呈现。pending placeholder 现于创建时复用 `TextArea.bounds` 裁剪；产品报告只在实际 handoff 为 `TransparentPlaceholder` 时计入其数量。renderer upload 的 `requeued_count` 与 `failure_count` 同时投影至 `RenderStats`，因此连续两帧 capture 条件为 pending、worker failure、missing source image、实际 visible placeholder、upload requeue 和 upload failure 均为 0。新增 bounds 外 placeholder 与 Glyphon fallback 两个回归，尚未运行 Cargo 或 WGPU，未生成 PNG，failure 保持 `open`。

2026-08-02 最终独立静态复核：P0=0、P1=0。复核确认 placeholder 创建时的 bounds 裁剪与正常 raster path 一致，`GlyphonFallback` 时产品统计为实际呈现的 0，且 renderer upload requeue/failure 从 prepare report 穿透到连续两帧 capture gate。未运行 Cargo，未修改代码，未生成 PNG；所有非验证实现已完成，failure 继续保持 `open`，待 coordinator wakeup 提交受管 Windows WGPU ignored product test。

2026-08-02 SDF/MSDF capture-path 复核：distance-field atlas 同帧同步提交 GPU texture write；其异步 glyph-generation pending、预算延后或失败会在当前 prepare 转为 native/overlay，由本记录的 durable native worker、source image、actual placeholder 与 renderer-upload 六项稳定条件统一覆盖。没有新增独立 SDF 等待状态，避免已正常可见的 native fallback 把产品帧误判为未稳定；未运行 Cargo 或 WGPU，未生成 PNG，failure 保持 `open`。

2026-08-24 PERF-MVP-231 帧级结构重开：本记录此前要求 mixed storage 只能在 renderer 具备有序 texture binding/plan 后合并，当前源已验证其“连续段拆分”保序，却同时把该拆分扩散为多份 frame submission。`text/native_bitmap_atlas/storage.rs` 逐段 clone `GlyphAtlasSet` 并对全帧多组数组 filter/collect，`frame.rs` 将连续段数直接投影为 `storage_submission_count`，`scene_renderer/ui/text.rs` 为每段 materialize source bytes，`atlas_renderer/renderer.rs` 为每段准备 instance buffer、upload plan 与 shadow commit。交替 `Alpha -> Color -> Alpha -> ...` 时，段数可等于 glyph 数，现实现的总扫描和 atlas clone 工作不再是线性帧级管线。

关闭本项的最低共享层不是按 format 全局 regroup，也不是再建 cache/slot/page owner。必须硬切为：一个 canonical frame submission、一个 source-image slice、一次 prepared upload/atomic shadow commit；上传按 page/resource format 路由，draw command 按原 painter token 顺序执行并仅在实际 resource/pipeline 变化时切 bind group。`storage_resource_count` 必须表示唯一 format/resource 数，`ordered_draw_segment_count` 才表示保序切换次数；后者可以随交错 glyph 数增长，前者不能驱动重复 atlas/plan 构建。

本轮状态为 `architecture_review_complete / profiling_plan_complete / canonical-frame_and_native-input_hard-cut_implementation_complete / managed_validation_pending`。一个 canonical frame、固定 format resource table、预检后的原子 upload/shadow commit，以及 canonical shaped glyph -> `GlyphRasterKey` -> bounded worker -> `GlyphAtlasSet` 输入链均已落地；旧 renderer 原始字符串/`TextArea`/layout-buffer 输入与 glyphon fallback 语义已删除。静态复杂度、直接布局不变量和 retired-symbol 扫描已复核；未运行基线或改造后的 CPU/GPU/功耗采样，也没有生成或登记任何 PNG。完整 workload、计数器和验证顺序见 [`2026-08-24-mixed-storage-frame-plan-and-profiling.md`](2026-08-24-mixed-storage-frame-plan-and-profiling.md)。本 failure 继续保持 `open`，且本条不改变其他 source/slot/page 预算修复的待验收状态。
