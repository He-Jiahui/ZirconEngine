---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: rhi-wgpu-submit-validation-and-copy-clones
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/rhi_wgpu/command_validation.rs
  - zircon_runtime/src/rhi_wgpu/command_validation/render_state.rs
  - zircon_runtime/src/rhi_wgpu/device.rs
  - zircon_runtime/src/rhi_wgpu/device/command_list.rs
  - zircon_runtime/src/rhi_wgpu/mod.rs
  - zircon_runtime/src/rhi_wgpu/render_pass_validation.rs
  - zircon_runtime/src/rhi_wgpu/tests.rs
  - zircon_runtime/src/rhi/tests/command_list.rs
  - zircon_runtime/src/rhi/tests/command_list/basic_commands.rs
  - zircon_runtime/src/rhi/tests/command_list/bind_groups.rs
  - zircon_runtime/src/rhi/tests/command_list/raster_draws.rs
  - zircon_runtime/src/rhi/tests/command_list/vertex_index_state.rs
  - zircon_runtime/src/rhi/tests/debug_markers.rs
  - zircon_runtime/src/rhi/tests/debug_status.rs
  - zircon_runtime/src/rhi/tests/device_contract.rs
  - zircon_runtime/src/rhi/tests/device_contract/basic_resources.rs
  - zircon_runtime/src/rhi/tests/device_contract/bind_groups.rs
  - zircon_runtime/src/rhi/tests/device_contract/framework_boundary.rs
  - zircon_runtime/src/rhi/tests/device_contract/invalid_descriptors.rs
  - zircon_runtime/src/rhi/tests/device_contract/texture_sampler_descriptors.rs
  - zircon_runtime/src/rhi/tests/device_contract/transfer_and_fences.rs
  - zircon_runtime/src/rhi/tests/pipeline.rs
  - zircon_runtime/src/rhi/tests/render_pass_clear_values.rs
  - zircon_runtime/src/rhi/tests/render_pass_command_list.rs
  - zircon_runtime/src/rhi/tests/render_pass_resolve.rs
  - zircon_runtime/src/rhi/tests/render_pass_state.rs
  - zircon_runtime/src/rhi/tests/render_pass_views.rs
  - zircon_runtime/src/rhi/tests/resource_lifecycle.rs
  - zircon_runtime/src/rhi/tests/texture_copy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_device_command_list.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/rhi_wgpu_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/rhi_device_contract.rs
  - docs/zircon_runtime/rhi/descriptors.md
---

# RHI WGPU contract device与产品backend边界

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：RHI/WGPU non-UI core 12/12 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：真实GPU queue/resource/fence、CPU/GPU profiling和RenderDoc证据由Render17拥有；performance audit不能把contract test double扩成第二套产品backend。

## 失败现象与复现证据

`WgpuRenderDevice`公开导出并声明async copy/debugger caps，但实现是单个`Arc<Mutex<WgpuRenderDeviceState>>`中的CPU `Vec<u8>`资源；submit在锁内完整validation后再执行第二遍，fence同步完成。调用图确认构造器当前只被`rhi/tests/**`调用，产品graphics backend不经过它。局部copy临时Vec/整资源clone已由PERF-MVP-226止损，但这不改变test-double身份。

## 最低共享层根因

neutral RHI contract test harness与真实wgpu backend缺少显式命名、可见性和接入边界：public `WgpuRenderDevice`同时承担测试语义与未来production名称，capability值也没有区分模拟能力与真实adapter能力。若产品误接，会把全局锁、host mirror、同步readback和即时fence带入帧路径；若继续只用于测试，则公开production形态会持续误导profiling与计划验收。

## 架构修复验收

- 在Render17/graphics backend文档明确唯一产品wgpu device/queue/resource/fence owner，并用调用图测试锁定F2/F4不经过CPU contract test double。
- 二选一硬切换：把当前实现改名并收紧为test-only deterministic device；或以真实wgpu adapter/device/queue/resource和异步fence实现替换，删除CPU mirror语义。不得长期保留同名双义。
- capability来自真实adapter或明确的test profile；test device不得声称未实现的async copy/graphics debugger语义。
- 真实产品copy/readback使用recorded GPU commands与有界异步readback ring；默认帧路径无caller-thread wait、无整资源host clone。
- 产品验收记录CPU submit、queue depth/fence latency、copy/upload/readback bytes、GPU timestamp及RenderDoc pass/resource；contract测试wall-clock不得替代GPU证据。

## 禁止临时方案

- 不得仅在全局mutex外包一层无界worker queue，继续保留同步CPU资源模型并称为production WGPU。
- 不得删除validation来降低测试耗时；应缓存immutable descriptor事实或在真实backend依赖wgpu validation/error scope，并保留错误合同。
- 不得用test double的即时fence、零GPU时间或CPU copy吞吐填充Render17产品基线。

## 修复结果与回传

Open state（2026-07-22）：Render17已把CPU mirror实现收敛为`cfg(test)`下的
`DeterministicRhiContractDevice`/`DeterministicRhiContractCommandList`，并用产品调用图合同锁定
`SceneRenderer -> graphics::backend::RenderBackend -> wgpu::Device/Queue`为唯一产品owner；test profile不再声明
surface、async copy、graphics debugger或indirect draw能力。独立审查已完成首轮C0/I2/M2，I2代码修正已落地；最终
public `WgpuRenderDevice`/`WgpuCommandList`测试兼容别名已删除，并由调用图与结构合同禁止回归；剩余内部
state也已硬切为`DeterministicRhiContractDeviceState`，当前源码不再包含旧`WgpuRenderDeviceState`；Rust 1.94.1
受管RHI回归、产品GPU/RenderDoc证据、upward gate、failure return/review/commit仍待完成，当前不声明fixed。

全树consumer审计已把successor的return前边界扩为exact36：除原exact33外，必须原子迁移仍断言旧名称/计数的两条Runtime15结构测试consumer，以及clean的canonical模块文档`docs/zircon_runtime/rhi/descriptors.md`。`docs/assets-and-rendering/render-framework-architecture.md`、`docs/zircon_runtime/structure/module-convention.md`和父计划文件当前是foreign-dirty full blob，继续作为独立docs-only owner交接，不吸收到本failure提交。
