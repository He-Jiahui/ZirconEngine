---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-07-18
summary_slug: render-graph-compile-analysis-scaling
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime/compiled_graph_cache/cache.rs
---

# RenderGraph compile analysis scaling

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/render_graph`当前源14/14 Rust文件及直接F2调用方
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 交接原因：依赖编译、culling、transient allocation、compiled cache与realtime graph owner均属于Render01；相关源码正由该计划活跃修改。

## 失败现象与复现证据

当前compile以HashSet clone传播manual reachability，多writer再做pair比较，每个transient read又全扫pass/access验证producer；culling按pass临时collect writes，allocation bucket/slot多处线性find。主pipeline虽有compiled cache，但immutable `graph.stats()`仍在每帧多遍扫描，realtime IBL有工作batch仍重新build/compile。性能切片已只在未冲突builder路径把每次resource access验证从O(resources)降为typed handle O(1)。

## 最低共享层根因

graph authoring/compile没有统一adjacency、resource access index和一次性compiled metadata，正确性检查各自重建局部集合并重复遍历；frame diagnostics与realtime IBL也未完整消费compiled generation事实。

## 架构修复验收

- 依赖、producer、WAW与root culling共享adjacency/resource index；不在nested pass loops clone HashSet或对每个read全图扫描。
- 16/64/256/1024 pass×resource的chain/fan-out/multi-writer/plugin-heavy基准记录compile p50/p95、edge/access visits、alloc bytes与复杂度斜率；目标接近O(P+E+A)，确需closure时使用有界bitset/索引并记录原因。
- `CompiledRenderGraphStats`在compile时生成，steady frame `stats()`为O(1)，pass/resource visited=0。
- realtime IBL按request/operation topology signature复用compiled graph；stable topology compile=0，变化generation只编译一次。
- transient bucket/slot计划避免bucket×resource与allocation×reservation线性find；保持descriptor bucket、readback lifetime、persistent/sparse bypass语义。
- current-source Cargo、RenderDoc graph dump/non-culled marker、pass/resource统计和产品像素对拍通过。

## 禁止临时方案

- 不得通过减少插件pass、关闭validation/culling/stats或只放宽测试图规模伪造改善。
- 不得把每帧compile/stat scan无界投递到worker；先以generation/cache消除稳定帧工作，再评估阈值并行。
- 不得覆盖当前Render01 transient-pool hard cutover或留下第二套graph metadata/cache事实源。

## 修复结果与回传

Open state: `非验收源码实现已完成；待 Render01 受管 current-source 规模、GPU/RenderDoc 与产品像素证据`。

- 依赖推导以 manual edge 为种子的单份去重 adjacency 与 per-resource access history 生成
  RAW/WAW/WAR；culling 反向遍历直接消费最终 dependency adjacency，且当前结构守卫禁止
  为每个 pass 收集临时 write list 或重建 manual WAW bitset closure。
- `CompiledRenderGraph` 在构造时预聚合 `CompiledRenderGraphStats`，稳态 `stats()` 仅返回
  缓存值；瞬态 allocation 先按完整 descriptor bucket 分组，再以有序 active/free slot
  集合分配，不做 bucket×resource 或 reservation×allocation 线性查找。
- 主 submission pipeline 的 `CompiledGraphCache` 已按 revision、compile fingerprint、选项与
  capability 摘要缓存；realtime IBL 另有固定上界的 topology variant cache，稳定 topology
  命中不会重新构建或编译 graph，并有 compile-count source guard。
- 尚未取得 16/64/256/1024 规模 p50/p95、current-source Cargo、真实 WGPU PNG 与 RenderDoc
  marker/dump 对拍。以上均由协调器管理，不能由静态源码结果替代或标记为 `fixed`。

## 2026-08-16 Structural Audit: external view provenance

The realtime IBL graph imports sampled and storage views for one physical cube
texture as distinct external handles. The compiler consequently cannot infer the
RAW/WAR/WAW hazards between those views; its `previous` pass chain is currently
standing in for the missing physical-allocation contract. Removing that chain
would be an invalid optimization because a source-mip read could be scheduled
without its producing storage-view write.

The staged repair is deliberately conservative:

1. Add an explicit external allocation-alias identity to RenderGraph imports.
   Same-identity views share one dependency history while their WGPU bindings
   remain distinct texture views.
2. Migrate realtime IBL source and PMREM view imports to that identity, then
   remove only the manual whole-plan predecessor chain. The compiler must retain
   the required RAW/WAR/WAW edges from the shared allocation history.
3. Keep shared-allocation tracking conservative for this MVP: different mips of
   one allocation remain ordered until a separately profiled subresource-range
   design proves that the extra concurrency survives WGPU recording and reduces
   measured GPU time. Do not claim async overlap from graph shape alone.
4. Make the compiled IBL artifact, cache hit/miss counters, and WGPU recording
   path consume the same topology before attempting cache or dispatch tuning.
   Validate this route with the managed 16/64/256/1024 compile fixture, product
   GPU timestamps, RenderDoc, and PNG evidence after the shared build lane is
   released.

## 2026-08-16 Structural Repair: feature resource version edges

`RenderFeatureResourceDescriptor` previously carried only a storage name and
access mode. Pipeline authoring therefore inferred ordering by finding a
unique writer for every resource, while post-process Bloom used a second
executor-id string table to move itself around exposure passes. Neither rule
identified the value a consumer intended to read, and terminal FXAA/SMAA could
be declared after the `output-transfer` pass that consumes
`FINAL_COMPOSITED`.

The replacement contract is `RenderFeatureResourceVersion`:

1. A token contains the resource name, resource kind, and producer **pass
   name**. It never uses a renderer executor id and does not merge storage
   identities.
2. A producer declares an ordinary write. A consumer opts into that exact value
   through `read_texture_from`, `read_buffer_from`, or `read_external_from`.
   Authoring builds edges only from these explicit values; it no longer guesses
   all readers of a unique writer.
3. The active descriptor set is catalogued before stage authoring. A producer
   in the same renderer stage creates a stable topological edge; a producer in
   an earlier declared stage is already ordered; a later-stage, absent,
   self-referential, mismatched, or cyclic token is a compile error.
4. The dynamic post-process routing now attaches the appropriate producer token
   when it selects TAA, depth-of-field, motion-blur, scene composite, blur,
   upscale, FXAA, or SMAA output. Bloom follows the selected scene-color value;
   it no longer has an executor-string rule relative to exposure.

Focused regressions cover a consumer declared before its producer, a missing
producer rejection, Bloom depending on motion-blur without an artificial
exposure dependency, and FXAA/SMAA preceding `output-transfer` through
`FINAL_COMPOSITED`. These source changes have passed formatting and static
contract checks only. Managed `zircon_runtime` compile and focused tests remain
required before this section is treated as validated; no timing, GPU, RenderDoc,
or PNG claim is implied.

## 2026-08-16 Structural Repair: compiled graph cache shape fingerprint

The cache key originally covered viewport, camera, selected feature options,
cookies, irradiance volumes, transmission, and late-forward material shape, but
omitted three `RenderPipelineAsset::compile` descriptor-filter inputs:

1. `advanced_lighting.oit.is_some()` gates OIT-only descriptors.
2. A selected texture camera matching a planar probe capture target gates
   planar-capture descriptors.
3. A current-view material reference to an active subsurface profile gates the
   deferred SSS descriptor.

Those omissions could return an incompatible compiled graph from a cache hit.
The frame fingerprint now records these three booleans. The subsurface value is
derived with the same bounded 16-slot active-mask semantics as the compile
path, but deliberately does not call `resolve_subsurface_profile_table`: that
helper materializes a `Vec`, which would introduce an allocation on every
stable cache-key lookup.

Focused cache-key regressions distinguish OIT presence, a matching planar probe
with an otherwise identical texture target, and an active subsurface profile.
This closes cache-key correctness only. The future managed profile must still
measure key-build work (including planar-probe/profile iteration), cache hit
rate, p50/p95 compile miss time, and steady-frame allocation count before any
claim of hot-path optimality or a decision to add extract-side O(1) shape
sideband data.

## 2026-08-16 Structural Repair: dynamic transmission value edge

The dynamically inserted transmission copy/draw pairs initially described the
draw read as an unversioned resource access. That left their required ordering
to the current insertion order even though the draw consumes the exact
`transmission.scene_copy[.N]` value produced in the same stage. Each draw now
uses `read_texture_from` with its matching copy pass name. The authoring
topology therefore retains the copy-to-draw edge if descriptor collection or
stable ordering changes, without restoring executor-id ordering rules.

The focused source regression asserts that this dynamic path carries an exact
version token and no longer creates an unversioned manual resource descriptor.
It is static-only evidence pending the shared managed Rust test lane; no
runtime timing, GPU, RenderDoc, or pixel-validity conclusion follows.

## 2026-08-16 Structural Audit: multi-writer dynamic feature stages

The half-resolution transparency insertion has a depth downsample producer,
an optional mesh writer, an optional particle writer, and a final composite
reader for the same half-resolution color/depth resources. A single
`RenderFeatureResourceVersion` identifies one produced value; it cannot model
the complete RAW/WAW chain when either optional writer is active. Assigning an
arbitrary one-producer token to the composite would make the ordering look
explicit while dropping a real hazard.

Unreal's `FRDGBuilder::AddCullingDependency` instead records resource producer
state per RHI pipeline and adds dependencies for every conflicting earlier
access, including the prior cross-pipeline read before a write. Zircon's lower
RenderGraph access history must retain that role. A future feature-authoring
extension needs either an explicit control dependency or a multi-input version
set before it can reorder this dynamic stage independently of insertion order.

No ordering change is made for half-resolution transparency in this slice.
Required future coverage is mesh-only, particle-only, mesh-plus-particle, and
both graphics/compute consumer variants, proving that every RAW/WAR/WAW edge
survives culling and WGPU submission. This is a structural correctness guard,
not a measured performance optimization.

## 2026-08-16 Cache-Hit Allocation Audit And Optimization Gate

Static inspection of `compile_submission_pipeline_with_options` found two
full `RenderPipelineCompileOptions` clones on its ordinary cache-hit path:
the submission path clones options to apply shader quality, then
`CompiledGraphCacheKey::from_inputs` clones them again for the `HashMap` key.
The type includes several `BTreeSet` fields, including
`disabled_plugin_features: BTreeSet<String>`, plus optional post-process and
IBL request descriptors. Non-empty sets can therefore allocate even when the
compiled graph itself is reused. This is a source-level risk, not a measured
claim about the current product's dominant CPU cost.

Before changing the cache representation, collect a managed Windows baseline
for a stable scene with both empty and non-empty compile-option sets:

1. Record per-frame cache hit/miss, key-build count, compile count, and CPU
   p50/p95 for submission preparation over at least 300 warmed frames.
2. Use an allocation-aware CPU profiler or a bounded test allocator probe to
   record allocations and bytes per stable hit; capture the caller stack to
   distinguish the submission clone from key construction and unrelated WGPU
   work.
3. Repeat after toggling OIT capacity fallback, shader quality, post-process
   stack, and an IBL request to establish miss correctness and transition cost.

Only if the data confirms material stable-hit allocation should the cache move
to a borrowed lookup probe plus an owned-on-miss key, or to a precomputed,
collision-safe compile-options identity owned by `ViewportRecordState`.
The latter must preserve full equality for all option fields and update
atomically with feature/capability changes; a lossy hash-only key is forbidden.
Compare the chosen design against the current fixed-capacity (16-entry)
`HashMap` using the same cache-hit and allocation measurements before claiming
an improvement.

## 2026-08-16 Structural Repair: production physical resource resolver

`RgResourceResolver` originally carried its physical resource table and its
texture/buffer lookup APIs only under `cfg(test)`. Production contexts instead
duplicated declaration validation followed by a direct
`RenderGraphExecutionResources` lookup. That split defeats the planned single
pass-scoped resolution boundary and makes a future executor bypass easier to
introduce.

The resolver now supports physical attachment in production. Attaching a GPU
context upgrades its resolver with the exact `RenderGraphExecutionResources`
reference, and the GPU facade performs the same upgrade for direct
construction. Common named texture and buffer lookups delegate through that
resolver; metadata-only contexts retain the existing resource-table fallback
because they intentionally have no physical table. Mip-specific view creation
remains in the GPU owner, but it continues to require resolver declaration
validation before accessing an owned backing.

The existing offscreen resolver regression now constructs a metadata resolver
and attaches the physical table through the production path. The GPU lookup
regression asserts that both its enclosing context and its GPU facade retain a
physical resolver before resolving declared resources. This is static-only
until the shared managed Rust test lane runs it; no WGPU submission, RenderDoc,
or product pixel claim follows.
