# Editor78 Export Profile Hash Index

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Status: `implementation_complete / managed_validation_pending`
- Batch: `optimization_batch_20260826co_`

## Problem

Build/export target projection loaded presets in sorted name order, but resolved every preset's
profile reference with a fresh linear scan of `manifest.export_profiles`. Rebuilding a project
with many presets and profiles therefore scaled as O(presets x profiles).

## Optimization

- Build one borrowed `HashMap<&str, &ExportProfile>` for the projection rebuild and resolve every
  preset through it.
- Preserve the old first-match rule for duplicate profile names with `entry(...).or_insert(...)`.
- Keep preset-name sorting, invalid-preset diagnostics, missing-profile diagnostics, and cloned
  profile ownership at the row builder boundary unchanged.

## Test And Performance Contract

- The behavior regression verifies first-duplicate selection, an independent profile, and a
  missing name.
- The source regression requires one capacity-sized hash index and rejects the old repeated
  `export_profiles.iter().find` path.
- Ignored release evidence prints `EDITOR78_EXPORT_PROFILE_HASH_INDEX_BENCH_V1` for 21 alternating
  sample pairs over 2,048 profiles and reverse-order profile requests. Index construction is inside
  the optimized timed region.
- Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 70`.

## Validation State

Rust 1.94.1 formatting and scoped static checks are required before submission. Cargo results,
exact P50/P95 values, commit SHA, push result, and WeCom delivery remain coordinator-owned terminal
evidence and are not claimed by this pending record.

