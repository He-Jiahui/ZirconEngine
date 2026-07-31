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
  - zircon_runtime/src/text/atlas/bitmap_run/types.rs
  - zircon_runtime/src/text/atlas/render_plan.rs
  - zircon_runtime/src/text/atlas/render_batch.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/instance.rs
  - zircon_runtime/src/text/atlas/render_submission/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/instance_buffer.rs
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

Open state: `等待Text04联动Render17/PERF231回传single draw artifact、instance或indexed quad、shader viewport transform、generation buffer reuse、current-source Cargo与产品GPU/像素证据`。
