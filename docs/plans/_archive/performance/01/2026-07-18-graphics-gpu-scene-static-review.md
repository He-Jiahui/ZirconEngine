---
related_code:
  - zircon_runtime/src/graphics/scene/gpu_scene
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_plugins/04-animation.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.h
  - dev/bevy/crates/bevy_pbr/src/render/gpu_preprocess.rs
  - dev/bevy/crates/bevy_pbr/src/meshlet/instance_manager.rs
tests:
  - complete gpu_scene directory fifteen of fifteen current Rust files reviewed, 3487 lines
  - full-upload dirty-discard regression contract RED then GREEN
  - stable morph upload regression contract RED then GREEN
  - stable virtual-geometry upload regression contract RED then GREEN
  - dirty-range in-place merge source contract passed
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 scale counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics GPUScene逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`graphics/scene/gpu_scene/**`当前15/15个Rust文件、3,487行，包括binding/ABI、primitive+instance+light owner、span allocator、dirty queue/upload、morph、virtual geometry、current/previous transform、morph weights、skinned palette/source history及测试。现有ABI offset、deferred free、dirty-range merge、stable primitive/instance/light upload、previous transform和palette双缓冲测试覆盖了基础合同；本记录只声明静态覆盖、源码合同和局部修复，不声明current-source动态验收。

## 本轮直接止损

primitive/instance buffer首次创建或扩容时必须全量上传CPU shadow，原实现随后仍调用dirty-range drain，为最终丢弃的范围执行sort、gap merge并分配upload-range `Vec`。现提供O(1) clear路径，全量上传后直接保留dirty队列容量并清空；增量路径的merge也改为在原dirty `Vec`内压缩，只分配最终提交ranges。

`upload_morph_buffers`原不比较内容，每次都clear/extend payload/delta/weight三份shadow并全量`queue.write_buffer`，内容完全相同也上传；`upload_virtual_geometry_resident_buffers`对page/cluster同样如此。现分别比较每个shadow，未变buffer不复制、不写入；稳定第二次调用的测试合同由`uploaded_bytes > 0`修正为0，变化的独立buffer仍保持全量写和原bind-group rebuild语义。

## PERF-MVP-405：GpuSceneDelta、持久arena与graph scatter upload

当前上游每frame遍历全部pending draws，构建live-key `HashSet`和返回entry `HashMap`，逐项register/hash lookup、primitive/instance完整比较并stage skin/morph history；frame末再扫描entries找stale key，并对palette/source/weight等多张map做retain/roll。previous transform每成功帧遍历所有live instances，morph weights把每个slice `to_vec`后又在roll时深clone全部weights。稳定场景虽已能做到primitive/instance GPU upload为0，但CPU仍是O(all visible instances + all history state)，不是scene generation delta。

Render03应接收scene/extract generation发布的added/changed/removed dense records，形成唯一`CompiledGpuSceneDelta`；primitive/instance/light/morph/VG/palette histories用dense slot+generation/epoch表达，previous/current以slot或buffer epoch翻转，禁止每帧复制整张HashMap/weight Vec。previous transform只处理新增或本帧motion-relevant changed instances。Render04的visible remap、indirect compaction与GPUScene条目共享stable slot，不再创建第二套identity map。

当前allocator对free spans做first-fit线性查找，pending free提交时对全部span排序合并；高churn和碎片下会退化。应改按size class/ordered interval索引的deferred epoch allocator，分配/释放近O(log F)，相邻合并无需每帧全排序，并以GPU completion epoch而非单一frame边界保证in-flight安全。morph/VG当前长度变化即按精确长度重建buffer，skinned实例各自持两份固定最大palette buffer并在stage时写全块；统一改为device级grow-only/suballocated arenas、active-prefix与dirty ranges，容量增长分档，缩短不重建，feature-off可选arena和fallback由device共享。

增量上传目前按merged range多次直接`queue.write_buffer`。Render01/03应把预尺寸staging/scatter upload作为主graph节点，大批量CPU pack超过阈值时有界并行，GPU copy/compute应用dirty records；小批量保留直接write的阈值路径。UE GPUScene以dirty filter跳过稳定全场景，按更新计数预尺寸scatter uploader，在阈值后`ParallelFor`打包，并用POT/16MiB块降低resize；Zircon采用相同原则但保持wgpu可移植实现。Bevy meshlet仍全实例遍历且源码TODO要求change events，因此不作为目标终态。

## 验收预算

按instances 0/1/1k/100k/1M、changed 0/1/1%/100%、free spans 0/1k/100k、churn 0/1%/50%、morph targets 0/8/64、weights 0/64/1k、VG pages 0/1k/1M、skinned joints 0/64/256、direct/scatter阈值两侧记录draw visits、HashSet/HashMap probes、history scans/deep-clone bytes、allocator probes/sort、dirty merge、CPU pack jobs、buffer/bind create、write/copy/dispatch count与bytes、capacity/fragmentation、CPU/GPU p95。当前stable primitive/instance/light/morph/VG upload=0，全量dirty discard sort/alloc=0，增量merge临时Vec=0；最终stable full-scene/history visits=0、changed工作近delta、allocator近O(log F)且无frame全排序、缩短不重建、palette bytes近active joints、large update走有界scatter。focused Cargo、F2规模counter、timestamp和DX12 RenderDoc完成前保留在`pending.md`，不进入`review.md`。

本轮协调验证器仍在Cargo启动前因`validate-matrix.ps1:187`的`ConvertFrom-Json`错误不可用，因此没有产生GPUScene focused test或package check结果；只保留静态与源码合同证据。
