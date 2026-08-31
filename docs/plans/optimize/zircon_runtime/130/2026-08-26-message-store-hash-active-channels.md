# Runtime130 Message Store Hash Active Channels

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Status: `implementation_complete / managed_validation_pending`
- Batch: `optimization_batch_20260826cm_`

## Problem

`MessageStore` used a `BTreeSet<TypeId>` for the private set of message channels that need
retention advancement. Writes, clears, and every retained-channel reinsertion therefore paid tree
membership costs even though channel ordering is not observable. `advance_frame` also moved the
set out with `mem::take`, forcing the next frame to rebuild all container storage.

## Optimization

- Replace the private active-channel tree with `HashSet<TypeId>` while retaining the existing
  `HashMap<TypeId, ...>` lookup owners and sorted `registered_type_names()` projection.
- Add one private spare hash set. `advance_frame` swaps the two sets and drains the previous active
  set, so steady-state frames reuse both allocation buffers.
- Preserve clear behavior, retention advancement, active visit metrics, public message order, and
  the absence of any cross-message-type ordering contract.

## Test And Performance Contract

- The behavior regression covers two message types, explicit clearing, one active visit, retained
  messages, and the following zero-visit frame.
- The source regression requires hash membership, the spare set, and `drain`, and rejects the old
  tree field and `mem::take` loop.
- Ignored release evidence prints
  `RUNTIME130_MESSAGE_STORE_HASH_ACTIVE_CHANNELS_BENCH_V1` for 21 alternating sample pairs over
  32,768 channels and eight frame turnovers.
- Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 70`.

## Validation State

Exact Rust 1.94.1 formatting and scoped static checks are required before submission. Cargo tests,
exact P50/P95 values, commit SHA, push result, and WeCom delivery remain coordinator-owned terminal
evidence and are not claimed by this pending record.

