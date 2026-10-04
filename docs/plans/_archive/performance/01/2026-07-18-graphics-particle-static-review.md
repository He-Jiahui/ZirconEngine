---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/particle
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/12-effects-particles.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraRendererSprites.cpp
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/draw.rs
tests:
  - particle subtree 19 of 19 Rust files reviewed, 1034 current lines
  - velocity graph ordering and LoadStore contract inspected
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 still not FIFO head
  - scale counters, F2 velocity and transparency pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/particle整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/particle/**`当前19/19个Rust文件、1,034行，覆盖颜色/速度CPU quad展开、previous identity匹配、顶点ABI、三条pipeline构造、depth/overlay/velocity提交与内嵌测试，并追到compiled graph的object→particle velocity顺序及Render12硬切计划。

## P0瓶颈与路由

- `ParticleRenderer::record`对同一current sprite列表分别调用depth-tested与overlay builder，执行两次layer/depth/size/color过滤；每个命中sprite在CPU计算sin/cos和6个完整world-space vertices。velocity又第三次扫描current并生成6份current+previous位置。稳定particle仍每frame全部重算。
- 三类结果每frame通过`create_buffer_init`创建新vertex buffer；颜色depth/overlay各开启独立LoadStore pass，velocity另开pass。renderer构造又无条件同步创建depth、overlay、velocity三条pipeline且descriptor cache均为None。
- velocity先调用`anonymous_stream_ambiguity_entities()`重建anonymous统计结构，再为previous建立`BTreeMap<identity, VecDeque>`并扫描current；同帧submission/stats已有另一套anonymous/previous索引。回链PERF-MVP-341，唯一`ParticleHistoryMatchReport`应同时向velocity发布matched previous range。
- compiled graph已断言`velocity-object`先写，`particle-velocity`固定LoadStore，因此空particle velocity直接返回不丢Clear语义；这里不新增修复。

新增PERF-MVP-396执行Render12既定FX-M2硬切：删除整个legacy CPU per-particle quad目录，以`ParticleSimOutput`/`BillboardInstanceData`为唯一实例artifact，vertex shader按`vertex_index`展开6角，CPU/GPU simulation共享persistent storage/indirect owner；velocity消费同一current/previous instance ranges。由于目标计划明确删除当前实现，本轮不增加一次性double-scan cache或临时arena。

参考UE Niagara renderer直接消费particle data buffer/indirect count，以及Fyrox的实例属性ABI；最终不保留CPU world-space quad作为第二渲染权威。

## 验收

按particles 0/1/1k/100k、depth:overlay 0:1/1:1/1:0、previous match 0/50/100%、anonymous/duplicate key、cameras 1/8、stable/1% changed、CPU/GPU sim记录current/previous visits、tree nodes、sin/cos、CPU vertex bytes、instance/dirty upload、buffer/pipeline/pass/draw、indirect args与CPU/GPU p50/p95/p99。最终CPU world-space quad vertices=0、每particle只上传实例delta、stable GPU object/create/upload=0、history index≤1/generation、颜色pass≤1 phase、velocity复用artifact、particle-off pipeline create=0。Cargo、F2透明/velocity/TAA像素、timestamp与DX12 RenderDoc通过前留在`pending.md`，不进入`review.md`。
