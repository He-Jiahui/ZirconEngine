---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-11
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: ASSET-A4 manifest save pre-write admission
session: astra-a4-manifest-save-record-20260911-01a090b7
---

# ASSET-A4 manifest save pre-write admission

The current worktree contains the ASSET-A4 save-boundary implementation. A
manifest is validated and serialized first, then its UTF-8 byte length is
checked against the same 4 MiB limit used by loading, before the atomic replace
path is entered. Oversized saves return the existing typed summary error and
leave an existing destination untouched.

## Finding status and lowest owner

| Finding | Status | Evidence and lowest owner |
|---|---|---|
| P1 `ASSET-A4` manifest save can produce a file that loading rejects | `implemented_pending_validation` | `zircon_runtime/src/asset/project/manifest/save.rs:23-36` calls `validate`, serializes the current manifest, rejects `document.len() > MAX_PROJECT_MANIFEST_BYTES` with `ProjectManifestSummaryError::DocumentTooLarge`, and only then calls `atomic_write_with_fault`. The lowest owner is `zircon_runtime::asset::project::manifest::save`. |
| Shared admission limit | `implemented_pending_validation` | `zircon_runtime_interface/src/project/manifest_summary/limits.rs:1-3` defines `MAX_PROJECT_MANIFEST_BYTES` as `4 * 1024 * 1024`; `zircon_runtime/src/asset/project/manifest/load.rs:14-18,46-70` applies the same limit to string and bounded-file reads. |

## Regression evidence

- `zircon_runtime/src/asset/project/manifest/save.rs:101-153` adds
  `manifest_save_accepts_exact_limit_and_rejects_limit_plus_one_before_replace`.
  It saves an exact-limit document, reopens it, then verifies a limit-plus-one
  save returns `DocumentTooLarge` and preserves the prior destination bytes.
- `zircon_runtime/src/asset/project/manifest/load.rs:106-157` covers bounded
  reader growth at limit-plus-one, exact-limit acceptance, invalid UTF-8, and
  the in-memory byte check before parsing.
- `zircon_runtime/src/asset/tests/project/manifest.rs:22-38` retains the
  oversized-file rejection regression; lines `243-273` retain atomic write
  fault recovery and previous-manifest readability checks.

## Static evidence and validation boundary

Read-only source assertions confirmed the shared-limit import, the typed
`DocumentTooLarge` branch before the atomic write call, the exact-limit/
limit-plus-one save regression, reopen coverage, bounded reader `limit + 1`
cap, and bounded-file counting regression. Scoped `git diff --check` exited
zero (Git emitted only its existing LF-to-CRLF normalization warnings).

Current source fingerprints observed during this audit:

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/asset/project/manifest/save.rs` | `8A251474650C8ADD87FD927E13F567FBB29E2DE947B59CC6F1C86617DAA85967` |
| `zircon_runtime/src/asset/project/manifest/load.rs` | `3B406D030D29D2DB0E273822CCB1E620A842FEEF147DE061F3D1EE9451EFA841` |
| `zircon_runtime/src/asset/project/manifest/save/borrowed_serialization_tests.rs` | `A392BD1868FB99E04D77BE594FA18DA7309D2D4A87656DE411786B458A7CB9EF` |
| `zircon_runtime/src/asset/tests/project/manifest.rs` | `23CE752C3887C9D0E203080DFDF23CCEA053DD57DC0E05B227C862A444CEA676` |

No Cargo, native, DLL, or product command was run. Managed Windows focused
tests, exact-limit/limit-plus-one execution, save/reopen, fault injection, and
release validation remain pending; this record therefore does not claim
acceptance.

## Coordination and remaining boundary

The source files are pre-existing foreign changes attributed by the coordinator
to stale, non-executable session `astra-optimize-20260909-root`; no live lease
currently covers them. Future source edits require an audited ownership
transfer/lease before touching that stale scope. This audit changed only this
child record, acquired one exact record-path lease, and created no commit.
