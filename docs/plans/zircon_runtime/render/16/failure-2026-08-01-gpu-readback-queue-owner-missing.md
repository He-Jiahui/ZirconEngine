---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: gpu-readback-queue-owner-missing
origin_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
fixing_plan: docs/plans/zircon_runtime/render/16-compute-neural.md
origin_child_dir: docs/plans/zircon_runtime/render/17
fixing_child_dir: docs/plans/zircon_runtime/render/16
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_pass_timer.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/readback/
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_diagnostics.rs
  - zircon_runtime/src/graphics/backend/render_backend/product_diagnostic_delivery_router.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector/
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/capture_mailbox.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_readback/
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
tests:
  - readback_callback_fires_after_n_frame_delay
  - readback_slot_reuse_is_refused_without_waiting_for_map_completion
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

- 已在 `rhi_wgpu/gpu_readback_queue/` 建立共享低层 owner，timer/UI 等选定消费者不再各自持有 ring。当前 scene/runtime production diagnostics 已进一步迁到同一 device 下的 `production/diagnostics/readback` service，并由 `RenderBackend` product-diagnostic router 统一 admission、copy/map 与 callback 派发；原文所列 `graphics/backend/render_backend/gpu_readback_queue` facade 已不存在，不能继续作为当前路径证据。
- 三槽 staging、空帧零分配、256-byte 请求布局、2 的幂增长、240 个全局帧低利用率收缩、N+1..N+2 非阻塞 poll、N+3 槽复用背压、ticket/cancel、panic 隔离、abort 错误完成和统计回传均已实现。二次性能复核修正了每槽只加1导致实际约720帧才收缩的偏差，现按该槽两次复用间经过的全局帧数累计，80次N+3复用即240帧触发一次减半。
- timer、realtime IBL timestamp、HZB stats/indirect args、mesh indirect args、Hybrid GI 与 particles 普通帧消费者已迁移；Virtual Geometry 的尚未接入生产的 GPU prepare 路径已改为只能通过 `RuntimePrepareCollectorContext::request_gpu_readback` enqueue，decoder 不再拥有 map 生命周期。
- Hybrid GI/Virtual Geometry 直接从原 storage buffer 进入共享 staging，删除 9 个每次 prepare 的中间 readback buffer 分配和 9 次冗余 buffer-to-buffer copy；仅保留 WGPU texture-to-buffer 所需的行布局中转。
- 二次审查发现并前向修复：容量布局失败会丢 callback、粒子实例归零后保留陈旧 future、迁移后死方法/死 helper、插件双重 copy，以及 staging 内部不变量依赖生产 `expect`/未检查 mapped slice。请求名现进入失败诊断，编码/映射/完成路径均以可恢复错误完成 callback；GPU 已提交后的 readback error 会先完成 transient pool、scene frame 状态或 UI surface present 再传播，不再留下半结束帧。
- 最新静态二次审查结论为 C0/I0；scoped `rustfmt`、`git diff --check`、生产 panic/dead-code guard 与私有 `map_async`/阻塞等待 source scan 通过。

### 2026-09-21 current-source reconciliation

- 槽复用的现行语义是非阻塞拒绝并累计 `slot_reuse_rejection_count`；旧测试名和计划中“第 4 帧阻塞等待”与禁止 caller-thread wait 的验收相冲突，已纠正为 `readback_slot_reuse_is_refused_without_waiting_for_map_completion`。
- `readback_no_private_map_async_source_scan` 当前覆盖列举的 ordinary production consumers，而不是对整个仓库做字符串白名单。显式 screenshot/headless、WGPU product-diagnostic service、IBL artifact persistence 与测试夹具拥有各自受约束的 map 生命周期；最终验收必须按角色核对这些例外，不能把窄扫描描述成全树唯一 owner 证明。
- viewport fallback 使用 product-diagnostic service；其 latest-ready mailbox 的晚到淘汰代际曾可把完整 RGBA payload 永久留在 `completed`。当前 source 修复以有界 armed generation window + request-drop cancellation 封住该缺口，并加入 callback-before-register、trimmed late completion、重复 late completion、显式 cancel 与 request-drop 回归。精确五文件快照独立终审 `review-render16-readback-chain-r2` 为 C0/I0/M0；该修复仍待 managed Cargo。
- 因此本记录仍为 `source_complete_dynamic_validation_pending`：低层 queue 的既有静态 ticket `e3df88f2a9f3455483cf45cddcfe3252` 只覆盖当前 `queue.rs` 等五个 blob，不覆盖本次 mailbox 变化，也不代替产品 WGPU/PNG/RDC、Render17 latency 或双中心 owner 角色审查。
- mailbox focused ticket `b3ae4113607a44b6a86d3ad3354ee24e` 冻结的是 rustfmt 修正前的两个源码哈希，并已因 validation copy 内相对 validator 路径不可解析而在 Cargo/test 前以 coordinator exit 1 终态失败；它不是测试失败且不能作为现行源码验收。唯一 current-hash successor `2a5e5ccf50c941d088a937c140b60459` 已改用仓库绝对 validator 路径与 validation-copy `-RepoRoot`，当前 `queued`，预期精确执行 6 个 `capture_mailbox` tests。

### 仍待接受的证据

- 当前源码受管 Windows 编译 request：`4dca61081c8a4e2b88cc857eb66dd89e`，仅确认 `session.register` post-response accepted timeout；按跨 Session 协调规则不轮询，validator 未启动 Cargo，因此不计为编译通过。
- `render_perf` 当前源码回归、真实 WGPU PNG 与 RenderDoc `.rdc` 尚未生成，因此本 failure
  保持 `source_complete_dynamic_validation_pending`，不得返回 `fixed`，PF-M1 也不得 accepted
  closeout。

Open state: `Render16 CN-M1 source implementation and second review are complete; managed current-source compile plus real WGPU PNG/RDC product evidence remain before fixed return.`
