---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_sprite_render/src/texture_slice/computed_slices.rs
  - dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/mod.rs
tests:
  - sprite subtree 6 of 6 Rust files reviewed, 1155 current lines
  - existing per-sprite transform and fixed-append source guard inspected
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 pending behind FIFO
  - scale counters, F2 pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/sprite整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/sprite/**`当前6/6个Rust文件、1,155行，覆盖phase取数、九宫/平铺切片、CPU顶点展开、相邻纹理批次、统计、GPU buffer创建、render pass提交及单元/source-guard测试；并追到compiled-scene统计、2D stage、transparent overlay和OIT消费者。

当前工作树中的相邻改动已经把九宫/平铺从每slice重复构造同一transform和6-vertex临时Vec，收敛为每sprite一次transform、精确reserve和固定顶点append；对应source guard存在。该局部修复有效，但不改变剩余的跨消费者重复prepare根因，本切片未覆盖或重写这组外部改动。

## P0瓶颈与路由

- `build_sprite_vertices`每stage先收集phase index Vec；phase无item时再全扫sprites构造fallback Vec。每个命中sprite仍单独持有一份vertex Vec，随后`prepare_sprite_draw_batches`按相邻texture把这些Vec移动/合并成第二层batch storage。
- `prepare_sprite_queue_stats`按所有active stages完整执行`build_sprite_vertices + prepare_sprite_draw_batches`，真实`SpriteRenderer::record`对同一stage再次完整执行。统计因此复制真实prepare的排序后访问、slice展开、顶点物化和批组织，不是O(1)消费已有report。
- 2D renderer为每个相邻纹理batch执行`create_buffer_init`并开启独立render pass；transparent overlay和OIT又各自调用`build_sprite_vertices(Transparent3d)`，且按sprite分别创建vertex buffer，OIT还按sprite创建texture bind group。普通透明、overlay与OIT形成多套CPU/GPU prepare owner。
- PERF-MVP-337与Render14/17必须硬切为generation-owned `PreparedSpriteArtifact`：phase ranges、slice/instance ranges、batch ranges、统计和GPU arena由一次prepare产出；2D、mixed transparent与OIT只消费同一identity。稳定generation的slice/vertex rebuild、stats额外build、buffer/bind-group create与upload均为0，pass按phase而非batch增长。

参考Bevy `ComputedTextureSlices`把切片计算绑定到sprite/image变化，以及tilemap chunk的缓存/增量重建边界；不在frame record路径增加局部memoization或第二套cache。

## 验收

按sprites 0/1/100/10k、slices 1/9/1k、phases 1/4、batches 1/100/10k、2D/Transparent3d/OIT、stable/1% changed记录phase/fallback visits、index/vertex/batch alloc与bytes、matrix/slice/vertex builds、stats extra work、upload bytes、buffer/bind-group/pass/draw数及CPU/GPU p50/p95/p99。当前matrix不超过每sprite一次build；最终stable generation的artifact rebuild/upload/GPU object create=0，stats额外vertex build=0，普通透明与OIT prepared identity一致。current-source Cargo、F2九宫/平铺/透明顺序像素、timestamp与DX12 RenderDoc全部通过前留在`pending.md`，不进入`review.md`。
