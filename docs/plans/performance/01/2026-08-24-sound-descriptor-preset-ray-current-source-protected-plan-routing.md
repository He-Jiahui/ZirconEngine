---
title: Sound Descriptor Preset and Ray Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-descriptor-preset-ray-current-source-algorithm-performance-review.md
---

# Sound Descriptor Preset and Ray Current-Source Protected Plan Routing

## Review ledger status

Sound descriptor validation, built-in presets, ray/IR cache and acoustics service boundary **25/25 Rust files** completed E3 current-worktree static review at `3e56d81da5e572849b51c50506ec65ec35fcf608`; fingerprint `53aa60e0033f06a4740c2d8b83a4cff2587ab60c037cf793bb5bed77b702a74d`. All files pass standalone rustfmt and scoped diff check; no source changed. Protected ledgers remain unchanged because Cargo, a current-source acoustic workflow, ETW/power and applicable RenderDoc evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Ray-tracing surface only ingests externally formed IR descriptors | Plugins11 + Runtime08b + Runtime08a + Plugins12 | Keep unavailable or rename truthfully until a versioned scene/query/acoustics provider exists. |
| Convolution/ray budgets are stored but never enforced | Plugins11 + Runtime08b + Runtime03 | Enforce exact count/frame/byte/work budgets and report requested/applied/observed values. |
| IR sample payload is retained twice and deep-cloned in snapshots | Plugins11 + Runtime08b | Establish one provider-owned immutable/prepared allocation and bounded summary snapshots. |
| Unsupported limiter/reverb presets are unconditionally discoverable | Plugins11 + Editor17 | Derive preset catalog/defaults/activation from the applied provider generation. |
| Source validation rebuilds graph track IDs for every source | Plugins11 + Runtime08b | Compile one graph validation index and benchmark `O(T+B+sends)` preparation. |
| Heavy validation occurs inside the global manager lock | Plugins11 + Runtime59 + Runtime03 | Validate/prepare outside lock; publish by immutable generation with lock-wait telemetry. |
| Acoustic status rescans metadata and reports declared rather than executed rays | Plugins11 + Runtime03 | Provider owns incremental actual-work counters and requested/active/last-good/failure states. |
| Cache has no world/source/listener/volume invalidation contract | Runtime05 + Runtime60 + Plugins11 + Editor17 | Add versioned scene-to-audio proxies and bounded add/update/remove commands. |
| No production caller or dynamic workload exists | Plugins11 + Editor17 + Runtime08b | Build current-source MVP and optional-provider scenarios before ETW/power/RenderDoc acceptance. |

## Acceptance routing

Implementation order is capability/preset truth -> compiled validation -> acoustic provider contract -> bounded IR residency -> scene/audio proxy lifecycle -> truthful diagnostics -> dynamic qualification. Do not close this scope by optimizing the current map scans while the system still lacks a provider, budget enforcement and lifecycle ownership.

Dynamic acceptance records exact source/build/project/target/provider/device/scene identity, frame and callback P50/P95/P99, lock wait, allocations, queue depth, submitted/completed/cancelled jobs, IR build latency, resident/prepared bytes, cache hit/eviction, underruns, RSS, CPU, wakeups and power. RenderDoc is required only after a current-source GPU acoustic/convolution path exists.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
