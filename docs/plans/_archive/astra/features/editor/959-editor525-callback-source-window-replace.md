---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor525-callback-source-window-replace.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs
tests:
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs
---

# Editor959 Editor525 callback source-window owner move

The callback source-window scope now moves the previous `MainPageId` with
`std::mem::replace`, then restores it after the callback. Nested visibility and
restoration semantics remain unchanged while the previous owned identifier is
no longer cloned. Marker `EDITOR525_CALLBACK_SOURCE_WINDOW_REPLACE_BENCH_V1`
models 65,536 rounds and reports zero optimized previous-owner clones. The
current source hash is `c7a46523f90efbe588cf6a97be21dfec1f86af418df6051528f6aa85a59774f2`.
Managed Editor Release validation remains pending.
