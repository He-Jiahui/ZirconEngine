---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: ASSET-A3 compound member-count admission
session: astra-compound-member-count-handoff-20260912-01a090b7
---

# Compound member-count admission owner handoff

## Finding

Compound discovery currently aggregates every matching path before the member
budget is checked. `zircon_runtime/src/asset/project/manager/collect_files.rs:9-52`
recursively walks a directory and unconditionally pushes each included regular
file at line 49. There is no count parameter or pre-push admission boundary.

The compound-specific call sites are
`zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs:122-141`
(targeted discovery, `collect_files` at line 123) and `:287-303` (full compound
discovery, `collect_files` at line 288). The same collector is also used for the
whole asset-root scan at `sources.rs:228-235` (line 229); that scan must not be
silently capped as a side effect of fixing compound membership. The existing
`MAX_COMPOUND_SOURCE_FILES` check at `sources.rs:378-390` runs only after the
unbounded `included_paths` vector has already been built, so zero-byte members
can still consume path/vector memory before rejection.

The lowest ownership boundary is therefore split but indivisible for a safe
implementation:

1. `zircon_runtime/src/asset/project/manager/collect_files.rs:9-52` owns the
   bounded traversal/push primitive and its focused 65,536-versus-65,537
   regression.
2. `zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs:122-141`
   and `:287-303` own the compound-only wiring and error context. A
   `collect_files`-only patch would be dead API; changing the existing generic
   function globally would also alter the unrelated root scan at line 229.

The downstream snapshot consumers are
`scan_and_import/full_generation.rs:145-169` and
`scan_and_import/targeted.rs:366-394`; they consume the already collected
`included_paths` and therefore do not provide an earlier admission boundary.
The shared cap is `zircon_runtime/src/asset/importer/ingest/auxiliary_source.rs:21`
(`AuxiliarySourceResolver::MAX_SNAPSHOT_FILES = 65_536`).

## Ownership evidence

| Path / lane | Coordinator evidence | Disposition |
|---|---|---|
| `zircon_runtime/src/asset/project/manager/collect_files.rs` | Ownership matrix marks the current hash `B7C0DE5A24F6FB7E197F90A30FAD7CF34F9498C839EEBAEF9064FEA3DD39F8D3` as modified with stale archived attribution `root-runtime-editor-optimize-20260829-r5`; no live lease | No executable owner is present, but this path cannot complete the production fix without the caller below. |
| `zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs` | Current hash `8740C89A58E4C69814554299997D9957EECCFFABE1FB1FD703FD8546643B77AC`; matrix attribution is stale/archived (`root-runtime-editor-optimize-20260901-r6`) and has no lease. Independently, executable finalizing session `astra-asset-build-identity-20260907` explicitly includes `sources.rs`, `full_generation.rs`, and `targeted.rs` in its write scope; last heartbeat `2026-09-07T15:09:15+08:00`, status `finalizing`, reason “Initial qualified identity candidate attributed; coordinator review correction continues in exact expanded context-integrity scope.” | **Blocking owner scope.** The finalizing scope must be explicitly released or handed off by its owner/coordinator maintainer; this session must not retire, cancel, or transfer it. |
| `zircon_plugins/gltf_importer/runtime/src/lib.rs` and `tests/source_snapshot.rs` | Waiting-validation session `astra-gltf-single-snapshot-20260911` owns these exact plugin paths; current hashes are `F11BE2A16B6E46019FECA45D58F178A1DA9B888D758A7C64586D47BE3EE261BE` and `D385D5D1492349A0C52943A73F901393FDCBA54507F51F57975EF7C95DB7B999`. | No direct path overlap with the collector fix. Preserve this lane; only the shared 65,536 contract is a semantic dependency. |

No matching live lease was found for `collect_files.rs`, `sources.rs`, or the
glTF paths. Absence of a lease does not clear the explicit executable
finalizing scope above under the cross-session ownership policy.

## Why this cannot be isolated safely now

The glTF lane is not itself a file-level blocker: the intended compound fix can
reuse the existing shared count constant without changing the glTF plugin.
However, production behavior cannot be changed in `collect_files.rs` alone.
The two compound callers in `sources.rs` must opt into the bounded collector,
while the root scan must remain unbounded. Those caller lines are inside the
finalizing asset-build-identity session's exact write scope. Adding only a unit
test or an unused helper would leave the unbounded production aggregation in
place and would not close ASSET-A3.

## Requested handoff and dependency-ready slice

The current finalizing owner/coordinator maintainer is requested to preserve
its existing diff and explicitly hand off the `sources.rs` (and, if retained in
the same candidate, `full_generation.rs`/`targeted.rs`) scope. After the
coordinator shows no executable owner and a new primary session has exact
leases, the smallest implementation slice is:

1. Add a bounded collector/push admission in `collect_files.rs` and a TDD
   regression proving exactly 65,536 synthetic zero-byte member paths are
   accepted while member 65,537 is rejected before insertion.
2. Wire the bounded API only at `sources.rs:123` and `:288`; keep the root scan
   at `:229` and metadata discovery at `:261` on their existing semantics.
3. Preserve the late `sources.rs:382-390` check as defense in depth and retain
   deterministic typed import-budget diagnostics.
4. Run scoped formatting/diff checks, then the managed Windows focused source,
   compound snapshot, targeted reimport, and release gates. No Cargo/native
   evidence exists for this handoff.

Prospective exact write paths after handoff:

- `zircon_runtime/src/asset/project/manager/collect_files.rs`
- `zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs`
- `docs/plans/astra/optimize/01/2026-09-12-compound-member-count-owner-handoff.md`

The record is intentionally `blocked_owner_scope`; it is not an implementation
or acceptance claim. This session made no source edits, ran no Cargo/native
commands, and created no commit.
