---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: dynamic-extract-artifact-and-stats
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
tests:
  - extract hit clone-byte and visit counters
  - F2 stable/dirty generation parity
---

# Runtime07：dynamic extract artifact与统计扫描

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-342 / PERF-MVP-431 dynamic extract artifact and stats
- 修复责任计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 交接原因：generation-owned extract、submission derived state 与统计扫描的共享边界由 Runtime07 所有。

## 失败现象与复现证据

cache hit仍返回完整`RenderFrameExtract::clone`，miss还clone一份进cache；每次capture/present随后遍历mesh/morph/VG/light/environment/post/UI等宽payload估算bytes。现有测试只断言诊断数值与cache hit，不限制deep clone bytes或payload visits。

## 最低共享层根因

immutable generation source 与 per-submission derived/selection state 尚未分离，导致 cache、renderer mutation 与 diagnostics observer 争用同一个宽 payload 合同。

## 架构修复验收

- 沿PERF-MVP-342发布generation-owned Arc extract，cache/renderer/diagnostics共享单一payload。
- 构建时单遍封存Copy `ExtractDiagnosticsSummary`；stable hit复用summary，diagnostics off不扫描。
- stable/1% dirty、1/1k/100k meshes与0/64MiB VG/UI记录build、clone bytes、stats visits：stable三项均0，changed generation≤1；回传PERF-MVP-431。

## 禁止临时方案

不得仅把 cache 改为 `Arc` 后依赖 `Arc::make_mut` 的整帧 COW clone，也不得用 default clone fallback 或每次 payload scan 宣称 shared extract 已收敛。

## 修复结果与回传

Open state: `待 Runtime07 联动Runtime10/Render17收敛extract owner与观察者成本`。

## 2026-07-27 contract recovery

- 当前 shared boundary 仍按值接收 `RenderFrameExtract`：cache 即使改为 `Arc`，runtime bridge 仍需在 submit/present 边界重建 owned payload。更重要的是 renderer camera loop 和 context builder 对 `Arc<RenderFrameExtract>` 使用 `Arc::make_mut` 写入 selected camera、viewport、material/PBR、particle、post-process 与 AA derived state；当 cache 同时持有 Arc 时，这会退化为整帧 copy-on-write clone。
- 因此本 lifecycle 的最低修复是 immutable generation source handle 加独立的 per-submission derived/selection state，并让 RenderFramework/Runtime bridge/renderer 在该 handle 上达成单一 ownership contract。不得把 cache Arc、renderer mutable working extract 或 diagnostics payload scan 混为同一对象，更不得以 default clone fallback 宣称 shared extract。
- 该 cross-boundary hard cut 需要对现有未归属的 framework/runtime-loop/renderer dirty source 先做 source attribution；当前 Runtime07 scope 只记录事实，不将这些工作区改动吸收为本 failure 的修复，也没有 Cargo green claim。

## 2026-08-13 forward continuation

- `RuntimeFrameExtractCacheEntry` now owns an immutable diagnostics summary created once when a
  generation is rebuilt. A stable cache hit reuses that summary, so recording
  `extract.output_bytes` no longer traverses meshes, lights, virtual-geometry payloads,
  post-process data, overlays, sprites, particles, or visibility payloads again.
- The real two-capture headless session regression requires payload-stat scan samples `[1, 0]`
  for rebuild then stable reuse. Existing `full_clones` and `full_clone_bytes` diagnostics remain
  explicit at `[1, 1]`: this slice does not misrepresent the still-owned submit boundary clone as
  eliminated.
- The broad immutable-source versus submission-derived-state hard cut remains separate work: the
  current RenderFramework trait, pipelined queue, and camera loop still accept an owned extract
  and perform legitimate per-submission mutations. No `Arc::make_mut` COW path was relabeled as a
  cache optimization. Rustfmt and scoped diff checks passed; no Cargo or performance command was
  run in this continuation. The canonical failure remains `open` pending its managed behavior and
  quantified performance gates.
