---
related_code:
  - zircon_runtime/src/core/framework/render/anti_alias/fallback.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mod.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mode.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/taa_quality.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/execute_taa_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/temporal_history_store.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_anti_alias/src/taa/mod.rs
tests:
  - anti_alias five of five Rust files reviewed
  - focused frame context TAA history graph and GPU execution callers traced
  - current-source Cargo, GPU counters, F2 traces and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render anti-alias逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/anti_alias/**`当前5/5个Rust文件、609行，并聚焦追踪frame submission、temporal feature descriptor、history store、reactive-mask writer与TAA resolve。framework合同全部为Copy enum/report与常数分支，resolve每帧成本为O(1)，无String/Vec/锁或独立CPU热点；双history texture也按size/format key持久复用。实际瓶颈位于graphics实现，不应把framework小函数微优化冒充产品收益。

## PERF-MVP-350：无reactive draw仍固定执行全屏mask clear pass

TAA graph固定声明`taa-reactive-mask-clear`、`taa-reactive-mask-mesh`、`taa-resolve`三pass。clear executor每个TAA帧都开启一次render pass并清写整张R8 mask；mesh executor在command stream为空时才直接返回。因此常见无authored reactive材质/透明物场景仍承担一次全屏attachment clear、pass begin/end与约width×height字节写入；有reactive draw时clear与mesh也被拆成两个pass。resolve还每帧创建包含六个binding的bind group，是否可按graph-resource generation安全复用尚无counter证据。

Render06/17应让0-command帧直接绑定共享black mask并跳过mask texture写入；有command时由唯一mesh writer以`clear_store`在同一pass清零并绘制，删除独立clear pass权威。bind group先记录create count与资源view generation；只有views/lifetime稳定时才缓存prepared bind group，resize、history flip、transient slot变化必须精确失效。Bevy TAA同样逐帧创建view-dependent bind group，说明不能仅凭源码把bind-group缓存宣称为确定收益；但其TAA没有Zircon额外的独立reactive clear pass。

## 验收要求

按720p/1080p/4K、reactive commands 0/1/100/10k、history cold/stable、resize/camera cut记录graph nodes、GPU render passes、mask attachment bytes、bind-group creates、CPU/GPU p50/p95：0-command mask pass=0且mask write bytes=0；有command mask pass=1而非2；TAA resolve仍恰好1 pass；若采用bind-group cache，stable resource generation create=0且resize/history slot切换不复用stale view。Off/FXAA/SMAA/MSAA/TAA fallback、history invalidation、reactive像素、透明/材质强度、camera cut与产品像素必须等价。current-source Cargo、timestamp/规模counter与RenderDoc未完成前，本批留在`pending.md`。
