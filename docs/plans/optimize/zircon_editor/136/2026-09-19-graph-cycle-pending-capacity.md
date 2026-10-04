---
title: Editor136 graph cycle pending capacity
category: zircon_editor
report_id: Editor816-graph-cycle-pending-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor816 · graph cycle pending capacity

## Scope

`default_connection_verdict` validates DAG and tree candidates through
`introduces_cycle`. The probe can enqueue the candidate target plus at most one
node for each existing edge: a node's adjacency list is expanded only on its
first visit, after the `visited` set accepts it. The scratch vector therefore
has a safe `edges.len() + 1` bound.

The implementation reserves that bound before pushing the candidate target.
Cycle detection order, duplicate handling, type/direction checks, and the
`GraphConnectRejection::Cycle` contract are unchanged.

## TDD and deterministic model

- The source contract was RED against the original `vec![candidate.to...]`
  initialization and GREEN at `3/3` after the bounded reservation and lower
  regression were added.
- The lower regression checks both the `edges.len() + 1` capacity rule and a
  three-node cycle verdict; the ignored marker
  `EDITOR816_GRAPH_CYCLE_PENDING_CAPACITY_BENCH_V1` is wired for the managed
  Release lane.
- For 4,097 pending slots, the legacy zero-capacity model performs 12
  geometric growth events; the reserved model performs 0 (`12 -> 0`). This is
  allocation-shape evidence only, not allocator, CPU, RSS, or product
  percentile evidence.

## Local validation

- `tools/tests/test_editor_graph_cycle_pending_capacity_performance_contract.py`:
  `3/3`.
- `python -m py_compile` for the contract: pass.
- Scoped `rustfmt --edition 2021 --check` for `zircon_editor/src/ui/graph/model.rs`:
  pass.
- Source SHA-256:
  `402DF3D2DA260F6FB2A0194E147F663810803158405969B1DBE2216BC28433C8`.
- Contract SHA-256:
  `8877AC18E2FFECE25A2EAF326C8E0B25FB00DD4D790B4854B30AFAC67E93010E`.

Managed Windows Cargo/Release execution, lower Rust execution, and graph
product p50/p95/p99 evidence remain pending under the shared external-worktree
admission gate. This session does not poll or monitor the coordinator; tooling
production remains deferred for the later Rust migration.
