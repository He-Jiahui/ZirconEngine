---
title: Editor63 History Route and Real Journal Completion List
category: astra-editor
report_id: Editor63-history-route-real-journal-2026-10-13
date: 2026-10-13
implementation_status: source_reviewed
validation_status: native_and_performance_pending
---

# Editor63 History Route and Real Journal Completion List

## Source work complete

- Reject a newly captured world route when retained undo/redo history belongs to another route, before activation, transaction allocation, or command execution.
- Preserve same-route nesting and allow a replacement after history clear or for an empty store.
- Add activation-counter regressions for both rejected `begin` and `execute_operation`, plus nesting and clear/empty cases.
- Retain V1's original flat 65,536-input / 8,192-unique / 17-sample contract, legacy-floor estimate at index 15, and 60% assertion.
- Add V2 over 16 independent 4,096-entry real journal windows, with 65,536 total sorted outputs and an identical-window `BTreeSet` baseline.
- Use nearest-rank P95 (index 16 for 17 samples), test sample counts 1/17/20/100, and count the actual visited journal range outside the per-entry loop.
- R3 source and patch review reconciles to the four owned files. The exact R3 change guard passed with 0 errors and 1 warning for the new ignored V2 release benchmark; that exact filter still needs a managed run.

## Native and performance acceptance pending

- Match the three route regressions and the quantile regression listed in `editor-core-r3-evidence.json` within the shared Editor regression arm; retain their named results.
- Match both V1 and V2 exact ignored producers within the shared Release arm with `--ignored --nocapture --test-threads=1`.
- Record actual P95 values. V2 must satisfy `optimized_p95_ns * 100 <= legacy_p95_ns * 60`; no such measurement or pass is claimed here.
- Complete managed Jenkins validation and reconcile its receipt. The batch08 JSON and Markdown file handoff has been written; queue, native, and performance acceptance remain pending actual receipts.

## Evidence

- Patch: `.codex/outbox/reports/optimize-20261013-01a13451/editor-core-r3.patch` (SHA-256 `8ed5acdc0c64bde0f7a6e78f2996ea813c2bd73d527878ae40d7ef8c12ddbf3c`).
- Source and validation evidence: `.codex/outbox/reports/optimize-20261013-01a13451/editor-core-r3-evidence.json` (SHA-256 `6bf8dd6b1a63c57cbde61e56e82b2a7a329d42c59c1679da26c6ac080765bad8`).
- Exact change guard: `.codex/outbox/reports/optimize-20261013-01a13451/editor-change-guard-r3.json` (SHA-256 `a4277512ee0e6fdfa6d6ed2d271ca27c0f5117b5a1e42285ee196102365e75be`; passed with 0 errors and 1 warning).
- Detailed performance and workload record: `docs/plans/optimize/zircon_editor/63/2026-10-13-history-route-and-real-journal-evidence-record.md`.
