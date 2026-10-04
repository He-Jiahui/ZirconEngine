---
title: Editor04 Release Preview Publication Admission Completion
category: zircon_editor
report_id: Editor04-preview-release-admission-completion-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_debug_and_release_pending
performance_status: not_measured
---

# Editor04 Release preview admission completion

Normal preview publication previously called `complete_refresh` only inside
`debug_assert!`. Release builds omitted the call, leaving the published UUID's
admission occupied. The real job then cleared `admission_armed`, so its drop
guard did not release it either. Ready and Error publications could eventually
fill all 64 slots and prevent subsequent preview requests from being admitted.

The production fix evaluates `complete_refresh` before the debug assertion and
asserts its returned value. Catalog publication, token checks, cancellation,
drop cleanup and change-stream ordering stay in their existing positions.

The new sibling `admission_tests.rs` submits the actual
`PreviewRefreshEditorJob` through the existing `test_job_system`. A complete
66-record catalog fixture first occupies 64 scheduler slots and rejects a
waiting dirty UUID. One real job then generates a placeholder thumbnail or
fails to decode an intentionally invalid image. Both cases must publish
`PreviewChanged`, advance only the publish epoch, project Ready/Error into the
catalog and metadata, release the completed token and preserve the other 63
tokens. The clean completed UUID stays unadmitted; the waiting UUID refills the
free slot, and a further request is rejected by the 64-slot cap. The job result
has a 30-second deadline; timeout requests cancellation and fails the test.
Temporary project cleanup is best effort.

The two ordinary `editor04_preview_publication_` regressions use the same path
in Debug and Release. The ignored Release-only
`editor04_preview_release_admission_refill_diagnostic` executes both cases and
prints `EDITOR04_PREVIEW_RELEASE_ADMISSION_REFILL_V1` after their assertions,
including OS, architecture and package version. A Debug result cannot prove
this Release repair; the managed Release batch must run the diagnostic or the
two ordinary tests with debug assertions disabled.

Static formatting, scoped diff and record checks are the local evidence.
Managed Cargo, the real Release job result and product acceptance remain
pending. This is a correctness and admission-liveness repair, with no timing
benchmark or claimed frame, memory, native present or million-asset gate.
