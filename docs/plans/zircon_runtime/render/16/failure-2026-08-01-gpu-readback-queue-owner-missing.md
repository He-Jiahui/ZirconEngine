---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-08-01
summary_slug: gpu-readback-queue-owner-missing
origin_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
fixing_plan: docs/plans/zircon_runtime/render/16-compute-neural.md
origin_child_dir: docs/plans/zircon_runtime/render/17
fixing_child_dir: docs/plans/zircon_runtime/render/16
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_pass_timer.rs
  - zircon_runtime/src/graphics/backend/render_backend/gpu_readback_queue/mod.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_readback/
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
tests:
  - readback_callback_fires_after_n_frame_delay
  - readback_slot_reuse_blocks_until_map_complete
  - readback_ring_grows_to_fit_frame_requests
  - readback_ring_shrink_delay_counts_global_frames_across_slot_reuse
  - readback_no_private_map_async_source_scan
  - readback_layout_failure_preserves_callbacks_for_abort
  - readback_queue_production_paths_are_panic_free
  - render_perf_gpu_timer_latency_within_three_frames
---

# Render16: shared GPU readback queue owner is missing

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 来源执行切片：PF-M1 GPU pass timer readback-ring closeout
- 修复责任计划：`docs/plans/zircon_runtime/render/16-compute-neural.md`
- 交接原因：`GpuReadbackQueue`、staging ring、ticket/callback lifecycle 和 private `map_async` 禁令由 Render16 CN-M1 唯一拥有；Render17 只能迁移 timer consumer，不能建立第二套 queue。

## 失败现象与复现证据

PF-M1 明确要求在 Render16 CN-M1 落地后，把 timer 私有三槽读取环硬切到 `GpuReadbackQueue`。当前全树不存在 `GpuReadbackQueue`、`ReadbackTicket` 或 `gpu_readback_queue/` owner；`rhi_wgpu/gpu_pass_timer.rs` 仍直接持有 `[TimerReadbackSlot; 3]`、`mpsc` receiver 与 `map_async`。生产 graphics 路径还存在多处直接 `map_async`，因此 Render16 的统一异步 readback 基础设施和迁移禁令均未建立。

## 最低共享层根因

Render16 CN-M1 切片 1.3 尚未实现。Render17 若只把 timer 的私有槽重命名，或在 `rhi_wgpu` 复制一个 timer-only queue，会留下多个 staging/map/callback owner，无法满足计划要求的统一背压、容量迟滞、统计和 N 帧延迟语义。

## 架构修复验收

- 按 Render16 计划在 graphics backend 建立唯一 `GpuReadbackQueue`、`ReadbackTicket` 和三槽 staging ring；空队列不得创建 staging 资源。
- 实现 256 字节请求对齐、按 2 的幂增长、240 个低利用率帧后收缩、槽复用背压和非阻塞完成派发。
- 将普通生产 readback consumer 迁到该 owner，并由 source-scan 禁止白名单以外的私有 `map_async`。
- Render17 随后删除 `TimerReadbackSlot`/timer 私有 `mpsc` 生命周期，改为 queue ticket consumer；保留 generation 有序回传与 `profile_latency_frames <= 3`。
- 通过 Render16 readback 测试、Render17 `render_perf` 回归和当前源码 WGPU 产品验收。

## 禁止临时方案

- 不得在 Render17 或各 executor 新建第二套 readback queue/ring。
- 不得把同步 `poll(wait_indefinitely)` 引入普通帧路径，或用 caller-thread wait 伪装 N 帧回传。
- 不得仅重命名 `TimerReadbackSlot`、放宽 source scan，或把缺失 GPU 时间写成 0。

## 修复结果与回传

### 2026-08-01 当前实现

- 已在 `rhi_wgpu/gpu_readback_queue/` 建立唯一 WGPU owner，并由计划指定的 `graphics/backend/render_backend/gpu_readback_queue` facade 暴露；`SceneRendererCore` 和 UI surface 分别持有与自身 device 对应的唯一实例，没有 timer/executor 私有 ring。
- 三槽 staging、空帧零分配、256-byte 请求布局、2 的幂增长、240 个全局帧低利用率收缩、N+1..N+2 非阻塞 poll、N+3 槽复用背压、ticket/cancel、panic 隔离、abort 错误完成和统计回传均已实现。二次性能复核修正了每槽只加1导致实际约720帧才收缩的偏差，现按该槽两次复用间经过的全局帧数累计，80次N+3复用即240帧触发一次减半。
- timer、realtime IBL timestamp、HZB stats/indirect args、mesh indirect args、Hybrid GI 与 particles 普通帧消费者已迁移；Virtual Geometry 的尚未接入生产的 GPU prepare 路径已改为只能通过 `RuntimePrepareCollectorContext::request_gpu_readback` enqueue，decoder 不再拥有 map 生命周期。
- Hybrid GI/Virtual Geometry 直接从原 storage buffer 进入共享 staging，删除 9 个每次 prepare 的中间 readback buffer 分配和 9 次冗余 buffer-to-buffer copy；仅保留 WGPU texture-to-buffer 所需的行布局中转。
- 二次审查发现并前向修复：容量布局失败会丢 callback、粒子实例归零后保留陈旧 future、迁移后死方法/死 helper、插件双重 copy，以及 staging 内部不变量依赖生产 `expect`/未检查 mapped slice。请求名现进入失败诊断，编码/映射/完成路径均以可恢复错误完成 callback；GPU 已提交后的 readback error 会先完成 transient pool、scene frame 状态或 UI surface present 再传播，不再留下半结束帧。
- 最新静态二次审查结论为 C0/I0；scoped `rustfmt`、`git diff --check`、生产 panic/dead-code guard 与私有 `map_async`/阻塞等待 source scan 通过。

### 仍待接受的证据

- 当前源码受管 Windows 编译 request：`4dca61081c8a4e2b88cc857eb66dd89e`，仅确认 `session.register` post-response accepted timeout；按跨 Session 协调规则不轮询，validator 未启动 Cargo，因此不计为编译通过。
- `render_perf` 当前源码回归、真实 WGPU PNG 与 RenderDoc `.rdc` 尚未生成，因此本 failure
  保持 `source_complete_dynamic_validation_pending`，不得返回 `fixed`，PF-M1 也不得 accepted
  closeout。

Open state: `Render16 CN-M1 source implementation and second review are complete; managed current-source compile plus real WGPU PNG/RDC product evidence remain before fixed return.`
