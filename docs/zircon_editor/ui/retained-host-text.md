---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/text_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/tests.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/text_style.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs (2026-07-03 retained-host per-grapheme spacing guard: passed)
  - cargo test -p zircon_editor runtime_positioned_glyphs --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-text-spacing-0703 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 retained-host per-grapheme spacing guard: timed out after 904s with no Rust diagnostics; no Cargo pass claimed; log docs/tests/runtime/text/runtime_text_editor_per_grapheme_spacing_guard_validation_20260703.log SHA256 48AFB99C98A9037F1B51E156257A7A43AD25F09D9E065EAA5E5A3D7FA10AB5C8)
  - docs/tests/runtime/text/runtime_text_editor_per_grapheme_spacing_guard_preview_20260703.png (2026-07-03 retained-host per-grapheme spacing guard visual proof: inspected; SHA256 7991429B66A536B05D538C05947EBAD6FB7D6049EF894F1D6EE4F8E32406A5F8; repo target, E:\cargo-targets, and D:\cargo-targets same-name match count 0; proof image is not a live editor capture)
status: in_progress
doc_type: module-detail
---

# Retained Host Text Bridge

This document tracks the editor retained-host text bridge where retained paint styles become runtime text commands. The bridge must not hardcode component fonts or create an editor-local measurement path. It should reuse retained-host typography preferences and pass the resolved text intent to runtime surface and text owners.

On 2026-07-02, the GPU draw-list bridge started preserving font family and weight. `runtime_draw_list/text_style.rs` maps `UiTextRunPaintStyle` to the retained-host UI/Strong/Mono font face helpers, then writes runtime family and normalized weight into `UiSurfaceCommandKind::Text`. Runtime WGPU text consumes those fields through glyphon attrs.

Validation for this bridge slice passed scoped formatting, runtime/editor library checks, and direct focused runtime/editor binaries. Visual proof is `docs/tests/runtime/text/runtime_text_editor_gpu_draw_list_font_projection_preview_20260702.png`, SHA256 `AE5D6B6847FD676E4876620D15073D320A1BF79561C0CD239B9C2745C0EADFB2`; repo and cargo target scans found no same-name image under target paths. The broader Workbench window font consistency QA, native/SDF paragraph parity, DPI/subpixel behavior, preference UI persistence, and stable face-id reconciliation remain open.

On 2026-07-03, the retained-host layout bridge added a per-grapheme spacing guard for the latest cropped editor-tab complaint. `paint_text/draw/layout.rs` still lets runtime layout own measurement, ellipsis, and advance projection, but it now compares the projected runtime advances against the selected host face's natural grapheme advances before rewriting glyph x positions. When total width looks acceptable but local advances drift, the bridge keeps host natural glyph positions so labels such as `editor base.zui` do not look uneven.

Validation for the per-grapheme guard passed scoped rustfmt. The focused `zircon_editor` Cargo test timed out after 904s without Rust diagnostics, so it remains unaccepted as a Cargo pass. The validation log is `docs/tests/runtime/text/runtime_text_editor_per_grapheme_spacing_guard_validation_20260703.log`, SHA256 `48AFB99C98A9037F1B51E156257A7A43AD25F09D9E065EAA5E5A3D7FA10AB5C8`; the proof image is `docs/tests/runtime/text/runtime_text_editor_per_grapheme_spacing_guard_preview_20260703.png`, SHA256 `7991429B66A536B05D538C05947EBAD6FB7D6049EF894F1D6EE4F8E32406A5F8`, with same-name target scans returning 0. Live Workbench/Asset Browser/Component Atlas typography capture remains open.
