---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands/record.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands/shapes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands/record.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands/shapes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/commands/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 continue editor UI architecture implementation
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - paint-frame root ownership scan
  - paint-frame frame storage ownership scan
  - paint-frame recording model/state ownership scan
  - paint-frame recording-frame command/state ownership scan
  - paint-frame recording-frame command shape/text/image/record ownership scan
  - touched-file whitespace scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Paint Frame

`paint_frame.rs` is the neutral retained-host RGBA frame module entry used by software replay, snapshot capture, and the temporary recording path that feeds `chrome_command_stream`. The root now keeps only child-module declarations plus `HostRgbaFrame` and recorded-command type re-exports.

`paint_frame/frame.rs` owns `HostRgbaFrame` storage fields, constructors, paint-clip accessor, dimensions, and byte accessors. `paint_frame/geometry.rs` owns visible-frame validation and clip intersection. `paint_frame/pixels.rs` owns rectangle fill, pixel-rect conversion, and span/channel writes. `paint_frame/recording.rs` is now a structural retained paint recording entry; `recording/model.rs` owns image UV/atlas, recorded paint kind, and recorded command DTOs, while `recording/state.rs` owns the command accumulator, record-only flag, z-index sequencing, and visible-frame filtering. `paint_frame/recording_frame.rs` is now a structural recording-facing extension entry; `recording_frame/commands.rs` is a structural command entry, `commands/shapes.rs` owns quad/border command recording, `commands/text.rs` owns text command recording, `commands/image.rs` owns image command recording, and `commands/record.rs` owns the shared recording handoff helper. `recording_frame/state.rs` owns record-only state and recorded command extraction. `paint_frame/tests.rs` owns the moved fill/clip/recording regressions.

The recording model remains a retained-host compatibility layer. `paint_recording.rs` builds a recording-only `HostRgbaFrame`, Workbench paint code records quads, borders, text, and image commands into it, and `chrome_command_stream/extraction.rs` converts those commands into neutral chrome commands for software replay and runtime draw-list projection.

The 2026-06-18 recording responsibility split reduced `paint_frame.rs` from 393 lines to 332 and created `paint_frame/recording.rs`. The follow-up 2026-06-18 paint-frame subtree split reduced `paint_frame.rs` further from 370 lines to 94 by moving geometry, pixel fill, recording methods, and inline tests into child modules. Current line counts are `geometry.rs` 32, `pixels.rs` 65, `recording.rs` 96, `recording_frame.rs` 120, and `tests.rs` 74.

Validation remained feature-first per the user's request. `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, the paint-frame root ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage.

The 2026-06-20 frame storage split reduced `paint_frame.rs` from 80 lines to an 11-line structural entry. `paint_frame/frame.rs` is 72 lines and owns `HostRgbaFrame` storage, constructors, paint clip replacement/access, dimensions, byte access, and byte extraction. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `FrameRect`, `HostPaintRecording`, struct fields, constructors, accessors, and byte methods no longer live in `paint_frame.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.

The 2026-06-21 recording model/state split reduced `paint_frame/recording.rs` from 98 lines to a 7-line structural entry. `recording/model.rs` owns image UV/atlas, recorded paint kind, and recorded command DTOs, while `recording/state.rs` owns `HostPaintRecording`, record-only state, z-index sequencing, visible-frame filtering, and command collection. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-frame recording model/state ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 recording-frame command/state split reduced `paint_frame/recording_frame.rs` from 111 lines to a 2-line structural entry. `recording_frame/commands.rs` owns `record_quad`, `record_border`, `record_text`, `record_image`, and the private command accumulator helper, while `recording_frame/state.rs` owns `is_recording`, `record_only`, and recorded command extraction. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-frame recording-frame command/state ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 recording-frame command shape/text/image/record split reduced `paint_frame/recording_frame/commands.rs` from 100 lines to a 4-line structural entry. `commands/shapes.rs` owns quad/border recording methods, `commands/text.rs` owns text recording, `commands/image.rs` owns image recording, and `commands/record.rs` owns the shared `record_command` helper. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-frame recording-frame command shape/text/image/record ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
