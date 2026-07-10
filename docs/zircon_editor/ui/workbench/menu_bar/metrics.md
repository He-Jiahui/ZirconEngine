---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/src/ui/workbench/menu_bar/mod.rs
  - zircon_editor/src/ui/workbench/menu_bar/metrics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/support.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs
implementation_files:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/src/ui/workbench/menu_bar/mod.rs
  - zircon_editor/src/ui/workbench/menu_bar/metrics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - rustfmt --edition 2021 --check zircon_editor/src/ui/workbench/menu_bar/mod.rs zircon_editor/src/ui/workbench/menu_bar/metrics.rs zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs zircon_editor/src/tests/host/retained_menu_pointer/layout.rs zircon_editor/src/tests/host/retained_menu_pointer/support.rs
  - static scan: menu touched paths contain no `chars().count()`, `TITLE_WIDTH_PER_CHAR`, `WIDTH_PER_CHAR`, or `* 7.0` text-width heuristics
  - docs/tests/runtime/text/runtime_text_editor_menu_bar_runtime_measure_preview_20260704.png
  - docs/tests/runtime/text/runtime_text_editor_menu_bar_runtime_measure_validation_20260704.log
  - cargo test -p zircon_editor --lib runtime_font_width --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-menu-0704 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-04: timed out after 15m without returned Rust diagnostics; matching validation processes stopped)
doc_type: module-detail
status: implemented-focused-and-visual-passed
---

# Workbench Menu Bar Metrics

`workbench/menu_bar/metrics.rs` is the shared geometry owner for top-level Workbench menu button widths. It keeps the menu title font size, minimum slot width, maximum slot width, and chrome reserve in one place so visual menu chrome and retained pointer hitboxes do not drift.

This module does not own menu contents, command dispatch, popup layout, or native painting. It only converts a runtime-measured label width into the slot width that the menu bar and pointer bridge both consume.

## Current Contract

- `WORKBENCH_MENU_SLOT_FONT_SIZE` projects `EditorTypographyTokens::WORKBENCH_BODY_SIZE`. Unreal Normal 10-point text is converted once to 13.33 logical pixels at the 96-DPI Slate baseline, so menu measurement no longer owns a separate 12-pixel default.
- `workbench_menu_slot_width_from_label_width(...)` accepts a width that has already been measured by the retained-host runtime text path and adds the shared menu chrome reserve.
- The helper clamps the result between 40 px and 128 px, preserving the prior compact menu affordance while removing character-count width estimation.
- Non-finite and negative measured widths fail closed to the minimum slot width.

## Consumers

`chrome_template_projection/menu_chrome.rs` measures each live menu label with `measure_runtime_text_width(..., WORKBENCH_MENU_SLOT_FONT_SIZE)`, then calls this owner to assign each `MenuSlot*` frame. The authored ZUI stencil still supplies y/height and the inter-slot gap, but the live label width now comes from actual glyph measurement.

`retained_host/menu_pointer/build_host_menu_pointer_layout.rs` uses the same measurement and width helper when it builds top-level menu button hitboxes. This keeps painted menu slots, popup anchors, scroll content width, and pointer routes aligned for labels with the same character count but different glyph widths, such as `iiiiiiii` and `WWWWWWWW`.

## Test Coverage

The focused helper test covers min, max, and non-finite clamps. Chrome projection tests assert that visual `MenuSlot*` frames use runtime glyph widths and that equally long narrow/wide labels no longer collapse to the same slot width. Retained menu-pointer tests assert the shared hitboxes use the same runtime widths as visual chrome.

The 2026-07-04 proof image `docs/tests/runtime/text/runtime_text_editor_menu_bar_runtime_measure_preview_20260704.png` compares runtime-measured menu slots against the removed character-count reference. The proof log records the measured widths and confirms the artifacts were not written to repo `target` or external Cargo target directories.

The 2026-07-10 unit correction adds `menu_slot_typography_uses_workbench_body_role` as a direct token-projection guard. Scoped rustfmt, the selected chrome scan, and the focused test pass. The refreshed 640/900/1260 and 1672 captures live under `docs/tests/editor`; the 1672 image still keeps composite toolbar spacing as an open layout concern rather than a typography failure.
