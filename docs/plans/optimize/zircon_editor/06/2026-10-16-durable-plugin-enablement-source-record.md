Editor06 E-PLUGIN-UX-P0-01 durable enablement source record - 2026-10-16

The actual checkout now contains corrected/rebased V3. The v1 names of this record set are logical output names; they do not mean the rejected V1 planned product was applied. Source implementation and byte-level static checks are complete. Supported guard, native execution and product acceptance remain OPEN.

- [x] Apply only33 changed files after fresh38 granted before bytes (308299 B, four absent) matched. Preserve5 unchanged files. Current38 total422121 B.
- [x] Double raw after hashes match V3; normal forward/reverse38 exact; all original40 ordinary identities preserved and16 new cases present.
- [x] HEAD f28ba39a5095f4964c046ecc702778627f80cfd5 and index7478958 B / 8ae41eb940729cf4f1ea1468d72fdbe257581bc1fc38ace9dce024c481400893 retained; exact Windows ticks639276485811354535 before/after source application.
- [x] V1/V2 artifacts remain immutable; V3 bridge preserves external formatting and the entire unknown-provenance resource testleaf. No source formatter ran.

- E:\Git\ZirconEngine\.codex\outbox\reports\optimize-20261013-01a13451\editor06-durable-plugin-enablement-v3.patch (138687 B), SHA256 `32be3f01be9876f6782f6d68d2392ea467049cd0613600d2cf3dbfe36702a827`.
- E:\Git\ZirconEngine\.codex\outbox\reports\optimize-20261013-01a13451\editor06-durable-plugin-enablement-v3-source-binding.json (202459 B), SHA256 `e125019ffc133e03a2bb8dfffd875e21ce82a04576ff2f4e103004c3e30d6d28`.

- A same bounded <=4MiB byte snapshot supplies parse and SourceRevision; plan carries expected revision plus physical project/session/catalog/native binding.
- Existing project owner serializes FIFO and OS generation lock; manifest-specific journal durably publishes manifest+pending before live apply; release I/O locks before capability/lifecycle work.
- A durable failure leaves live owner, capabilities, lifecycle and the published report Arc unchanged; CleanupDeferred and CommitRecoveryDeferred have different read/apply gates.
- Binding is captured before prepare, compared after prepare, then rechecked after durable commit; an old plan cannot be stamped with a newer catalog generation.
- Registered Project/native metadata with no proven live handle yields RuntimeRepairRequired+NotApplied; ordinary restart is not claimed to apply it.
- Manager-private RuntimeRepairRequired fence is independent of disposable report Arc; Unknown and original primary/rollback causes survive Inspect/status replacement/clear.
- Same physical Ready session blocks every new SetEnabled intention, Retry and live Revert irrespective of new op/plugin/revision; no callback replay after Unknown.
- Only explicit writer-admitted non-Ready oldop+expected committed revision durable new resolution, matching original receipt identity and fresh preflight, may clear the fence.
- Journal/pending recovery gate runs before authoritative manifest parse; Inspect/RecoverFiles never auto-apply; Unload/HotReload early routes remain.
- Typed phase/durable/applied/recovery plus primary+rollback causes reach existing status report and projected recovery rows; new Arc invalidates ptr_eq caches.
- Two Editor startup preflight-error callers preserve the primary error and project authenticated recovery actions; upstream App-before-host path remains OPEN.

The two formatting deltas were reverse-hash-proven against original before bytes. source_revision/tests.rs current3717 B / b0e909abf0c96b225c80b4987f686f7230e3f123fce015615c7155ed325b1470 is unchanged by E06; its prior full semantic delta and actor remain UNKNOWN. Existing module_plugin_details.rs unknown13 B baseline is preserved too.

Applied source process: cell5/268162 actualexit0/no session, PS20108/Py31600. Git26408/21404/3140/30060 each communicate+wait0 joined. The33 after hashes are in the implementation receipt/source_files; all38 correspond exactly to V3 binding compact_source_activation_rows.

Current pre-write failures retained:

- 3/5a7672: actual exit1, PS2200/Py22728, source0/output0; KeyError fixed38_binding in root V3 grant; actual key binding. Before Git/source writes, mandatory compact38 and changed33/unchanged5 matched; schema corrected without changing frozen source pins.
- 4/4a3189: actual exit1, PS25100/Py12700, source0/output0; Second legacy fixed38_binding reference remained because first JavaScript replacement only replaced first occurrence; all legacy references now removed in RAM. Before Git/source writes.

Earlier V1/V2/V3 attempt failures and historical spawn_ready UNKNOWN identities remain recorded in JSON; known current process exits do not close those unknowns.

Source regression inventory: original40 ordinary/0ignored, added16 ordinary, selected lexical56/0. These are identities, not native passes or native floor. All16 exact names follow.

| Package / binary | Exact new ordinary FQN | cfg | Execution |
| --- | --- | --- | --- |
| zircon_editor / lib | `core::project::tests::preflight::plugin_enablement_gate::journal_gate_wins_before_malformed_authoritative_manifest_parse` | `all(test, windows)` | NOT RUN |
| zircon_editor / lib | `core::project::tests::preflight::plugin_enablement_gate::durable_pending_gate_wins_before_malformed_authoritative_manifest_parse` | `all(test, windows)` | NOT RUN |
| zircon_editor / lib | `tests::host::manager::minimal_host_contract::native_plugins::durable_enablement::manifest_replace_failure_preserves_live_owner_and_report_arc` | `all(test, windows)` | NOT RUN |
| zircon_editor / lib | `tests::host::manager::minimal_host_contract::native_plugins::durable_enablement::generation_change_during_prepare_cannot_stamp_an_old_plan` | `all(test, windows)` | NOT RUN |
| zircon_editor / lib | `tests::host::manager::minimal_host_contract::native_plugins::durable_enablement::generation_change_after_commit_keeps_typed_pending_without_runtime_apply` | `all(test, windows)` | NOT RUN |
| zircon_editor / lib | `tests::host::manager::minimal_host_contract::native_plugins::durable_enablement::registered_project_package_without_live_handle_requires_explicit_repair` | `all(test, windows)` | NOT RUN |
| zircon_editor / lib | `tests::host::manager::minimal_host_contract::native_plugins::durable_enablement::cold_preflight_projects_admitted_revert_and_then_allows_a_fresh_open` | `all(test, windows)` | NOT RUN |
| zircon_editor / lib | `ui::retained_host::app::module_plugin_projection::pane_data::view_rows::enablement_state_tests::typed_operation_projects_reachable_recovery_actions_without_a_live_plugin_row` | `test` | NOT RUN |
| zircon_runtime / lib | `asset::project::plugin_apply::tests::stale_manifest_revision_rejects_manifest_and_pending_publication` | `all(test, windows, feature = "native-host")` | NOT RUN |
| zircon_runtime / lib | `asset::project::plugin_apply::tests::interrupted_manifest_replace_blocks_parse_until_owned_recovery_restores_pair` | `all(test, windows, feature = "native-host")` | NOT RUN |
| zircon_runtime / lib | `asset::project::plugin_apply::tests::cleanup_and_commit_recovery_dispositions_have_different_read_gates` | `all(test, windows, feature = "native-host")` | NOT RUN |
| zircon_runtime / lib | `asset::project::plugin_apply::tests::stale_acknowledgement_keeps_pending_and_preflight_blocked` | `all(test, windows, feature = "native-host")` | NOT RUN |
| zircon_runtime / lib | `asset::project::plugin_apply::tests::explicit_preopen_revert_is_a_new_durable_resolution_at_expected_revision` | `all(test, windows, feature = "native-host")` | NOT RUN |
| zircon_runtime / lib | `asset::project::plugin_apply::tests::recovery_refuses_a_foreign_target_before_mutating_its_bytes` | `all(test, windows, feature = "native-host")` | NOT RUN |
| zircon_runtime / lib | `asset::project::plugin_apply::tests::bounded_manifest_snapshot_rejects_oversize_before_parse` | `all(test, windows, feature = "native-host")` | NOT RUN |
| zircon_editor / lib | `tests::host::manager::minimal_host_contract::native_plugins::durable_enablement::inspect_preserves_unknown_runtime_fence_and_rejects_retry_revert_and_new_intent` | `all(test, windows)` | NOT RUN |

Future combined validation recipes (recorded only, never executed by this task):

- `cargo test -p zircon_editor --lib`: package default=[]; actual Runtime dependency target-editor-host plus dev test-support; NOT RUN.
- `cargo test -p zircon_runtime --lib --no-default-features --features target-editor-host,test-support asset::project::plugin_apply::tests:: -- --test-threads=1`: existing target-editor-host implies native-host; explicit test-support enables zr_resource fault support; seven Windows-only proposed regressions; NOT RUN.
- `cargo test -p zr_resource --lib io::source_revision::tests:: -- --test-threads=1`: existing resource reader namespace; preserved tests, no new mirrored unit case; NOT RUN.

- [ ] supported >=3.11 change guard NOT_RUN_OPEN
- [ ] all Cargo/build/native/bench/Python unittest NOT_RUN_OPEN; native requests=0
- [ ] full cold UI and end-to-end product acceptance OPEN
- [ ] upstream App-before-host preflight error UX OPEN
- [ ] exactly-once callbacks and full cold recovery OPEN
- [ ] external noncooperating-writer CAS OPEN
- [ ] performance acceptance OPEN; no samples or budgets changed
- [ ] ordinary lexical56 is source inventory, not passed tests or native floor
- [ ] Windows fixture creation/last worker close/cleanup are source obligations, not executed cleanup evidence
- [ ] historical spawn_ready PID/session/terminal UNKNOWN remain UNKNOWN

All controlled fixtures require physical E:/cargo-targets/zircon-local before mkdir/write, then real last Runtime/manager/handle/worker closure before owned-fixture cleanup. No fixture ran here, no cleanup execution is claimed, released space0. No Cargo/native/bench/guard/CI/Python unittest ran or was queried.
