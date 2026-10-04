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
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/queue.rs
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

2026-08-13 source closeout：`RenderSubmissionConfig::hzb_diagnostics_readback` 以默认关闭的
统一开关显式控制 stats、indirect args 与 draw-count CPU 诊断。普通产品帧不再申请任何 HZB
readback copy；只有显式启用且本帧实际 dispatch HZB 时才进入共享 `GpuReadbackQueue`。诊断帧在
stats 与 indirect pending 队列均有容量时才整体准入，容量不足只累计 drop，不生成半帧诊断。
ready 结果按 source frame FIFO 延迟消费并报告 generation/frame、pending、drop 与 age；共享三槽
staging ring 只执行 non-blocking poll，busy slot 不等待 GPU。

本切片已完成 scoped `rustfmt`、旧配置/API 名扫描与 `git diff --check`，源码合同锁定默认零读回、
显式 opt-in、whole-frame admission 和 skipped-frame 诊断。Cargo/WGPU、1/100/10k candidate timing、
RenderDoc capture、copy/allocation 计数和 `docs/tests/runtime/render` 当前源码 PNG 仍由 coordinator
执行；因此该 failure 仅为 source complete，尚未 accepted closeout。
### 2026-09-19 corrected static source-contract receipt

Coordinator-managed ticket `e3df88f2a9f3455483cf45cddcfe3252` passed with
job/run `57ce0d2fe4e44b69884b8936c2be6358` /
`e3df88f2a9f3455483cf45cddcfe3252` (exit 0,
`RENDER04_HZB_ASYNC_READBACK_SOURCE_CONTRACT_PASS`, `CHECKED_PATHS=5`).
The sealed source-manifest hash is
`8c4ad57cf3209ecd067c878b081b3698704b87de76bdae294dc4787c750bc81d`.
This corrected rerun uses `rustfmt --config skip_children=true --check`
after the earlier coordinator-only nested `tests.rs` resolution failure and
does not alter source. It covers the default HZB enablement, bounded delayed
diagnostic readback, GPU-resident indirect arguments, and shared queue
contract. Render11-owned `submit_compiled_scene_frame.rs` remains explicitly
deferred. Managed Cargo/WGPU, Render16 upward acceptance, scale/performance
evidence, independent C/I/M review, canonical failure return, and closeout
remain pending; the failure stays `open`.

### 2026-09-20 independent source review r3

Reviewer session `review-render04-hzb-r3` completed an independent read-only
review of the current owned source and recorded `Critical=0 / Important=0 /
Moderate=0`. The review confirmed that the ordinary product path is gated by
`RenderSubmissionConfig::hzb_diagnostics_readback` (default `false`), that the
HZB stats queue is bounded and FIFO with source-frame/pending/drop/age
diagnostics, and that stats plus indirect-args readbacks use the product
diagnostic callback route rather than a synchronous map or wait. The
production portions of the owned files contain no `wait_indefinitely`, direct
`device.poll`, or `map_async`; the `GpuReadbackQueue` ring keeps polling and
slot reuse non-blocking. Indirect args remain GPU-resident for replay and are
only copied for explicit diagnostics. The Render11-owned submit path remains
out of scope and was not absorbed.

The review snapshot hashes at the time of the receipt were:

```text
zircon_runtime/src/graphics/pipeline/compile_options/default.rs                                      0426686b995331ac1ac5e9c5727e512293872cc2140c0e54492ac03a4e7fdc99
zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs                          0539c0d43b0452d45afe901c083d17cca7e28ae69d89a671c4ba704ddb5c442d
zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs            97f6db0d216dd17e5a6f4ef08564219725bd434e414d56dc2723259288f8c2bb
zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/queue.rs                                     12a34c829492e0961097f8134f2e173c4a328fa938d256838f9ff1bf05d43868
```

This is a source-only review receipt. It does not replace the required
current-source managed Cargo/WGPU tests, Render16 upward acceptance, 1/100/10k
scale/performance evidence, canonical `fixed-*` return, or coordinator
closeout.

### 2026-09-25 current-source rolling reconciliation (Render04 owner)

- Session `failure-roll-01a084c8-render04-hzb-r4` owns this reconciliation. Snapshot `3814` seals the four Render04-owned paths and records the Render11-owned submit path for foreign provenance:
  - `zircon_runtime/src/graphics/pipeline/compile_options/default.rs` — `0426686b995331ac1ac5e9c5727e512293872cc2140c0e54492ac03a4e7fdc99`
  - `zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs` — `0539c0d43b0452d45afe901c083d17cca7e28ae69d89a671c4ba704ddb5c442d`
  - `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs` — `97f6db0d216dd17e5a6f4ef08564219725bd434e414d56dc2723259288f8c2bb`
  - `zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/queue.rs` — `12a34c829492e0961097f8134f2e173c4a328fa938d256838f9ff1bf05d43868`
  - Render11-owned `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs` — `22ab6bba7820d84c81d336bb9d356d8b0b0adb48308073d4583390078c21f8df` (dirty foreign path, not leased or absorbed).
- `git diff --check` reports no whitespace errors (only normal LF→CRLF notices). `rustfmt +1.94.1 --edition 2024 --config skip_children=true --check` was run on all four owned paths; existing import ordering/line-wrap drift in `hzb_occlusion_culler.rs`, `indirect_draw_execution.rs`, and `gpu_readback_queue/queue.rs` makes the aggregate check non-passing, so no formatting GREEN is claimed. The default pipeline path passes its formatter check.
- Current source inspection confirms `RenderSubmissionConfig::hzb_diagnostics_readback` remains default-off, diagnostics admission is whole-frame and bounded, HZB stats and indirect summaries carry source frame/generation/pending/drop/age state, `GpuReadbackQueue` uses bounded staging slots and non-blocking `PollType::Poll`, and indirect draw execution remains GPU-resident unless explicit diagnostics are requested. The Render11 submit path is intentionally left to its owner.
- Managed Windows Cargo/WGPU, Render16 upward acceptance, 1/100/10k timing/RenderDoc/copy-allocation evidence, formatter repair, canonical `fixed-*` return, closeout, and WeCom remain pending; this failure stays `open`/`resolving_failure`.

### 2026-09-25 independent static review receipt

- Reviewer `review_editor03_gizmo_private` checked snapshot `3814`, all four owned hashes, and the Render11-owned dirty submit path. Result: Critical/Important/Moderate = `0/0/0`.
- The review confirms default-off diagnostics, bounded FIFO generation/pending/drop/age state, GPU-resident indirect args, non-blocking `PollType::Poll`, and no production blocking wait in the owned HZB paths. Formatter, Cargo/WGPU, scale, and upward gates remain pending.
