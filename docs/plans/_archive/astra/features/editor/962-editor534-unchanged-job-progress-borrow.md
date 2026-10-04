---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor534-unchanged-job-progress-borrow.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/status.rs
tests:
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
---

# Editor962 Editor534 unchanged job-progress borrow

Job-progress publication now compares the borrowed projected snapshot before
cloning and only owns strings on the changed path. The workbench bridge still
receives changed progress, while unchanged updates return early. Marker
`EDITOR534_UNCHANGED_JOB_PROGRESS_BORROW_BENCH_V1` models 32,768 unchanged
updates with zero optimized pre-comparison string clones. Managed Editor
Release validation remains pending.
