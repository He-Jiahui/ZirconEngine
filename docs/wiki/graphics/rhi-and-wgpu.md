---
related_code:
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/crates/zr_rhi/src/lib.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_device.rs
  - zircon_runtime/crates/zr_rhi/src/submission.rs
  - zircon_runtime/crates/zr_rhi/src/surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/lib.rs
implementation_files:
  - zircon_runtime/crates/zr_rhi/src
  - zircon_runtime/crates/zr_rhi_wgpu/src/production
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/render/20-hybrid-raster-raytracing-rhi.md
tests:
  - zircon_runtime/crates/zr_rhi/src/tests
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests
doc_type: module-detail
---

# RHI 与 WGPU 后端

## RHI 的范围

`zr_rhi` 定义后端中立的设备、资源描述符、代际句柄、命令列表、提交、Surface、上传、诊断和 UI Surface 契约。`zircon_runtime::rhi` 是运行时的 curated facade；普通调用者优先从这里导入。

`zr_rhi_wgpu` 是当前生产实现。它同时包含中立 `RenderDevice` 的 WGPU 实现和若干 WGPU-native 产品接口。后者用于当前场景后端收敛过程，不应被当作多后端稳定接口。

## 资源与代际句柄

RHI 资源包括 Buffer、Texture、TextureView、Sampler、BindGroupLayout、BindGroup、ShaderModule、PipelineLayout 和 Pipeline。创建返回轻量 typed handle，不暴露 native WGPU 对象。

每个 handle 隐含 `DeviceId + DeviceGeneration + slot generation` 所有权。设备重建、handle 释放后重用或跨设备传递会被验证拒绝。正确模式是让持有者在设备代际变化时重新创建派生资源，而不是缓存裸索引。

```rust
use std::time::Duration;

use zircon_runtime::rhi::{BufferDesc, BufferUsage, RenderDevice};

fn allocate(device: &dyn RenderDevice) -> Result<(), zircon_runtime::rhi::RhiError> {
    let desc = BufferDesc::new(
        "frame-uniforms",
        256,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    );
    let buffer = device.create_buffer(&desc)?;
    let upload = device.write_buffer(buffer, 0, &[0; 64])?;
    device.flush_submissions()?;
    device.wait_for_submission(upload, Duration::from_secs(1))?;
    device.destroy_buffer(buffer)?;
    Ok(())
}
```

上传返回 `SubmissionTicket`，因此 `destroy_buffer` 是否立即允许取决于后端对 in-flight 引用的验证与延迟销毁实现。高频上传应使用 `BufferUploadBatch` / `TextureUploadBatch`，避免把每个小写入拆成独立逻辑提交。

## 描述符

- `BufferDesc`：大小与 `BufferUsage`。
- `TextureDesc`：维度、extent、format、mip、array layers、sample count、usage、residency 和 view formats。
- `TextureViewDesc`：base mip/layer、数量、aspect 和 view dimension。
- `SamplerDesc`：address/filter/mipmap/compare 等。
- `ShaderModuleDesc`：stage/entry/source。
- `PipelineDesc`：pipeline kind、shader、layout 和 raster state。

描述符在创建前必须满足格式、usage、对齐、采样和 binding 限制。`RenderBackendCaps` 与 `RenderDeviceLimits` 描述后端事实；不要硬编码桌面 GPU 上限。

## 命令与提交

`RenderDevice::create_command_list(queue, label)` 返回 `Box<dyn CommandList>`。可录制 copy、render pass、draw/dispatch 等中立命令；后端在执行前检查每条命令对应的 `RenderOperation` 是否已获能力 admission。

多个 command list 可组合为一个不可变 `RhiSubmissionPacket`，共享一个逻辑 `SubmissionTicket`：

1. `create_submission_packet`；
2. `enqueue_submission_packet`，状态进入 Accepted；
3. `flush_submissions`，把已接受 packet 按服务顺序提交；
4. `poll_submissions` 非阻塞推进完成；
5. `submission_status` 查询；必要时 `wait_for_submission(ticket, timeout)`。

`submit` / `submit_packet` 是便利函数，仍经过唯一 submission service。`cancel_submission` 只能取消尚未交给 native queue 的工作。

## SubmissionTicket 状态

票据带 device ID、generation、queue class 和单调 sequence。公开构造函数用于序列化/诊断，不构成伪造提交的权限；后端必须在已发行表中验证。终态历史有容量上限，长期诊断系统不应假设所有旧票据永久可查询。

## Surface 生命周期

Surface 使用严格 lease 模型：

1. `create_surface_session`；
2. `acquire_surface_frame` 获得最多一个短生命周期 `SurfaceFrameLease`；
3. 命令引用该 target 并产生同设备 submission ticket；
4. `present_surface_frame(lease, ticket)` 消费 lease，但不再次 queue submit；
5. 失败或图裁剪时 `discard_surface_frame`；
6. resize/outdated 时 `reconfigure_surface_session`，旧 session/lease 立即失效；
7. `destroy_surface_session` 收尾。

零尺寸窗口返回 typed non-renderable outcome。`SurfaceAcquireOutcome` 还区分 retry reason，调用者应跳过本帧并等待宿主事件，而非循环重试。

## WGPU 生产扩展

`WgpuRenderDevice` 提供 native recorder lease、native submission packet、同设备 UI context、GPU query/readback 和批量 native upload。这些 API 为当前产品路径服务，具有真实实现，但绑定 WGPU。需要跨后端的插件应使用 `RenderDevice` 和中立 descriptor；只有图形产品内部才应依赖 WGPU-native 类型。

## 诊断与内存

- `debug_instrumentation_status()`：debug marker/group、graphics debugger 等能力快照。
- `memory_snapshot()`：区分物理资源 retention 和 CPU upload staging。
- `transient_allocator_stats()`：瞬态 allocator 统计。
- diagnostic query/readback：以帧、预算和 submission ticket 限定，异步交付。
- WGPU `GpuPassTimer` / pipeline statistics：只有设备 feature 支持时可用。

## 当前限制

中立 RHI 及 WGPU 实现为**默认可用**；但产品 SceneRenderer 尚未完全切到 `RenderDevice`，多后端等价性也不是当前承诺。Acceleration structure、inline ray query、bindless 能力只能在 `RenderBackendCaps` 明确报告并由对应产品路径消费时使用。
