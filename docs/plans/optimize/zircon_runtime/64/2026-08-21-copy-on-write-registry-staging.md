# Runtime64 Copy-on-Write Registry Staging Optimization Record

- Date: 2026-08-21
- Owner: `optimize-runtime64-cow-registry-staging-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md`, RAR-P2-002
- Status: implementation complete; combined managed validation pending

## Problem

`ResourceRegistry::begin_staging` previously cloned both complete registry
indexes and then built a third full identity map. Even a candidate that was
discarded or rejected before its first mutation deep-cloned every
`ResourceRecord` and every locator string. Targeted project scans therefore
paid work proportional to total registry size before doing any useful staging.

## Change

- Registry indexes are immutable `Arc<HashMap<...>>` snapshots. Cloning a
  registry or opening a staging candidate now increments shared owners instead
  of cloning its contents.
- Records and locator keys are independently shared through `Arc`, so the first
  staged mutation copies hash buckets and Arc pointers while unchanged records
  and locator strings retain their original allocation.
- The staging identity guard reads historical kind/locator authority from the
  shared original registry. Only IDs first introduced by the current candidate
  occupy the incremental identity map.
- Existing lookup, `Deref<ResourceRegistry>`, removal, replacement, identity,
  locator-collision, and `finish()` contracts remain unchanged.

## Deterministic Performance Evidence

The release workload opens staging candidates over 4,096 registry records for
16 rounds per timing sample:

| Measure per `begin_staging` | Legacy | Optimized | Gate |
|---|---:|---:|---:|
| Deep `ResourceRecord` clones | 4,096 | 0 | eliminated |
| Deep locator clones | 12,288 | 0 | eliminated |
| Timing distribution | 21 samples | 21 samples | alternating first-run order |
| Nearest-rank P95 | pending | pending | optimized <= 25% of legacy |

The 12,288 legacy locator clones comprise the locator inside each cloned
record, the locator index key, and the historical identity entry. The
deterministic regression additionally performs one staged replacement and
requires an unchanged record to retain the same locator allocation as the
original registry.

Exact Windows P50/P95 values remain pending the combined coordinator batch and
must be written here before integration acceptance.

The pinned Runtime64 child validator is
`zircon-validation-runtime64-registry-staging.ps1` at SHA-256
`0AE778E17C16E77DDC22CCD87820874DBA49FF89A0388B7349E79F6445FAB989`.
It is aggregated with five other Rust optimization tasks by
`zircon-validation-runtime-rust-followup-six.ps1` at SHA-256
`8EF1FDC6A784D3EFD97A422F08D296E5383F39DA661B0DC3DC825ADE3FD6D6D2`.
Both scripts have zero PowerShell AST parse errors.

## Acceptance

- Existing staged locator-collision and remove/reinsert identity tests remain
  required.
- `registry_staging_shares_unchanged_record_storage` proves that a first write
  does not deep-clone unchanged records.
- `registry_staging_copy_on_write_release_gate` emits 21 alternating raw sample
  pairs, nearest-rank P95 values, clone counts, and a 75% P95 reduction gate.
- Exact-file Rustfmt, scoped diff checks, Cargo regressions, and release timing
  run in one managed multi-task Windows validation copy; no per-task Cargo
  invocation is used.

## Remaining Scope

This slice removes full payload and identity deep clones from staging startup.
The first mutation still copies standard `HashMap` bucket arrays containing Arc
pointers. Persistent generation structures, mutation overlays, and scale-based
choice of a different map remain part of the broader RAR-P2-002 work and must be
justified by product-scale measurements rather than assumed here.
