---
doc_type: optimization-profile
status: profile_executed_normal_tests_and_product_gates_open
plan_source: docs/plans/optimize/zircon_runtime/210/2026-09-27-random-checkpoint-stream-count-budget.md
implementation_files:
  - zircon_runtime/crates/zr_contracts/tests/runtime210_checkpoint_decode_profile.rs
tests:
  - runtime210_checkpoint_decode_release_profile
---

# Runtime210: bounded checkpoint decode Release profile

This isolated profile supplies the Release decode-cost and allocation evidence
left open by Runtime1019. It calls the public `RandomServiceCheckpoint` serde
decoder and does not change production code, checkpoint wire bytes, or the
count-budget contract.

## Measurement protocol

- Run the ignored test as a Windows Release test.
- Exercise canonical checkpoints with 1, 1,024, and 65,536 streams. The final
  size is the exact `RandomServiceCheckpoint::MAX_STREAMS` limit. The fixture
  uses the JSON representation already exercised by the contract tests and
  invokes the production `Deserialize` implementation through `serde_json`;
  it does not assert which codec an application persists.
- Build each checkpoint and serialize its JSON fixture before sampling. Each of
  31 raw samples times one `serde_json::from_slice::<RandomServiceCheckpoint>`
  call; equality, stream-count, and canonical-order checks run after timing.
- Emit encoded payload bytes, every sample's elapsed nanoseconds, allocator
  request count and bytes, plus nearest-rank p50/p95/p99 latency.
- The allocator reports successful `alloc`, `alloc_zeroed`, and `realloc`
  requests made during decode. Its byte total is cumulative requested bytes;
  it is not live heap, peak memory, or RSS, and it does not subtract deallocs.
  A thread-local active flag and counters exclude unrelated allocations on
  other threads. Decode latency includes the counting-wrapper overhead, so the
  raw samples are instrumented observations rather than uninstrumented product
  latency.

## Acceptance limits and open gates

Correctness requires all three in-budget fixtures to decode equal to their
source checkpoint, retain the expected stream count, and remain in canonical
key order. The existing hard count ceiling is 65,536 streams. No absolute
latency or allocated-byte ceiling is defined by Runtime210, so this profile
prints raw measurements without asserting an invented threshold or claiming a
performance pass. Allocation counts are not expected to be zero.

The managed Windows Release invocation must select the
`runtime210_checkpoint_decode_profile` integration target and exact
`runtime210_checkpoint_decode_release_profile` test, enable ignored tests, and
serialize the test binary while preserving stdout (the equivalent libtest
options are `--ignored --exact --test-threads=1 --nocapture`).

This profile does not bound serialized payload bytes, nesting depth, digest
work, or cancellation/deadline. Runtime154-P1-005 and G09 therefore remain
partial; ordinary grouped compilation/tests remain required. The v45 Release
run passed and its complete raw samples are archived below; those results do not close the remaining G09 budgets.

## 2026-09-30 UTC: independent review and grouped validation preparation

Independent static review found no concrete must-fix in the current decoder,
count-boundary regressions, or Release profile. The review receipt is
`.codex/state/session-coordinator/async-validation-batches/offline-candidates/contracts-runtime210-independent-review/review.json`
(SHA-256 `c840bca769bc609289f2d1c8432e0bacd1dd0dbeaffb87b896c0150436369f5f`).

The original v38 grouped submission used 21 admitted source/configuration inputs,
with source fingerprint
`9253cd9424ebd69d33f445b52b18a63c1e8748e84d1ecf9c67d3b56565625b78`.
The original package-source and workspace-manifest evidence is recorded in
`.codex/state/session-coordinator/async-validation-batches/offline-candidates/contracts-v38-root-closed-component/ready-receipt.json`.
Canonical plan records are outside the compile source manifest.

A single reconciliation of the three original outer requests returned:

| Lane | Submission evidence | Acceptance |
|---|---|---|
| Package check | Admission failed: an external `zr_vm` review TSV changed during capture | No ticket or compilation result |
| Normal library tests | Ticket `17ba90403a7d4d0d973ed132f5924659` failed during metadata closure planning | Cargo/test execution did not start |
| Release checkpoint profile | Admission failed: an external `zr_vm` review TSV shortened during capture | No ticket or timing sample |

The immutable reconciliation is
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-v38-single-outer-command-reconciliation.json`
(SHA-256 `08fd79da4b9e523aea4585d79582b969300d16d7542ee98a135616011b752338`).
It queried the original command requests once, without replay or compiler-status
queries. The external capture failures concern
`docs/code-review/coverage/zr_vm_parser_type_inference_type_display_alias.tsv`
and `docs/code-review/coverage/zr_vm_parser_dataflow_ownership_moves.tsv`;
they are submission failures, not failed Rust tests.

No executed test, raw timing sample, or performance pass is established by
these receipts. The downstream Runtime/Editor and remaining G09 gates stay open.

The normal-library ticket was checked once after subsequent F1 v2 source and
canonical-record work. Its terminal receipt is
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-normal-lib-single-receipt-validation-at-f1-v2-boundary.json`
(SHA-256 `452c219950d5a406113cdc81b158c3d36b428b5128343bb5fc6f09fc01569197`).
The coordinator reported `pinned_cargo_metadata_failed` at `closure_planning`,
with `cargoMs=0`: the sealed workspace graph contains the
`zr_dev_deps_dylib` manifest but lacks its `src/lib.rs` target entry. The
21-input package proof is therefore insufficient for Cargo metadata planning.
Seven target-entry source inputs have now been admitted for the new grouped
submission below; this original failure is not a failed checkpoint regression.

The prepared two-lane v40 recovery was not submitted because it reused that
incomplete source manifest. Its independent recipe review is
`.codex/state/session-coordinator/async-validation-batches/offline-candidates/contracts-v40-two-lane-full-worktree-recovery/recipe-review.json`
(SHA-256 `16d7b87b87826eb7be29a76f3a41940f934999f46e35b0325e0a22c6b31a9d18`).
No compiler status was continuously monitored and no existing ticket was replayed.

## 2026-09-30: corrected metadata inputs and asynchronous batch

The failed v38 metadata plan was corrected by admitting seven current default or
explicit target-entry source files. The v42 compile input manifest contains
28 paths, with fingerprint
`2004c264412147788d7d5e3c0db69001ebc0f49ba7383d7cf74a7ca687e534e5`.
The independent entry review is
`.codex/state/session-coordinator/async-validation-batches/offline-candidates/contracts-workspace-target-input-closure/review.json`
(SHA-256 `21c1e12274dc76f784ab4c9d49ba2250ec716a83f6cfe10b418f939abee6dae1`).
The package check, library tests and named profile test have reviewed input
coverage; this manifest does not assert full current-workspace target-list parity.
Unselected current-only auto-discovered build and test targets need separate
closure if the command scope expands.

The three validation requests were posted in one asynchronous wave using the
complete dirty external worktree capture and a freshly observed external HEAD.
The client submission log records `submission=accepted` for all three requests,
but each client call timed out before returning a terminal outer receipt. A
later exact outer-receipt reconciliation resolved all three during admission.
No validation ticket was created and Cargo work did not start.

| Lane | Outer request | Client submission observation | Reconciled terminal outcome |
|---|---|---|---|
| Package check | `807f7989e9b942b3b42d92cfe70cd47f` | `command_post_timeout`; submission accepted | Admission failed: `validation_ticket_external_archive_short_read`; `tests/parser/test_ssa_execbc_vm_fixtures.inc` read 39,640 of 39,642 inventoried bytes |
| Normal library tests | `60de058ff13d48e78589f88812e908da` | `command_post_timeout`; submission accepted | Admission failed: `validation_ticket_external_worktree_conflict`; external worktree capture conflicts with the discovered commit |
| Release checkpoint profile | `fbb9842e12c240c2acae85ecbcccbe7c` | `command_post_timeout`; submission accepted | Admission failed: `validation_ticket_external_worktree_conflict`; external worktree capture conflicts with the discovered commit |

The original client submission log is
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-v42-three-lane-client-submission-wave.json`
(SHA-256 `98e2c867a2d67353dceb5bcfda26445d261731a593fbf20de8e8159ed2de471d`). It
records that each `validation.submit` detail said submission accepted and that
no compiler-status query or wait followed. The exact terminal reconciliation
is
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-v42-single-outer-reconciliation-at-editor04-union-freeze.json`
(SHA-256 `5ad5a2871d583915cbd839701a914be0df92d38aec960a3e0da3c8b6555ce6e8`).
It records three admission failures, zero compiler-status queries, zero waits,
zero replays, and zero source writes. This terminal receipt supersedes the
client log's initial pending outer-status snapshot; accepted submission is not
a validation ticket, test result, or performance result.

2026-10-01 UTC: After the EditorHarness source repair, one finite reconciliation of the original v45 outer journals completed with empty admission blockers and sealed three queued validation tickets: package check `e9e90966dc7041b9a9f87bac12543016`, normal library tests `5d0dea91993e4ddea7cd93aa97899f2c`, and the Release profile `ebbf4f7655714a5a9a4984e82d7e5ed6`. Each ticket receipt records source manifest fingerprint `2004c264412147788d7d5e3c0db69001ebc0f49ba7383d7cf74a7ca687e534e5` over 28 paths and the external capture `E:\Git\zr_vm` at commit `c5715d296de0b6ef59fbe49c1f0253be992c326a`. These are historical submission/queue receipts, not live compiler status. Cargo execution, tests, profile samples, and performance acceptance remain pending; no replay or compiler-status query was made. Reconciliation: `.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-v45-single-outer-reconciliation-at-editor-harness-source-repair-boundary.json` (SHA-256 `f78a2a347d878ef4d56bd7f5030c391182e9eaccfaf81d0189eff86c2f353252`). Original v45 client wave: `.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-v45-three-lane-client-submission-wave.json` (SHA-256 `8a7de17f69fa8e1a4d31492dc8f99afde3c38539724e381cb159be2a2d1b56eb`).

These outcomes do not change the original v38 records: the v38 package and
Release requests failed external TSV capture admission, and the v38 normal
library ticket failed Cargo metadata closure planning with `cargoMs=0`. That
older ticket was not replayed. Managed tests, downstream integration and all
performance acceptance gates remain open.

## 2026-10-01 UTC: one result reconciliation at the palette source repair milestone

The three original v45 tickets were read once after the independent Editor
palette source repair. The immutable result snapshot is
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-v45-one-three-ticket-result-snapshot-at-palette-source-repair-boundary.json`
(SHA-256 `d57c7899cb7d012d61a5fafb13a18a029e3a1bdf5083ba07e49290a714f22b64`).

| Lane | Terminal outcome | Current evidence |
|---|---|---|
| Package check `e9e90966dc7041b9a9f87bac12543016` | Failed before Cargo start | `validation_ticket_cargo_storage_override`, `cargoMs=0` |
| Normal library tests `5d0dea91993e4ddea7cd93aa97899f2c` | Failed before Cargo start | Same rejected inline profile configuration; no library tests executed |
| Release profile `ebbf4f7655714a5a9a4984e82d7e5ed6` | Passed | Exit code 0; 1 test passed; production decoder equality/count/order assertions ran at 1, 1,024 and 65,536 streams |

The coordinator added inline development profile overrides to the ordinary
commands. Its immutable-input policy rejected those overrides before
compilation; this is a coordinator configuration failure, not a failed Rust
regression. The next grouped check/library-test request uses the supported
`toolchain.linkMode=static` setting with the same 28-source component.
The existing Release evidence will be reused while those inputs remain equal.
No tooling source changes or acceptance-policy bypass are part of this repair.

The passed Release snapshot reports source fingerprint
`2004c264412147788d7d5e3c0db69001ebc0f49ba7383d7cf74a7ca687e534e5`,
31 samples per size, and the following complete sample groups in its stdout tail:

| Streams | JSON bytes | p50 ms | p95 ms | p99 ms | Requests per decode | Cumulative requested bytes per decode |
|---|---:|---:|---:|---:|---:|---:|
| 1,024 | 220,004 | 0.5005 | 0.8513 | 0.8952 | 9 | 212,576 |
| 65,536 | 14,450,228 | 37.8585 | 43.7778 | 45.4592 | 15 | 13,631,072 |

The one-stream run passed its assertions, but its complete sample header and
latency array are outside the retained public stdout tail. A single artifact
lookup for the completed Cargo job returned `No managed run`; local expected
log paths were absent. The lookup receipt is
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-runtime210-completed-profile-run-artifact-receipt.json`
(SHA-256 `fbef728137abf971b152ec9619b244278d5df7f90a2051c7260eb4e293d5e67b`).
Complete raw-sample archival for all three sizes therefore remains open; no
numbers are reconstructed for the missing group.

These are instrumented decoder measurements. Allocator requested bytes are
cumulative requests, not live heap, peak memory or RSS. Runtime210 defines no
absolute profile latency/allocation ceiling. Ordinary library validation,
downstream Runtime/Editor tests, the remaining G09 limits and actual product
performance gates remain open.

## 2026-10-01 UTC: v46 admission outcome after the keymap repair

After the actual Editor keymap repair and its canonical records, the two original
v46 outer requests were reconciled once. Both are terminal admission failures:
package check `9e4b85c5763d40eea01cc75ebc0da33e` reports
`validation_ticket_external_worktree_changed` (HEAD changed from
`9de3d7398c8971c1a22a908597dcf5286f4dfbf0` to
`49a01febdc57ab83b2616bea7c8e38e560677f2b` during capture); library tests
`32016be6f1454d8caf070b2a107038d0` report
`validation_ticket_external_worktree_conflict`. Neither created a validation
ticket or executed Cargo/tests. This does not supersede the passed v45 Release
profile; normal library validation remains open.

The exact saved outer receipts are
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-v46-one-two-lane-reconciliation-at-keymap-source-and-record-boundary.json`
(SHA-256 `0cb1d6cb065cd7f0888e9739d16fb25bb843cf0f1247e28fcc9710d2cb1d5364`).
The terminal interpretation below corrects that snapshot's generic
no-unique-ticket label using its already-saved explicit `request.status=failed`;
it makes no additional API query, replay, or compiler wait:
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-contracts-v46-terminal-admission-interpretation-at-keymap-boundary.json`
(SHA-256 `fcb867bd28eeba650577765e1f5ecc1184f552e0cb0a253030210ebfc3d6e10a`).

## 2026-10-01 UTC: complete Release sample archival

The exact Cargo run ID `e9758992758d4d138df3bc7b49d960bd` identified the
actual Cargo job directory `754316746aae459698151cbcdd960222`. The earlier
artifact request had used the separate validation metadata job ID. The full
stdout and stderr are now archived in
`.codex/state/session-coordinator/async-validation-batches/offline-candidates/contracts-v45-runtime210-full-release-profile-artifacts-v1`.

The archive receipt is `receipt.json` in that directory
(SHA-256 `4002aec0e4805316e92769037b3aac5615cf2e3c03ca74da47e205e297719596`).
Original stdout SHA-256:
`b83f55a2124a40622cf6c882c32adbd822830566cafd02892be69bc176c6c036`;
stderr SHA-256:
`e3bc63f950329325fcc47c79895fb05adb1b07d0833fc727caf6def10cf3e167`.
All three groups contain 31 actual latency, allocation-count and requested-byte
samples, and every reported p50/p95/p99 matches nearest-rank recomputation.
The first group is a complete report on the same line as libtest's test-name
prefix; it is now retained without reconstructing data from the truncated tail.
No compiler query, wait or test replay was made during this artifact retrieval.
Normal library validation, G09 and product performance gates remain open.

| Streams | JSON bytes | p50 ms | p95 ms | p99 ms | Requests per decode | Cumulative requested bytes per decode |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 311 | 0.0008 | 0.0013 | 0.0023 | 1 | 416 |
| 1,024 | 220,004 | 0.5005 | 0.8513 | 0.8952 | 9 | 212,576 |
| 65,536 | 14,450,228 | 37.8585 | 43.7778 | 45.4592 | 15 | 13,631,072 |

The table is instrumented decoder evidence and cumulative allocator requests;
it does not establish a live-memory/RSS or product-latency threshold pass.
