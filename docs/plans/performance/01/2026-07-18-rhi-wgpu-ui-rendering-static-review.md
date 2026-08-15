---
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry/tests.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/pipeline.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/render_pass.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/retained_cache.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/surface_setup.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/text.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/render_pass.rs
  - dev/slint/internal/renderers/skia/lib.rs
tests:
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry/tests.rs
  - current-source Windows ui_surface tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# WGPU UI rendering逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`geometry/pipeline/render_pass/retained_cache/surface_setup/text`及geometry tests当前源7/7个Rust文件、1,822行已逐文件阅读；连同前一批root/batching/dependency/tests 4文件，`zr_rhi_wgpu/src/ui_surface`当前源11/11已静态覆盖。动态Cargo、产品规模counter、Softbuffer像素与RenderDoc仍未完成，所以整个RHI批次继续留在`pending.md`。

## 热路径结论

`geometry::draw_items`每present收集并sort全部commands；每个solid item持一份vertex Vec，batch layer再收集一份，`WgpuUiDrawBuffers::new`又汇总第三份CPU Vec并每帧`create_buffer_init`两个GPU buffers。rounded rect/border每次重算points与sin/cos。image key在item和per-resource BTreeMap间重复String clone。该链即使draw-list generation与surface size未变也全部重跑。

`WgpuUiTextRenderer::prepare`虽然保留FontSystem/SwashCache/atlas，却每present清空batches；每个text batch新建buffers/commands/clips/text-areas Vec，每个command新建glyphon Buffer并`Shaping::Advanced`，每batch再新建TextRenderer。它缓存glyph raster但不缓存layout/shape，稳定编辑器chrome仍支付CPU shaping与prepare/upload判定。

root presenter复核补充：cache未超256项时的每帧全表prune Vec和upload-key String多clone已直接止损；但编辑器GPU stream固定携image bytes，`prepare_image_resources`仍会对每个带RGBA的可见静态image/atlas逐帧`queue.write_texture`，必须由command/image generation把bytes只放在首次/变更帧。

原`record_draw_ops_to_view`每个DrawOp开启一个render pass；同一draw plan在FullRedraw对surface/cache各录一遍，在DamagePatch先full-screen restore再对surface/cache各录一遍。PERF-MVP-227已把连续solid/image ops合并为单pass，text因glyphon borrow保持一op一pass：每target由D passes降为`non-text runs + text ops`，顺序、pipeline/bind group和load/store不变。plan仍被录两次、GPU buffers仍每present创建，属于Render17结构修复。

pipeline/layout/sampler只在surface初始化创建；`pollster::block_on(request_device)`也是初始化同步点，不是每帧等待。retained texture/bind-group/restore buffer仅创建或resize；damage patch的full-screen restore是带宽成本但不是CPU重建。UI pass均`timestamp_writes: None`，尽管device会申请timestamp features，当前只能用RenderDoc离线估计。

Bevy UI在prepare阶段形成batch ranges并由render phase复用统一buffer/pass owner；Slint partial renderer持有dirty-region与partial-rendering state，只重绘受影响区域。Zircon应以draw-list/layout generation拥有ordered items、geometry、text layout与upload ranges，再让damage只筛选/patch，而不是把worker用于无界重复重建。

## 动态验收

受管`ui_surface` Cargo必须覆盖batch/dependency/geometry/text attrs/cache/mode/source guards。补1/100/1k/10k mixed solid/image/text与stable/damage workloads，记录command sort、rounded trig、shape、glyph miss、CPU alloc、vertex copy/upload bytes、new GPU buffer、render pass/draw count及p50/p95。RenderDoc需验证FullRedraw/DamagePatch的pass/marker/resource、surface/cache内容与像素；generation缓存、persistent upload和single projection完成前不得进入`review.md`。
