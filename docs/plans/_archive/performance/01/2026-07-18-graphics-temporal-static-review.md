---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalAA.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VelocityRendering.cpp
  - dev/bevy/crates/bevy_anti_alias/src/taa/mod.rs
  - dev/bevy/crates/bevy_pbr/src/prepass/mod.rs
tests:
  - temporal subtree 9 of 9 Rust files reviewed, 997 current lines
  - empty object velocity Load+Store pass source guard RED then GREEN
  - rustfmt check and scoped diff check passed
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 pending behind FIFO
  - scale counters, F2 velocity/TAA/cut pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/temporal整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/temporal/**`当前9/9个Rust文件、997行，覆盖TAA resolve/params/history flip、camera velocity、object velocity replay与camera-cut兼容性tests。产品热点不是参数数学，而是无效pass、每camera GPU binding重建和多个消费者重复构造camera matrices。

## 已直接止损

`record_velocity_object_to_resource`原在stream为空时仍先开启带velocity color与depth load的render pass，然后才返回。现在仅当attachment ops为Load+Store且stream为空时，在资源/context合同验证后、`begin_render_pass`前返回；该组合没有clear、discard或draw副作用。Clear/Discard空流仍按原路径录制，确保first-writer与graph store语义不变。源码门禁先RED后GREEN，新增Rust source guard，`rustfmt --check`与scoped diff通过。

## P0瓶颈与路由

- PERF-MVP-350：TAA无reactive command仍有独立mask clear，resolve每camera创建6-entry bind group；现有任务要求0-command共享black mask/零pass并按resource-view generation缓存resolve binding。
- PERF-MVP-346：`VelocityCameraParams::from_cameras`重新构建current/previous `ViewProjectionMatrixPair`并求inverse，与scene uniform/froxel/post重复。Render06须消费唯一prepared-camera matrices。
- PERF-MVP-368：camera velocity每frame创建2-entry bind group，object velocity有draw时重建forward shadow-receiver group；Render02/06须用resource-generation bundle与dynamic params ring。当前empty Load+Store object pass已为0。
- PERF-MVP-395：TAA history store本身为正确固定双槽，但外层history整包feature/resize owner和每frame view clone仍待拆分。

参考UE TemporalAA/VelocityRendering的view-state与velocity pass边界，以及Bevy TAA/prepass specialization；保留camera-cut阈值、projection兼容、unjittered matrices、history flip、object velocity clear/discard和像素语义。

## 验收

按TAA off/on、reactive commands 0/1/1k、object velocity draws 0/1/1k/100k、camera stable/cut/missing previous、cameras 1/8、720p/1080p/4K记录pass/draw/clear bytes、matrix builds/inverses、params writes、bind-group creates、history view clones与CPU/GPU p50/p95/p99。当前empty Load+Store object pass=0；最终0-reactive mask pass=0，stable matrices≤1/camera generation，TAA/velocity bind create=0且history slot按feature生成。current-source Cargo、F2 motion-vector/TAA/cut逐像素、timestamp与DX12 RenderDoc通过前留在`pending.md`。
