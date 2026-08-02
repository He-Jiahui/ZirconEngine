---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: glyph-atlas-draw-vertex-duplication
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/atlas/render_batch.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/instance.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/tests.rs
  - zircon_runtime/src/text/atlas/shaders/glyph_atlas_pipeline.wgsl
  - zircon_runtime/src/text/atlas/render_submission/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/instance_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
---

# Glyph atlas draw双层六顶点物化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/atlas`当前源47/47 Rust文件，聚焦render key/batch/plan/GPU/submission剩余24文件
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 联动责任：GPU buffer/pipeline owner联动Render17，stable slot回链PERF-MVP-231。
- 交接原因：glyph slot到draw occurrence的数据形态属于Text04；最终vertex/instance buffer与shader viewport transform属于Render17，必须共同硬切而非增加第三个DTO。

## 失败现象与复现证据

PERF-MVP-244：run重复保存draw glyph；batch为每glyph物化6个像素空间vertex，GPU plan再复制并逐vertex除法生成6个52B NDC vertex。两层仅vertex为624 B/glyph，10k约6.24 MiB/帧，FG/BG颜色和layer各重复6次。

本轮已用RED→GREEN源码门禁为GPU plan vertices/batches/draw commands按已知精确数量reserve，消除增长reallocation；rustfmt和diff check通过。结构性双层vertex仍open。证据见`docs/plans/performance/01/2026-07-18-text-atlas-render-submission-static-review.md`。

## 最低共享层根因

render plan把quad当成六个完整CPU顶点，GPU plan又把viewport transform当CPU projection；slot occurrence、batch quad和GPU payload分别拥有同一几何/颜色信息，没有canonical draw artifact或generation复用。

## 架构修复验收

- run只保留slot occurrence及screen/clip identity；删除`draw_glyphs`与`glyphs[*].draw_glyph`双份所有权。
- 优先每glyph写一个instance（screen rect、UV rect、FG/BG、page layer/contract）并使用static quad/index；若backend约束不允许，至少4 vertices+6 shared indices。
- viewport pixel→NDC进入uniform/shader；CPU per-vertex viewport division为0，resize只更新uniform而不重写全部glyph geometry。
- ordered batch必须保留Alpha/Color/Subpixel painter顺序；不能为减少draw而跨overlap非法重排。
- stable layout/atlas generation复用compiled instance/vertex ranges和resizable GPU buffer；hover/clip/viewport变化只更新必要动态段。
- 1/100/1k/10k glyph记录slot occurrence、DTO/vertex writes与bytes、alloc/realloc、division、upload、draw和CPU p50/p95；中间6-vertex层=0。
- current-source Cargo、clip/UV/padding/subpixel background/face generation、Softbuffer/WGPU/RenderDoc像素与draw parity通过。

## 禁止临时方案

- 不得只继续reserve Vec并宣称完成；双层六顶点与每帧重建必须删除。
- 不得把两个RGBA颜色继续重复到每个corner；instance或压缩vertex contract应按glyph保存一次。
- 不得仅缓存NDC vertices而在viewport resize全量重建；viewport transform应由uniform处理。
- 不得跨不同render contract/page的overlap glyph重排破坏painter order。

## 修复结果与回传

Open state: 非验收实现与二次静态审查已完成；仅等待 managed current-source Cargo、ignored 规模报告和真实 WGPU 产品像素验收。成功回执前保持 `open / implementation_complete / resolving_failure / managed_validation_pending`，不写成 blocked，也不以旧截图关闭。

- 2026-08-01 已删除两层 `GlyphAtlasGpuVertex`/CPU NDC 路径。clipped draw occurrence 只投影为每 glyph 一个 68 B `GlyphAtlasGpuInstance`；固定六角点由 WGSL `vertex_index` 展开，viewport pixel 到 NDC 只读取 16 B vertex uniform，CPU viewport division 为 0。
- WGPU owner 使用 `VertexStepMode::Instance` 与 `draw(0..6, instance_range)`；仅合并相邻且 page/render-contract 相同的 batch，Alpha/Color/Subpixel 的非相邻 painter order 不被重排。
- instance buffer 以至少 4 KiB 的二次幂容量持久复用；稳定或缩小帧只写有效的 `68N` bytes，容量不足才重建，prepare report 显式记录 capacity/reallocation。显式 idle 释放历史 mixed-storage draw buffers。
- 新增 1/100/1k/10k 规模门禁：严格锁定 `N` slot occurrences、`N` instances、`0` CPU quad vertices、相邻同合同场景 `1` draw、`68N` instance bytes，并锁定同容量稳态 reallocation 为 0。ignored `render_text_atlas_gpu_plan_reports_scale_p50_p95` 用 31 samples 输出 p50/p95，不以机器时间阈值伪造正确性 gate。
- 2026-08-01 二次静态审查修正 p95 nearest-rank 下标与误导性的 batch helper 命名；当前 owner 拆分为 render-plan/instance/instance-buffer/state leaf，旧 vertex leaf 均已删除。scoped rustfmt 与 diff check 通过，未发现新的 actionable P0/P1/P2。
- 待 coordinator wakeup：执行 Text04 focused scale/instance-buffer tests、ignored p50/p95 exporter、upward renderer tests与 exact ignored WGPU product framebuffer；验证图只允许成功后写入 `docs/tests/runtime/text`，不得写入任意 target。
