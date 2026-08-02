---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: hzb-per-frame-blocking-readback
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/04-visibility-culling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/pipeline/compile_options/default.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
  - zircon_runtime/src/rhi_wgpu/gpu_readback_queue/queue.rs
tests:
  - default pipeline steady frame has no blocking HZB readback
  - delayed HZB stats ring freshness and drop test
  - GPU indirect args remain GPU-resident product parity test
---

# Render04：默认 HZB 每帧同步读取 stats 与 indirect args

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F2 默认 pipeline、HZB culler、indirect execution 与 submit 后诊断静态审查
- 修复责任计划：`docs/plans/zircon_runtime/render/04-visibility-culling.md`
- 共同责任：`docs/plans/zircon_runtime/render/16-compute-neural.md`
- 交接原因：Render04 明确拥有 HZB/GPU occlusion；Render16 拥有统一 async readback，不应在性能计划复制另一套 readback framework。

## 失败现象与复现证据

默认 forward+、deferred pipeline 都启用 HZB occlusion。只要本帧有 candidate，submit 后 `attach_hzb_occlusion_readback_stats` 立即调用 `collect_last_readback_stats`，其中执行 `map_async + device.poll(wait_indefinitely)`。

同帧还为最多四个 HZB indirect phase 创建 args readback buffer，并按 capability 创建 draw-count readback buffer。submit 后逐 buffer 调用 `collect_pod_buffer`，每次再次 `map_async + wait_indefinitely`。最坏静态形态是 1 次 stats 加最多 8 次 args/count blocking poll；这些 CPU 数据只用于诊断 summary，GPU draw 本身不需要回读。

## 最低共享层根因

产品 culling 与诊断 introspection 没有分层：GPU-resident indirect execution 在每帧结尾被同步 CPU inspection 强制收敛。readback 没有 generation、ring、ready/pending/drop 或 age 契约。

## 架构修复验收

- HZB stats 使用 Render16 统一的持久多缓冲 async readback，N 帧后消费；not-ready 不阻塞，只更新 age/drop/pending 诊断。
- 正常产品帧不创建/读取每 phase args/draw-count MAP_READ buffer；只有显式调试采样才异步读取，默认频率为零。
- 保持 GPU compaction、visible remap、indirect draw 与 cull report 的确定性/能力回落；诊断允许延迟但必须带 generation/age。
- 1/100/10k candidate 的 WPR/Tracy/RenderDoc 证明无主线程 GPU wait，并给出 draw/cull parity、copy bytes 和 buffer allocation。

## 禁止临时方案

- 不得仅把 `wait_indefinitely` 换成 busy poll，或在线程池任务里继续每帧同步等待并无限积压。
- 不得为了消除 readback 关闭 HZB/GPU-driven 或删除可观测性；应把可观测性降频异步化。

## 修复结果与回传

Render04 now reserves bounded HZB stats/explicit debug args queues, consumes only ready FIFO
results, and reports pending/drop/age diagnostics. The shared WGPU readback queue now rejects a
busy frame slot after a non-blocking poll instead of waiting for GPU completion; the normal render
path already treats that rejection as a skipped diagnostic frame. Default indirect-args inspection
remains opt-in. Focused source tests and managed WGPU performance/capture validation remain pending.
