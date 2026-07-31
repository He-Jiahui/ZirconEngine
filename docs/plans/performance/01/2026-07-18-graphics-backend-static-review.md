---
related_code:
  - zircon_runtime/src/graphics/backend
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
tests:
  - current graphics backend Rust source census 20 of 20 files reviewed, 2185 lines
  - cube mip readback source contract has one buffer, encoder, submit and wait
  - 4x4 three-mip pure staging layout regression test added
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2, scale counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics backend全目录静态审查（2026-07-18）

## 当前源覆盖

`zircon_runtime/src/graphics/backend/**`当前20/20个Rust文件、2,185行已逐文件静态阅读：backend/root/config/device/capture 6文件，offscreen target及构造5文件，buffer/texture/IBL readback 7文件，viewport surface 1文件，以及backend module root 1文件。当前21个单元测试也已阅读；它们只作为合同代码静态覆盖，未冒充Cargo执行通过。

## 发现与直接止损

`read_texture_rgba16float_cube_mip_chain`原来对每个face×mip分别创建MAP_READ buffer和encoder，分别submit、`map_async`并`poll(wait_indefinitely)`。标准128 face/8 mip为48次GPU/CPU同步；完整PMREM+SH9+IEM artifact连同SH9和IEM约55次。本轮按RED→GREEN增加4×4/3 mip纯布局测试，把所有region编码到一个row-aligned staging buffer，一次submit/map/wait后按face-major顺序剥离padding；源码合同确认cube函数buffer/encoder/submit/wait均为1且不再调用单region readback。标准路径48→1，完整artifact约55→3。

这只是同步stopgap。`read_texture_rgba`、buffer/region/3D readback和完整IBL sections仍会创建临时staging对象并阻塞调用线程；普通offscreen frame readback继续归`PERF-MVP-023`，IBL bake归`PERF-MVP-402`。最终owner采用Bevy本地参考的shared frame encoder、buffer pool、跨帧`map_async`/bounded completion，以及UE `FRHIGPUTextureReadback`的可复用staging+fence边界；不复制任何引擎的具体API。显式graphics-debugger capture stop的等待属于用户请求的capture边界，不按稳定帧热点误报。

`ViewportSurface::present_texture`每帧创建source bind group、独立encoder/render pass并额外submit；sampler/layout/shader/pipeline又按surface重复构造。新增`PERF-MVP-407`要求present进入主graph/encoder，pipeline按device+format共享，binding按source generation更新。

`OffscreenTarget`固定创建final/scene/GI/bloom/三张GBuffer/normal/AO/depth十张texture+view及cluster buffer，output或render extent任一变化就整包替换。新增`PERF-MVP-408`要求按compiled resource mask拆slot生命周期，minimal/headless不分配未使用的deferred/post/advanced资源。backend/config/request-device其余代码只在初始化或显式能力/capture边界执行，没有发现需要独立编号的每帧算法热点；35次测试backend初始化继续归`PERF-MVP-406`。

## 验收状态

局部文件已通过rustfmt、单batch源码合同和`git diff --check`。Cargo协调验证器在启动Cargo前解析协调器JSON失败，故未得到current-source编译/测试结果；F2 native/offscreen像素、不同WGPU backend、规模counter、GPU timestamp与DX12 RenderDoc也未完成。该目录只更新`pending.md`，不进入`review.md`。
