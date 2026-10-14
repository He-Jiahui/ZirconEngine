# Runtime 03 shared Config persistence snapshot source record

**Status:** The seven-file batch09 source snapshot is frozen, reviewed, and passed its static change guard. Its file handoff was written successfully. Two shutdown-fixture observation edits after batch09 have since frozen in `config_manager_tests.rs` at SHA-256 `a524d887dde2ec18d77e66fa39164eccd34692a17ffa1a5c9bf18c13395b1407`; independent review found no blocker and their separate scoped guard passed. Batch10 must bind these postimages. Batch09 cannot accept them. Rust type-check, Native execution, queue acceptance, and performance measurements remain pending.

- **Task:** optimize-20261013-01a13451
- **Baseline:** 5a547d4e2727d27ddf99578f548efb7e6d19d915
- **Date:** 2026-10-13

## Source scope

The shared persistence snapshot carries immutable `Arc<Value>` owners into serialization. `ConfigStore::shared_snapshot_values` copies map keys and Arc handles while holding the values-map mutex; it builds the returned map and serializes JSON after the lock is released. The private `CoreHandle::config_snapshot_source` closure captures a cloned `ConfigStore`, not `CoreHandle` or `CoreRuntime`. The public `snapshot_values` and `snapshot_config_values` APIs still return independent owned `HashMap<String, Value>` snapshots.

The worker still pretty-serializes through `serde_json::to_vec_pretty`, writes the same JSON object schema, and uses the existing flush target, admission fence, cancellation, error reporting, and real shutdown join behavior. The source removes the redundant deep JSON payload clone before each persistence attempt; JSON encoding and file I/O remain necessary work.

The batch adds two ordinary regressions in [shared snapshot tests](../../../../../zircon_runtime/src/core/runtime/config_store/tests/shared_snapshot_tests.rs): one verifies Arc reuse, a 1 MiB payload's backing allocation, stable serialization after a concurrent raw replacement, and weak-runtime lifecycle; the other verifies public owned snapshots remain independent. These tests are present but have not run. The batch09 postimage also contains the inherited three ConfigStore concurrency regressions and two shutdown fixtures. Their 100 ms observation window, and the 2 s / 30 s wait or shutdown limits, are synchronization and lifecycle controls rather than performance budgets; the finite observation caveat from the [batch08 record](2026-10-13-config-comparison-and-shutdown-fixture-record.md) remains. After batch09, the owner changed only the two shutdown-fixture observations in `config_manager_tests.rs`; the follow-up is frozen at SHA-256 `a524d887dde2ec18d77e66fa39164eccd34692a17ffa1a5c9bf18c13395b1407` and independently reviewed without a blocking finding. See the [shutdown-return fixture record](2026-10-13-shutdown-return-observation-record.md) for its separate patch and guard. Batch09 source hashes, patch, evidence, handoff, and guard do not bind or accept this later source; batch10 must bind the updated postimage before validation.

The seven frozen batch09 source postimages are:

| Source path | SHA-256 |
| --- | --- |
| `zircon_runtime/src/core/runtime/config_store.rs` | `0ba1e999954fa1eb670cf0bd195cdae4a0564ba9360301a40a3e6dc40bb698f2` |
| `zircon_runtime/src/core/runtime/tests/config_store.rs` | `aededa5d82746fa3e17deddbcdf3926b28d0c0c3b80f861b0900118a2000d46b` |
| `zircon_runtime/src/foundation/runtime/config_manager.rs` | `d2dfb80e47431cc73ba25fafabd0402eca84c8f0576238d0c9f2208619fed1fa` |
| `zircon_runtime/src/foundation/runtime/config_manager/worker.rs` | `50c811b43af2f309692a29348a88893694e544d01bfbe075f4a700f45ece1c01` |
| `zircon_runtime/src/foundation/runtime/tests/config_manager_tests.rs` | `af0b48b1ce84d19b140d2e0aaf744a4999b78f5b542a27c3289d380783d6073c` |
| `zircon_runtime/src/core/runtime/handle/events.rs` | `c9dccabbc70af73ef6d8dbaad8b40921fa3542e5db95f78d4af33eb22b18432a` |
| `zircon_runtime/src/core/runtime/config_store/tests/shared_snapshot_tests.rs` | `52b21607a59091d716d307d68d080f3db2f53e86bafc13a9dbf293060c1d182e` |

## Review and static checks

The author-reported affected-file `rustfmt --check` and `git diff --check` passed. Independent read-only review found no blocking ownership, lock-scope, public snapshot, serialization-schema, or worker lifecycle issue. The cumulative [batch09 source patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shared-snapshot.patch) is 27,499 bytes, SHA-256 `650541196ce60873b116360f1e01b3d9363b198ff98bf089dd50650ed685f4e4`; the [batch09 evidence](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shared-snapshot-evidence.json) is 27,127 bytes, SHA-256 `20512588299ff1c9ebe9032acf77c9c5dd52b267fd63a504c0fb0e16e00c8149`.

The [batch09 static guard receipt](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shared-snapshot-guard.json), SHA-256 `6a66b428a8d064103472e26881e3887bee29e1d577b8cffb3f6a0781d40ee92a`, reports `passed`, 0 errors, and 1 warning. The warning points to the existing newly added ignored large-value Release probe at `core/runtime/tests/config_store.rs:263`; it is not an ordinary regression being disabled. Include that selector in the named `#9` validation run.

## Performance criteria and handoff status

The user authorized an absolute persistence P95 budget, and root accepted these per-cell flush / last-owner-shutdown caps in milliseconds. They are criteria for future raw samples, not measurements or passes:

| Payload | Distinct burst keys | Flush path cap | Last-owner shutdown path cap |
| --- | ---: | ---: | ---: |
| 1 B | 1 | 50 ms | 75 ms |
| 1 B | 1,000 | 75 ms | 100 ms |
| 1 MiB | 1 | 75 ms | 100 ms |
| 1 MiB | 1,000 | 100 ms | 125 ms |
| 100 MiB | 1 | 1,500 ms | 1,600 ms |
| 100 MiB | 1,000 | 1,750 ms | 1,850 ms |

For each path and cell, both `derived_burst_plus_completion_ns` P95 and the API-observed `completion_ns` P95 use that path's cap, calculated from its five raw rows; with `n=5`, the maximum observed value is the gate. The derived sum excludes initial payload setup and is not full session wall-clock time. The small sample count limits statistical confidence. The formal [absolute P95 criteria record](2026-10-13-config-persistence-absolute-p95-budget-record.md) (SHA-256 `09b256c60c22d4fdd91caf1512f14dac12d4e05d5c8b11f5acbd4dc5e5ea4391`), [criteria JSON](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json) (SHA-256 `1075e7ac1195d9b8d5521eb20fda659506aa4759f4b9f5af7736dc1dcd4c77f5`), and [completion list](../../../astra/features/runtime/2026-10-13-runtime03-absolute-p95-budget-completion-list.md) (SHA-256 `824fa44723be0fe8464b9589860e3a479880420721321336d1234c7944fd5c2a`) are recorded; their criteria are not measurements or passes.

Source structure shows that persistence no longer deep-clones each JSON payload into a new owned map before encoding. It does not establish a measured speedup or allocation reduction. The original Runtime 03 plan did not supply an absolute Config latency threshold; the newly authorized caps fill that acceptance criterion once formally recorded and measured. The local 70% / 75% helper thresholds and original 45 / 36×3 / 60 matrix remain separate gates. The original [60-sample producer](../../../../../zircon_runtime/src/foundation/runtime/worker/tests/config_original_matrix.rs) remains unchanged and pending; it covers 1 B / 1 MiB / 100 MiB payloads across the original key-burst and flush/shutdown cases. No sample has been run for this packet.

The batch09 file writer exited 0. At write time the handoff JSON SHA-256 was `c934086320a3c4598f8b441b253fdbcf6d029348c7c00b745d5df1977d4a653f` and Markdown SHA-256 was `d127f64409f02a97fcbdd71120cb48b53a06974e5816a655424ff442ae316e4b`; each retained report copy matched its canonical `.jenkins/records` file byte-for-byte. The [retained JSON copy](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.json) and [retained Markdown copy](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.md) remain available. A later normal link check observed the canonical batch09 JSON/Markdown paths missing again; the cause is unknown, and their present absence does not erase the historical writer result. The report handoff declares 15 selected inputs, 15 Cargo arms, and 1 Python command. This is successful file delivery only: it does not show that commands were queued, run, or accepted.

The historical batch07 and batch08 handoff/intake/prospective-response records checked by root remain missing; the cause is unknown. Retained earlier write hashes are historical evidence only and do not prove consumption, queueing, or acceptance. See the [continuation log](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/continuation-log.md). Batch09 is the first currently verified file handoff in this record, while queue, Native, performance, product, Jenkins, and whole-milestone acceptance remain pending. The concurrent observed HEAD `1de7f1af1f86b124645acdb2743bfe9064f6da8b` is recorded in the evidence but is not attributed to this task and does not establish acceptance.

No Rust build, type-check, ordinary test, ignored probe, original matrix, or Native execution is recorded here. Editor 07 continues asynchronously under its separate owner; this Runtime record does not close that work.

The matching [Runtime 03 completion list](../../../astra/features/runtime/2026-10-13-runtime03-shared-persistence-snapshot-completion-list.md) tracks source completion and pending acceptance separately.
