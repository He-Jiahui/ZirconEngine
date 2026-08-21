# Plugins10 Lazy Replication Payload-Clone Optimization Record

- Date: 2026-08-19
- Owner: `plugins10-rpc-heap-lazy-snapshot-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md`, NNET-P1-041
- Status: implementation complete; combined managed validation pending

## Problem

Replication scheduling cloned every complete snapshot, including all field
payload bytes, before interest, update-frequency, snapshot-count, and byte-budget
admission. A client interested in one small slice still paid O(total payload
bytes) allocation and copy on every schedule call.

## Change

- Candidate ordering now carries only the stable `(object, component type)` key,
  priority, and update interval.
- Interest, due-time, and budget checks borrow the canonical snapshot.
- The complete snapshot is cloned only after all admission checks pass.
- Priority/object/component ordering, skip/defer counters, byte accounting, and
  replication timestamps remain unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Candidate build: 8,192 snapshots x 4,096 payload bytes | 33,554,432 eager payload-clone bytes | 0 eager payload-clone bytes | 100% |
| Interest/due/budget rejected snapshot | full snapshot clone | no snapshot clone | 100% |
| Admitted snapshot | one snapshot clone | one snapshot clone | unchanged |

## Acceptance

- Existing replication priority, interest, due-time, snapshot-count, and byte
  budget regressions remain in the combined replication test batch.
- `lazy_replication_payload_clone_release_benchmark_evidence` compares 21
  paired, alternating release samples for the 32 MiB candidate workload and
  computes nearest-rank P50/P95.
- Timing gate: lightweight candidate P95 must be no more than 50% of legacy P95.
- Exact-file Rustfmt, Cargo regression, and release P50/P95: pending one batched
  Windows coordinator validation with the RPC priority-heap task.

## Remaining Scope

Scheduling still scans and sorts every snapshot key each tick, clones component
key strings, and uses a feature-local manager. This record closes eager payload
deep-copy amplification only; incremental dirty scheduling, persistent priority,
complete wire-byte accounting, typed interpolation/prediction, and
NNET-P1-041/G24 remain open.
