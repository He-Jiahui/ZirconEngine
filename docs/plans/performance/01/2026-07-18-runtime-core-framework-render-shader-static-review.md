---
related_code:
  - zircon_runtime/src/core/framework/render/shader
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache
  - zircon_runtime/src/graphics/shader/variant_cache
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
tests:
  - shader nineteen of nineteen Rust files reviewed
  - mesh batch variant runtime disk cache prewarm and IDE import callers traced
  - runtime dimension token allocation source guard RED to GREEN
  - disk cache include-hash clone source guard RED to GREEN
  - rustfmt and scoped git diff check passed
  - shader focused Cargo scale counters F2 trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render shader逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/shader/**`当前19/19个Rust文件、3,460行，包括production与tests，并追踪mesh pass variant resolution、同步磁盘cache/pipeline创建、shader prewarm manifest及asset/IDE include解析。两个无语义风险的分配已直接止损：runtime pass/quality维度命中不再每次构造`String`，disk cache key不再克隆全部include hash再做Blake3。其余四类根因涉及帧级variant所有权、异步pipeline状态机、prewarm schema与import合同，必须由责任计划收口，不能以局部微调替代。

## PERF-MVP-355：每batch/pass重复构造variant key与诊断字符串

`MeshPassBuildContext::pipeline_variant_id`对每个mesh batch和pass调用`MeshPipelineVariantRegistry::resolve_variant_for_geometry`。函数在`HashMap::get`之前完整构造owned key：复制`PipelineKey`、生成`ShaderVariantKey`并为固定`wgpu-runtime`分配`platform_token`。命中后`ShaderVariantMissReport`原又为pass、geometry、shading与quality四个BTree维度逐次创建字符串；frame末还深clone整份map进入`RenderStats`。本轮已让静态pass/quality token先借用查询、只在该frame首次出现时分配，但owned variant key、两个数值维度和report clone仍在热路径。

Render03/08/17应把compiled `MeshPipelineVariantId`纳入material/static-batch generation artifact，queue只引用dense id；registry用无分配borrowed probe或interned platform token，统计先写dense enum/id counters，sealed report/UI导出时才物化字符串。稳定material/scene generation不得重复resolve或重建dimension map。

## PERF-MVP-356：首遇variant在pass编码线程同步I/O、解压、写盘与pipeline创建

`ensure_pipeline_for_variant`由base scene pass直接调用。cache未命中GPU module时，它先同步组装整份WGSL，再由`ShaderVariantCacheDisk::lookup`执行`exists`、meta/WGSL两次`fs::read`、JSON解析与zstd解压；disk miss继续在同线程zstd压缩、pretty JSON、两次atomic write，随后同步`create_shader_module`与`create_render_pipeline`。因此冷启动、新材质、热重载或插件variant可把不可控I/O/driver compile尖峰塞入提交线程。

Render08/17应建立`Queued -> Loading/Compiling -> Ready/Error`状态机：asset/IO lane负责磁盘与解压，async compute/driver lane负责source assemble和pipeline创建；render只做O(1)状态读取并使用明确的skip/depth-only/error-material策略。Bevy的`PipelineCache`把descriptor排队、维护waiting集合，并在允许多线程的平台用`AsyncComputeTaskPool`创建pipeline；Zircon应采用同类非阻塞生命周期，同时保留自己的disk artifact与generation失效合同。

## PERF-MVP-357：prewarm按variant复制WGSL并串行重复hash/validate/write

`prewarm_requests_for_source_with_dimensions`在pass×quality×geometry笛卡尔积中把WGSL、include hashes、source label与版本字符串复制进每个request；manifest因此按variant数量重复保存相同大文本。执行器又串行遍历所有variant，逐个Naga/WGPU验证、构造disk key、压缩写盘；`record_written_cache_entry`还对同一WGSL每variant重新Blake3并克隆provenance字段。大材质集会形成variants×source bytes内存、序列化、hash与单worker瓶颈。

Render08联动Runtime04/17发布prewarm schema v2：唯一source/provenance表由content hash索引，variant只引用source id和维度；按内存预算以bounded worker queue并行assemble/validate/compress，disk publish保持原子且同cache key去重。报告按source聚合一次，不再次扫描WGSL。

## PERF-MVP-358：asset/IDE include解析重复扫描与临时行Vec

`ShaderTemplateInclude::new`及import/IDE生成对同一source先`wgsl_include_paths`、再`strip_wgsl_include_directives`，至少扫描两遍；strip还先收集`Vec<&str>`再join。IDE preview路径会再次resolve/strip，shader package ingest也重复调用。它不属于稳定渲染帧热点，但直接影响基础编辑器的shader保存、hot reload与大include图导入延迟。

Render08联动Runtime04/Editor09提供单遍parser产物（stripped source、dependencies、line mapping、content hash输入），由import、template registry与IDE preview共享generation-owned结果；依赖图只对changed module增量失效，禁止每个consumer重新扫描全文。

## 验收要求

PERF-MVP-355按batches 1/1k/100k、passes 1/6、stable/1% material changed记录variant resolves、key/String alloc、dimension materialization与report clone bytes：stable resolve/alloc=0，changed每唯一variant≤1。PERF-MVP-356按cold/warm variants 1/100/10k及disk hit/miss/corrupt记录frame-thread fs/decompress/compress/write/driver compile time、queue depth/latency与fallback draws：frame-thread阻塞工作全部=0，warm miss=0。PERF-MVP-357按sources 1/100、variants 1/1k/100k、WGSL 4KB/1MB记录manifest bytes、WGSL copies、hash visits、worker利用率与peak RSS：source text唯一存储、同source provenance hash=1、in-flight有界。PERF-MVP-358按4KB/1MB source与include depth 1/100/1k记录source scans、temporary line Vec、graph rebuild nodes：每changed source scan=1、stable scan=0。shader/material产品像素、cache restart/hot reload、插件variant、Cargo、F2与RenderDoc通过前，本模块留在`pending.md`。
