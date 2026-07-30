# Validation-copy source hash canonicalization

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M6.8
Status: accepted
Files: ["tools/session_coordinator/cargo_jobs.py", "tools/session_coordinator/tests/test_cargo_reservations.py", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-30-validation-copy-source-hash-canonicalization-return.md", "docs/plans/zircon_editor/editor/12/fixed-2026-07-30-validation-copy-source-hash-canonicalization.md", "docs/plans/zircon_runtime/runtime/11/fixed-2026-07-29-cpu-reservation-ledger-consume-fifo-divergence.md"]

## Scope delivered

- Canonicalized the persisted materialized validation-copy hash at the Cargo source-copy admission boundary before comparing it with canonical compatibility metadata.
- Preserved Session ownership, materialized status, source root, per-file source-manifest revalidation and distinct-hash rejection.
- Returned the child-only failure to Editor12 and recorded the Coordinator receipt without changing either parent plan.
- Normalized the source-executor field delimiters in the prior Runtime11 fixed return so the Coordinator01 scoped failure graph remains canonical at finalize time.
- Loaded the repair through admission-preserving rollover action `9c6760e3a21a4ab0b65829a86065acfb`; successor instance `23ab68c8df8044a98ff0faa1eada842e` is healthy.

## Fresh testing evidence

- Focused lowercase/distinct-hash regressions: `2 passed / 0 failed` on the final code hashes.
- Complete `tools.session_coordinator.tests.test_cargo_reservations`: `49 passed / 0 failed / 226.000s` on the same production/test code hashes. A later redundant full rerun reached the outer 360-second command limit without failure output; it is non-diagnostic and does not replace the completed run.
- Snapshot 1351 final fixed-return exact5: `5/5` unchanged. The finalize audit added one owned historical fixed record, so the refreshed milestone manifest is exact6.
- Managed `coordinator-actions` validation on the pre-audit exact5 source copy: `34 passed / 0 failed / 137.621s`; a fresh exact6 binding is required before commit.
- Live admission: snapshot 1349 was `71/71` unchanged with 65 materialized source hashes, and validation copy `0927082468ef4419b090af071059da5e` successfully created FIFO reservation `d3179b6bf5394717a1e49dfa3eb60d46`.
- `git diff --check` passed; only repository line-ending warnings were emitted.

## Review

- Independent review: Critical `0`, Important `0`, Minor `0`.
- The review confirmed the fix normalizes representation only; it does not weaken provenance or permit foreign, stale, incomplete or distinct source copies.
- Catalog focused/broad Rust validation remains owned by its open Plugins01 failure and is not claimed as GREEN by this Coordinator milestone.
