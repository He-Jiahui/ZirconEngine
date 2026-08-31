---
title: Hybrid GI Validation Current-Source Coverage Review
date: 2026-08-24
scope:
  - zircon_plugins/hybrid_gi/editor/src/tests.rs
  - zircon_plugins/hybrid_gi/runtime/tests
  - zircon_plugins/hybrid_gi/runtime/src/test_support
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/**/tests
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/**/*tests.rs
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/19-first-party-hybrid-gi-source-runtime-editor-dist-catalog-scene-representation-surface-cache-global-sdf-radiance-cache-probe-trace-denoise-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCacheFeedback.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeGather.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadianceCache.cpp
---

# Hybrid GI Validation Current-Source Coverage Review

## 1. Coverage

At revision `79f64878f3b9526517644c055ad3bf5cadfccd0f`, dedicated validation is **40/40 Rust files**, **12,277 physical / 11,559 non-empty lines**, **469,720 bytes**, **139 test attributes** and **20 ignored tests**. Its ordered `workspace-relative path + NUL + raw bytes + NUL` fingerprint is `2a0b32136d27bc24bdeb23254badb43e082ad983d756f75355edcc72d8c2cd4d`.

Together with the 210-file non-validation set, Hybrid GI is **250/250 Rust files**, **37,945 physical / 35,078 non-empty lines**, **1,419,343 bytes**, **240 test attributes** and **21 ignored tests**. Composite fingerprint: `69adb0c4b76810600a3cb466e6a63d18e9bebbfbbecdd9f251cc142146385a63`.

All **17/17** files under `runtime/src/hybrid_gi/test_sources` are wired through the current three module roots. This is materially better than merely storing unwired test source, but it does not mean that GPU paths execute on the validation machine.

## 2. Findings that block acceptance

### P0: GPU-unavailable paths can return a successful test

There are **26 direct** `let Some((device, queue)) = test_device() else { ... }` paths that can leave a test successful without an adapter, plus **2 helper** `test_device()?` paths that propagate absence. Eighteen explicit messages say the test is being skipped. A green aggregate therefore does not prove adapter acquisition, command submission, map completion or shader output.

GPU-required tests must emit a structured executed/skipped/failed manifest including adapter/backend/driver and required feature/limit reasons. Product acceptance requires `executed`, not merely process exit success.

### P0: ignored tests contain much of the scale/performance evidence

The plugin has **21 ignored tests**: 20 in dedicated validation and one inline production release-performance gate. These cover important release-scale and elapsed-time behavior. They are not evidence for default CI and must not be merged into a generic test count without the ignored split.

### P1: host timing and allocation assertions do not validate GPU topology

The suite is strong at DTO packing, deterministic state transitions, buffer contents and small offscreen contracts. It does not establish that the product avoids per-frame pipeline creation, overlaps queues, consumes newest-ready readback, incrementally updates a large scene or produces physically meaningful GI. Host `Instant` around encode calls cannot stand in for timestamp queries.

The one default/release elapsed gate should be converted to semantic equality plus deterministic operation/allocation counters; release timing belongs in a controlled benchmark with raw distributions. The same rule applies to future comparisons: do not hard-assert scheduler-sensitive wall-clock superiority in ordinary correctness tests.

### P1: legacy captures cannot qualify current source

PNG, trace or RDC artifacts without exact source fingerprint, executable hash, adapter/driver, scene, resolution, profile and capture command are historical evidence only. RenderDoc acceptance must launch the current-source product and validate real pixels, draws/dispatches, resources, barriers and timestamp correspondence.

## 3. Required dynamic matrix

1. Build through the managed Windows validator with target/temp/cache paths on D/E/F; record source fingerprint and executable SHA-256.
2. Run CPU-only semantic suites separately from adapter-required suites and publish the executed/skipped manifest.
3. Exercise cold start, unchanged warm frames, camera motion, one-object transform, material edit and one-light edit on small/medium/large scenes at 1080p and 4K.
4. Use GPU timestamp queries for every Hybrid GI stage; use WPR/ETW for main/render/worker scheduling, waits, wakeups, allocations, memory and package power.
5. Use RenderDoc only after the current-source executable reaches the target frame; verify pixels, dispatch dimensions, persistent resources, resource hazards and fallback mode.
6. Report raw samples plus p50/p95/p99, not a single best run. Keep all captures and temporary artifacts off C.

Deterministic counters must include scene add/update/remove, dirty objects/regions/pages, surface captures, cache hits/misses/evictions, probes/traces, overflow/fallback, pipeline/layout/bind-group/buffer creation, uploads/readbacks, queue submissions, timestamp validity and history rejection.

## 4. Execution status

- Static validation review: **40/40 complete**; full plugin static review: **250/250 complete**.
- Cargo tests are pending because the managed Windows validation session is unavailable; raw Cargo was not used.
- WPR and WPAExporter are installed, but capture was not started because no launchable current-source product executable exists.
- RenderDoc CLI is unavailable. No current-source GPU capture exists.
- No bottleneck is declared removed, no latency/power parity is claimed, and no protected-ledger update, milestone commit or WeCom completion message is warranted.
