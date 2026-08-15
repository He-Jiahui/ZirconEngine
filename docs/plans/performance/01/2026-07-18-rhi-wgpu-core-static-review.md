---
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/bind_group_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/capabilities.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation/render_state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/command_list.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/lib.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/pipeline_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/render_pass_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/resource_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/mod.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/texture_copy.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHICommandList.h
tests:
  - zircon_runtime/crates/zr_rhi/src/tests
  - current-source Windows zircon_runtime RHI tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# RHI/WGPU non-UI core逐文件性能静态审查（2026-07-18）

## 范围与调用图

当前源12/12个Rust文件、3,074行已逐文件阅读，覆盖resource/pipeline/bind-group/render-pass/command validation、recorded command list、CPU resource state、copy execution与capability mapping。非测试调用图对`WgpuRenderDevice::{new_headless,new_with_surface_support}`为零；当前消费者全部位于`zircon_runtime/crates/zr_rhi/src/tests/**`，产品graphics backend与UI native presenter不经过该device。

因此这里的`Arc<Mutex<WgpuRenderDeviceState>>`、CPU `Vec<u8>`资源、submit校验+执行双遍、即时完成fence和同步readback是contract test double事实，不是当前F2/F4产品帧证据。`caps`却声明async copy/debugger capture且类型公开名为WGPU，若未来误接产品会形成全局串行、双份host memory和虚假fence语义；该边界已交接Render17。

## 逐文件性能结论

descriptor/resource/pipeline validators只在create或test submit触发；`BTreeSet`去重、bind-group layout/entry的nested find、render-state `BTreeMap`与attachment format字符串构造在当前小规模测试契约内可接受，不据此建产品热点。`transient_allocator_stats`会显式全扫buffer/texture表；command list无预留capacity；submit持全局锁完成validation与CPU copy。这些项目只在1/100/10k contract-command基准证明测试吞吐后再优化，禁止为了跑分快而删除错误校验。

PERF-MVP-226已直接删除copy execution的三类无必要临时分配：same-buffer overlap用`copy_within`保持memmove语义，不同buffer用`get_disjoint_mut`直接复制，buffer↔texture利用独立resource tables并行借用。源码守卫禁止恢复range `to_vec`、whole-buffer contents clone或whole-texture resource clone；现有cross-resource transfer/texture copy测试加overlap self-copy覆盖。

Bevy真实readback在command encoder记录GPU copy，提交后异步map并通过channel回送；Unreal RHI在command list上记录`CopyBufferRegion`/`CopyTexture`并由RHI context执行。两者都不会把整张GPU资源镜像成一个全局mutex下的CPU Vec后称为production WGPU backend，Zircon未来产品RHI接入必须遵守同一owner/queue/fence边界。

## 动态验收

待受管Cargo运行`rhi`/`texture_copy`/`device_contract` focused tests，至少覆盖distinct buffer、overlap self-copy、buffer↔texture row stride、usage/range错误与source guard。再记录1/100/10k contract commands的validation/execute scans、alloc bytes和lock hold time；这些数据只评价测试反馈速度。产品GPU性能仍必须在`graphics`真实wgpu backend以CPU scope、timestamp、marker与RenderDoc验证，完成前本批保持`pending.md`。
