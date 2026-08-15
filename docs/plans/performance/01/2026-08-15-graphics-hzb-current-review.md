---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_workspace.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneTextureReductions.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DeferredShadingRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingMergedContext.cpp
tests:
  - current HZB culler slice 7 of 7 Rust files reviewed, 1660 lines, 17 tests
  - direct HZB build and indirect owner chain 6 files rechecked, 2007 lines, 10 tests
  - scoped rustfmt 7 of 7 clean
  - current-source Cargo, F2 counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics HZB current-source结构审查（2026-08-15）

## 当前范围与结论

`zircon_runtime/src/graphics/scene/scene_renderer/hzb/**`当前物理清单7/7个Rust文件：1,660行、1,519个非空行、17条测试，fingerprint为`EA732943B34B2C4D3FF2BA9C7B57ED4A7A409BF76F4644F1F9E78DA7F6931FD1`。另复核直接HZB build与indirect owner链6个文件、2,007行、10 tests，以及single/MSAA WGSL。7/7 HZB文件通过`rustfmt 1.8.0 --edition 2021 --check`。这些文件含其他会话未提交改动，本轮未修改生产源码。

当前实现有三项应保留的进展：没有candidate args时整项早退；diagnostics readback默认关闭、只在显式开启且实际dispatch后进入最多4帧的异步队列；`HzbOcclusionParamsWorkspace`和64项LRU bind-group cache让稳定workspace identity不再逐帧创建参数buffer/bind group，args count不变时参数upload为0。

参数表本身没有淘汰，但当前所有权不构成无界泄漏。一个`SceneRendererCore`同时持有一个固定9-phase的`MeshIndirectDrawWorkspace`和一个`HzbOcclusionCuller`，HZB只读取opaque、alpha-mask、advanced-PBR-opaque、velocity四个workspace；renderer销毁时二者共同销毁。因此当前最大参数entry基数是4/renderer，而不是随frame增长。后续若引入动态workspace注册，必须同时新增retire/generation合同，不能把当前固定所有权误报成现有瓶颈。

## P0：声明3个reduction batch，产品仍录制11/12个mip pass

`HzbBuildPlan`声明`MAX_MIPS_PER_REDUCE_PASS=4`。1923x1081得到1024x1024、11 mip、`reduce_pass_count=3`；4K得到2048x2048、12 mip、3批。但`computed_resources.rs`完全忽略`reduce_pass_count`，按`0..mip_count`逐mip创建source/target view并调用executor。每次调用都copy 16-byte params、解析binding、创建bind group、开启独立compute pass并dispatch。稳定1923x1081因此仍约21个mip views、11个bind groups、11个buffer copies、11个compute passes和1个每帧upload buffer；4K为12组。

当前shader每个输出texel固定读取4个parent texels并写1个Rgba16Float texel。仅按源码logical payload计算，不含缓存、压缩、barrier和driver开销：

| HZB base | mips | logical texels | single-sample read+write | MSAA4 read+write |
|---|---:|---:|---:|---:|
| 1024x1024 | 11 | 1,398,101 | 37.33 MiB/frame | 85.33 MiB/frame |
| 2048x2048 | 12 | 5,592,405 | 149.33 MiB/frame | 341.33 MiB/frame |

4K MSAA4的源码logical payload为20.00 GiB/s@60，仅用于确定测量优先级，不能冒充显存总线或GPU耗时。四mip batch若在workgroup内复用中间结果，理论上还可减少后续mip的global parent loads，但具体收益必须由目标kernel、timestamp和capture证明，计划不预填收益数字。

Unreal `SceneTextureReductions.cpp:164-174`以`FHZBBuildCS::kMaxMipBatchSize`选择批宽；`201-216`为一批mip绑定UAV数组；`246-268`把实际mip数写入shader permutation并只添加一个compute pass；`321-359`首批从depth读取、后续批只读取上批末mip。Zircon已有同构的四mip计划却没有执行层闭环，因此PERF-MVP-367继续是P0结构问题，不是微优化。

## P0：四个phase仍各自clear和compute

有candidate时，culler遍历固定四个HZB phase。每个phase先对visible-index、draw-count、compacted-args三个buffer执行clear，再锁参数workspace、锁bind cache、开启独立compute pass并dispatch。四phase全开时静态上限为12个buffer clear commands和4个compute passes；report已记录phase/groups、params create/upload、bind-group create，但没有clear command/bytes和compute-pass count。

PERF-MVP-376应在已有grow-only workspace基础上继续收敛到以`(camera, sampled HZB identity)`为bin的共享compaction artifact：phase只提供range/header，统一清理packed outputs并在一个compute pass/dispatch处理多个phase，replay仍消费各phase range。Unreal `InstanceCullingManager.cpp:17-21,55-96`显式允许batched `BuildRenderingCommands`并按HZB分bin；`InstanceCullingMergedContext.cpp:30-44,56-145,171-215`先合并各context的batch、indirect args和compaction offsets，再执行统一GPU处理。Zircon不应把render phase边界直接放大为同一HZB下的独立GPU pass。

## 测试漂移

`hzb_occlusion_uploads_phase_params_in_encoder_order`仍要求production source包含`zircon-hzb-occlusion-cull-params-upload`和`encoder.copy_buffer_to_buffer(`，而当前production两项均为false；新实现明确改用workspace buffer和`queue.write_buffer`。该测试若被执行将确定失败，且断言方向与稳定帧upload=0的性能合同相反。应由当前HZB源码所有者把它改成行为/counter门：首次每active phase create/upload各1、稳定相同args count为0、changed count只upload 16 bytes、workspace identity稳定且resource revision变化只重建受影响bind group。不能恢复旧staging-copy路径来迎合测试。

## 实施与验收

1. Render04完成PERF-MVP-367的1-4 mip batch kernel、UAV array/permutation、首批MSAA depth与后续普通HZB source；Render01/PERF-MVP-366提供physical backing的mip-view和batch bind bundle，Render17记录真实pass/dispatch/view/bind/copy/object counters。
2. Render03/04完成PERF-MVP-376的phase range/header和按HZB identity合批，保持opaque/alpha/advanced/velocity replay语义；diagnostics继续归PERF-MVP-373，history owner/copy归PERF-MVP-625。
3. 先修正stale源码形状测试并跑focused WGPU behavior；再恢复current F2产品，采集WPR CPU record scope、GPU timestamp和DX12 RenderDoc。当前没有可运行current-source产品二进制，旧capture不得用于验收。

矩阵：1x1/1080p/1923x1081/4K，MSAA1/4，HZB off/on，phases 0/1/4，args 0/1/1k/100k，history absent/stable/resize/cut，diagnostics off/on。硬门：1923x1081和4K的HZB build reduction passes/dispatches均<=3；warm upload-buffer/mip-view/bind-group create=0；四phase cull按同一HZB identity的clear commands<=3、compute pass/dispatch<=1；stable params create/upload/bind create=0；diagnostics off copy/map=0；mip texels、closest/furthest、occlusion result、indirect replay、AO/SSR消费者通过Cargo/F2 pixels/timestamp/RenderDoc。动态证据完成前本模块保留在`pending.md`，不进入`review.md`。
