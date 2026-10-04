---
title: Sound HRTF and IR Resource Service Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-hrtf-ir-resource-service-current-source-performance-review.md
---

# Sound HRTF and IR Resource Service Current-Source Protected Plan Routing

## Review ledger status

The remaining Sound HRTF/IR resource service **2/2 Rust files** completed E3 current-worktree static review at `c02a7fb7c4b90381b9e701008bc8a2898fc09263`; fingerprint `74764428af26f4fec5667f2bb8f6373ec31db6651dbcb370744ebb5ae5b638c9`. A redundant full ray-descriptor/sample-map clone on IR removal was removed; direct rustfmt, source contract and scoped diff checks pass. Protected ledgers remain unchanged because managed Cargo and current-source dynamic evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| HRTF/IR maps have no format/count/frame/byte/prepared-residency budget | Plugins11 + Runtime08b + Runtime04 + Runtime64 | Admit versioned provider assets under exact raw/prepared budgets and leases. |
| Static overwrite can leave ray-derived metadata/status on unrelated samples | Plugins11 + Runtime08b | Use one canonical entry with explicit origin/provider/generation and atomic replacement. |
| Any HRTF edit clears all active HRTF render states | Plugins11 + Runtime08b + Runtime59 | Maintain reverse dependencies and invalidate/switch only the affected generation. |
| HRTF catalog deep-copies all kernels under the global manager lock | Plugins11 + Editor17 + Runtime03 | Publish lightweight immutable metadata generations; full payload access is explicit and bounded. |
| Accepted raw resources are not compiled into an actual render provider | Plugins11 + Runtime08b | Prepare sample-rate/channel/partition-specific immutable objects and prove render reachability. |
| IR removal cloned all ray descriptors/samples to refresh status | Plugins11 + Runtime08b | Scoped fix completed: clone count 1 -> 0 and copied cached sample elements -> 0; retain in regression coverage. |

## Acceptance routing

Implementation order is resource identity/budgets -> prepared immutable handles -> selective dependency invalidation -> lightweight catalogs -> truthful incremental status -> dynamic qualification. Do not close this scope on the local clone removal; it only removes avoidable work from an otherwise unbounded and currently unreachable provider model.

Dynamic acceptance records exact source/build/project/provider/device identity, resource count/raw/prepared bytes, prepare/switch P50/P95/P99, callback HRTF/convolution time, copied/allocated bytes, lock wait, cache hit/eviction, tail discontinuity, RSS, underruns, CPU, wakeups and power.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
