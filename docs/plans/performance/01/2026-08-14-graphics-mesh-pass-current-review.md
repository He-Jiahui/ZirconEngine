---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/assign_execution_owned_indirect_args.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommandStats.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PrimitiveSceneInfo.cpp
tests:
  - current mesh_pass slice 28 of 28 Rust files reviewed, 6931 lines, 74 inline tests
  - scoped rustfmt 28 of 28 clean
  - persistent indirect workspace source tests require first prepare 5 buffers and stable second prepare 0 creates, 0 uploads
  - current-source Windows Cargo, F2 counters, WPR, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics mesh pass current-source结构审查（2026-08-14）

## 当前范围与结论修正

`zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/**`当前物理清单28/28个Rust文件：6,931行、6,270个非空行、74条内联测试，fingerprint为`2481EE7186ABF02CEA04B363C84D14F91381B988DA2B17F06E095DFE6C8325CC`。相对2026-07-18旧报告新增indirect plan、persistent workspace、dirty-range upload及其测试，因此旧23文件结论不能继续作为current source事实。本轮逐文件复读全部28文件及render/HZB直接consumer；28/28通过`rustfmt 1.8.0 --edition 2021 --check --config skip_children=true`。17个tracked modified和5个untracked产品文件属于其他会话，本轮未改生产代码。

两条旧结论已经被源码修正。第一，HZB diagnostics默认关闭，`render.rs:530-569`只有显式启用且确有dispatch才请求readback；`hzb_occlusion_culler.rs:95-175,371-449`使用最多4帧的共享异步队列，满时drop，不再在产品提交路径`wait_indefinitely`。第二，stats不再第二次构造indirect batcher：`MeshPassIndirectDrawPlans`同时封存execution plan与stats，render只build一次。后续不得继续按“无条件同步readback”或“stats重复batch build”实施。

正向变化还包括每个indirect phase的grow-only persistent workspace。当前测试要求首次prepare创建5个buffer，完全稳定的第二次prepare创建0、上传0、identity不变；单个args变化只上传20 bytes的一段。replay会缓存pipeline、material、GPUScene和geometry状态，并支持multi-draw/count路径。这些行为应保留。

## P0：并行边界仍把重工作留在主线程

生产render只要存在compute task pool就调用parallel builder。该builder先把所有owned `MeshBatchRef`收进Vec并按`source_draw_index`排序；并行门槛仅为2个batch。随后`has_duplicate_cache_keys`为四个phase建立`HashSet`并全扫batch，任一重复key会在已经collect/sort/hash后整帧退回串行。

更关键的是`prepare_batch_plan`在`task_pool.install`之前串行执行，包含command cache lookup/mutation、phase spec选择和pipeline variant resolve。worker只执行末端`build_prepared_batch_chunk`：每batch已有一张容量6的commands Vec，worker再建commands/cache-stores Vec、clone待缓存command，最后主线程逐chunk合并。也就是说当前并行的是较窄的对象物化尾部，而不是命令生成、排序、合并的主要权威工作；2-draw threshold还可能使task/Vec开销大于收益。

Render02/Runtime11应把generation-owned visible phase ranges、cache identity与已编译pipeline handle作为输入；经过实测阈值后，worker直接构建独立phase range和local arena，主线程按range做确定性linear merge。cache/pipeline owner不能依赖并行共享可变表，重复cache key应在generation编译期规范化，不能让一项重复触发整帧串行fallback。

## P0：command arena仍反复分相、排序和clone

`MeshPassCommandBuffers::from_command_list`每次新建10张Vec，搬移全部command，再对10个phase分别sort。render先取得prebuilt buffers，再构造residual buffers，最后`extend`固定调用10次`append_command_list`；该函数即使source为空也执行extend后全量sort。full static cache hit因此仍会clone cached `MeshDrawCommand`、分相并重排已有有序phase。

`CachedMeshDrawCommands::lookup_status`命中返回完整`MeshDrawCommand::clone`，而command拥有PipelineKey、geometry/bind handles与可选GPU buffers；`retain_generation`又在每帧对整张HashMap执行retain。静态稳定帧的缓存维护仍为O(N)，没有做到generation复用和dirty/remove-proportional。

目标仍是PERF-MVP-382/383：单一phase-owned command arena与range table由scene/mesh generation持有；static hit挂shared range/handle，residual按affected phase生成，有序range只做linear merge。删除每帧全cache retain，改为scene removal/dirty command驱动的精确失效或分桶渐进sweep。

## P0：GPU buffer稳定不等于indirect artifact稳定

每帧仍对9个indirect-capable phase调用`IndirectDrawBatcher::build`和compaction-plan构造。batcher逐command重建wide key，包含phase、kind、variant、完整PipelineKey、geometry/material/GPUScene ids；PipelineKey在key构造时clone，新batch又clone一次。`IndirectDrawBatch.pipeline_key`当前除构造与测试外没有消费点。args、batch ranges和metadata CPU Vec仍逐帧重建，dirty uploader在stable frame也必须线性比较整段内容；碎片变化会产生多次`queue.write_buffer`。

workspace prepare后，render无条件调用`attach_visible_remap_scene_bind_groups`，为最多9个execution逐帧调用`GpuScene::create_scene_bind_group_for_visible_instance_remap`。这些ephemeral execution已带稳定resource identity，却没有按workspace id/resource revision/GPUScene layout generation缓存bind group。更上游的execution-owned mesh indirect args路径仍逐帧扫描draw、创建args GPU buffer、逐draw copy并把Arc写回draw，与persistent phase workspace形成两套args ownership。

Render03/04应让compiled command generation同时拥有batch ranges、args、compaction metadata和visible-remap binding identity；stable generation复用CPU plan与GPU workspace，1% change仅重建affected ranges。wide key改为编译期state bucket id；移除未消费字段。execution-owned原始args与phase args必须收敛为一个allocator/upload plan，bind group按resource identity精确失效。

## 统计可信度与工具门禁

`stats_with_indirect_plan`不再重复batch build，但仍重新扫描10张command list统计kind/instances。更直接的测量缺陷是`MeshDrawCommandReplayer::draw_indexed`无条件增加`direct_draw_call_count`，而其下层`record_indexed_draw`同时支持`IndexedIndirect`；indirect batcher保留的fallback indirect command可能被计为direct。这会污染优化前后draw-path比例，Render17应先以draw-args kind修正分类，并让默认关闭的统计从sealed artifact读取；UE `MeshDrawCommandStats.cpp:45-52,249-260,465-474`同样只在CVar/dump/CSV启用时建立统计数据。

## Unreal Engine本地源码依据

- `MeshDrawCommands.cpp:1016-1174`明确把dynamic mesh command generation、view override、sort、InstanceCulling setup和stats作为一个parallel pass setup task，而不是只并行最终command clone。
- `MeshDrawCommands.cpp:1407-1475`先按MaxNumDraws reserve arena，再由threading-for-performance、render-thread和CVar共同决定并行；`1707-1725,1803-1818`又用worker数和`MinDrawsPerCommandList`计算任务数。这直接否定Zircon固定2 batches门槛。
- `MeshDrawCommands.cpp:1165-1173,1654-1668`把有序visible commands交给`InstanceCullingContext`生成rendering commands，command与culling artifact拥有明确阶段边界；Zircon应收敛两套args owner，而非逐帧互相投影。
- `PrimitiveSceneInfo.cpp:572-650,1546-1599`以primitive add/remove/change维护cached mesh draw commands，支持dirty-driven cache lifetime；不要求stable frame全表retain。

## 目标结构与实施顺序

1. Render02先定义`MeshCommandGeneration`：phase arena、sorted ranges、compiled state bucket、cache revision和visibility generation是唯一权威，prebuilt/residual/fallback只发布range。
2. Render03把indirect batch/args/compaction/visible-remap资源作为generation artifact；一个CPU plan驱动execution与sealed stats，persistent GPU workspace只按revision增量更新。
3. Runtime11在cache/pipeline mutable准备移出worker前，先量出serial preparation、worker tail和merge占比；并行任务按eligible command数、worker预算和测得task overhead分块，低于阈值保持serial。
4. Render04把per-view visible range直接接到phase generation，避免不可见draw进入command准备；HZB只改visible/count/compacted输出，不回滚为CPU同步。
5. Render17补齐generation build、sort/merge、key/command clone、cache visits、CPU plan bytes、buffer/upload/bind-group及queue-age counters，并先修正direct/indirect统计分类。

## 动态验收矩阵与阻塞

矩阵：draws 0/1/2/32/1k/100k，phases 1/9/10，static hit 0/50/100%，stable/1/100% changed，duplicate cache key 0/1/100%，views 1/4/12，threads 1/2/8/64，GPU-driven/HZB/diagnostics off/on。记录main-thread prepare wall、worker wall/occupancy/task count、collect/sort/hash probes、phase allocations/sort comparisons、cache clone/retain visits、batch key/PipelineKey clones、CPU plan bytes、GPU creates/upload ranges/bind-group creates、draw-path stats、CPU p50/p95/p99、CSwitch/ReadyThread、GPU timestamp与energy。

硬门：stable generation command partition/sort/cache full-scan/CPU plan rebuild/GPU create/upload/bind-group create均为0；100% hit command/resource clone=0；1% changed工作近affected ranges；duplicate key不触发整帧串行；parallel threshold由测量导出且2/32 draw不回退；stats分类与RenderDoc draw事件一致；F2画面、direct/indirect/multi-draw/HZB parity通过。

当前只有源码清单、rustfmt和测试合同证据。最近managed Windows lib-test在843.4秒后因361个共享foreign编译错误结束，0 tests执行；当前源码没有可运行MVP二进制，现存`target/profiling/zircon_editor.exe`早于本轮源码，不能作为WPR、GPU timestamp、energy或RenderDoc样本。本记录留在`pending.md`，不进入`review.md`。
