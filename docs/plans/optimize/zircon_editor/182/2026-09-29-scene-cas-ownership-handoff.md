---
title: Editor182 scene save CAS ownership handoff
category: zircon_editor
report_id: Editor182-P0-01-CAS-handoff-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_editor/182-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
implementation_status: blocked_by_active_shared_scope
validation_status: not_run_no_implementation
performance_status: not_measured
---

# Editor182: scene save CAS handoff

The open `ED182-P0-01` data-loss path spans ordinary Save and the external-conflict Save choice. Both can reach an unconditional Runtime scene write. Changing the conflict dialog alone would leave ordinary Save and explicit conflict Save able to overwrite a newer external revision.

A complete slice must capture the exact source revision during startup/open/reload, retain it in the active document session, and send it to Runtime's serialized conditional publish. Runtime must compare the expected revision under its writer lease, return a typed conflict without changing source bytes, and distinguish a persisted source from a later Editor projection failure. Editor must preserve dirty state and suppress Saved on conflict. Dual-writer and external replace/delete/recreate regressions are needed. Scene staging creation must use an expected-missing condition; the existing Save caller API can remain stable, avoiding the MVP Play-owned menu action path.

The 2026-09-29 14:23 UTC coordinator snapshot showed active `Source comment audit M1` Session `1d9a143c-c6a1-4da3-a702-eeeff3e26b8f` owning `zircon_editor/src/core/project/scene_document.rs`, a required revision-propagation path. That file and the conflict/reload paths already contain changes from other work; Runtime's `scene_asset.rs` contains light-shadow serialization work to preserve. Two registered MVP Play sessions own `zircon_editor/src/ui/host/editor_event_execution/menu_action.rs`. No CAS production file was changed for this handoff. Reconcile the active scopes before implementing the complete ED182-M0 slice.

No Rust test, managed validation, external-writer matrix, or performance measurement was run. `ED182-P0-01` and its source revision gate remain open.
