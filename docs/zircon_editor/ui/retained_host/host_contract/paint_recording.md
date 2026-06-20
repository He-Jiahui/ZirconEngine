---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording/damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording/record.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording/damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording/record.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - paint-recording record/damage ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Paint Recording

`paint_recording.rs` is the retained-host bridge from legacy Workbench paint calls into recorded paint commands used by `chrome_command_stream/extraction.rs`. It is now a structural entry that only re-exports the production recording entry.

`paint_recording/record.rs` owns `record_host_frame_commands(...)`: zero-size guard, recording-only frame construction, shell background fill, workbench command drawing, and recorded-command extraction. `paint_recording/damage.rs` owns frame-bounds construction plus damage-to-frame clipping and local visibility/intersection checks.

The 2026-06-20 record/damage split reduced `paint_recording.rs` from 52 lines to a 3-line structural entry. `damage.rs` is 36 lines and owns frame bounds plus damage clipping; `record.rs` is 27 lines and owns the recording flow. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming data/frame/theme/workbench imports, shell-background constant, recording body, frame intersection, and visible-frame helper no longer live in `paint_recording.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.
