---
doc_type: source-manifest
status: source_bound_validation_pending
owner_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
milestones:
  - M1
  - M3
  - M4
supersedes_for_validation:
  - docs/plans/zircon_editor/editor/14/2026-08-11-m1-job-system-current-source-manifest.md
  - docs/plans/zircon_editor/editor/14/2026-08-11-m3-progress-snapshot-current-source-manifest.md
  - docs/plans/zircon_editor/editor/14/2026-08-11-m4-job-scheduler-current-source-manifest.md
exact_path_count: 40
input_path_count: 39
---

# Editor14 M1/M3/M4 Current-Source Manifest

This is the only current-source validation input for the combined M1 admission,
M3 primary-progress generation, and M4 scheduler hard-cut work. The table has
39 frozen inputs; this manifest is the 40th exact path. It replaces the earlier
separate candidates only for validation, not for ownership, historical evidence,
or failure lifecycle.

The previous M4 candidate included `system/mod.rs` but did not include every
declared leaf module. This manifest includes all ten current module owners and
the M1 admission/test sources that they require. It excludes Editor02 message
delivery files and all unrelated working-tree changes. The JobPump failure
remains open until the Editor02 producer contract, managed Cargo, scale matrix,
Windows WPR, and independent second review are complete.

| Path | SHA-256 |
| --- | --- |
| `docs/plans/zircon_editor/editor/14/2026-08-11-job-scheduler-fairness-and-module-boundary.md` | `7ec589977da9000f0381bab1694afdeac5537a0ecd3bb3c5b6c2f57d212b6087` |
| `docs/plans/zircon_editor/editor/14/2026-08-11-m1-job-system-current-source-manifest.md` | `75c310276676bae10d1acb110b9e41957c3ccacec3dc91c85eb444ccf9039477` |
| `docs/plans/zircon_editor/editor/14/2026-08-11-m3-progress-snapshot-current-source-manifest.md` | `49e8a65a14aedc3dbdd79e310e2f12f066910c5a4d181091813289422d5fff21` |
| `docs/plans/zircon_editor/editor/14/2026-08-11-primary-progress-generation-architecture.md` | `e5ccde1e50192f74a1cdbfc85a5d4fc7b82180e32689007f14ac7b4941c2411c` |
| `docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md` | `bcc446fa9f8dcfabe7b238c6e6dcec061dcc01992121e21c03b231d1584aae9f` |
| `zircon_editor/src/core/jobs/admission.rs` | `88c04938c57a7428be650c90cc2227290a096dbc08a12a9c911d8298c587eb34` |
| `zircon_editor/src/core/jobs/error.rs` | `9e0bb14940556ce94d03eedb82fb24d8065b3804952409448956cd04d4480889` |
| `zircon_editor/src/core/jobs/event.rs` | `fb866ccabe2dd0e99a58bc56dbc4eb79c600cc88ab72c106920f592be40ccc15` |
| `zircon_editor/src/core/jobs/event_sink.rs` | `256fb9cdab0a49d6c2157614fbb4243477a1d09b1fd08a35a2cac55f89e28d3c` |
| `zircon_editor/src/core/jobs/limits.rs` | `7161c54c63ab1e4915fedbefd6dde215c5c44d4844279879fc3170164af98ec6` |
| `zircon_editor/src/core/jobs/mod.rs` | `0769cb3026b177cfa77f96b07426d49dc6b2e24b72e7b72eefaffb280b2b71ff` |
| `zircon_editor/src/core/jobs/progress.rs` | `68542104a8e189d4983cc952268db749d1f7ac7562a8d851934cae96d3698314` |
| `zircon_editor/src/core/jobs/progress/primary_generation_tests.rs` | `fc6e9c74ad278e977f8cd46ffd996887bd3f93d10eec8560e679dd6fe6cc7804` |
| `zircon_editor/src/core/jobs/quota_settings.rs` | `8e164c5075b69607c239ef2cff4fee8f0702a6cead27ec6353f00114ee14b778` |
| `zircon_editor/src/core/jobs/spec.rs` | `656af883a62422954af4d22a0e4a5f14472110e99f26611d923a677ff3fab30c` |
| `zircon_editor/src/core/jobs/system/admission_ledger.rs` | `fdcc83d236c6ee1f70bcc0a8d3a1499f09256178152c04cbc214b1dfb1ed1de9` |
| `zircon_editor/src/core/jobs/system/admission_reservation.rs` | `325040a061f49df04d16b8710615f4b8a5b77045a5de86657dd291a28c5bfc83` |
| `zircon_editor/src/core/jobs/system/construction.rs` | `445af90b620ee48d22261d3c0aaae3a9401fb615eb9318b014b43a3a5993cc1d` |
| `zircon_editor/src/core/jobs/system/lifecycle.rs` | `2c3dba51aa331d44d7e8cb548c7ae2f82319a490019d6135b7688994ced8baee` |
| `zircon_editor/src/core/jobs/system/mod.rs` | `d549f3f8736c17bd04a386ca8a9769dabcf2a5e3c4f2a5f5f31264353beb513e` |
| `zircon_editor/src/core/jobs/system/pending.rs` | `0bafe00418cbacf6889e97edb10c0157cefe61c9e49d19abd7ef7f3671d03805` |
| `zircon_editor/src/core/jobs/system/pending_task.rs` | `a9fc2e08d14927ec33baa1ba1f141d22a7d0d7a79cd3dca0d8d100402456888e` |
| `zircon_editor/src/core/jobs/system/pending/tests/admission.rs` | `9dba4012fb32079948f1b4cf69607e732d28e4e0ec8258bef4778d0ffb42ca59` |
| `zircon_editor/src/core/jobs/system/pending/tests/fairness.rs` | `637d9566f1d2c13174f27b9a09497c685e634295989a8046895c98b1f81f06be` |
| `zircon_editor/src/core/jobs/system/pending/tests/mod.rs` | `5ccb74e3cd2fddd94b8b3c90a93383874693319878d4fae84f0e3c1485bd1ac3` |
| `zircon_editor/src/core/jobs/system/progress_observer.rs` | `3cc57720e623ecf6828559f43ceff8198e33b64638695c0cc1cf1882bc05cc46` |
| `zircon_editor/src/core/jobs/system/scheduling.rs` | `3220f06513d6830194bdaf7172cd7a4e2b34e07a55099b1fcfe67225b59cbf8d` |
| `zircon_editor/src/core/jobs/system/state.rs` | `d0346c540edefe0a6aff9ee772ab31ec4a98784888c4842a3fc607498a71bc90` |
| `zircon_editor/src/core/jobs/system/submission.rs` | `9ed622c4b24c3033f7818b7e7943778d22d17fe787571d4f7953306d7b152c59` |
| `zircon_editor/src/core/jobs/tests/admission_scaling_contract.rs` | `cbef22f0ef0895602df598336e678cd97a2a37e585610240c6b51f38a6d15d5f` |
| `zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs` | `7ea9e5c27f9db2a885f497afeba9ca9fe43302df1a769a8b55269a00f563bd71` |
| `zircon_editor/src/core/jobs/tests/admission_scaling_contract/keyed.rs` | `16c75ccc632a6df8bd66383c0a1b87d92b1fc6c2b60ad3c6b9a40649d57c47a9` |
| `zircon_editor/src/core/jobs/tests/admission_scaling_contract/reservation.rs` | `adea721ad851c9334b63d5e10237de23adba8f395210883a482ed52659064ae9` |
| `zircon_editor/src/core/jobs/tests/admission_scaling_contract/support.rs` | `11be5c539048e1116dd212d76d1eebaf3d6934c1ba2defc66f33dc20709e4524` |
| `zircon_editor/src/core/jobs/tests/background_storm_contract.rs` | `d783b45d158ab20d40ae8593c5d084f9914f87da2789b95f867979021aadaa02` |
| `zircon_editor/src/core/jobs/tests/quota_settings_contract.rs` | `3a88bfc7ddf5a0bf85ffd39072070386e08aa2bff44dcf36c97329c486d98ec3` |
| `zircon_editor/src/core/jobs/tests/scheduling_contract.rs` | `7170645a5f17d9caff952ba4ed0e3fc96a959ea53a4fea88cc04aee8cd203061` |
| `zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs` | `0328318cb142ce23592d5c3db554a441d1fae5764358b3d52b5bdd8e7a6bcba5` |
| `zircon_editor/src/core/jobs/tests/thread_ownership_contract/scanner.rs` | `266417fed7457f10e8757fb68b919108eed8455b064008434f3cee6c59c8aefa` |

## Required Managed Validation

- `cargo test -p zircon_editor --lib ready_background_job_is_selected_within_one_weighted_fairness_round --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib system_root_is_a_structural_leaf_module_entry --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib cloned_events_share_the_job_stable_label_allocation --locked --jobs 1 -- --test-threads=1`
- current-source `zircon_editor` library validation, 1k/10k admission matrix, and Windows WPR after the existing Editor02 lower-layer handoff becomes terminal.
