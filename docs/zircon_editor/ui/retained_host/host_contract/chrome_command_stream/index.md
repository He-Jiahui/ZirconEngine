---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/command/kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/command/layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/images.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/shapes.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/shapes/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/shapes/border/rect.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/shapes/quad.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/text_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stats.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/extend.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/shape.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/extraction.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/stream_model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/atlas_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/tests.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/command/kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/command/layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/images.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/shapes.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/shapes/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/shapes/border/rect.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay/commands/shapes/quad.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/text_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stats.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/extend.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/shape.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream/push/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/extraction.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/stream_model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/atlas_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - user: 2026-06-17 continue editor UI architecture implementation
tests:
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/extraction.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/stream_model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/atlas_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/tests.rs
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-17 after chrome command-stream neutralization: passed with existing warning noise only)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-17 after runtime draw-list projection ownership split: passed with existing warning noise only)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check (2026-06-17 after chrome command-stream stats responsibility split: passed)
  - chrome_command_stream source scan for stats ownership (2026-06-17 after stats responsibility split: clean; stats aggregation lives in stats.rs and test assertions remain in tests.rs)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-17 after chrome command-stream stats responsibility split: passed with existing warning noise only)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check (2026-06-17 after chrome command-stream software replay responsibility split: passed)
  - chrome_command_stream source scan for replay ownership (2026-06-17 after replay responsibility split: clean; replay implementation lives in replay.rs and mod.rs keeps only the re-export)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-17 after chrome command-stream software replay responsibility split: passed with existing warning noise only)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check (2026-06-17 after chrome command-stream atlas sampling responsibility split: passed)
  - chrome_command_stream source scan for atlas sampling ownership (2026-06-17 after atlas responsibility split: clean; atlas_subimage_rgba and atlas_uv_pixel_rect live in atlas.rs)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-17 after chrome command-stream atlas sampling responsibility split: passed with existing warning noise only)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check (2026-06-17 after chrome command-stream stream model responsibility split: passed)
  - chrome_command_stream source scan for stream model ownership (2026-06-17 after stream responsibility split: clean; stream types and push/accessor methods live in stream.rs)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-17 after chrome command-stream stream model responsibility split: passed with existing warning noise only)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; paint-frame recording ownership scan; touched-file whitespace scan; scoped git diff --check; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after paint-frame recording responsibility split: passed with existing warning noise only)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; chrome command DTO ownership scan; touched-file whitespace scan; scoped git diff --check; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after chrome command DTO responsibility split: passed with existing warning noise only)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; runtime draw-list/GPU presenter test ownership scan; touched-file whitespace scan; scoped git diff --check; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after boundary test responsibility split: passed with existing warning noise only; cargo check -p zircon_editor --lib --tests remains blocked by pre-existing unrelated lib-test errors under render_framework_boundary, paint_template_nodes test imports, profiling_artifacts tests, and retained_tab_drag tests)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; chrome command-stream test-tree ownership scan; touched-file whitespace scan; scoped git diff --check; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after command-stream test tree responsibility split: passed with existing warning noise only)
  - chrome_command_stream replay command/image/shape ownership scan (2026-06-21 after replay command/image/shape split: clean; replay.rs keeps region replay and sorting, commands.rs keeps command-kind dispatch, images.rs keeps image fallback/atlas draw, shapes.rs keeps quad/border draw)
  - chrome_command_stream runtime draw-list command/text-style/geometry ownership scan (2026-06-21 after runtime draw-list projection split: clean; runtime_draw_list.rs keeps stream-to-list orchestration, command.rs keeps command-kind projection, text_style.rs keeps text style mapping, geometry.rs keeps rect/UV conversion)
  - chrome command-stream replay shape quad/border ownership scan (2026-06-21 after replay shape split: clean; shapes.rs keeps structural exports, shapes/quad.rs owns quad replay, shapes/border.rs owns rounded and rectangular border replay)
  - chrome command-stream stream model/push/geometry ownership scan (2026-06-21 after stream model/push/geometry split: clean; stream.rs keeps structural exports, model.rs owns state/accessors, push.rs owns command construction, geometry.rs owns size/frame validation)
  - chrome command-stream stream push command-family ownership scan (2026-06-21 after push command-family split: clean; push.rs keeps structural child declarations, push/shape.rs owns quad/border, push/text.rs owns text, push/image.rs owns image, push/clip.rs owns clip, push/extend.rs owns bulk extension, and push/command.rs owns visible-frame gated insertion)
  - chrome command-stream extraction entry/command/image/model/visibility ownership scan (2026-06-21 after extraction split: clean; extraction.rs keeps structural exports, entry.rs owns recording extraction, command.rs owns recorded-kind conversion, image.rs owns image payload conversion, model.rs owns extraction DTO, visibility.rs owns frame filtering)
  - chrome command-stream extraction command kind/layer ownership scan
  - chrome command-stream replay border rect ownership scan
doc_type: module-detail
---

# Chrome Command Stream

## Purpose

`chrome_command_stream` is the neutral retained-host command stream for editor chrome while the
Workbench shell is being moved toward runtime UI extract and GPU command ownership. It owns
`ChromeCommandStream`, `ChromeCommand`, image payload metadata, damage clipping, software replay,
the conversion from recorded paint commands into chrome commands, and the projection from chrome
commands into runtime `UiSurfaceDrawList`.

This module is deliberately not a presenter backend. `presenter/snapshot.rs`, `presenter/softbuffer.rs`,
`presenter/softbuffer/diagnostics.rs`, `presenter/softbuffer/surface_io.rs`, and `presenter/gpu.rs`
consume the stream; they no longer own the stream type, extraction logic, or runtime draw-list projection.

## Related Files

`mod.rs` declares the child modules, re-exports the public internal stream helpers, and owns the
`build_chrome_command_stream(...)` assembly entry. `command.rs` defines `ChromeCommand`,
`ChromeCommandKind`, `ChromeCommandLayer`, and the chrome image payload/UV DTOs. `stream.rs`
is now the structural stream entry: `stream/model.rs` defines `ChromeCommandStream`,
construction, stored command access, and stream metadata; `stream/push.rs` is the structural push
entry; and `stream/geometry.rs` owns surface-size clamping plus visible-frame validation. The push
children keep command construction separated by family: `push/shape.rs` owns quad and border
commands, `push/text.rs` owns text commands and line-height/style defaults, `push/image.rs` owns
viewport image commands, `push/clip.rs` owns clip commands, `push/extend.rs` owns bulk command
extension, and `push/command.rs` owns the shared visible-frame gated insertion path. `replay.rs`
owns software replay entry points, command ordering, and region repaint helpers.
`replay/commands.rs` owns command-kind dispatch, `replay/commands/images.rs` owns image fallback
and atlas-backed image draw, and `replay/commands/shapes.rs` is now a structural shape replay entry:
`shapes/quad.rs` owns quad replay, while `shapes/border.rs` owns rounded-border versus rectangular-border
dispatch and `shapes/border/rect.rs` owns rectangular border segment geometry and paint.
`atlas.rs` owns atlas UV validation and RGBA subimage sampling for software replay. `stats.rs` owns
`ChromeCommandStreamStats` plus image upload and draw-call aggregation. `extraction.rs` is now the
structural extraction entry: `extraction/entry.rs` owns the bridge from
`paint_recording::record_host_frame_commands(...)` to extraction output, `extraction/command.rs`
is now the structural recorded-command conversion entry. `extraction/command/layer.rs` owns recorded-paint to chrome layer selection, `extraction/command/kind.rs` owns recorded-paint kind to chrome-command kind conversion, `extraction/image.rs` owns chrome image
payload and atlas UV conversion, `extraction/model.rs` owns `ChromeCommandExtraction`, and
`extraction/visibility.rs` owns recorded-frame filtering.
`runtime_draw_list.rs` is now the structural runtime draw-list projection entry. It converts the
neutral chrome stream into runtime `UiSurfaceDrawList` commands for GPU presentation, while
`runtime_draw_list/command.rs` owns per-command `UiSurfaceCommandKind` projection,
`runtime_draw_list/text_style.rs` owns retained text-style mapping, and
`runtime_draw_list/geometry.rs` owns frame rectangle and image UV conversion.

The old `host_contract/presenter/command_stream.rs`,
`host_contract/presenter/command_stream/`, and `host_contract/presenter/extraction.rs` paths are
deleted. Current tests live beside the neutral module: `tests.rs` is the structural entry,
`tests/support.rs` owns shared fixtures and pixel helpers, `tests/stream_model.rs` owns stream
construction/stat assertions, `tests/extraction.rs` owns recorded-paint conversion assertions, and
`tests/replay.rs` owns software replay parity assertions.

## Behavior Model

The stream has two update modes:

- Full rebuild: all visible recorded paint commands become static, text, or viewport commands.
- Patch: damage is clipped before extraction, and non-text/non-image commands are treated as dynamic.

Images carry a stable `resource_key`, size, upload byte accounting, optional CPU RGBA bytes, and
optional atlas UVs. `include_image_bytes` controls whether CPU bytes are retained for software
snapshot/replay callers or omitted for upload-oriented paths.
`ChromeCommandStream::stats()` reports layer counts, draw-call count, clip count, image command
count, and unique upload bytes without making `mod.rs` own the accounting details.

The 2026-06-18 command DTO split reduced `stream.rs` from 245 lines to 172 and created
`command.rs` at 54 lines. Validation confirmed command layers, command kinds, image payloads, image
UVs, and command records no longer live in the stream container file; `stream.rs` now owns only
stream state, construction, clamping, push helpers, and accessors. Full Cargo test expansion remains
deferred to the milestone testing stage per the current feature-first cadence.

The 2026-06-18 boundary test split moved runtime draw-list projection regressions into
`runtime_draw_list/tests.rs` and GPU presenter stream/diagnostic regressions into
`presenter/gpu/tests.rs`. Production files now keep only the projection/submission logic and a
`#[cfg(test)] mod tests;` hook. `runtime_draw_list.rs` is 95 lines, `runtime_draw_list/tests.rs` is
94 lines, `presenter/gpu.rs` is 166 lines, and `presenter/gpu/tests.rs` is 209 lines.

The 2026-06-21 runtime draw-list command/text-style/geometry split reduced
`chrome_command_stream/runtime_draw_list.rs` from 95 lines to a 26-line stream-to-list orchestration
entry. `runtime_draw_list/command.rs` owns command-kind projection, `text_style.rs` owns text style
mapping, and `geometry.rs` owns frame rectangle plus image UV conversion. Validation used
`cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a runtime draw-list
command/text-style/geometry ownership scan, scoped trailing-whitespace scan, and scoped
`git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's
feature-first instruction.

The 2026-06-18 command-stream test-tree split converted the previous 648-line
`chrome_command_stream/tests.rs` into a folder-backed test subtree. The root `tests.rs` now only
declares child modules, while `tests/support.rs` is 280 lines, `tests/stream_model.rs` is 141 lines,
`tests/extraction.rs` is 64 lines, and `tests/replay.rs` is 191 lines. This keeps test ownership
parallel to the production responsibilities instead of using a single regression bucket.

The 2026-06-21 replay command/image/shape split reduced `chrome_command_stream/replay.rs` from 203
lines to 31 lines. `replay.rs` now owns only software replay entry points, stable z ordering, and
damage-scoped repaint orchestration; `replay/commands.rs` is a 38-line command-kind dispatch owner;
`replay/commands/images.rs` owns fallback image fill, atlas subimage sampling handoff, and
resource-key image replay; `replay/commands/shapes.rs` owns quad, rounded-border, and rectangular
border segment drawing. Validation used `cargo fmt -p zircon_editor`,
`cargo fmt -p zircon_editor --check`, a chrome command-stream replay command owner scan, scoped
whitespace scan, and scoped `git diff --check`; package-level
`cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` was
attempted and timed out after 300 seconds before producing actionable editor diagnostics.

The 2026-06-21 replay shape quad/border split reduced `replay/commands/shapes.rs` from 110 lines
to a 4-line structural entry. `shapes/quad.rs` owns quad and rounded-quad replay, while
`shapes/border.rs` owns rounded-border versus rectangular-border dispatch, and
`shapes/border/rect.rs` owns rectangular border segment geometry and paint.
Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a chrome
command-stream replay shape quad/border ownership scan, scoped trailing-whitespace scan, and scoped
`git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's
feature-first instruction.

The 2026-06-21 replay border rect split reduced `replay/commands/shapes/border.rs` from 97 lines
to a 25-line focused rounded/rect dispatch entry. `shapes/border/rect.rs` owns rectangular border
width normalization, segment iteration, segment frame geometry, and clipped rect paint. Validation
used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a chrome command-stream
replay border rect ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`;
package-level Cargo check and full Cargo tests remain deferred per the user's feature-first
instruction.

The 2026-06-21 stream model/push/geometry split reduced `chrome_command_stream/stream.rs` from
172 lines to a 5-line structural entry. `stream/model.rs` is 45 lines and owns stream state,
constructors, accessors, and stored command exposure; `stream/push.rs` is 128 lines and owns quad,
border, text, image, clip, extend, and internal command construction; `stream/geometry.rs` is 14
lines and owns surface-size clamping plus visible-frame validation. Validation used
`cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a chrome command-stream stream
model/push/geometry ownership scan, scoped whitespace scan, and scoped `git diff --check`.
Package-level Cargo check remains covered by the earlier 2026-06-21 timeout before actionable
editor diagnostics, and full Cargo test matrix remains deferred to the milestone validation stage
per the user's instruction.

The 2026-06-21 stream push command-family split reduced `chrome_command_stream/stream/push.rs`
from 137 lines to a 6-line structural entry. The new child owners are `push/shape.rs` at 50 lines,
`push/text.rs` at 32 lines, `push/image.rs` at 22 lines, `push/clip.rs` at 21 lines,
`push/extend.rs` at 11 lines, and `push/command.rs` at 27 lines. The existing
`ChromeCommandStream::push_*` method names stay attached to `ChromeCommandStream`; only their
implementation ownership moved. Validation used `cargo fmt -p zircon_editor`,
`cargo fmt -p zircon_editor --check`, a stream push command-family ownership scan, scoped
trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo
tests remain deferred to the milestone validation stage per the user's instruction.

The 2026-06-21 extraction entry/command/image/model/visibility split reduced
`chrome_command_stream/extraction.rs` from 146 lines to a 9-line structural entry.
`extraction/entry.rs` is 24 lines and owns recorded command extraction and damage return,
`extraction/command.rs` is 94 lines and owns layer/kind conversion, `extraction/image.rs` is 47
lines and owns payload/upload-byte/atlas UV conversion, `extraction/model.rs` is 6 lines and owns
the extraction DTO, and `extraction/visibility.rs` is 9 lines and owns visible-frame filtering.
Validation used `cargo fmt -p zircon_editor --check`, a chrome command-stream extraction
entry/command/image/model/visibility ownership scan, scoped whitespace scan, and scoped
`git diff --check`. The first `cargo fmt -p zircon_editor` run timed out at 120 seconds after
formatting the touched files; package-level Cargo check remains covered by the earlier 2026-06-21
timeout before actionable editor diagnostics, and full Cargo test matrix remains deferred to the
milestone validation stage per the user's instruction.

The 2026-06-21 extraction command kind/layer split reduced `extraction/command.rs` from 98 lines
to a 36-line structural recorded-command conversion entry. `extraction/command/kind.rs` owns
recorded paint kind to `ChromeCommandKind` conversion and image payload delegation, while
`extraction/command/layer.rs` owns recorded paint kind to `ChromeCommandLayer` selection.
Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a chrome
command-stream extraction command kind/layer ownership scan, scoped trailing-whitespace scan, and
scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the
user's feature-first instruction.

## Flow

`paint_recording` records Workbench chrome into `HostRecordedPaintCommand` values owned by
`paint_frame/recording.rs`. The extraction subtree filters invisible frames, classifies command
layers, converts text/image/quad/border payloads, and returns clipped damage. `mod.rs` stores the
resulting command list. `replay.rs` can replay the full
stream into `HostRgbaFrame` or repaint a damaged region from those stored commands, while its child
command owners emit the concrete software replay primitives. `stats.rs` derives command accounting
from the same stored commands. `runtime_draw_list.rs` maps the same stream to runtime
`UiSurfaceDrawList` values, preserving corner radius, text style, image upload metadata, atlas UVs,
clip commands, damage, and surface size through its command/text-style/geometry children.
`atlas.rs` is deliberately kept below replay because it is a software fallback sampler, not the
runtime GPU atlas contract.

The GPU presenter now asks this module for a runtime draw list and then only handles surface
submission, cache bootstrap, damage diagnostics, and profiling counters.

## Edge Cases

Frames with non-finite or non-positive extents are dropped during extraction. Atlas image commands
preserve UV coordinates even when CPU bytes are omitted. Missing image bytes in software replay use
the stream fallback image color rather than making the frame blank.

## Test Coverage

The module-level tests cover replay parity against the test-only direct painter, damage clipping,
command ordering, stats, image upload accounting, atlas UV propagation, and runtime draw-list
projection of rounded geometry and atlas payloads. The test tree now separates stream-model,
extraction, replay, atlas, runtime draw-list, and GPU presenter coverage. GPU presenter tests cover
runtime surface failure propagation, upload/draw-call diagnostics, damage patch submission after
cache bootstrap, and resize invalidation. Full focused and real-window validation remains deferred
for this feature-first pass per the current user direction.

## Open Issues

`chrome_command_stream` is still a retained-host compatibility bridge. The remaining architecture
work is to shrink this module and `paint_template_nodes/` toward runtime extract and GPU command
stream ownership, then delete the temporary software recording path when the editor shell no longer
needs it. The presenter side should stay limited to concrete backend lifecycle and diagnostics.
