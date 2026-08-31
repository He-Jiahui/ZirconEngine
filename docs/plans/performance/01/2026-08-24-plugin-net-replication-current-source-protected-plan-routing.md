---
title: Plugin Net Replication Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-net-replication-current-source-algorithm-performance-review.md
---

# Plugin Net Replication Protected Plan Routing

## Review ledger status

Replication **24/24** Rust files completed E3 current-source static review. Shared changes in `apply.rs` and `state.rs` were preserved; this audit made no source change. Protected `review.md` and `pending.md` remain unchanged because Cargo, product, transport-loss, scale/soak and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Feature ignores NetManager and has no World/Reflection/connection/transport caller | Runtime08E P1-15/M6 and Plugins10 M6 | Install one per-World replication owner consuming authenticated canonical connections and ECS/Reflection dirty state. |
| Strategy, authority, type and delta-compression descriptors are inert | Plugins10 NNET-P1-039/G22 and Editor26 schema owner | Compile versioned stable IDs, serializers, conditions, authority, interpolation and migration into a BuildSet artifact. |
| Collect full-materializes fields and performs `O(F^2)` comparisons | Runtime08E M6 | Consume dense ECS/Reflection dirty member masks; no-change work and deltas must be proportional to dirty members. |
| Every connection/tick scans and sorts all snapshots under one mutex | Plugins10 NNET-P1-041/G24 and Runtime08E M6 | Persist global/per-connection candidate state, prefilter, age priority and bounded-partial-select outside main-thread locks. |
| Schedule sends full snapshots and marks time before encode/admission/ACK | Plugins10 NNET-P1-040/G23 and Runtime08E M6 | Add known-object lifecycle, ACKed/pending baseline, packet commit/loss retry, resync and late-join create stream. |
| Payload-only budget can defer oversized or low-priority state forever | Runtime08E M6 and Plugins10 G19/G24 | Budget encoded wire bits/packets, add huge-object path, fairness/aging and typed cannot-fit receipts. |
| Public despawn resets sequence and can make respawn permanently stale | Plugins10 G23 | Unify lifecycle with generation-safe tombstone/create semantics and per-connection acknowledgment. |
| Transform interpolation uses component-name matching and raw first four bytes | Plugins10 NNET-P1-041 and Editor26 | Move interpolation/quantization/time policy into compiled field traits and publish batched immutable presentation buffers. |
| Local lazy clone and borrowed interpolation lookup improvements are unexecuted | Plugins10 current implementation record | Preserve the improvements where compatible, execute ignored baselines and measure the final product pipeline. |

## Acceptance routing

Implementation must start with compiled schema and World dirty extraction, then per-connection graph/baseline lifecycle, scheduling/encode and presentation. Further HashMap/String micro-optimization before these owners exist would optimize the temporary model. Dynamic acceptance must bind all tail latency, CPU, RSS, wire, wakeup and energy results to the same BuildSet and declared multi-client/object workload.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
