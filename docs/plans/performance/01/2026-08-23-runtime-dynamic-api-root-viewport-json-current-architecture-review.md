---
related_code:
  - zircon_runtime/src/dynamic_api/*.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/render/03-render-framework.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Viewport.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/Windows/WindowsD3D12Viewport.cpp
tests:
  - current dynamic_api root 9 of 9 Rust files and 7 tests reviewed
  - current-source Cargo and F2/F4 traces blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Runtime dynamic API root viewport/JSON当前架构复审（2026-08-23）

## 范围与当前性

已逐行复读`zircon_runtime/src/dynamic_api`根层 **9/9** 个Rust文件：`bounded_json.rs`、`camera_controller.rs`、`exports.rs`、`frame.rs`、`mod.rs`、`runtime_loop.rs`、`session.rs`、`shader_prewarm.rs`、`surface.rs`，共 **2,292行、78,216 B、7 tests**；逐文件manifest SHA256为`c285c59e1fe6289a561a0cdc7b74cab649a4e6de883d0e6170846308e86d48d3`。`frame.rs`包含本Session早先host-intent borrowed-page改动，其余根文件未接管或改写。

本切片沿生产链读到App resize/rebind/redraw、session extract/capture/present、manager registry resolution、WGPU viewport record/surface以及Unreal D3D12 viewport resize。`shader_prewarm.rs`只负责manifest/template组装和离线入口；实际磁盘、Naga/WGPU validation与execution budget属于后续`dynamic_api/shader_prewarm/**`切片，本报告不拿root组装成本冒充编译瓶颈。

## P0：resize被错误建模为viewport销毁/重建

App对每个不同`SurfaceResized`先发送runtime resize，再重新bind native surface。`RuntimeRenderBridge::ensure_viewport`只要像素尺寸变化就执行`destroy_viewport -> create_viewport`；destroy会删除viewport record、释放全部frame histories、删除viewport products并清除graphics debugger状态。随后bind又创建全新WGPU surface和`SurfaceBlitResources`，其中包含sampler、bind-group layout和render pipeline。连续窗口拖拽因此把每个尺寸事件放大为完整viewport生命周期、时序历史丢失及present pipeline重建，而不是单一surface/back-buffer resize。

底层`ViewportSurface::resize`已经能原位更新surface configuration，但`RenderFramework`没有viewport resize contract，dynamic bridge无法正确调用它。这是跨`core/framework/render`和`graphics/runtime/render_framework`的接口缺口，不能靠在`dynamic_api`延迟若干毫秒或缓存旧descriptor来修补。

Unreal依据是`D3D12Viewport.cpp:300-435`与`WindowsD3D12Viewport.cpp:213-344`：resize保留`FD3D12Viewport`身份，在明确flush/idle同步点释放并重建back buffers、调用`ResizeBuffers`，而不是销毁整个viewport owner；`ConditionalResetSwapChain`也只在无效/fullscreen状态下显式重置。Zircon应继承“稳定viewport owner + 显式resize同步 + 可观察资源重建”的结构，不复制D3D12 API。

计划要求：为`RenderFramework`加入显式`resize_viewport(handle, size)`；WGPU owner原位resize surface和size-dependent render targets，只失效确实依赖尺寸的history/product generation，保留viewport handle、surface target、调试订阅及尺寸无关pipeline；App一次resize事件只提交一次resize，不再次create surface/present pipeline。记录resize events/coalesced、viewport create/destroy、surface configure/create、pipeline create、history invalidation、GPU idle/wait、CPU p50/p95/p99/max及首个稳定帧延迟。

## P1：bounded JSON成功路径为2至3次全量遍历

入站`decode`先按4 KiB chunk扫描nesting，再用`JsonItemSeed`完整反序列化一次JSON图，最后再业务类型反序列化；成功路径为约`3N`字节/语法访问。图预解析的limit是`max_encoded_bytes + 1`，并非业务`max_items`，它只能以wire-byte ceiling防止极端值图；精确业务项限制仍在typed decode之后计算。当前输入上限为256 KiB或1 MiB、deadline 10/25 ms，所以它主要影响profile/plugin subscribe/operation/world query/watch/accessibility输入及恶意输入风暴，不是普通RGBA frame path。

该双/三遍结构同时存在于host foreign-output decoder，说明它是跨ABI防御策略而非本文件偶然重复。优化前必须用合法/畸形/深嵌套/宽图的1 B、256 KiB、1 MiB payload测量scan bytes、JSON values、alloc bytes、deadline checks与p95/p99；再评估使用单次typed deserializer中的depth/value budget visitor，或证明预解析带来的攻击面收益值得保留。不得删除安全pass后只比较happy-path均值。

出站`encode`为单次bounded writer，但accessibility/profile/world payload在序列化前另行遍历计算业务项数；这是准确业务配额所需，只有owner能随generation封存item-count summary时才可消除，归PERF-MVP-431/433的数据产物所有权收敛，不在ABI边界另建缓存。

## P1：帧/输入热路重复解析manager service

`RuntimeRenderBridge`保存`ManagerServiceHandle`，但每次capture/present/bind/unbind/drop都调用`resolve_manager_service`。已解析实例路径仍会取得core services锁、按name查表、校验index/generation/lifecycle、clone type-erased Arc、downcast，再clone内层render Arc；input event也采取同一模式。单次成本可能小，但pointer 240 Hz加present 60/120 Hz会形成共享registry锁流量。

这里不能直接永久缓存Arc，因为handle/generation的意图是检测module reactivation和stale service。Runtime07/10应明确session激活后manager lifetime：若session内不允许热替换，则construction resolve一次并持有typed Arc；若允许替换，则registry发布generation-stamped typed snapshot，consumer只在generation变化时重新resolve。验收记录1/60/120/240 Hz与1/8/64 sessions的registry read locks、lookup/downcast、Arc clones、lock wait/hold和stale-manager语义。

## 已排除与验收状态

- camera controller只在实际drag/scroll读取active camera的local transform，没有full SceneNode clone；resize相同尺寸已在App和controller双层no-op。
- FFI table/panic boundary、surface ABI validation、RGBA ownership转移未发现新的根层热路径算法问题。
- shader manifest固定6个pass并复用每pass source生成quality requests；quality去重是小集合`Vec::contains`，当前不作为MVP瓶颈。实际shader validation/compile并发与cache命中率需在子目录复审。
- 本切片仅静态完成；受管current-source Cargo不可执行，WPR/Tracy/allocator/功耗与RenderDoc产品帧均无新证据。动态门通过前继续留在`pending.md`，不得迁入`review.md`，也不得声明耗时或功耗已接近Unreal。
