---
title: UI Source Replay and Canonical Savepoint Repair
category: optimization-record
report_id: Editor63-ui-source-replay-savepoint-2026-10-14
date: 2026-10-14
implementation_status: source_reviewed
validation_status: native_and_performance_pending
---

# UI source replay and canonical savepoint repair

This repairs existing editor persistence and basic authoring behavior. A failed local Undo/Redo could consume history or change source/selection before a later projection error. A durable canonical save could leave source-relative history ranges attached to the previous text, and returning to saved text could leave the source-buffer dirty owner active. These are existing F3/F4 save/close blockers; the historical internal slice name F2 does not refer to the Scene-runtime MVP milestone.

## Implementation

- Prepare the source splice, document replay, kind/V2 projection and palette catalog before installing local state. Peek at history first and move its cursor only after local replay succeeds. Invalid source keeps the existing diagnostic and last-good-preview behavior.
- At canonical save completion, use the existing source-revision CAS and mutable session to prepare both adjacent source-neutral Undo/Redo runs. Rebase neutral entries to canonical-to-canonical source no-ops and reconnect the first source-changing edge on each side to its authored neighbor. Preserve all history, labels, document commands, external effects and ordering.
- Compute one UTF-8-safe cursor mapping; map both neutral-entry endpoints and the current-side boundary cursor. Refresh current/last-valid source text and outline caches only after checkpoint preparation succeeds. Saves and edits are not mutually exclusive, and no new history-generation protocol is introduced.
- Include actual selection/theme/style/cursor changes in replay's changed result so existing host Undo/Redo consumers synchronize navigation-only replay. Saved-source comparison and dirty accounting remain independent.
- Clear only the typed source-buffer dirty owner through the existing revision CAS after checking the session is still clean. Release the session lock before final dirty projection and preserve other dirty owners.
- Retain the actual disk receipt and published-byte baseline when local checkpoint completion fails. Preserve an already-recorded publication, durability or SaveCtx error as the primary error.
- Remove only the newly unused private palette-refresh wrapper and its solely unused import. Keep active layout/V2/import/selection helpers and their module wiring.

Local preparation covers local replay/checkpoint state. Later host external participants and IO retain their existing failure behavior; this record does not claim an atomic whole-host transaction.

## Work and memory

Checkpoint preparation traverses the adjacent neutral runs and reconstructs source at at most two source-changing boundaries. Neutral entries prepare cursor metadata and source no-ops; they do not clone full text or the full stack per entry. Boundary preparation temporarily reconstructs and inverse-validates neighbor text; prepared boundary replacements and existing source/outline caches still retain source-sized storage. No CPU, allocation-byte, peak-memory or P95 measurement has been run for this repair.

## Frozen source and static evidence

The nine-file patch is relative to named base d588b90c0e3ca36503ac6b4f2c5d2d96ba7acfef. The packet records raw before/after hashes separately from Git's line-ending-normalized blobs, and preserves shared-index attribution. HEAD later moved externally to f28ba39a5095f4964c046ecc702778627f80cfd5; this task issued no Git mutation and does not attribute that movement.

- [Source patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-source-checkpoint.patch): 55,141 bytes; SHA-256 7157fa7a08afd97579568444bec468573c7db186ba441af83c71892652a07edf.
- [Source, regression and cost evidence](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-source-checkpoint-evidence.json): 107,624 bytes; SHA-256 9c2fbbde1c5263f8935b52ce90d5718853cd71e341acc12574c1200b14d675b1. The owner metadata writer/readback exited 0.
- [Scoped patch guard](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor-source-checkpoint-guard.json): 1,818 bytes; SHA-256 2e15bc3490a304eed82f9acf688538ea62cd60c6075c598e89fc3bf89249e551. Guard subprocess exited 0, with zero errors, warnings and suppressed findings.
- Scoped formatting and named-base whitespace checks exited 0. Root independently matched all nine current source hashes to the frozen packet. Independent semantic review by editor_peer12 (gpt-6-luna max) found no static blocker in the frozen source packet; native tests have not been executed by the source agents or root.

## Combined validation still required

The next planned batch retains all original 15 Cargo arms, both existing Python arms, all original producer identities and numeric gates, then adds one separate grouped Windows F1 arm. It must retain complete actual compiler-input/config/lock/features/toolchain/environment seals; selected file hashes alone are insufficient.

- Add 60 disjoint ordinary Editor cases: 11 newly authored regressions and 49 existing cases newly selected by namespace. They cover early and late replay rejection/retry, both history directions, UTF-8/comments, multiple source-neutral entries, a wholly neutral history, stale save completion, invalid checkpoint preparation, and real manager durable-save/dirty-owner behavior.
- Add the three existing native command-palette consumer cases. Preserve the previous ordinary floor 32; the resulting ordinal-3 floor is 95. Require every named case individually, not only an aggregate count.
- Preserve the Editor150 producer unchanged: 21 paired samples, 2,048 builds per sample, 256 effects per build, optimized nearest-rank P95 at most 70% of legacy. Also select the unchanged Editor811 producer: 68 descriptors, actual final capacity at least 68 and positive arithmetic legacy growth. Its printed growth model is not a measured per-push allocation or CPU/P95 result. Together these raise ordinal-12's preserved floor from 10 to 12; they are additional existing producers and do not replace the historical 16-producer inventory.
- Retain every Runtime03 sample and relative gate, the frozen absolute criteria, and Runtime44's full RSS matrix and both 128 MiB caps. No runtime gate is relaxed to accommodate this editor repair.
- Jenkins owns compilation, native fixtures and cleanup. Before creation it must register the physical task fixture root under E:/cargo-targets/zircon-local, bind TEMP/TMP for every native child and verify the resolved temporary directory. It must join owned children and report actual released bytes and retained failure evidence.

At this source-record freeze, Batch12 has not yet been handed off. Subsequent file-handoff and receipt evidence is recorded separately in the task manifest. Compilation, native/product/UI/MVP acceptance and performance results remain open. After the actual file handoff, root continues the separate Runtime worker-admission repair without waiting for compilation or monitoring coordinator status.

See the corresponding [feature completion list](../../../astra/features/editor/2026-10-14-ui-source-replay-and-savepoint-completion-list.md).
