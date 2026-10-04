---
related_code:
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/tests/ui_painter_style_contracts.rs
  - zircon_runtime_interface/src/tests/ui_theme_contracts.rs
  - zircon_runtime/src/ui/theme/mod.rs
  - zircon_runtime/src/ui/tests/theme_registry.rs
implementation_files:
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/tests/ui_theme_contracts.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
tests:
  - cargo test -p zircon_runtime_interface --lib ui_theme --locked --target-dir target/codex-editor-ui (2026-06-12: passed, 3 passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir target/codex-editor-ui-runtime-check --message-format short --color never (2026-06-12: blocked by unrelated graphics render pass errors in zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs)
doc_type: module-detail
---

# UI Style Contracts

`zircon_runtime_interface::ui::style` owns shared style DTOs that can cross runtime, editor, and native painter boundaries. The existing painter selector remains the canonical pseudo-state priority table. The 2026-06-12 editor UI architecture slice adds theme token contracts without moving or deleting the existing `style.rs` API.

## Theme Document

`UiThemeDocument` describes the editor theme baseline:

- `palette` contains four surface layers, text colors, accent, semantic colors, and separators.
- `typography` stores named text variants such as body, caption, and title.
- `shape` stores standard radius buckets.
- `spacing` stores spacing steps.
- `control_sizes` stores default, compact, and dense control heights.
- `elevation` stores neutral shadow metrics.

The default document is `zircon.dark`, matching the planned near-black editor chrome with teal accent. Nested theme structs use field-level serde defaults so theme assets can override only the tokens they need while retaining the dark baseline for missing fields.

## Token References

`UiThemeTokenRef` is a transparent string wrapper. It serializes as a plain string, for example `"palette.surface.1"`, so TOML/JSON theme-aware assets can store stable token ids without depending on runtime registry implementation details.

The interface module does not resolve tokens. Runtime resolution lives in `zircon_runtime::ui::theme::UiThemeRegistry`, which keeps token lookup and hot-reload fingerprinting out of the shared DTO crate.

## Painter State Selector

`UiPainterStyleSelector` remains the single priority table for visual state folding. Runtime v2 style resolution now derives explicit `:resolved-*` pseudo states from this selector before matching style rules, so authored `:resolved-pressed` or `:resolved-disabled` rules follow the same priority as native editor painters. Existing raw retained-state selectors such as `:hover`, `:active`, and `:focus` remain available for assets that need boolean state overlays rather than a folded visual state.
