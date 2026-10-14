# Runtime 03 shared persistence snapshot completion

**Status:** The seven-file batch09 source snapshot is complete, frozen, and statically reviewed; its file handoff was written successfully. Two later shutdown-fixture observation edits are frozen at SHA-256 `a524d887dde2ec18d77e66fa39164eccd34692a17ffa1a5c9bf18c13395b1407`, independently reviewed without a blocker, and covered by a separate passing scoped guard. They still require a batch10 source binding. Rust type-check, Native tests, queue acceptance, performance measurements, product, Jenkins, and whole-milestone acceptance remain pending.

- **Task:** optimize-20261013-01a13451
- **Baseline:** 5a547d4e2727d27ddf99578f548efb7e6d19d915
- **Date:** 2026-10-13

## Source complete

- [x] Retain immutable `Arc<Value>` owners for persistence snapshots. The map lock covers only copying keys and Arc handles; map construction and JSON serialization happen after unlock.
- [x] Keep the private snapshot closure limited to a cloned `ConfigStore`, with no strong `CoreRuntime` capture. Preserve public owned snapshot APIs and their independent `HashMap<String, Value>` behavior.
- [x] Preserve pretty JSON encoding, persisted schema, flush targets, commit admission, cancellation, errors, and real shutdown join behavior.
- [x] Add two ordinary shared-snapshot regressions for Arc/payload reuse, stable serialization after raw replacement, weak-runtime lifecycle, and owned snapshot independence. Source presence is recorded; tests have not run.
- [x] Retain the batch08 comparison/shutdown source: three ConfigStore concurrency regressions, two shutdown fixtures, and a 15-pair ignored Release probe across 1 B / 1 MiB / 100 MiB. The finite 100 ms fixture observation limitation is recorded in the [batch08 source record](../../../optimize/zircon_runtime/03/2026-10-13-config-comparison-and-shutdown-fixture-record.md).
- [x] Author-reported affected-file rustfmt and whitespace checks passed; independent read-only review found no blocking contract issue.

The cumulative [batch09 source record](../../../optimize/zircon_runtime/03/2026-10-13-shared-config-persistence-snapshot-record.md) lists all seven frozen postimage hashes. Its [source patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shared-snapshot.patch) has SHA-256 `650541196ce60873b116360f1e01b3d9363b198ff98bf089dd50650ed685f4e4`; its [evidence](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shared-snapshot-evidence.json) has SHA-256 `20512588299ff1c9ebe9032acf77c9c5dd52b267fd63a504c0fb0e16e00c8149`.

## File handoff

- [x] Batch09's file writer exited 0. The JSON SHA-256 was `c934086320a3c4598f8b441b253fdbcf6d029348c7c00b745d5df1977d4a653f`; the Markdown SHA-256 was `d127f64409f02a97fcbdd71120cb48b53a06974e5816a655424ff442ae316e4b`. At write time the [retained JSON](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.json) and [Markdown](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.md) copies byte-matched the canonical `.jenkins/records` files. A later normal link check observed the canonical batch09 paths missing again; the cause is unknown. The retained report copies remain available, and this historical file-write result does not prove queueing or execution.
- The handoff declares 15 selected inputs, 15 Cargo arms, and 1 Python command. This confirms file handoff only; it does not confirm queueing or command execution.
- The historical batch07 and batch08 intake/handoff/prospective-response artifacts remain missing for unknown reasons. Their retained historical write hashes do not establish consumption, queueing, or acceptance; see the [continuation log](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/continuation-log.md).

## Static guard

- [x] The [batch09 Runtime change-guard receipt](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shared-snapshot-guard.json), SHA-256 `6a66b428a8d064103472e26881e3887bee29e1d577b8cffb3f6a0781d40ee92a`, binds the cumulative patch and reports `passed`, 0 errors, and 1 warning. The warning is the ignored large-value Release probe at `core/runtime/tests/config_store.rs:263`; it is a required named `#9` follow-up, not a disabled ordinary regression.

This static result does not establish compilation, Native correctness, a performance improvement, or product acceptance.

## Validation and acceptance pending

- [ ] Reconcile actual queue and Native receipts for batch09. Successful file handoff does not establish that any command was queued, run, or accepted.
- [ ] Run the focused ordinary Rust regressions and obtain type-check/Native evidence through the authorized validation path.
- [ ] Run the ignored Release selector as a named `#9` command and retain its raw samples. The batch08 100 ms observation window and 2 s / 30 s lifecycle timeouts are not performance budgets.
- [ ] Execute the unchanged original Runtime 03 60-sample Config matrix (1 B / 1 MiB / 100 MiB payloads across the original key-burst and flush/shutdown cases).
- [ ] Evaluate the original 70% / 75% helper thresholds and retain the complete 45 / 36×3 / 60 matrix under their own gates.
- [ ] Collect raw samples for the user-authorized absolute P95 caps in the [source record's six-cell table](../../../optimize/zircon_runtime/03/2026-10-13-shared-config-persistence-snapshot-record.md#performance-criteria-and-handoff-status). Root's formal [criteria record](../../../optimize/zircon_runtime/03/2026-10-13-config-persistence-absolute-p95-budget-record.md) and [completion list](2026-10-13-runtime03-absolute-p95-budget-completion-list.md) are recorded. The [criteria JSON](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json) has SHA-256 `1075e7ac1195d9b8d5521eb20fda659506aa4759f4b9f5af7736dc1dcd4c77f5`. Each flush and last-owner-shutdown path uses its cap for both `derived_burst_plus_completion_ns` P95 and API `completion_ns` P95 over five raw rows; the maximum observed value is the gate. The derived sum excludes initial payload setup and is not full session wall-clock. No result has been measured or passed.
- [x] Independently review the frozen two-fixture shutdown-observation edit at `config_manager_tests.rs` SHA-256 `a524d887dde2ec18d77e66fa39164eccd34692a17ffa1a5c9bf18c13395b1407`; no blocker was found. See the [separate shutdown-return source record](../../../optimize/zircon_runtime/03/2026-10-13-shutdown-return-observation-record.md) and its scoped guard.
- [ ] Bind the updated fixture source postimage in batch10 and run the two focused Native selectors. The seven batch09 hashes, patch, evidence, handoff, and guard do not cover or accept the later edit.
- [ ] Complete product, Jenkins, and whole-milestone acceptance.
- [ ] Reconcile Editor 07’s separate asynchronous handoff and receipt under its owner; this list does not close that work.

The shared snapshot changes `config_store.rs` and the persistence worker beyond the batch08 source postimages, and adds the private CoreHandle source and ordinary tests. Batch09 seals those seven postimages; the later two shutdown-fixture observation edits require batch10 and must be peer-reviewed after freeze. Batch08/07 missing-file history and the concurrent observed HEAD `1de7f1af1f86b124645acdb2743bfe9064f6da8b` carry no queue, Native, or acceptance credit.
