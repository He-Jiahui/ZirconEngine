---
title: Hybrid GI Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_reviews:
  - docs/plans/performance/01/2026-08-24-hybrid-gi-product-ownership-current-source-algorithm-performance-review.md
  - docs/plans/performance/01/2026-08-24-hybrid-gi-scene-cache-trace-current-source-algorithm-performance-review.md
  - docs/plans/performance/01/2026-08-24-hybrid-gi-validation-current-source-coverage-review.md
---

# Hybrid GI Current-Source Protected Plan Routing

## Review ledger status

Hybrid GI's **210-file non-validation set** and **40-file validation set** completed current-worktree static review at repository revision `79f64878f3b9526517644c055ad3bf5cadfccd0f`. Their fingerprints are `1f52074f574732fe0dfa89d6fd27a275c6caaad9e057017c0768f3db67a1199a` and `2a0b32136d27bc24bdeb23254badb43e082ad983d756f75355edcc72d8c2cd4d`; the **250-file** composite fingerprint is `69adb0c4b76810600a3cb466e6a63d18e9bebbfbbecdd9f251cc142146385a63`. Dynamic validation is unavailable, so protected `review.md` and `pending.md` remain unchanged.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Core visibility/post-process and plugin both own GI probe/trace/history/composition | Runtime09f3 + Runtime98 + Runtime89 + Plugins19 | Hard-cutover to one typed GI product and delete duplicate core loops/state after migration. |
| Core visibility constructs empty Hybrid GI probe/update/feedback/request state | Runtime94 + Runtime09b + Runtime09f3 | Define visibility-owned inputs only; publish real view/depth/normal/motion generations to the GI owner. |
| Three executors recreate layouts, shaders, pipelines, bind groups and params per execute | Runtime89 + Runtime09a + Runtime09c + Plugins19 | Move device-generation resources and specialization cache to the render resource authority; require zero warm-frame creation. |
| One global mutex spans scene projection, Global SDF/HGI encoding and readback enqueue | Runtime89 + Runtime09a + Plugins19 | Publish immutable generations and schedule independent bounded queues without a cross-subsystem critical section. |
| Front-only completion and shared HGI/Global-SDF in-flight budget cause head-of-line blocking | Runtime89 + Runtime09a + Plugins19 | Consume newest ready generations, add supersession/cancellation and separate bounded feedback rings. |
| Full scene/light sort/clone, spherical cards and first-card screen probes | Plugins19 + Runtime98 + Runtime09d | Implement incremental render scene, cooked cards, sparse surface cache and view-derived adaptive probes. |
| Global SDF rebuilds CPU page state and creates seven buffers per batch | Plugins19 + Runtime98 + Runtime09d | Persist clipmap page tables/allocators/atlases and compact changed-object work on GPU. |
| Any light change dirties all resident pages | Runtime95 + Runtime09e + Plugins19 | Route light influence bounds/generations to affected cards/pages only. |
| Plugin GI, history and baked ambient are composed through overlapping ownership | Runtime97 + Runtime09f2 + Runtime09f3 | Define one lighting-composition contract and explicit baked/dynamic fallback precedence. |
| Hardware trace would duplicate scene ownership without acceleration-structure authority | Runtime28 + Runtime09f3 + Plugins19 | Add HRT only as a registered trace backend after Runtime28 owns AS lifetime and scheduling. |
| Editor default-enables experimental GI and points at incomplete authoring asset | Plugins08 + Plugins19 + Editor22 + Editor58 + Editor68 + Editor69 | Make capability/profile truthful; expose real generations, invalidation, budgets, timestamps and capture controls. |
| 26 direct and 2 helper GPU-unavailable paths can leave tests green | Plugins19 + Runtime89 | Publish adapter-required executed/skipped manifest and require executed current-source GPU qualification. |
| Host encode spans are reported without GPU timestamps | Runtime89 + Plugins19 + Editor69 | Add timestamp-query ownership and calibrated delayed resolve; label host encode time accurately. |

## Acceptance routing

Implementation order is unique GI authority -> persistent render resources -> immutable scene deltas and independent queues -> real surface cache -> camera-derived probes -> persistent Global SDF -> directional radiance cache -> temporal reconstruction -> authoring/scalability -> dynamic qualification.

The existing Runtime09f3 plan already owns its P0 Hybrid GI hard-cutover items. This record revalidates and routes current-source evidence; it does not duplicate those items or mutate protected plans.

Dynamic acceptance must identify exact source/build/executable/config/device/scene and record CPU scheduling, allocations, scene deltas, page/probe/trace work, upload/readback bytes, dispatches, timestamped GPU passes, frame p50/p95/p99, RSS/VRAM, wakeups, power and pixel/resource parity. WPR/WPAExporter are present, but no launchable current-source executable exists; RenderDoc CLI is absent. No Git milestone commit or quantified WeCom message is warranted by this static routing record.
