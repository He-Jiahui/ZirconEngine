---
related_code:
  - zircon_runtime/crates/zr_rhi/src/capabilities.rs
  - zircon_runtime/crates/zr_rhi/src/descriptors.rs
  - zircon_runtime/crates/zr_rhi/src/descriptors/pipeline.rs
  - zircon_runtime/crates/zr_rhi/src/device.rs
  - zircon_runtime/crates/zr_rhi/src/device/handles.rs
  - zircon_runtime/crates/zr_rhi/src/lib.rs
  - zircon_runtime/crates/zr_rhi/src/ui_surface.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_resource/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHICommandList.h
tests:
  - zircon_runtime/crates/zr_rhi/src/tests
  - current-source Windows zircon_runtime RHI tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# RHI neutral contract生产面逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/crates/zr_rhi/src/{capabilities.rs,descriptors.rs,descriptors/pipeline.rs,device.rs,device/handles.rs,lib.rs,ui_surface.rs}`生产面当前源 **7/7** 个Rust文件、**2,695** 行已逐文件阅读。范围是backend capability、buffer/texture/sampler/pipeline descriptors、typed handles、command/device traits、UI surface DTO/stats与root factory；`zr_rhi/src/tests/**`及`zr_rhi_wgpu`其余实现另行验收。

## 性能结论

Capability queue使用小Vec，但有效枚举最多Graphics/Compute/Copy三项且仅设备初始化/显式查询，不值得增加复杂索引。descriptor/pipeline主要是plain data与const bit/size helpers；`TextureDesc::checked_storage_size_bytes`按mip levels循环并执行checked arithmetic，但合法mip上限由u32 extent约束，发生在资源规划/创建而非pixel/frame inner loop。

`RenderDevice`与`CommandList`是neutral trait；owned debug label、attachment Vec、descriptor clone的实际频率由backend实现/recording owner决定，不能在contract文件凭类型形状认定每帧重复。Bevy同样把buffer/texture/pipeline/cache拆成render-resource contracts，Unreal RHI command list把draw/dispatch/copy作为recorded command边界；Zircon的owner分层与成熟引擎一致，优化应落具体WGPU recorder、pool/cache和submit路径。

`UiSurfaceDrawList::stats`是本范围唯一直接F4热实现：它扫描commands并用BTreeSet去重upload keys，而WGPU batch/image prepare又重复扫描。该根因已归PERF-MVP-225与Render17交接，本证据不重复建项。其余neutral生产文件未发现锁、filesystem I/O、thread spawn、channel、busy wait或无界frame循环。

## 动态验收

待受管Cargo运行RHI contract/descriptor/command tests；对1/100/10k command lists记录recorded command allocation、descriptor lookup clone bytes和submit validation CPU，并将实际热点归到对应WGPU实现。GPU marker/timestamp、resource create/destroy/pool、copy/readback和RenderDoc资源证据必须随`rhi_wgpu`逐文件审查补齐。完成前保持`pending.md`，不进入`review.md`。
