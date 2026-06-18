---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 continue editor UI architecture implementation
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - paint-frame root ownership scan
  - touched-file whitespace scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Paint Frame

`paint_frame.rs` owns the neutral retained-host RGBA frame state used by software replay, snapshot capture, and the temporary recording path that feeds `chrome_command_stream`. The root now keeps the `HostRgbaFrame` storage fields, constructors, paint-clip accessor, dimensions, byte accessors, recorded-command type re-exports, and child-module declarations.

`paint_frame/geometry.rs` owns visible-frame validation and clip intersection. `paint_frame/pixels.rs` owns rectangle fill, pixel-rect conversion, and span/channel writes. `paint_frame/recording.rs` owns the retained paint recording DTOs and internal command accumulator. `paint_frame/recording_frame.rs` owns the `HostRgbaFrame` recording-facing methods for quad, border, text, image, record-only state, and recorded command extraction. `paint_frame/tests.rs` owns the moved fill/clip/recording regressions.

The recording model remains a retained-host compatibility layer. `paint_recording.rs` builds a recording-only `HostRgbaFrame`, Workbench paint code records quads, borders, text, and image commands into it, and `chrome_command_stream/extraction.rs` converts those commands into neutral chrome commands for software replay and runtime draw-list projection.

The 2026-06-18 recording responsibility split reduced `paint_frame.rs` from 393 lines to 332 and created `paint_frame/recording.rs`. The follow-up 2026-06-18 paint-frame subtree split reduced `paint_frame.rs` further from 370 lines to 94 by moving geometry, pixel fill, recording methods, and inline tests into child modules. Current line counts are `geometry.rs` 32, `pixels.rs` 65, `recording.rs` 96, `recording_frame.rs` 120, and `tests.rs` 74.

Validation remained feature-first per the user's request. `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, the paint-frame root ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage.
