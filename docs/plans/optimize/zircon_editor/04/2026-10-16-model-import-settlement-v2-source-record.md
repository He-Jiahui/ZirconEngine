# Editor04 model import settlement v2 source record

Date: 2026-10-16. Narrow committed-import to Editor projection/Scene settlement slice; the full Editor04 P1-03 row remains OPEN.

- [x] Fixed source12 applied and exact after hashes verified.
- [x] Independent scoped source review and manual raw/normal evidence recorded.
- [ ] Execute new9/preserved13 and satisfy native, GUI, whole guard, configuration and performance acceptance.

A successful durable import receipt remains owned through projection errors, cancellation and close settlement. Retry performs projection/settlement only; it never reruns or rolls back the durable import. Published readiness requires the same activation and matching committed records. A started Scene apply that later fails enters RepairRequired and cannot replay. Resume requires typed PendingDocumentSave/QueuedDocumentSaveAll refusal in actual Decision with no possibly-started close. Actual Quiescing retains close intent and forbids Scene insertion; stale Resume is revalidated.

Writer123 Py30572/PS10724 remains actual exit1: six writes completed and workspace.rs held an exact candidate prefix plus old tail. The errno13 compound statement does not establish a unique failing suboperation or permission cause. Root authorized ordinary same-sandbox recovery. Reader125 Py37020/PS38596 exited0. Standard apply_patch126 returned an empty object with no PID exposed. Writer127 Py48860/PS48460 exited0 (chunk c8d005), verified the fixed workspace after, retained seven completed files and wrote only the remaining five. Its Git children43700/48196/36240/48076/49060/36460/48876 all exited0 and joined. The earlier parser prewrite exit1 and all historical UNKNOWN evidence remain preserved.

Post writer at creation: Py45124/PS44140. Its actual external terminal is reported after exclusive output creation; these documents do not preclaim their own writer exit0.

Immutable inputs:

- E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor04-model-import-settlement-v2.patch | 68459 bytes | SHA256 63e84392677548e9fb1b8bfa06c2f8d48fe62055d304c9250f13dbac1647b06d
- E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor04-model-import-settlement-v2-binding.json | 91716 bytes | SHA256 4ec77086ad4a12f53a5d236888d48c20f11721c00f5c9718bf5641cd004ef4f9
- E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor04-model-import-settlement-v2-source-activation.json | 28239 bytes | SHA256 903996043dabc1e3af345d6ee968ab559dccdf03caf025e153243d86a8e55515
- E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor04-model-import-settlement-v2-source-activation.md | 671 bytes | SHA256 70fbc4d44bfaed8c8b074a925ce13eba82319f98fb852b395dc90fbfdff8ef10
- E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor04-model-import-settlement-v2-partial-recovery-activation.json | 20766 bytes | SHA256 f8870dfb99d5951967585645b2db26fda93f6ddf11534269137ce1e986d977b9
- E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor04-model-import-settlement-v2-partial-recovery-activation.md | 745 bytes | SHA256 0c70bf1f7cac9c47f31a4c4431ddba0a4c77c3cce6ec9b8adaf8456d6bd4888c

Source after pins:

- zircon_editor/src/core/asset/import_flow/model_ticket.rs | 1637 bytes | SHA256 f17c4fa07c3f21d69854ba7e439433eab4820a1428fbc1b5d91b1b23228e9704
- zircon_editor/src/core/asset/import_flow/model_submit.rs | 2214 bytes | SHA256 7c7bbb568fef4bd1197f1804a10ec4a7dfbbc60fd0d6e7a9c9a2b9be8efd461e
- zircon_editor/src/ui/host/assets/api.rs | 3602 bytes | SHA256 2caef6cc55439f36a8d8b4992b96abc5d2157c122be7064c287437d567a27600
- zircon_editor/src/ui/host/assets/mod.rs | 1156 bytes | SHA256 59414f33550954b2ed7b4e6409334b8dbd6e610f062a8802ba692ff91d653948
- zircon_editor/src/ui/host/assets/manager/assets/preview_trait_bridge.rs | 3441 bytes | SHA256 ea1b50ae9ad3490af53a15fefae5669208f6c8ee2b965e42f1e37d02dffddb45
- zircon_editor/src/ui/host/assets/manager/assets/mod.rs | 8246 bytes | SHA256 981438bf1c38b1bc1fd3bfab070deff248314fc476e6c916d8d48ca341a042c4
- zircon_editor/src/ui/retained/app/assets/workspace.rs | 20423 bytes | SHA256 47ae6692c3865f51278de51574bef5fb4c7d7931483b0931b9a96e1b01675ab3
- zircon_editor/src/ui/retained/app/assets/workspace/model_import_settlement.rs | 27527 bytes | SHA256 59847733ede2e97a8c254d8f7a78fe287b2942d099656fe6c62b14cf9eeca41f
- zircon_editor/src/ui/retained/app/assets/workspace/tests/reload.rs | 39333 bytes | SHA256 af93cd2aa3214378a2c229d19d74bc54637dbf16f03b5d1b8c03a51153aa7fc7
- zircon_editor/src/ui/retained/app/assets/workspace/tests/model_import_settlement.rs | 21067 bytes | SHA256 b3883e1d28032fc24197420be3b06f970902f4539196c6cf335476490fd2ac02
- zircon_editor/assets/i18n/en.toml | 50987 bytes | SHA256 ec312170b1f9b9278f4528014afd29e011811d92a8461e28f394869da8e9c3ed
- zircon_editor/assets/i18n/zh-CN.toml | 49811 bytes | SHA256 78b97115c1a22efd03ea45d4f463bda3e79fcdb63b9b936eb584c17efc941372

Route ordinal3: zircon_editor debug test -p zircon_editor --no-default-features --lib --locked --. Append OR selector ui::retained_host::app::assets::workspace::reload_tests::. Real parents: workspace.rs cfg(test), alias reload_tests; reload.rs file cfg(windows); child model_import_settlement_tests. Preserved13 and new9 are named AND requirements, not a floor. Old167, all earlier named/subset/typed predicates and all22 performance gates remain independent AND.

Declared cases, all UNRUN:

- ui::retained_host::app::assets::workspace::reload_tests::save_overwrites_the_observed_revision_once (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::save_after_a_second_external_change_preserves_source_dirty_and_checkpoint (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::ordinary_save_rejects_an_unobserved_external_change (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::conflict_requires_a_new_decision_before_retry (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::discard_after_a_new_edit_since_the_prompt_requires_a_new_decision (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::an_edit_after_discard_submission_survives_real_scene_job_completion (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::unchanged_discard_authorization_allows_the_real_scene_job_to_commit (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::a_scene_activation_supersedes_the_pending_discard_without_clearing_history (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::a_project_generation_supersedes_the_pending_discard_without_mutating_the_world (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::a_gizmo_preview_survives_completion_and_requires_a_new_decision_after_commit (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::cancelling_a_preview_resumes_the_same_authorized_prepared_reload (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::a_clean_scene_preview_defers_the_same_job_until_the_interaction_ends (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::play_and_simulate_keep_pending_authoring_reload_out_of_both_history_domains (old, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::durable_import_projection_io_failure_retries_only_settlement_once (new, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::deferred_and_replaced_decision_tickets_cannot_replay_a_committed_import (new, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::newer_generation_with_replaced_receipt_record_is_not_publication_proof (new, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::reopened_same_root_activation_retires_the_old_import_decision (new, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::close_retry_preserves_close_intent_until_explicit_resume (new, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::scene_apply_started_fence_preserves_receipt_and_forbids_retry (new, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::refresh_ok_without_matching_project_projection_is_not_a_receipt_acknowledgement (new, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::quiescing_close_failure_never_offers_resume_or_inserts_the_committed_model (new, ordinary, UNRUN)
- ui::retained_host::app::assets::workspace::reload_tests::model_import_settlement_tests::stale_resume_ticket_cannot_reopen_a_real_quiescing_close (new, ordinary, UNRUN)

Open acceptance:

- compiler_configuration_seals: OPEN
- cross_process_exactly_once: OPEN
- full_callback_downstream_reentrancy_concurrency: OPEN
- general_outer_publication_full_Editor04_P1_03: OPEN
- native_GUI: OPEN
- native_tests: UNRUN
- performance_RSS: OPEN
- predecessor_old167_named_subset_typed_and22perf: REQUIRED_AND_UNRUN
- whole_change_guard: NOT_RUN

No fixture ran here. Before native execution the owner must preregister all fixture/cache physical paths under E:/cargo-targets/zircon-local and bind TEMP, TMP and RUSTC_TMPDIR in the command configuration. No repository/C-drive temporary fixtures or path aliases are authorized.

Manual static evidence is not an executed whole change guard. Full callback downstream/reentrancy/concurrency remains OPEN. Released space:0 bytes. Immutable evidence retained; no fixture, build, native test, CI query, Git mutation or root metadata write ran in this source/post slice.
