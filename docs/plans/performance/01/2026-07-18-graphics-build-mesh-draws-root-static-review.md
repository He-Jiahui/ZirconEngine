---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_plugins/04-animation.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/material_bind_groups.rs
  - dev/bevy/crates/bevy_render/src/mesh/morph.rs
  - dev/bevy/crates/bevy_pbr/src/render/morph.rs
  - dev/bevy/crates/bevy_pbr/src/render/skin.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GPUSkinCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
tests:
  - build_mesh_draws subtree 33 of 33 Rust files reviewed, 6877 current lines
  - fixed single-raster-draw storage source guard RED then GREEN
  - fixed dense phase-input lookup source guard RED then GREEN
  - rustfmt and scoped git diff check passed
  - current-source Cargo, F2 counters and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics build_mesh_draws整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`build_mesh_draws/**`当前33/33个Rust文件、6,877行，覆盖pending draw扩展、material输入、phase ordering、command-cache提取、GPU Scene同步、morph/skinning、Virtual Geometry indirect/resident upload和最终`MeshDraw`创建。该模块位于render submission主线程，当前把多类本应按asset/scene/pose generation准备的工作压进每camera/frame：形变delta与weights全量重建、骨架拓扑/inverse-bind/CPU vertex skin重复、VG draw/page/buffer重建、动态mesh GPU资源创建，以及纯诊断二次扫描。

当前只完成静态审查、两项局部RED→GREEN止损和源码级校验；Windows current-source `cargo test -p zircon_runtime --lib` reservation `0e49ba8bd6574fa2b77191847df06961`已排队但consume返回`cargo_cpu_reservation_not_fifo_head`，尚未启动。F2规模counter、像素对拍、GPU timestamp和DX12 RenderDoc capture也未完成，因此不进入`review.md`。

## 已直接止损

1. `raster_draws_for_mesh`合同固定只返回一项，却原来为每mesh instance分配一项Vec；现改为栈上固定数组并删除恒假的empty分支，三处caller顺序不变。
2. material-adjusted phase queue投影原对每个queued mesh线性扫描`phase_inputs.iter().find`，形成O(M²)。现先构造一次`mesh_index -> first GeometryPhaseInput` dense表，再O(1)投影，保留重复input取首项的旧语义。源码门禁先RED后GREEN。
3. pending command cache 100% hit仍每draw分配phase/command小Vec并clone cached command；production root被活动Render02租约持有，本轮未越权修改，继续归PERF-MVP-382。

## P0瓶颈与责任

- PERF-MVP-381/382/384：pending draw后的诊断重扫、cache hit临时Vec/owned handle，以及material/palette bind group逐draw创建，分别由Render02/03/08/17收口为唯一generation artifact。
- PERF-MVP-385：morph target静态delta应由Runtime04导入/编译一次；Plugins04只发布weights generation；Render03持久保存delta与weight slots并做dirty upload。
- PERF-MVP-386：skeleton name/index、parent topology和inverse bind应由Plugins04编译一次。GPU路径只准备palette，不得先CPU skin全部vertex或clone primitive；CPU fallback进入有界worker。
- PERF-MVP-387：本轮只把phase-input lookup由O(M²)降为O(M)。Render09仍须提供per-view generation phase artifact，使stable queue物化、material lookup和sort为0。
- PERF-MVP-388：VG每帧clone draw/segment、新建5个GPU buffers并全量重建resident page payload。Render03/04应交付persistent page/segment/args allocator与dirty scatter upload，联动PERF-MVP-376。
- PERF-MVP-389：`Dynamic`与CPU-morphed source可在最终draw转换时调用`GpuMeshResource::from_asset`，按draw/camera创建并上传GPU mesh。Runtime04/Render03应发布content+device generation resident handle；真正变形走385/386的persistent deform buffers。

参考路由以Bevy分离morph target allocator、morph descriptors/weights和skin prepare为Rust/WGPU基线；以UE GPUScene默认dirty upload、pooled upload buffer、parallel threshold和GPU skin cache为大规模场景基线。只借鉴生命周期和测量策略，不复制引擎API。

## 验收

按meshes/draws 0/1/1k/100k、cameras 1/8、morph targets 0/1/64、bones 16/128/256、VG pages 0/1/1k、stable/1% changed/能力fallback矩阵记录frame-thread visits、HashMap/Vec/clone bytes、GPU object creates、upload bytes/ranges、queue age/drop及CPU/GPU p50/p95/p99。最终stable generation的phase sort、morph delta build/upload、skeleton compile/CPU vertex skin、VG map/buffer/upload、dynamic mesh create均为0；changed工作近dirty delta。Cargo、F2像素/motion-vector、binding/indirect parity、timestamp与DX12 RenderDoc通过后才能移入`review.md`。
