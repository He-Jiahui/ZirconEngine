---
title: Sound Spatial Environment Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-spatial-environment-current-source-algorithm-performance-review.md
---

# Sound Spatial Environment Current-Source Protected Plan Routing

## Review ledger status

Sound spatial/environment **38/38 Rust files** completed E3 current-worktree static review at `9a217cce07c574cbec8dda70b3e1142eeedbc9a9`; fingerprint `44f668ed5a1d791ebaed4d89d8b9fee3d09d7d2b477ef8cc37dd88caaed5f4c6`. All files pass standalone rustfmt and scoped diff check. The shared low-pass allocation-removal edit is preserved; no additional source changed. Protected ledgers remain unchanged because Cargo, current-source Sound activation, callback profiling, audible parity, ETW and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Spatial/environment code has no production render consumer | Plugins11 + Runtime08b + Editor17 | Establish the current-source runtime/preview call graph and truthful capability surface before performance claims. |
| Existing HRTF/convolution/Doppler algorithms are not acceptable for product integration | Plugins11 + Runtime08b | Keep advanced features Unsupported until stateful provider adapters exist; implement pitch Doppler, not gain modulation. |
| DSP lacks persistent per-source state and callback-safe ownership | Plugins11 + Runtime08b + Runtime59 | Add per-source processors, bounded audio commands, reusable scratch/tails and zero steady-state callback allocation. |
| Listener/volume/acoustic selection repeatedly scans full collections | Runtime05 + Runtime60 + Plugins11 | Extract stable slots/generations and spatially query changed sources outside the audio callback. |
| Ray/acoustic selection has no physics-query owner or worker budget | Runtime08a + Plugins12 + Runtime59 | Batch asynchronous physics/acoustic work, cache by scene generation and publish compact source parameters. |
| Low-pass resets state at each block; pan lacks equal-power/layout behavior | Plugins11 + Runtime08b | Replace block-local helpers with continuous multi-channel processors and deterministic split-block fixtures. |
| Convolution is direct, duplicate-prone and truncates IR tails | Plugins11 + Runtime08b | Use a proven partitioned/FFT provider with tail/quality/overload policy; do not tune the current loop. |
| Current allocation benchmark preserves legacy wrong semantics and is unexecuted | Runtime03 + Plugins11 | Retain it only as local regression evidence; add callback, waveform, frequency and continuity product gates. |

## Acceptance routing

Implementation order is capability truth -> scene extraction -> backend processor ownership -> MVP spatial correctness -> continuous filter -> advanced provider adapters -> dynamic qualification. MVP attenuation/pan/pitch must be correct and bounded before HRTF, convolution or ray acoustics expands the callback surface.

Dynamic acceptance records exact source/build/config/device/workload identity, active/changed source and volume counts, block/rate/channel/IR/HRIR sizes, audio/control/main CPU, callback P50/P95/P99/max, allocations, wakeups, underruns, latency, RSS, power, direction/impulse/frequency parity and overload degradation behavior.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
