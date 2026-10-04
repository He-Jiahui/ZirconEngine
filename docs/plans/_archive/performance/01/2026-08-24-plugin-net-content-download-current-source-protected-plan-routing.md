---
title: Plugin Net Content Download Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-net-content-download-current-source-algorithm-performance-review.md
---

# Plugin Net Content Download Protected Plan Routing

## Review ledger status

Content Download **20/20** Rust files completed E3 current-source static review. Shared changes in `manager.rs` and `manager/{attempts,bitmap,resume,state}.rs` were preserved. M0 added checked manifest arithmetic and two unexecuted overflow regressions. Protected `review.md` and `pending.md` remain unchanged because Cargo, product composition, real staging/install, crash/fault, WPR and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Production canonical manager does not receive the private HTTP backend; tests inject it | Runtime08E P1-1/M1-M3 and Plugins10 NNET-P1-043/M7 | Register HTTP capability into one NetworkRuntimeInstance and add real catalog App/Hub/export/native composition tests. |
| Manifest lacks provenance/version/root identity and uses unchecked sizes/ranges | Plugins10 NNET-P1-045/G26 and Tooling09 release manifest owner | Compile a signed versioned artifact manifest with checked limits/layout, target/dependency/origin policy and rollback identity. |
| Sync full-body/prefix/hash path multiplies memory and can block the caller for 30 seconds | Runtime08E P1-4/P1-10/P1-18 and Plugins10 NNET-P1-044 | Replace with async ticket, bounded block staging, incremental hash, cancellation and worker-owned CPU/disk work. |
| Bool bitmap/cache-hit APIs accept unverified completion and stale download IDs retain old state | Plugins10 NNET-P1-044/M7 and Runtime25 | Namespace resume/cache by manifest digest and verify persistent staged/cache bytes before reuse; retire all old generation state. |
| Corrupt prefix is reused across mirrors and cannot trigger full repair | Plugins10 M7 | Invalidate suspect resume checkpoint, bounded-full-refetch, separate origin failure from local corruption and emit typed repair receipt. |
| Cancel does not stop I/O/admission or clean memory; concurrent same-chunk fetch races attempts | Runtime08E cancellation owner and Plugins10 G14/G19/G26 | Add one generation-fenced ticket state machine, in-flight dedup, cancel/join and exactly-one terminal outcome. |
| No stage/space/fsync/verify/atomic publish/rollback/crash recovery | Runtime25 atomic I/O + Tooling09 install owner + Hub03 consumer + Plugins10 NNET-P1-046/M7 | Build durable stage/construct/verify/publish/last-good transaction and fault/kill-point recovery matrix. |
| Local mirror/bitmap/lock/key allocation wins are unexecuted and payload memory still unbounded | Plugins10 current implementation record | Preserve compatible wins; measure only after the bounded product pipeline exists. |

## Acceptance routing

M0 checked manifest arithmetic and regression source are implemented but not Cargo-executed. Product implementation starts with canonical HTTP composition and trusted manifest identity, followed by bounded scheduling, Runtime25 staging/resume/cache, independent verify and atomic publication. Further HashMap/String tuning before these owners exist would optimize a disconnected full-memory model.

Dynamic acceptance requires real App/Hub/export/native catalog activation, remote/local origins, large artifacts, warm/cold cache, corrupt range, slow/stalled origin, disk-full/read-only, cancel race, process kill/restart and soak. WPR/ETW results must bind tail latency, CPU, RSS, disk/network bytes, wakeups and energy to one BuildSet/workload. RenderDoc is used only after the installed asset enters a real rendered frame.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
