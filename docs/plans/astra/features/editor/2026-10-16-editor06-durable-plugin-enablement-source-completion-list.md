Editor06 source completion list - 2026-10-16

E-PLUGIN-UX-P0-01 has an applied V3 source fix and byte-level static evidence. Full acceptance remains OPEN. This list records the scoped source slice and does not close the entire Editor06 plan or upstream App UX.

- [x] Expected-revision bounded document prepare, durable manifest+pending owner transaction, then generation-bound runtime apply.
- [x] Durable failure leaves live state and prior report Arc unchanged; typed durable/apply/recovery failures remain actionable.
- [x] Early preflight journal/pending gate and explicit admitted oldop+revision resolution; observation never auto-applies.
- [x] Manager-private monotonic repair fence protects Unknown/original causes across Inspect and status eviction, including new SetEnabled intentions.
- [x] Three real path API fixes, actual Sync fault branch, real Project-generation producer, and real catch_unwind16th regression are in checkout.
- [x] Exact38 double raw/normal reverse+forward proof; only33 written,5 preserved; original40 ordinary names retained,16 new names added.
- [x] Read-only V2 core and V3 bridge peer findings incorporated.

Actual source evidence: [source record](E:/Git/ZirconEngine/docs/plans/optimize/zircon_editor/06/2026-10-16-durable-plugin-enablement-source-record.md) and [implementation receipt](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor06-durable-plugin-enablement-v1-implementation-receipt.json). These logical-v1 filenames bind corrected/rebased V3.

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

Validation recipes, NOT RUN:

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

Source activation actual5/268162 exit0 PS20108/Py31600, four joined Git children0. The4 record writer actual terminal is external evidence reported after file fsync/readback; it is not inferred from this document body. Old UNKNOWN process and source attribution history stays retained. No fixtures/helper/build outputs were created, released space0.
