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
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/storage.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
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

Open state: `CPU source-cache MVP implemented / managed_validation_pending；等待跨 source/slot/GPU page 的统一 budget-pressure eviction、300 empty-frame 规模 counter、current-source Cargo 与产品 WGPU/RenderDoc 像素证据，收到成功回执前保持 open`。
