---
title: Virtual Geometry Validation Current-Source Coverage Review
date: 2026-08-24
scope:
  - zircon_plugins/virtual_geometry/editor/src/tests.rs
  - zircon_plugins/virtual_geometry/runtime/src/tests.rs
  - zircon_plugins/virtual_geometry/runtime/src/test_support
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/**/tests
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/**/*tests.rs
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/17-first-party-virtual-geometry-source-runtime-editor-dist-catalog-asset-cook-cluster-page-streaming-culling-raster-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteCullRaster.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Rendering/NaniteStreamingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Nanite/NaniteStreamingPageUploader.cpp
---

# Virtual Geometry Validation Current-Source Coverage Review

## 1. Coverage

At revision `79f64878f3b9526517644c055ad3bf5cadfccd0f`, dedicated validation is **65/65 Rust files**, **28,821 physical / 27,146 non-empty lines**, **1,063,175 bytes**, **247 test attributes** and **30 ignored tests**. Its ordered `workspace-relative path + NUL + raw bytes + NUL` fingerprint is `981677bdce5a2557902dacd5331e278df0e6c912c987222cca54b8f63518d2f4`.

Together with the 200-file non-validation set, Virtual Geometry is **265/265 Rust files**, **44,656 physical / 41,643 non-empty lines**, **1,641,840 bytes**, **295 test attributes** and **33 ignored tests**. Composite fingerprint: `6a04da8a30e9a93b78f8c4e90e4128e9988b7d40452b78e11f67bc8883220396`.

| Validation scope | Files | Tests | Ignored | Static result |
|---|---:|---:|---:|---|
| Editor/runtime harness and support | 4 | 4 | 0 | Registration and fixture wiring reviewed. |
| `test_sources` | 14 | 115 | 0 | Five sources are wired; nine renderer-era sources are deliberately unwired. |
| Executed-selection and node/cull pass test folders | 13 | 52 | 0 | CPU worklist, metadata, packing and buffer-handle assertions reviewed. |
| Allocation/performance/packing files | 34 | 76 | 30 | Local equivalence and microbenchmark contracts reviewed. |

## 2. Findings that block acceptance

### P0: 88 renderer tests are source inventory, not executable coverage

`virtual_geometry/mod.rs:12-26` wires only five `test_sources` files, totaling **2,001 lines / 27 tests**. `virtual_geometry_renderer_test_promotion_guard.rs:3-24` explicitly requires nine other files to stay unwired until their runtime-owner imports and renderer-private fixtures move to plugin-local/public neutral contracts. Those nine files contain **15,289 lines / 88 test attributes**. They cover GPU uploader/readback, prepared rendering, args/submission authority, execution order and unified indirect behavior, but current Cargo tests do not compile or execute them.

The active Rust test tree therefore contains **207 test attributes**, not 295: 174 default plus 33 ignored. Test-file presence must not be reported as executed coverage.

### P0: offscreen pass tests do not submit GPU work

Node/cull test support creates a high-performance offscreen adapter, device, encoder and a fresh `VirtualGeometryGpuResources` for every call (`tests/prelude.rs:38-61`, `tests/support.rs:44-66`). It invokes the CPU pass builder but never calls `queue.submit` or finishes/submits the encoder. Assertions that GPU buffers are `Some` prove allocation/packing only, not compute execution, completion, resource lifetime, readback latency or rendered output.

The three wired public framework-stat tests call `submit_frame_extract` and then inspect counters. They do not assert a draw/dispatch command, timestamp, per-pixel visbuffer/depth, frame latency or power. This matches the production finding that registered executors only validate context.

### P1: local microbenchmarks cannot qualify the product topology

Thirty performance tests are ignored release-only comparisons of allocation capacity, stable versus unstable sort, repeated scans versus indexes, or single versus multiple projections. They do not traverse the registered product executors, cook/load a self-contained cluster artifact, stream real page bytes, reuse a persistent page pool, submit GPU culling/raster, or measure a complete frame.

Eight default test files use `Instant::now` and hard-assert that one local implementation is faster: descendant collection, page-payload packing, eviction selection, pending-request preparation, visible-cluster preparation, child decision, page-request deduplication and overlay lookup. These are vulnerable to scheduler/frequency noise and should not be CI correctness gates. Preserve semantic equivalence tests; move elapsed comparisons to controlled release benchmarks and gate complexity with deterministic counters.

### P1: the wired CPU reference validates a debug teaching model, not Nanite scale

The 1,248-line CPU-reference suite repeatedly rebuilds the same three-cluster fixture and validates many overlapping maps/worklists. It is useful as a small semantic oracle, but it does not test partition quality, DAG reduction, compressed byte layout, large hierarchy traversal, request overflow, dependency fixups or differential GPU results. The imported-extract test also creates temporary files through `std::env::temp_dir`; future execution must redirect `TEMP`/`TMP` to an E/D/F validation directory so artifacts do not land on C.

## 3. Unreal-grounded validation target

The acceptance suite must observe the ownership demonstrated by Unreal rather than merely reproduce Zircon DTOs:

- `NaniteStreamingManager.cpp:2536-2609` and `:3305-3342` separate begin/async/end update and wait for task-graph completion before upload.
- `NaniteStreamingPageUploader.cpp:137-239` uploads actual page/dependency bytes through reusable storage; `:309-361` dispatches dependency-ordered GPU transcode.
- `NaniteCullRaster.cpp:804-1025`, `:4486-4860` and `:1517-2228` execute real instance/node/cluster cull and hardware/software raster paths with indirect GPU work.

Add deterministic counters for cooked/source/resident/upload/readback bytes, loaded models, hierarchy rebuilds, CPU-reference builds, buffer/pipeline creations, queue submissions, dispatches/draws, visited nodes/clusters, requests/overflows and debug snapshot bytes. Then run cold/warm and scale matrices with raw samples plus p50/p95/p99 CPU/GPU time, RSS/VRAM, scheduler wait, wakeups and power. Validate real depth/visbuffer pixels and ordinary-mesh fallback before RenderDoc inspection.

## 4. Execution status

- Static validation review: **65/65 complete**; full module static review: **265/265 complete**.
- Owned-report diff, frontmatter-shape and referenced-path checks: passed; protected ledger hashes are unchanged.
- Repository-wide plan-record audit still reports four pre-existing child-record-limit violations under Editor and Shader plans; none is owned by this review.
- Cargo tests: pending because the managed Windows validation session is unavailable; raw Cargo was not used.
- WPR and WPAExporter are installed, but ETW/power capture is pending because no launchable current-source product executable exists.
- RenderDoc CLI is unavailable; GPU timestamps and pixel/draw/resource capture remain pending.
- No bottleneck is declared removed, no Unreal latency/power parity is claimed, and no protected ledger update, milestone commit or WeCom completion message is warranted.
