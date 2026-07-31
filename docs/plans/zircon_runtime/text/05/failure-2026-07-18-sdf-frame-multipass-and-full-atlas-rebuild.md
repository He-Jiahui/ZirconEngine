---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: sdf-frame-multipass-and-full-atlas-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
---

# SDF frame多轮准备与整页重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/sdf`当前源24/24 Rust文件及产品调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md`
- 联动责任：Text09负责cache/frame budget，Render17负责persistent GPU page/buffer与dirty upload；回链PERF-MVP-231/243/244。
- 交接原因：SDF prepare/failure/bake/atlas artifact owner属于Text05，GPU写入由Render17消费。

## 失败现象与复现证据

PERF-MVP-249：产品每帧先做generation-failure bake，再做fallback CPU preparation，renderer又重复CPU preparation并build atlas。单次prepare最多三次分配render scalar Vec。glyph cache hit深clone bitmap；atlas每帧分配/清零所有R8/RGBA pages、复制全部slot、全atlas扫描nonzero，随后上传并新建vertex buffer。

## 最低共享层根因

failure、advance/decoration、glyph bitmap、slot page与GPU plan没有共享一个generation artifact；cache只避免FDSM计算，没有避免bitmap ownership、page materialization和render submission重建。

## 架构修复验收

- 每frame generation对每个text/run只构建一次scalar/key/metrics/failure结果，fallback与renderer共享artifact；无重复`prepare_sdf_runs_cpu`。
- glyph cache返回Arc/slot handle，不按cache hit clone bitmap；key共享font/family/language identity，不按glyph分配String。
- atlas page跨帧驻留；stable frame page alloc/zero/copy/nonzero scan/upload=0，changed glyph只写dirty rect并增量更新report。
- GPU texture/vertex/instance buffer按capacity与generation复用；viewport只更新uniform，device loss显式重建。
- 1/100/10k glyph稳定300帧记录prepare pass、Vec/String alloc、bitmap clone bytes、page touched/upload、buffer create与CPU/GPU p50/p95；增长按O(delta)。
- fallback whole-batch重规划最多一次有界二阶段提交，failure与slot generation一致；SDF/MSDF/MTSDF/offline/dynamic、clip/effects、reload/device loss、current-source Cargo与像素通过。

## 禁止临时方案

- 不得只缓存FDSM输出但仍每帧clone bitmap并重建整页。
- 不得关闭nonzero report而保留其他整页扫描并宣称完成。
- 不得用无界page/glyph cache换取稳定帧0工作；bytes/eviction/age必须可观测。

## 修复结果与回传

Open state: `等待Text05联动Text09/Render17回传single compiled SDF frame、persistent dirty pages/buffers、规模counter、current-source Cargo与RenderDoc像素证据`。
