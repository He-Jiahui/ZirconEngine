---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-08-16
summary_slug: editor-context-autosave-construction-drift
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/17
related_code:
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/recovery/autosave_service.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/retained_host/app.rs
tests:
  - powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-editor.ps1 -OutputDirectory E:\ZirconBuilds\editor-context-composition
---

# Editor17: context autosave construction drift

## Failure evidence

Current `EditorContext::new` requires `Arc<EditorLogService>` at parameter five and
`EditorAutosaveService` after notifications (`editor_context.rs:38-52`). Current
`EditorContextBuilder::build` creates a value `EditorLogService`, passes it at parameter five and
passes `EditorTransactionEngine` immediately after notifications (`builder.rs:296-332`). No autosave
service is constructed or passed. The production `EditorManager` uses this builder.

This is a deterministic source-signature mismatch. It prevents current context construction before
performance measurement can begin. The foreign diffs show two incomplete migrations: quota/settings
startup in `builder.rs`, and shared logging plus autosave ownership in `editor_context.rs`.

## Ownership and required repair

Performance01 owns only `docs/plans/performance`; the overlapping Rust changes are foreign and were
preserved. Editor17 should complete the composition with Editor14:

1. create one `Arc<EditorLogService>` and configure that exact shared instance;
2. create one `EditorAutosaveService` using a clone of the already-created `EditorJobSystem` handle
   and the accepted Editor17 autosave policy;
3. pass autosave between notifications and transactions without changing the public aggregate order;
4. prove autosave admission and progress use the same job-system authority and that shutdown does not
   terminate or duplicate the shared scheduler;
5. keep project-generation fencing, payload admission and retained-tick activation in the existing
   recovery plan rather than declaring them solved by constructor wiring.

Do not create a second scheduler, job system, log store, message bus or service registry. Do not wire
the currently uncalled retained autosave poll merely to make it reachable before its P0 recovery scale
and generation contracts pass.

## Acceptance

- The focused context/recovery composition tests compile and pass under the managed Windows validator.
- Product `EditorManager` constructs exactly one complete context; log identity and autosave job
  authority are shared and observable in tests.
- Forced failure at every assembly stage publishes no partial context and leaves no worker/service
  owner behind.
- Managed editor build and `--help` smoke pass in an approved D/E/F output root; no artifact is written
  to C:.
- F0 startup receipt reports settings, quota, owner construction, wiring and publication separately.

## Current external blocker

The approved-root separator defect in `tools/build-editor.ps1:130` currently prevents the product
command above from reaching Cargo. That independent failure is recorded in
`failure-2026-08-15-build-editor-approved-root-separator.md`. This record remains open until both the
source repair and managed validation are green.
