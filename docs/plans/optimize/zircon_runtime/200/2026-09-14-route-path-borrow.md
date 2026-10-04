---
title: Runtime200 Generic Route Preview Path Borrow
category: zircon_runtime
report_id: Runtime200-route-path-borrow-2026-09-14
date: 2026-09-14
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime200 Generic Route Preview Path Borrow

## Scope

`populate_generic_route_trace` already owns the bubble and focus paths that
become the public route trace. It previously cloned whichever path was chosen
into a temporary `route_path` solely to build the reversed preview tunnel,
then moved the original vectors into the trace. The implementation now selects
a borrowed preview source and constructs the same tunnel before moving the
owned vectors. Route precedence, empty-bubble fallback, truncation, and wire
shape are unchanged.

This is a narrow input-diagnostics allocation reduction. It does not alter
route authority, capture identity, transaction rollback, or the broader
Runtime200 proposal/commit migration.

## Complexity and deterministic target

For a selected route of depth `H`, the old path paid one temporary `Vec` clone
and `H` node copies before the required reversed preview allocation. The new
path keeps the same `O(H)` preview materialization but removes the intermediate
clone and its allocation (`0` extra path copies). The ignored Release benchmark
uses 512-node routes, 4,096 iterations per sample, 17 alternating pairs, and
emits `RUNTIME200_ROUTE_PATH_BORROW_BENCH_V1`; the dynamic gate requires an
optimized P95 no greater than 80% of the legacy path.

## TDD and local evidence

- The source contract was RED before implementation because the production
  body built `route_path` with `focus_path.clone()`/`bubble_path.clone()` and
  the regression module did not exist.
- The focused route-path contract passes `3/3`.
- The combined Runtime/Editor pointer, reference, history, and input batch
  passes `76/76` in one invocation after this slice.
- Current-source performance-contract discoveries pass Runtime `1201/1201`
  and Editor `622/622` in separate batched invocations.
- A subsequent single-process cross-surface discovery loaded both patterns and
  passed `1823/1823` tests in `22.612s`; this is a local source/model receipt,
  not managed Cargo or Release evidence.
- The Rust regression checks bubble-over-focus precedence and empty-bubble
  fallback; the ignored release marker reports paired samples and explicit
  legacy/optimized copy counts. Scoped Rustfmt, Python compilation, and
  `git diff --check` pass.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/input/route_policy.rs` | `1533D38FE2E7E4C7FE4E18255D20BD8EBCAAC315AD04010EF72417CB7FDAA22F` |
| `zircon_runtime/src/ui/surface/input/route_policy_tests.rs` | `4650255E9CED7F61372B346D375AB107F179AC7431F2AADA7861653C496EB172` |
| `tools/tests/test_runtime_ui_route_path_borrow_performance_contract.py` | `7165F2DF96805654A370860298F8B948037B9E3AD027B2AFC4EED3A8D1663E56` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor Windows Release
batch. Managed compilation, allocator behavior, and input p50/p95/p99 remain
pending under the shared external-worktree admission blocker; no standalone
Cargo invocation or coordinator polling was performed. Tooling production work
remains deferred.
