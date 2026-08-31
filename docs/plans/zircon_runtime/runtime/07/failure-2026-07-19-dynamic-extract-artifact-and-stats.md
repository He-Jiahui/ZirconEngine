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

## 2026-08-30 current-source architecture and baseline review

### Canonical owner and mutation admission

- This file remains the canonical Runtime07 lifecycle for F3 shared extract. Runtime43
  `DYN-P1-029` is a current-source consumer finding, not a second implementation owner;
  Runtime15/Render17 own cooperating submission/renderer boundaries.
- `dynamic_api/session/extract_cache.rs` and `extract.rs` have no active coordinator attribution.
  `events.rs` is archived Runtime117 work and `state.rs` is actively Runtime22-owned. The current
  render framework/extract/submission tree is also broadly modified or untracked by foreign work.
  This review therefore did not mutate production source and did not claim those blobs.
- Current hashes preserved by this review are:
  - `extract_cache.rs`:
    `d2acbfc73885a5d8d507100d0569ee146a75328e6f88d305940ea6c384eed81e`;
  - `extract.rs`:
    `89cf1b3a03fb53464dc367e6fb5651ee62dc83f5ac9736385de120178ed3d226`;
  - `events.rs`:
    `db316a91c843f0480c8641862f80e0038b533d9910291321b67a811d035415c7`;
  - `state.rs`:
    `a79255874bdddf1c2cdc3d9e11597954d1197356fb96685611c3bb0d8d31ffba`.

The lowest legal production slice is Runtime07-owned cache/submission contract work after exact
transfer of the dirty RenderFramework/renderer files. Runtime22 `state.rs` must remain untouched;
its two call sites can be migrated only by its owner or an explicit coordinated transfer.

### Current data flow and structural bottleneck

1. `RuntimeFrameExtractCacheKey` correctly includes `ChangeTick`,
   `lifecycle_visibility_revision`, `active_camera`, and `viewport_size`; these four components
   remain mandatory in the hard cut.
2. A miss builds the full `RenderFrameExtract`, deep-clones it into the cache, and returns the
   original. A stable hit deep-clones the cached value. `current_extract` then mutates only editor
   camera/view state and timing before capture or present receives the owned value.
3. The public `RenderFramework` contract still accepts `RenderFrameExtract` by value for submit
   and present. `ViewportRenderFrame` and `FrameSubmissionContext` wrap it in `Arc`, but the current
   submit builder has nine `Arc::make_mut` sites and the camera loop has additional mutation sites.
   Mutations include viewport/dynamic resolution, selected camera, temporal jitter, material/PBR
   and subsurface projections, previous particles, effective post process, hydrated environment,
   and VG/HGI payload-slot moves.
4. Consequently, retaining the cache owner and replacing only the cached value with
   `Arc<RenderFrameExtract>` would force copy-on-write of the full frame before rendering. It does
   not satisfy stable-frame allocation or copied-byte gates.

### Unreal reference boundary

Local Unreal source confirms a three-lifetime design rather than a copied monolithic frame DTO:

- `Renderer/Private/ScenePrivate.h:1492-1497` states that `FScene` stores renderer state independent
  of any view or frame and owns primitive/light add/remove state.
- `Engine/Public/SceneView.h:2327-2437` makes `FSceneViewFamily` hold a scene pointer, frame time,
  render target, and the views for one submission; `FSceneView` separately owns per-view matrices,
  rectangles, location, and state.
- `Renderer/Private/SceneRendering.h:2158-2219,2239-2252` makes the short-lived renderer reference
  the persistent scene while owning `FViewInfo` and view-family derived state for the render call.

Zircon should therefore keep one immutable generation-qualified scene payload, build a small
submission/view overlay, and let renderer-owned context hold effective derived state. It should not
copy the scene payload to make renderer mutations convenient.

### Required hard-cut contract

The implementation owner must introduce one canonical submission path with these responsibilities:

- `RenderFrameScenePayload`: immutable, generation-qualified heavy domains for geometry,
  animation poses, authored lighting/environment/post-process inputs, debug payload, sprites,
  particles, and visibility. Cache entries retain `Arc<RenderFrameScenePayload>` plus the sealed
  diagnostics summary.
- `RenderFrameSubmission`: the only submit/present DTO. It carries the shared scene payload,
  the scene generation, timing, selected/editor camera and viewport/view-family overlay, and UI
  submission. It must not expose a mutable path to the scene payload.
- `FrameSubmissionContext`: renderer-owned effective state. Dynamic resolution, AA/jitter,
  material features, subsurface profiles, hydrated environment handles, previous particles,
  post-process graph/effect settings, visibility/history, VG/HGI decisions, and runtime overlays
  are derived here without mutating or taking fields from the shared scene payload.
- Cache reuse clones only `Arc` handles and the bounded view/timing overlay. Capture and present
  consume the same hard-cut DTO shape and preserve the same scene generation; no compatibility
  overload accepting an owned `RenderFrameExtract` remains in production.

This is a semantic hard cut, not an `Arc::make_mut` conversion. Direct public field consumers and
synthetic test fixtures must migrate to payload/submission constructors or read-only accessors in
the same source change so two authorities cannot survive.

### Direct-current-source baseline

An F-drive Rust harness directly includes the current production `extract_cache.rs` behind a
minimal contract shell and instruments global allocations plus the mock wide-payload Clone
implementation. The payload has 10,000 mesh, 10,000 light, and 10,000 sprite records, each 64
bytes (`1,920,000` logical payload bytes). It runs 31 samples on Windows 11 build 26200, AMD Ryzen
7 5800H (8C/16T), optimized `rustc -C opt-level=3`; build inputs and outputs stay under
`F:/codex-targets/019ffe-runtime07-f3-shared-extract-20260830`.

| operation | P50 | P95 | allocations/op | allocated bytes/op | deep clones/op | copied bytes/op |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| current stable hit | 836.2 us | 1,350.7 us | 3 | 1,920,000 | 1 | 1,920,000 |
| current miss | 2,477.5 us | 6,679.4 us | 6 | 3,840,000 | 1 | 1,920,000 |
| proposed shared base + overlay model | 4 ns | 17 ns | 0 | 0 | 0 | 0 |

Harness source SHA-256 is
`81bfd65314298f5113208aca8b6e1bab6dcb640bb515edee401588fdc7cdbf1f`. The proposed model is a
layout/ownership microbenchmark with 100,000 iterations per sample; it proves the order-of-growth
and allocation consequence of the ownership split, not product renderer latency, RSS, power, or
performance parity with Unreal. Those claims remain prohibited until current-source managed and
matched product measurements exist.

### RED/GREEN and performance acceptance

- unchanged frames keep pointer identity for every large domain; overlay camera/timing changes are
  visible without mutation or COW of the cached base;
- each preserved key component independently forces exactly one rebuild;
- capture and present observe the same scene generation and use the same submission DTO;
- production source contains no `Arc::make_mut` or `take()` path for the immutable scene payload;
- an ignored current-source 10k mesh/light/sprite benchmark reports allocations, allocated bytes,
  deep clones, copied bytes, and P50/P95 elapsed time. Stable reuse must report all four allocation/
  clone/copy counters as zero; rebuild count is at most one per key change;
- managed Windows behavior tests, matched renderer profile, RSS/CPU, and power evidence remain
  required. The microbenchmark does not close this failure.

Current status:
`open / architecture_reviewed_baseline_profiled / foreign_render_contract_transfer_pending`.

## 2026-08-31 M1 shared-storage source slice

Runtime07 has now acquired the exact source leases and implemented the first dependency-ordered
production slice without touching Runtime22 `state.rs`:

- `RenderFrameExtract` is a bounded timing/view overlay over one
  `Arc<RenderFrameScenePayload>`; geometry, animation, lighting, environment, post process, debug,
  sprites, particles, and visibility are independently shallow-shared Arc domains.
- Cache hit and miss-retain clone the compact overlay and domain handles. `current_extract` applies
  editor camera and timing only to the returned submission. The sealed diagnostics summary remains
  generation-owned and `full_clones/full_clone_bytes` now report zero for these cache operations.
- World and level producers use the canonical payload constructor. Current source contains no old
  `RenderFrameExtract { ... }` literal, and whole-domain synthetic assignments were migrated to the
  COW-domain contract instead of preserving an owned compatibility DTO.
- Focused regressions prove unchanged clone pointer identity, submission-local view/timing,
  real-cache overlay isolation, per-domain COW isolation, and independent rebuild admission for
  `ChangeTick`, `lifecycle_visibility_revision`, `active_camera`, and `viewport_size`.
- The ignored 1/1k/10k mesh/light/sprite benchmark now reports allocation count, requested bytes,
  copied scene bytes, peak live bytes, and P50/P95 elapsed time for the exact clone operation used
  by cache retain and cache return.
- `RenderFrameExtract::DerefMut` and mutable shared-domain COW remain transitional renderer
  migration surfaces. The submission type is not yet immutable, so this slice must not be reported
  as the canonical architecture hard cut or as M4 zero-allocation acceptance.

Static evidence: core touched files pass
`rustfmt +1.94.1 --check --config skip_children=true`; all leased paths pass `git diff --check`;
the current exact source hashes are recorded in
`2026-08-31-shared-render-frame-scene-payload-current-review.md`.

Managed release ticket `693854d59b4140968c949926560c2d5f` was submitted with initial status
`queued`, then superseded by the real-cache regression, diagnostics correction, and V3 benchmark.
It is not polled and cannot accept current source. No Cargo-green or optimized measurement is
claimed yet. The earlier
baseline ticket `5979086274ae4e01b969a6155934c39f` failed before the benchmark on an incomplete
validation source union; its missing method already exists in current source, so no duplicate fix
was admitted.

The superseding current-source behavior ticket is `b27f2c10f9114236973377181a51b9cb` and the
release V3 performance ticket is `e479bfc0b2e64aaea6689f589b339a92`. Both use source-manifest
SHA-256 `067bda0cb4e18f1adf2ed54864c0a3548762149572918b25e8a502df35b23cd3`,
returned initial status `queued`, and are intentionally not polled.

Current status:
`open / M1_shared_storage_source_complete / immutable_submission_gate_open / managed_acceptance_pending / M2_renderer_derivation_and_M3_generation_parity_open`.
