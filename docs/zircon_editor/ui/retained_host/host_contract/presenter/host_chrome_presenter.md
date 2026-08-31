---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - host chrome presenter trait/tests ownership scan
  - scoped trailing-whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Host Chrome Presenter Trait

`presenter/host_chrome_presenter.rs` owns the neutral presenter trait boundary used by native editor windows. The trait exposes resize, ordinary present, interactive native-resize present, and diagnostics-snapshot operations without binding callers to the GPU or softbuffer backend. The native-resize method has a conservative full-present default; concrete backends may override it only when they preserve a frozen transaction snapshot and force an ordinary fresh present after resize reflow commits.

`presenter/host_chrome_presenter/tests.rs` owns the trait-object regression that proves a boxed backend can receive resize/present calls and merge invalidation diagnostics into refresh diagnostics. Keeping this regression outside the trait file keeps the production boundary declarative while preserving test access to private host-contract types.

Concrete backend state remains in `presenter/gpu.rs` and `presenter/softbuffer.rs`. New backend-specific lifecycle, present, diagnostics, or surface I/O behavior belongs in those backend subtrees instead of the trait module.

## Validation Notes

The 2026-06-21 host chrome presenter trait/test split reduced `presenter/host_chrome_presenter.rs` from 77 lines to a 15-line trait boundary. `presenter/host_chrome_presenter/tests.rs` owns the moved boxed-backend regression. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a host chrome presenter trait/tests ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
