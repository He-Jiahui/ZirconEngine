---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphPass.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
tests:
  - render pass execution context root and resource resolver two of two Rust files reviewed, 1058 lines
  - compiled pass declaration lifetime and access index regressions reviewed
  - partitioned viewport attachment semantics regressions reviewed
  - no separate source edit; owned metadata hard cut remains PERF-MVP-343
  - current-source Cargo scale trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics render-pass context与resource resolver逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`render_pass_execution_context.rs`与`resource_resolver.rs`当前2/2个Rust文件、1,058行，并追踪产品`execute_graph_stage`构造点。resolver已使用compiled graph的pass/declaration/lifetime/access索引，handle与name lookup不再扫描pass/resource Vec；partitioned viewport attachment策略也只做标量判断。剩余热点是context同时持有owned pass元数据副本和resolver handle，正是PERF-MVP-343的上游根因，本轮不重复编号。

## 当前有效基础

`RgResourceResolver`只保存`&CompiledRenderGraph + RenderPassId`，Copy返回；resource declaration/lifetime/pass/access均委托compiled索引。产品context存在resolver时，name access、attachment ops、read kind与required texture/buffer先校验compiled pass合同；owned `resources`线性fallback只服务无resolver构造路径。当前测试还锁定topological reorder后stable pass id仍命中正确access，说明后续hard cut可以保留现有handle语义。

## PERF-MVP-343补充：context不应再复制resolver已能提供的元数据

`RenderPassExecutionContext`仍公开拥有`pass_name: String`、`executor_id`、`dependencies: Vec`和`resources: Vec`。产品每pass从compiled pass clone这些字段，再附加指向同一compiled pass的resolver；执行结束后execution record又复制同一元数据。也就是说resolver已经提供唯一权威，但compatibility DTO仍在产品热路径形成第二份authority。

Render01的后续hard cut应让产品context以`CompiledPassHandle`/borrowed pass metadata为主，executor通过只读accessor取得name、queue、flags、dependencies/resources；无graph的测试/工具context可保留独立owned fixture builder，但不得迫使产品构造clone。`RenderPassExecutorId`同样应引用compiled/interned executor identity，错误文本按需格式化。Unreal RDG把pass对象和资源句柄保留在graph allocator中，execute阶段围绕`FRDGPass*`收集/执行，不为每次调用重建pass name与resource access DTO；Zircon可沿用已有resolver而无需新抽象。

按passes 16/64/256/1024、dependencies/resources 0/8/64 per pass记录context builds、pass/executor String clone bytes、dependency/resource Vec clone bytes、resolver lookups与CPU p95。产品resolver-backed路径要求context owned metadata clone=0、fallback Vec scan=0、compiled lookup保持O(1)；fixture路径保留现有错误、attachment/read/access与topological reorder语义。Cargo、F2产品trace与RenderDoc完成前保留在`pending.md`。
