---
title: Runtime 73 Flat Prototype Owned-field Move
category: zircon_runtime
report_id: Runtime73-flat-prototype-node-move-2026-08-25
date: 2026-08-31
session_id: root-runtime73-flat-prototype-release-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime 73 Flat Prototype Allocation Reduction

## Scope

This batch advances `RST-P1-026` and the allocation evidence required by `RST-G32`. It removes
redundant owned-field clones while materializing the already-owned flat node table into the
prototype arena, and removes owned node-ID clones from successful reachability validation. It does
not claim completion of the parent plan's computed-style arena,
structural sharing, typed property, selector dependency, theme, or product-scale qualification.

## Change

`FlatUiAssetDocument::into_raw_prototype` previously iterated over `&self.nodes`, cloning the node
ID, fifteen owned node fields, and each child mount/slot even though the document was consumed.
Materialization now consumes the node table and each `FlatUiNodeDefinition`, moving those owners
directly into `UiNodePrototype`. The stable handle index remains the only required node-ID clone;
missing-child diagnostics, handle ordering, and prototype output are unchanged.

Reachability validation previously cloned the root ID, every visited ID, and every traversed child
ID into its DFS stack and two `BTreeSet`s. The stack now borrows IDs from the immutable flat node
table, while one dense three-state vector indexes `Unseen/Visiting/Visited` by the already-validated
prototype handle. Missing-node and cycle diagnostics allocate owned IDs only on failure. Successful
validation semantics and deterministic traversal order remain unchanged.

For a 10,000-node chain with 9,999 child edges, deterministic owned clone operations change from
`189,998 -> 10,000`, a `94.737%` reduction. The ignored release benchmark builds fixtures outside
the timed region, runs four paired warmups, and records 21 alternating sample pairs with
nearest-rank P50/P95. Acceptance requires optimized projection P95 to be at most 50% of legacy
P95; measured timings remain pending the managed Windows-native validation ticket.

For the same 10,000-node chain, successful reachability validation changes deterministic node-ID
clones from `20,000 -> 0`, a `100%` reduction. A second four-warmup, 21-pair alternating release
benchmark requires borrowed-validation P95 to be at most 60% of the legacy owned traversal.

## Validation

- Current baseline: Git HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784`, coordinator epoch
  `575`.
- Original TDD red state: the source contract errored because the consuming helper and benchmark
  module did not exist.
- Release-r2 TDD red state: the new four-paired-warmup guard failed while the other three source
  contracts passed against the single-warmup benchmark.
- Release-r2 source performance contract after implementation: 4/4 passed.
- `rustfmt +1.94.1 --check` and scoped whitespace validation: passed.
- A dependency-free `rustc -C opt-level=3` preflight of the same 10,000-node traversal measured
  `28.026 ms -> 13.992 ms` P95 (50.075% reduction). This is diagnostic evidence only; final timing
  remains bound to the managed integrated ticket.
- Rust regressions compare the move projection against the prior clone projection and compare
  borrowed validation against the owned traversal for success, missing-child, and cycle outcomes.
- Coordinator ticket `e5ed453e3c0841e9ac4e1542ac0ed20f` failed before executing Rust because a
  concurrently changed graphics test referenced absent `graphics/dynamic_api/runtime_loop.rs`
  during source-closure planning. No Runtime73 test or performance gate ran in that ticket.
- Release-r2 validation request `1bb16ae1ba94429088347f8284cd9d8b` batches two behavior-parity
  regressions and both ignored release performance gates under the `flat_prototype_` filter. The
  managed command is `cargo +1.94.1 test -p zircon_runtime --locked --release --jobs 1 --
  flat_prototype_ --include-ignored --nocapture --test-threads=1` with four expected tests.
- Focused Runtime UI behavior and both performance gates are pending this one asynchronous
  Windows-native coordinator ticket. Terminal evidence must report raw sample arrays and exact
  P50/P95 for both paths before commit, push, optimization-record closeout, or WeCom notification.
- No local Cargo lane or Cargo dry-run was launched, polled, or terminated.

## Remaining Parent-plan Work

Runtime73 still requires the typed style schema, compiled bundle parity, component scope and part
semantics, computed-style sharing, precise mutation dependency indices, theme generation/reload,
transition runtime, MUI authority cutover, and full WOC/Editor pixel and scale qualification.
