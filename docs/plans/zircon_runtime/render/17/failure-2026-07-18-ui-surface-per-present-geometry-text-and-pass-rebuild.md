---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: ui-surface-per-present-geometry-text-and-pass-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry/clipping.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/render_pass.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/retained_cache.rs
---

# UI surface每present geometry text与pass重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：WGPU UI surface当前源11/11 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：generation-owned compiled UI geometry/text/upload、GPU pass profiling与cache/surface投影由runtime renderer拥有，不能由editor painter建立平行缓存。

## 失败现象与复现证据

每present重新sort commands；solid vertices经历item、layer和global upload三层Vec分配/复制并新建GPU buffers；rounded geometry重算trig。text每command新建glyphon Buffer并Advanced shape，每batch新建TextRenderer。原每DrawOp开一个render pass，且FullRedraw/DamagePatch都把同一plan分别录到surface与retained cache。局部已把连续non-text ops合并到一个pass，但稳定generation的全部CPU/GPU重建和双target recording仍存在。

## 最低共享层根因

WGPU presenter没有以draw-list/layout generation为key的compiled presentation owner。geometry、batch ops、text layout、GPU upload buffers和retained projection是present-local临时值；damage仅裁剪command rect，没有复用上一generation的shape/vertex/batch/upload事实。统计也不暴露sort/shape/buffer-create/pass count，无法设置F4预算。

## 架构修复验收

- draw-list/layout generation稳定时command sort、rounded points/trig、text shape、TextRenderer build与GPU buffer create均为0；surface resize/atlas/font generation显式失效。
- compiled presentation一次生成ordered visible projection、solid/image vertex ranges、text layout/batches和resource keys；damage只筛选或patch受影响ranges。
- solid/image使用有界可增长persistent upload buffers；同一vertex payload不再经过item/layer/global三份owned Vec。
- 明确surface/cache唯一权威写入策略；FullRedraw与DamagePatch不再把同一plan完整record两次，retained内容、direct surface alpha/gamma与像素保持一致。
- 暴露command-sort、shape/glyph miss、alloc、vertex upload/new-buffer、pass/draw及surface/cache bytes counters；1/100/1k/10k规模给p50/p95与预算。
- UI pass接marker/timestamp；current-source Cargo、Softbuffer像素及RenderDoc pass/resource/像素对拍通过。

## 禁止临时方案

- 不得把每present的sort/shape/geometry rebuild无界投递worker；先用generation消除稳定工作，再对真实miss做有界并行。
- 不得通过打乱painter order、合并相交layers、跳过text/image或降低rounded segments来伪造性能收益。
- 不得保留editor与runtime两份geometry/text cache，或用test-double RHI wall-clock替代native WGPU证据。

## 修复结果与回传

Open state: `generation compiled presentation、stable text batch reuse、solid/image persistent vertex upload、single retained-cache authority以及command scan/batch/pass/draw/vertex/text/image preparation counters已落地；图像上传现于纹理创建前校验设备2D尺寸上限、checked bytes-per-row与payload长度，active image working set超过软预算时不再逐帧淘汰重建，subpixel裁剪UV按真实frame比例计算，非有限/非正矩形在stats与geometry层一致剔除；圆角quad/border现按原始frame生成并由独立clipping owner裁剪三角形，command clip/damage边界不再被错误重建为新圆角；待current-source managed Windows validation、完整规模预算、GPU/Softbuffer像素与RenderDoc pass/resource parity后回传`。
