---
related_code:
  - zircon_runtime/src/text/atlas/raster_key
  - zircon_runtime/src/text/atlas/render_batch.rs
  - zircon_runtime/src/text/atlas/render_contract.rs
  - zircon_runtime/src/text/atlas/render_plan.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan
  - zircon_runtime/src/text/atlas/render_submission.rs
  - zircon_runtime/src/text/atlas/render_submission
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/zircon_runtime/render/17-ui-wgpu-surface-and-render-graph-integration.md
reference_sources:
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
tests:
  - source-level RED to GREEN exact GPU-plan reserve guard passed
  - rustfmt check and scoped diff check passed
  - current-source Windows zircon_runtime atlas render tests pending
  - 1/100/1000/10000 glyph CPU allocation and upload counters pending
  - WGPU/Softbuffer/RenderDoc draw and pixel parity pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text atlas render/submission逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/atlas/{raster_key/**,render_batch*,render_contract*,render_plan*,render_gpu_plan*,render_submission*}`剩余当前源24/24个Rust文件已逐文件阅读，覆盖raster identity、clip/UV/quad、batch、shader/blend contract、GPU vertex/pipeline/draw plan、retry frame state/driver/submission/report及全部测试。加上上一记录23文件，`zircon_runtime/src/text/atlas`当前源47/47文件静态审查完成。

## 现有回链

`GlyphRasterKey`拥有face/glyph/size/subpixel/format/hinting/smoothing/synthetic完整identity，但产品调用图只命中本模块测试；native source/run仍没有该key或slot引用，回链PERF-MVP-231。1px size bucket与Auto route没有进入产品，回链PERF-MVP-241。page/slot全量重建、retry source多轮clone和storage report重复规划继续回链PERF-MVP-231，不重复编号。

## PERF-MVP-244：draw DTO与六顶点展开重复物化

run同时保留`draw_glyphs: Vec<GlyphAtlasDrawGlyph>`与`glyphs[*].draw_glyph`；submission再从前者生成batch quads。每个quad保存6个`GlyphAtlasDrawVertex`，随后GPU plan对6个顶点逐个执行pixel→NDC除法并写第二份6个`GlyphAtlasGpuVertex`。GPU vertex固定52 B，前景/背景两个RGBA及page index在同glyph六个顶点中重复；两层vertex单glyph约624 B，10k glyph约6.24 MiB，尚未计run/batch/command和renderer buffer copy。

本轮以源码门禁先确认三类输出未reserve，再按`draw_plan.vertex_count/batches.len()`为vertices、batches、draw commands精确reserve，RED→GREEN、rustfmt/diff check通过。最终应消除中间quad vertex层和CPU NDC展开：优先一条instance保存screen rect、UV rect、FG/BG及layer，配static quad；至少也应采用Bevy UI/UE Slate每quad 4 vertices+6 indices。viewport尺寸通过uniform由shader转换，stable generation复用renderer buffer。

## PERF-MVP-245：backpressure实现存在但产品配置为unlimited

retry planner已支持due retry/new source两类数量预算、defer frame和telemetry，测试覆盖限流；但唯一产品入口`native_bitmap_atlas_retry_frame`调用`GlyphAtlasBitmapRetryFrameDriverConfig::with_defaults()`，其`backpressure_policy`固定`unlimited()`。因此page容量不足时，所有到期blocked source与全部新source在同帧进入run allocation、draw batch和GPU plan；再次失败后又用新Vec整体替换queue。没有queue count/bytes hard cap、CPU/staging预算、key dedup或age fairness。

Text04需从Text09全局frame budget取得retry/new quotas，并与PERF231 persistent slot结合，让容量压力只处理真正slot miss/evict。队列保存轻量key/generation/age，按old retry和new source公平调度；产品默认不得再命中unlimited。测试中手工custom policy通过不代表产品已受控。

## 参考引擎结论

Bevy UI为glyph quad写4个vertices和共享6 indices，并使用长期`RawBufferVec`；UE Slate text同样为每glyph写4个`FSlateVertex`再追加quad indices。两者都避免Zircon当前“6个像素顶点DTO再复制成6个NDC顶点”的双层结构。Bevy/UE的font atlas以glyph→atlas location持久映射，容量处理围绕持久页/显式flush，而不是每帧把全部visible source放进unlimited retry规划。

## 责任计划与验收

Text04分别收到`failure-2026-07-18-glyph-atlas-draw-vertex-duplication.md`与`failure-2026-07-18-bitmap-atlas-retry-budget-not-wired.md`，vertex/buffer联动Render17，frame budget联动Text09。1/100/1k/10k glyph记录DTO/vertex bytes、alloc/realloc、viewport divisions、upload及CPU；stable build/upload为0。max-pages=0/1的due/new风暴记录attempt/defer/queue bytes/age，任何帧不越预算且无饥饿。current-source Cargo与产品WGPU/Softbuffer/RenderDoc证据完成前，atlas 47/47仍保留pending。
