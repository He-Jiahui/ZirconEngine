---
title: Sound Test DSP and Filter Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-test-dsp-filter-current-source-algorithm-performance-review.md
---

# Sound Test DSP and Filter Current-Source Protected Plan Routing

## Review ledger status

Sound test DSP/filter **17/17 Rust files** completed E3 current-worktree static review at `39f7f45c5671b1b8515685198f000989a0f1d82a`; fingerprint `89b9bbebe742954cc19036c928cffdb242aa0346dd6b80c4db2306bcc0f38edf`. All files pass standalone rustfmt and scoped diff check; no source changed. Protected ledgers remain unchanged because managed Cargo and a current-source effect workload are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| All 17 files and 36 tests are `cfg(test)` only with no production consumer | Plugins11 + Runtime08b | Decide hard cutover: remove misleading duplicate DSP or retain only truthful independent reference utilities. |
| Golden tests preserve placeholder sample values, not product effect contracts | Plugins11 + Runtime03 | Replace with response, continuity, latency/tail, allocation and overload fixtures against the selected provider. |
| Phaser/limiter/reverb/modulated feedback names overstate their algorithms | Plugins11 + Editor17 | Do not expose these as capabilities; rename truthful utilities or keep the feature Unsupported. |
| Block clones and front-drained history would be callback hazards if promoted | Runtime08b + Runtime59 | Backend processor owns preallocated circular/fixed-capacity state and bounded scratch. |
| Stateful biquad is more correct than production low-pass but unreachable | Plugins11 + Runtime08b | Integrate persistent filter state through the backend owner with parameter ramps; do not maintain two filter paths. |
| Meter reads only stereo block data and has no publisher/ballistics | Plugins11 + Editor17 | Publish bounded backend-owned meter generations; Editor must not scan audio buffers. |

## Acceptance routing

Implementation order is capability inventory -> test-contract cleanup -> test-only hard-cutover decision -> backend processor lifecycle -> effect admission -> dynamic qualification. Do not count `cfg(test)` DSP tests as evidence that runtime or Editor effects exist.

Dynamic acceptance records exact source/build/config/device/workload identity, source/track/submix/effect counts, block/rate/channel sizes, audio/control/main CPU, callback P50/P95/P99/max, allocations, latency/tails, underruns, RSS, wakeups, power and response parity.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
