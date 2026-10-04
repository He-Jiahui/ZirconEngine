---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: ASSET-A2/A3 glTF external-source admission
session: astra-gltf-external-snapshot-handoff-20260912-01a090b7
---

# glTF external-source boundary and immutable-snapshot owner handoff

## Finding

The runtime importer already has the intended lowest-layer path contract. `zircon_runtime/src/asset/importer/ingest/auxiliary_source.rs:86-176`
rejects rooted/drive/scheme-qualified references, percent-decodes glTF URI
paths before lexical resolution, enforces the admitted asset root, and provides
bounded reads. `:178-310` binds admission, no-follow/opened-handle identity,
size/mtime checks, and the returned bytes. The runtime glTF decoder consumes
that contract through `gltf_decode/sources.rs:28-85`; when a context snapshot is
authoritative it reads the snapshot and refuses an absent member, and otherwise
it caches the first admitted read. Its external buffer/image tests cover encoded
parent traversal and snapshot-over-disk behavior at
`zircon_runtime/src/asset/importer/ingest/gltf_decode.rs:227-313`.

The separately shipped glTF plugin still has an open ASSET-A2/A3 path. In
`zircon_plugins/gltf_importer/runtime/src/lib.rs:64-75`, `import_gltf` parses
`context.source_bytes` once but passes the document/blob and the source-path
parent to `gltf::import_buffers` and `gltf::import_images`. Those APIs reopen
external URI files from the filesystem rather than consuming the immutable
`AssetImportContext` auxiliary snapshot. The preflight helper at
`:300-328` only checks buffer existence using `base_dir.join(uri)` and
`.exists()`; it does not reject absolute/drive/UNC, `file:`/other schemes,
percent-encoded traversal, or symlink/reparse escapes, and it does not inspect
external image paths at all. A path can therefore pass validation and then be
reopened through a different filesystem object (or fail after a snapshot was
captured), violating the single-generation input invariant and the shared
auxiliary budget.

The current plugin regression at
`zircon_plugins/gltf_importer/runtime/src/tests/source_snapshot.rs:16-39`
only proves that an embedded primary source can be imported after its path is
missing. It does not cover an external buffer/image whose context snapshot
differs from, is deleted, or is replaced on disk, nor legal nested URI versus
encoded/root/link rejection.

## Ownership evidence

| Path / owner | Current coordinator evidence | Disposition |
|---|---|---|
| `zircon_plugins/gltf_importer/runtime/src/lib.rs` | Exact current SHA-256 `F11BE2A16B6E46019FECA45D58F178A1DA9B888D758A7C64586D47BE3EE261BE`; session `astra-gltf-single-snapshot-20260911`, status `waiting_validation`, last heartbeat `2026-09-11T21:21:31+08:00`; scope explicitly includes this file; no live lease currently present. | **Blocking owner scope.** Preserve the waiting-validation candidate; do not edit or claim this path from this handoff session. |
| `zircon_plugins/gltf_importer/runtime/src/tests/source_snapshot.rs` | Exact current SHA-256 `D385D5D1492349A0C52943A73F901393FDCBA54507F51F57975EF7C95DB7B999`; same waiting-validation session and explicit scope; staged/dirty source is foreign. | **Blocking owner scope.** External-source regressions belong with the plugin implementation owner. |
| `zircon_runtime/src/asset/importer/ingest/auxiliary_source.rs` | Current SHA-256 `92639AD7E3A1C15CEA64E1F0D393CF021355C27080536B50A36CE236066A31DE`; staged foreign source has no executable matrix owner/lease in the current snapshot. Existing path/handle tests and the runtime decoder already consume it. | No new runtime resolver change is justified by this audit; preserve the foreign source candidate. |
| `zircon_runtime/src/asset/importer/ingest/gltf_decode.rs` | Current SHA-256 `301A6BDE8C545A35871F6E644C61C836E9A12C1A0FDF3FC3A7560144BF276F9E`; decoder/snapshot producer is already implemented and is a semantic dependency, not a safe plugin-only edit target. | No source edit in this handoff. |
| `zircon_runtime/src/asset/project/manager/scan_and_import/{sources,full_generation,targeted}.rs` | The finalizing session `astra-asset-build-identity-20260907` explicitly owns these generation/snapshot callers (status `finalizing`; review correction still active). | Do not expand this handoff into generation or compound collector changes; the separate compound member-count handoff remains blocked. |

No matching live lease exists for the plugin paths, but the explicit
waiting-validation owner and dirty candidate remain authoritative under the
cross-session policy. Lease absence is not permission to take another session's
scope.

## Why the slice is not independently safe now

Fixing the plugin requires both (1) a path-admission/URI resolver and external
buffer/image byte provider, and (2) a way for the plugin crate to consume the
runtime-owned immutable snapshots or a runtime decode facade. The existing
`AssetImportContext::source_file_snapshot` API is crate-private to
`zircon_runtime`; exposing or relocating it would touch the runtime importer
contract and its consumers, while editing the plugin implementation/tests is
already owned by `astra-gltf-single-snapshot-20260911`. A test-only patch would
not close the raw `join`/reopen behavior, and a resolver-only patch would leave
`gltf::import_buffers/import_images` reopening the path.

## Requested handoff and dependency-ready slice

The waiting-validation glTF owner/coordinator maintainer should preserve its
current candidate, then take the following exact paths after a fresh lease and
contract review:

1. `zircon_plugins/gltf_importer/runtime/src/lib.rs`: route every external
   buffer and image URI through the admitted-root resolver and consume the
   corresponding immutable snapshot bytes (including legal nested references,
   encoded traversal rejection, absolute/scheme rejection, and link/reparse
   rejection). Do not use `gltf::import_buffers/import_images` for external
   path reads after preflight.
2. `zircon_plugins/gltf_importer/runtime/src/tests/source_snapshot.rs` (and,
   if needed, the owner’s existing plugin test module): add TDD regressions for
   snapshot-wins-over-disk, missing/replaced external files, legal nested URI,
   encoded/root/scheme/link rejection, and deterministic bounded file/byte
   errors for buffers and images.
3. If the plugin cannot access the runtime snapshot without a public contract,
   coordinate a separate exact owner for
   `zircon_runtime/src/asset/importer/contract.rs` or a small runtime decode
   facade; do not silently widen this plugin handoff into the finalizing
   `astra-asset-build-identity-20260907` scope.

The plugin lane is therefore **not dependency-ready for an unowned
implementation in this session**. It becomes ready only after the waiting
validation owner (and, if required, the runtime contract owner) explicitly
hands off/leases the exact paths. The manifest ASSET-A4 slice is already
implemented_pending_validation separately; its exact-limit/+1 and old-file
preservation regression is present at
`zircon_runtime/src/asset/project/manifest/save/borrowed_serialization_tests.rs:101-153`.

This record is intentionally `blocked_owner_scope`; this session made no
source edits, ran no Cargo/native commands, and created no commit.
