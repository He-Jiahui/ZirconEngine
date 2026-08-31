---
related_code:
  - zircon_runtime/src/asset/migration
  - zircon_runtime/src/asset/registry/inspection.rs
  - zircon_runtime/src/core/resource/io/atomic_file
  - zircon_runtime/src/core/resource/io/transaction
  - zircon_runtime_interface/src/project/retired_asset_ref_migration
implementation_files:
  - zircon_runtime/src/asset/migration/mod.rs
  - zircon_runtime/src/asset/migration/document.rs
  - zircon_runtime/src/asset/migration/error.rs
  - zircon_runtime/src/asset/migration/mode.rs
  - zircon_runtime/src/asset/migration/options.rs
  - zircon_runtime/src/asset/migration/report.rs
  - zircon_runtime/src/asset/migration/resolver.rs
  - zircon_runtime/src/asset/migration/run.rs
  - zircon_runtime/src/asset/migration/scan.rs
  - zircon_runtime/src/asset/migration/sidecar.rs
  - zircon_runtime/src/asset/migration/transaction.rs
  - zircon_runtime/src/asset/migration/transaction/journal_owner.rs
  - zircon_runtime/src/asset/migration/transaction/recovery.rs
  - zircon_runtime/src/core/resource/io/transaction
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/safe_project_path.rs
  - zircon_runtime/src/asset/assets/project_document.rs
  - zircon_runtime/src/asset/assets/project_document/codec.rs
  - zircon_runtime/src/asset/assets/project_document/material.rs
  - zircon_runtime/src/asset/assets/project_document/model.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/registry/inspection.rs
plan_sources:
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/asset/tests/migration/mod.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/command_flow.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/document_migration.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/source_boundary.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/transaction_recovery.rs
doc_type: module-detail
---

# Project Asset Migration

## Boundary and flow

`zircon_runtime::asset::migration` is the business owner behind the application `migrate-assets` commandlet. It loads the current `ProjectManifest`, preflights the current and retired sidecars in memory, builds one immutable registry snapshot, and visits the first-wave textual authoring formats: `.scene.toml`, `.zmaterial`, and `.model.toml`.

The inspection snapshot is deliberately read-only. Unlike registry rebuild/import, it never remints duplicate GUIDs, rewrites sidecars, or persists `.zircon/registry`; duplicate GUIDs and duplicate `res://` paths are commandlet failures. Each retired reference must resolve by both GUID and locator to the same registry entry, and its physical source must exist under exactly one manifest root. Current `AssetRef` GUIDs are authoritative: a missing GUID with an occupied path hint is a typed candidate/error, not an automatic GUID replacement, while a GUID/subasset mismatch is a conflict. Missing GUIDs, missing paths, conflicts, ambiguous roots, unsupported schemes, malformed documents, and unsafe paths are typed `AssetMigrationIssueKind` entries.

Scene, model, and material candidates are not accepted as generic TOML after rewriting. Their production authoring readers deserialize the discriminated `PersistedAssetReference` contract and require an injected read-only registry resolver before constructing the non-persistent runtime asset representation. Project references carry `AssetRef`; builtins carry only their stable builtin locator. Runtime `AssetReference` remains an in-memory resolved value rather than a project-file identity contract.

## Dry run and apply

Dry-run parses and resolves every candidate and writes nothing, including when an interrupted transaction journal exists. A valid pending journal is reported as `PendingRecovery`; only Apply may recover it. Apply performs the same complete preflight; if any issue exists, no authoring file is written. Retired `.meta.toml` names and pre-v7/source-hash sidecars are converted to the strict v7 `.zmeta` contract in the same transaction as their authoring documents. Runtime import stays strict and does not perform this repair.

All changed files are staged, flushed, and synchronized before the first commit. Existing-file backups are reopened with read/write access before durability synchronization because Windows rejects `FlushFileBuffers` on a read-only handle; this is a transaction-owner rule, not a commandlet platform workaround. An intent journal is durable before staging, and the transaction then persists `active`, `all_committed`, and cleanup phases. A journal loaded from disk is untrusted input: recovery validates the journal owner, target whitelist, sibling artifact names, transaction ownership, states, and digests before removing only reserved artifacts and the journal. It never copies backup or staging bytes over a live target and never deletes a live target. After that cleanup, the same Apply invocation reruns current preflight and migration so interrupted work converges forward to current bytes. In-process failures still use the transaction's in-memory rollback path. Completed phases preserve all-new bytes and idempotently clean remaining artifacts, with the journal removed last. Typed errors identify recovery, stage, commit, or rollback failure. Current TOML omits a null optional `sub` field because TOML has no null value, while the shared `AssetRef` JSON and bincode contracts remain unchanged.

A complete commit frame whose synchronization fails is a distinct terminal disposition: the shared engine keeps the new bytes and recovery evidence, while Apply reports an actionable pending-recovery error instead of claiming durable success. Rerunning Apply performs recovery before a new transaction. Cleanup deferral remains a committed success because `all_committed` is already durable.

The scanner never follows links/reparse points, skips `.zircon` subtrees, and ignores external source formats, binary artifacts, stale sidecars without a source file, and unrelated TOML. Importer-recognized sources without sidecars are different: migration mints their strict v7 `.zmeta` documents in the same transaction and reports those paths alongside authoring rewrites. Already-current persisted references retain their exact field set instead of being typed-deserialized and then reserialized; absent optional fields therefore cannot become synthetic JSON nulls and cannot create a false second-run change. A same-GUID stale path hint may be published as a generation-bound observation for the editor health surface, but migration does not silently persist that update. A second run sees no retired exact shapes or sidecars, performs no writes, and preserves identical bytes.
