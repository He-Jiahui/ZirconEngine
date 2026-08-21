# Runtime24 Checked Level Handle and Stable Traversal Record

- Date: 2026-08-21
- Owner: `optimize-identity-handle-batch-r2-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md`, IDENTITY-P1-029
- Status: implementation complete; combined managed validation pending

## Problem

`DefaultLevelManager` allocated `WorldHandle` values with unchecked atomic
addition. Exhaustion could wrap to the reserved zero identity and overwrite a
registry entry. Multi-level world traversal also inherited randomized HashMap
iteration order, so callbacks and multi-world lock acquisition had no stable
owner order.

## Change

- `try_create_level` uses checked atomic update. Counter state `u64::MAX - 1`
  emits `WorldHandle::MAX` once; state `u64::MAX` returns typed
  `CoreError::LevelHandleExhausted` without mutation.
- The existing infallible convenience methods remain explicit panic wrappers;
  fallible runtime paths retain the typed result.
- World snapshots and multi-world lock acquisition are sorted by raw
  `WorldHandle` before traversal. Registry locking remains separate from normal
  per-world callback work.
- Source guards reject the retired unchecked fetch-add path and require all
  three traversal owners to use the common handle sorter.

## Deterministic Evidence

| Contract | Before | After |
|---|---|---|
| Terminal allocation | wrap to zero | `MAX` once, then typed exhaustion |
| World traversal order | HashMap iteration | ascending `WorldHandle` |
| Live registry collision at wrap | possible | rejected before insertion |

No isolated timing improvement is claimed for the ordering repair. The same
identity batch carries the measured Interface01 allocator gate; Runtime24 is
accepted on boundary behavior and deterministic ordering.

## Acceptance

- `level_handle_allocation_accepts_the_maximum_once_then_reports_exhaustion`
  proves the terminal value, persistent error, live lookup, and zero exclusion.
- `level_manager_registry_orders_world_snapshots_by_handle` proves ascending
  traversal independently of HashMap seed.
- `level_manager_registry_locks_do_not_cross_world_work` locks the shared sorter,
  checked allocator, and callback lock boundary.
- The pinned combined validator is `zircon-validation-identity-handle-batch.ps1`,
  SHA-256 `13037D4A78F75FC3A63DA7BE9BA33F97D1622312E62E29115D2772A93580DE93`.
- Exact-file Rustfmt and scoped diff checks pass. Cargo behavior remains pending
  the post-Main managed copy.

## Remaining Scope

This slice does not add owner epochs or make serialized `WorldHandle` a durable
asset identity. World replacement epochs, stale-reference classification,
generation rollover, and the other IDENTITY-P1 findings remain open.
