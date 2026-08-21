# Runtime49 Shared Immutable Gizmo Asset Commands

- Date: 2026-08-20
- Owner: `optimize-runtime49-gizmo-shared-asset-r2-01a00797-20260820`
- Source plan: `docs/plans/optimize/zircon_runtime/49-runtime-debug-gizmo-command-buffer-retained-extract-filter-budget-render-product-integration-review.md`, GIZMO-P1-006 / M2
- Status: implementation complete; combined managed validation pending

## Problem

`GizmoAsset` owned a `Vec<GizmoCommand>`. Cloning an asset or a
`RetainedGizmo` therefore allocated and copied every command even though the
asset command stream is immutable after construction. N retained instances of
one asset paid O(N * commands) memory traffic before any overlay extraction.

## Change

- `GizmoAsset` stores commands in `Arc<[GizmoCommand]>`.
- `from_buffer` still performs one owned snapshot of the mutable source buffer.
- Asset and retained-instance clones now share that immutable snapshot and only
  update the Arc reference count.
- The public `commands()` slice and derived `{ commands: [...] }` serde shape are
  unchanged. A round-trip regression locks the serialized contract.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 20,000 commands, 32 clones/sample, 21 sample pairs | 13,440,000 command copies | 0 command copies | 100% |
| One asset/retained clone | O(commands) + allocation | O(1) Arc clone | one complexity class |
| Initial mutable-buffer snapshot | one command copy | one command copy | unchanged |

The ignored release benchmark alternates legacy and shared-first order across
21 pairs, emits every raw nanosecond sample, and computes nearest-rank P50/P95.
Acceptance requires shared-clone P95 to be no more than 25% of legacy P95.
Exact Windows timing values remain pending the serialized coordinator batch.

## Acceptance

- `gizmo_asset_clones_share_immutable_commands_and_preserve_serde_shape`
  requires asset and retained clones to share the same command slice and proves
  serde round-trip equality.
- Existing extraction tests continue to consume `commands()` without learning
  the storage owner.
- `gizmo_asset_shared_command_clone_release_benchmark_evidence` provides the
  21-pair raw distribution and 25% P95 gate.
- The current combined managed validator covers eight logical tasks in twelve
  Cargo groups: Runtime45, Runtime48, Runtime49, and the five prepared
  Runtime08C animation slices. It retains seven independent performance gates;
  Runtime45 owns four, Runtime49 owns one, and Runtime48 changes no production
  behavior and receives none. Validator SHA-256:
  `A2C1864BDCA19026FD02493EC066031AF95CE6A050E59A608859C64FBC9E0943`.
- Exact-file Rust 1.94.1 rustfmt and scoped `git diff --check`: passed.
- Cargo regressions and release timing: pending behind the active Main batch.

## Remaining Scope

This slice removes repeated command copies but does not yet introduce a
qualified asset handle/generation, retained remove token, TTL, world/session
scope, owner teardown, or plugin quiescence. Those remain open in Runtime49 M2.
