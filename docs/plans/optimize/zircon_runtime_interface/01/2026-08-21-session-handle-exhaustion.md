# Runtime Interface 01 Checked Session Handle Allocation Record

- Date: 2026-08-21
- Owner: `optimize-identity-handle-batch-r2-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md`, P1-23
- Status: implementation and deterministic evidence complete; combined managed validation pending

## Problem

The dynamic session registry allocated handles with `AtomicU64::fetch_add` even
though allocation already occurred under the registry mutex. Exhaustion wrapped
to the reserved zero handle and could later collide with a live or retired
session identity. The C ABI create entry also left caller-owned output unchanged
when validation failed before registration.

## Change

- The registry mutex now owns a checked scalar allocator. It emits every value
  from 1 through `u64::MAX` at most once and uses zero only as the permanent
  exhausted state.
- Both linked and C ABI creation use fallible insertion. Linked callers receive
  `SessionHandleSpaceExhausted`; the ABI returns `LimitExceeded` with a stable
  diagnostic.
- Failed allocation shuts down the newly constructed session before returning,
  so no runtime resources survive a rejected admission.
- The ABI invalidates `out_session` before any config validation, preventing a
  stale caller value from appearing valid after failure.
- Runtime 15 structure guards now lock the fallible registry owner and checked
  allocation contract instead of the retired atomic insertion text.

## Deterministic Performance Evidence

The release gate isolates the allocator operation that executes under the same
registry mutex in production. Each sample performs 250,000 allocations; the
legacy branch uses `SeqCst` atomic fetch-add and the optimized branch uses the
checked scalar owner.

| Measure | Legacy | Optimized | Gate |
|---|---:|---:|---:|
| Sample pairs | 21 | 21 | alternating first-run order |
| Allocations per sample | 250,000 | 250,000 | exact |
| Nearest-rank P95 | pending | pending | optimized ratio <= 7,500 bps |

The marker emits both raw 21-value nanosecond arrays. The external validator
recomputes nearest-rank P95 and the basis-point ratio before accepting the
threshold. Exact Windows values remain pending the post-Main managed copy.

## Acceptance

- Near exhaustion allocates `u64::MAX` once, then returns the typed exhaustion
  result on every later attempt; zero is never returned as a live handle.
- Invalid ABI configuration leaves `out_session` equal to
  `ZrRuntimeSessionHandle::invalid()`.
- Four handle-allocation behavior tests, two Level registry contracts, and the
  release benchmark run in one identity batch rather than per-task Cargo runs.
- The pinned validator is `zircon-validation-identity-handle-batch.ps1`,
  SHA-256 `13037D4A78F75FC3A63DA7BE9BA33F97D1622312E62E29115D2772A93580DE93`.
- Exact-file Rustfmt and scoped diff checks pass. Cargo regressions and timing
  remain pending; no performance result is inferred from source inspection.

## Remaining Scope

This closes P1-23 for runtime session allocation only. Generation-qualified
reuse, cross-owner identity taxonomy, capacity telemetry, and the other ABI
handle families remain owned by their numbered findings.
