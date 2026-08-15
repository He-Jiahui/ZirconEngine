---
doc_type: source-manifest
status: superseded_for_validation
owner_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
milestone: M4
supersedes_for_validation:
  - docs/plans/zircon_editor/editor/14/2026-08-11-m3-progress-snapshot-current-source-manifest.md
superseded_by:
  - docs/plans/zircon_editor/editor/14/2026-08-11-m1-m3-m4-current-source-manifest.md
exact_path_count: 23
---

# Editor14 M3/M4 当前源码清单（已由完整 successor 取代）

此文件原本试图固定 M3 progress generation，以及 M4 JobPump fairness、稳定标签和 `system/` 硬切的当前源。
后续 source-closure 审计确认它列出的 22 个输入路径没有包含 `system/mod.rs` 声明的三个 M1 leaf owner
`admission_ledger.rs`、`admission_reservation.rs` 和 `pending_task.rs`，也遗漏了其余 M1 admission/test
source。因此它不能作为可独立验证的 current-source 清单，不能复用任何旧 receipt。

M3 snapshot `1632` 的 progress generation 实现没有回滚；但 `EditorJobSpec.label` 改为 `Arc<str>` 后，
`progress.rs` 必须显式构造 UI snapshot 的 `String`。后续 M1/M3/M4 current-source 验证唯一使用
[`2026-08-11-m1-m3-m4-current-source-manifest.md`](2026-08-11-m1-m3-m4-current-source-manifest.md) 的
40-path union。它不包含任何外部 dirty hunk 或 Editor02 bus 代码。JobPump 的 queue-front retry 和
lifecycle delivery reservation 仍依赖 Editor02 的 non-consuming lossless producer contract；该 successor
不能替代此 lower-layer handoff。

| 路径 | SHA-256 |
| --- | --- |
| `docs/plans/zircon_editor/editor/14/2026-08-11-m3-progress-snapshot-current-source-manifest.md` | `49e8a65a14aedc3dbdd79e310e2f12f066910c5a4d181091813289422d5fff21` |
| `docs/plans/zircon_editor/editor/14/2026-08-11-primary-progress-generation-architecture.md` | `e5ccde1e50192f74a1cdbfc85a5d4fc7b82180e32689007f14ac7b4941c2411c` |
| `docs/plans/zircon_editor/editor/14/2026-08-11-job-scheduler-fairness-and-module-boundary.md` | `c3c3b7341d0c9af76c479c52b7a28361181f1d698c2bc1a77288b131a13d2a3a` |
| `docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md` | `6cdfbe7639ffda9ccde6075d5e82dd15ac9418d6127849f92cca6bdc9f2c3f7a` |
| `zircon_editor/src/core/jobs/mod.rs` | `0769cb3026b177cfa77f96b07426d49dc6b2e24b72e7b72eefaffb280b2b71ff` |
| `zircon_editor/src/core/jobs/spec.rs` | `656af883a62422954af4d22a0e4a5f14472110e99f26611d923a677ff3fab30c` |
| `zircon_editor/src/core/jobs/event_sink.rs` | `256fb9cdab0a49d6c2157614fbb4243477a1d09b1fd08a35a2cac55f89e28d3c` |
| `zircon_editor/src/core/jobs/progress.rs` | `68542104a8e189d4983cc952268db749d1f7ac7562a8d851934cae96d3698314` |
| `zircon_editor/src/core/jobs/progress/primary_generation_tests.rs` | `fc6e9c74ad278e977f8cd46ffd996887bd3f93d10eec8560e679dd6fe6cc7804` |
| `zircon_editor/src/core/jobs/system/mod.rs` | `d549f3f8736c17bd04a386ca8a9769dabcf2a5e3c4f2a5f5f31264353beb513e` |
| `zircon_editor/src/core/jobs/system/construction.rs` | `445af90b620ee48d22261d3c0aaae3a9401fb615eb9318b014b43a3a5993cc1d` |
| `zircon_editor/src/core/jobs/system/submission.rs` | `9ed622c4b24c3033f7818b7e7943778d22d17fe787571d4f7953306d7b152c59` |
| `zircon_editor/src/core/jobs/system/lifecycle.rs` | `2c3dba51aa331d44d7e8cb548c7ae2f82319a490019d6135b7688994ced8baee` |
| `zircon_editor/src/core/jobs/system/scheduling.rs` | `3220f06513d6830194bdaf7172cd7a4e2b34e07a55099b1fcfe67225b59cbf8d` |
| `zircon_editor/src/core/jobs/system/progress_observer.rs` | `3cc57720e623ecf6828559f43ceff8198e33b64638695c0cc1cf1882bc05cc46` |
| `zircon_editor/src/core/jobs/system/pending.rs` | `0bafe00418cbacf6889e97edb10c0157cefe61c9e49d19abd7ef7f3671d03805` |
| `zircon_editor/src/core/jobs/system/pending/tests/mod.rs` | `5ccb74e3cd2fddd94b8b3c90a93383874693319878d4fae84f0e3c1485bd1ac3` |
| `zircon_editor/src/core/jobs/system/pending/tests/admission.rs` | `9dba4012fb32079948f1b4cf69607e732d28e4e0ec8258bef4778d0ffb42ca59` |
| `zircon_editor/src/core/jobs/system/pending/tests/fairness.rs` | `637d9566f1d2c13174f27b9a09497c685e634295989a8046895c98b1f81f06be` |
| `zircon_editor/src/core/jobs/system/state.rs` | `d0346c540edefe0a6aff9ee772ab31ec4a98784888c4842a3fc607498a71bc90` |
| `zircon_editor/src/core/jobs/tests/scheduling_contract.rs` | `7170645a5f17d9caff952ba4ed0e3fc96a959ea53a4fea88cc04aee8cd203061` |
| `zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs` | `7ea9e5c27f9db2a885f497afeba9ca9fe43302df1a769a8b55269a00f563bd71` |

## 受管验证

- `cargo test -p zircon_editor --lib ready_background_job_is_selected_within_one_weighted_fairness_round --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib system_root_is_a_structural_leaf_module_entry --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib cloned_events_share_the_job_stable_label_allocation --locked --jobs 1 -- --test-threads=1`
- current-source `zircon_editor` library validation, 1k/10k admission matrix, and Windows WPR, after the existing Editor02 lower-layer handoff becomes terminal.
