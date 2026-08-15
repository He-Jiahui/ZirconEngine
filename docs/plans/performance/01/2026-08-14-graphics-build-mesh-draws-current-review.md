---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_plugins/04-animation.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PrimitiveSceneInfo.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RendererScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GPUSkinCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SkeletalRenderGPUSkin.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SkeletalMeshLODRenderData.cpp
tests:
  - current build_mesh_draws slice 33 of 33 Rust files reviewed, 7394 lines, 6844 nonblank lines, 70 inline tests
  - all 33 current Rust files pass rustfmt 1.8.0 check
  - build order and feature-off allocation source gates passed
  - RenderDoc 1.44 and WPR available; Tracy unavailable
  - current-source Windows Cargo, F2 counters, WPR, GPU timestamps and RenderDoc capture blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics build_mesh_draws current-source结构审查（2026-08-14）

## 当前范围与证据身份

`zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/**`当前物理清单33/33个Rust文件，覆盖root/build 16个文件、mesh-instance子目录2个、command-cache extract子目录10个、skinning子目录1个和root draw转换4个文件；共7,394行、6,844个非空行、70条内联测试，fingerprint为`D919302CD0B8A5F4F96DFBFE315FEE44F4386D71C1044914002983E7CE0FC1B4`。33/33通过`rustfmt 1.8.0 --edition 2021 --check`。24个产品文件已有其他会话修改，本轮不越权改产品代码。

旧2026-07-18报告的两项局部止损仍成立：单raster draw不再用一项Vec，phase-input lookup已从O(M^2)降为dense O(M)。当前代码比旧基线增加517行，command-cache late extract、GPUScene history和staging也有正向变化，但frame构建的owner与顺序没有收敛。本轮没有把结构风险伪装成可局部修复的问题。

## P0：可见性晚于全部昂贵准备

`build_mesh_draws`的current顺序是：`phase_ordered_meshes` -> 全部mesh-instance pending draw展开 -> Virtual Geometry indirect plan -> VG resident upload -> Morph upload -> light pack/upload -> GPUScene sync -> `mesh_visibility_states` -> command-cache extract -> final `MeshDraw`。源码位置分别为`build.rs:134-188`，字符顺序门禁也得到5,574 < 6,152 < 6,336 < 6,556 < 7,235 < 7,663。phase ordering只按camera layer过滤，不消费frustum/shadow-view bitset。

因此同layer但主视图和全部shadow view均不可见的对象，仍先完成材质解析、模型/骨架读取、Morph/CPU skin、VG segment展开及buffer创建、GPUScene register/history和值比较。late visibility只允许满足静态command-cache合同的draw在extract阶段返回空commands；动态、skinned、reactive或material phase miss仍被物化。即使最终pass根据flags跳过draw，两个13-entry material bind groups和可选palette bind group已在`create_mesh_draw.rs:49-72`创建。

结构目标不是再加一个late `if`，而是让Render04/05发布`ViewId/ShadowSlot -> dense visible primitive slots/ranges`，成为mesh prepare入口。camera-neutral scene artifact由Render03持久化，view阶段只对visible或明确shadow-visible ranges准备动态draw。静态command cache只读scene generation identity，visibility裁剪不得等待material、deformation、VG或GPUScene工作完成。

## P0：蒙皮与Morph按primitive/frame重复资产级工作

model与direct mesh两条路径各自对`animation_poses`执行线性`iter().find`。model路径还在render preparation同步调用`load_model_asset`和`load_animation_skeleton_asset`，然后对每个primitive调用`prepare_skinned_model_primitive`。该函数每次重建bind-local Vec、pose-name HashMap、bind/posed world矩阵、每bone inverse-bind和joint Vec，再clone完整source primitive、CPU skin全部vertices并clone indices；即使GPU palette可用，CPU fallback也已支付。复杂度约为O(mesh instances x poses + skinned primitives x (bones + vertices))，而不是O(changed characters x active bones)。

Morph路径对每instance/current+previous active target逐vertex访问position/normal/tangent/color并固定输出4个delta rows，随后复制delta和weight Vec。`Arc`只让同一draw payload的segment clone按指针去重；不同instance对同一mesh重新生成的payload有不同identity，静态delta不能跨实例共享。active weight跨0还会改变target layout和后续slot偏移。

Runtime04/Plugins04应先发布compiled skeleton和morph artifacts：dense bone index、parent topo、inverse-bind、mesh-generation static delta ranges；animation只发布dense pose/weights generation。Render03以character slot生成一次palette，GPU path禁止CPU vertex skin和primitive clone；CPU fallback必须只在能力/ABI失败时进入有界worker。Morph静态delta按mesh generation驻留，instance只更新current/previous weight slots。

## P0：Virtual Geometry关闭路径仍按draw分配，开启路径每帧重建

`VirtualGeometryIndirectDrawPlan::empty(draw_count)`在feature disabled、无snapshot、无segment等常见路径仍建立5张长度为draw_count的`Vec<Option<_>>`。100k draws即500k个Option slots和5次heap allocation，虽然实际没有一个indirect draw。

开启路径先把execution segments clone进`BTreeMap`，再drain pending draws并按segment clone完整`PendingMeshDraw`。随后为args、submission、authority、draw-ref和segment每frame创建5个GPU buffer，并建立多张per-draw Option Vec。resident upload又重建page/payload HashMap和cluster words。stable generation没有persistent range/allocator owner，工作量仍接近O(draws + segments + pages)，而非O(changed pages/segments)。

Render03/04应以snapshot/page generation维护persistent page、segment、args和draw-ref arenas；feature off返回零长borrowed slices而不是N长度占位表；draw引用segment range/slot，不clone pending draw。只对新增、淘汰、状态和LOD变化页做dirty scatter upload，并与GPUScene统一upload plan。

## P0：phase/material/command cache边界仍重复工作

material-adjusted phase queue每camera/frame重新物化并排序。initial extend先取material common/revision/override，后续push路径再次解析material、texture set、uniform和pipeline key，tint判断又可再次查material。最终每个residual draw无条件准备sampler variants并创建custom/standard两套bind group。

command-cache full hit已能延迟构造residual `MeshDraw`，这是正向变化；但它发生在pending draw、VG/Morph/GPUScene和visibility之后，且仍为每draw建立phase/command小Vec并clone cached commands。正确边界是scene/material generation变化时编译static command与binding handles；per-view只投影visible cached ranges。动态miss才进入frame arena，不能让cache hit仅省最后一段工作。

## P1：诊断统计无条件重扫产品数据

`summarize_pending_mesh_command_cache_plan`和`prepared_mesh_queue_stats_for_pending_draws`在prepare后再次扫描全部pending draws。后者重算geometry/profile、查询GPUScene entry、构造并hash宽batch key；visibility state又新建main/shadow集合和输出map。Render17应让diagnostics off不执行unique/key工作，sealed generation直接消费command/VG/GPUScene主构建时的计数。

## Unreal Engine本地源码依据

- `SceneVisibility.cpp:3835-3851,4029-4044`先初始化primitive visibility map并用`ParallelFor`执行frustum cull；`4851-4867`只把visibility bitset中的primitive送入relevance，再按view mask gather动态mesh。Zircon当前顺序与此相反。
- `SceneVisibility.cpp:4141-4182`为dynamic gather提供render-thread或async task路径，`4185-4210`只对已有view mask的primitive调用`GetDynamicMeshElements`。并行化对象是已裁剪eligible集合，不是全scene错误工作。
- `PrimitiveSceneInfo.cpp:572-650,1546-1599`在static scene info加入或变化时缓存支持的mesh draw commands；`RendererScene.cpp:6632-6649`把cache作为可异步setup task。`MeshDrawCommands.cpp:606-699`再从visible dynamic elements和build requests生成visible commands。
- `GPUSkinCache.cpp:450-529,546-622`以bone buffer指针和revision复用双position buffer并持久保存skin-cache entry；Zircon不应在每primitive先CPU skin再决定使用GPU source。
- `SkeletalRenderGPUSkin.cpp:261-365,837-852,985-1014`持久拥有可双缓冲Morph vertex buffers，按revision和active morph变化决定更新；`SkeletalMeshLODRenderData.cpp:263-277`在LOD resource边界初始化静态Morph target render data。Zircon当前把静态delta展开留在per-instance render prepare，生命周期明显过晚。

## 目标算法与实施顺序

1. Render03/04/05先定义唯一scene generation、persistent primitive slots和per-view/per-shadow dense visibility ranges，frustum/layer/relevance在dynamic/material/deformation准备前完成。
2. Runtime04/Plugins04发布compiled mesh、morph和skeleton artifacts；pose/weights只携带dense slots与revision。Render03建立persistent deform/palette/morph arenas和current/previous epochs。
3. Render08/02按material/static-state generation编译binding与static command handles；visible ranges直接引用shared command ranges，dynamic miss使用可复用frame arena。
4. Render03/04把VG page/segment/args与GPUScene upload合成generation-owned persistent plan；feature off零分配，changed只更新dirty ranges。
5. Render09持久化per-view phase artifact；Render17把统计融合到主构建并加diagnostics gate。完成上述owner切换后，才依据WPR/Tracy阈值对eligible dynamic/deform pack启用有界task。

## 动态验收矩阵与阻塞

矩阵：draws/primitives 0/1/1k/100k/1M，visible 0/1/10/100%，static/dynamic/skinned 0/50/100%，cameras 1/2/8，shadow views 0/1/4/12，materials 1/100/1k，bones 16/64/256，vertices 1k/100k，morph targets 0/1/8/64，VG segments/pages 0/1/1k/100k，stable/transform-only/1% material/pose/weight/page changed。记录pre/post-cull visits、asset loads、material probes、palette/inverse/vertex visits、Morph delta rows/copy bytes、pending-draw clones、Option slots、GPU object creates/upload、cache clone/Vec alloc、main-thread CPU p50/p95/p99、CSwitch/ReadyThread、GPU timestamp和energy。

硬门：fully culled对象的material/deform/VG/GPUScene prepare=0；stable scene的static command/phase/material/VG artifact build=0；GPU skin path CPU vertex visits和primitive clone=0；skeleton/morph static artifact build不超过1/asset generation；feature-off VG Option slots和GPU creates=0；stable VG/morph upload=0且payload compare bytes=0；unique material bind create不随draw数增长；diagnostics off extra scan/key/set work=0；changed工作近dirty slots/ranges。

当前source-order、5张feature-off Option Vec和33文件格式门禁通过，但current-source动态验收不可成立：managed `zircon_app` build-only在324.2秒后因6个foreign `zircon_runtime`错误退出；focused `zircon_runtime` lib-test在843.4秒后以361个编译错误、1,520条warning退出，0 tests执行。RenderDoc 1.44和WPR可用，Tracy不可用；`target/profiling/zircon_editor.exe`时间为2026-08-10、SHA256为`56965EEC9D80CF9660145B743EC1CFD453EFAE25314F95FFE762FED8F1083888`，早于2026-08-14 current源码，不能用于current capture。故F2 counters、WPR/energy、GPU timestamp和RenderDoc保持pending，本记录不进入`review.md`。
