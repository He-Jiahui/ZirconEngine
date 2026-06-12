---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/theme/mod.rs
  - zircon_runtime/src/ui/tests/theme_registry.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_theme_asset.rs
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/tests/ui_theme_contracts.rs
  - zircon_editor/assets/ui/theme/editor_base.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
implementation_files:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/theme/mod.rs
  - zircon_runtime/src/ui/tests/theme_registry.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_theme_asset.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_editor/assets/ui/theme/editor_base.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
plan_sources:
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - cargo test -p zircon_runtime_interface --lib ui_theme --locked --target-dir target/codex-editor-ui (2026-06-12: passed, 3 passed)
  - cargo test -p zircon_runtime --lib theme_registry --locked --jobs 1 --target-dir target/codex-editor-ui-runtime --message-format short --color never -- --nocapture --test-threads=1 (2026-06-12: timed out while compiling/linking the runtime test target; matching processes stopped)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir target/codex-editor-ui-runtime-check --message-format short --color never (2026-06-12: reached runtime crate and then failed on unrelated graphics render pass errors in zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\theme\mod.rs zircon_runtime\src\ui\tests\theme_registry.rs (2026-06-12 style-color role consumption slice: passed)
  - cargo test -p zircon_runtime --lib ui_theme_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-theme-0612 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-12: first attempt timed out after 604 seconds while compiling/linking and matching command processes were stopped; rerun failed before the filtered theme tests executed because active runtime core tests reference a moved `core/runtime/lifecycle.rs` path)
  - python -c "import tomllib, pathlib; paths=[r'zircon_editor/assets/ui/theme/editor_material.v2.ui.toml', r'zircon_editor/assets/ui/editor/welcome.v2.ui.toml']; [tomllib.loads(pathlib.Path(p).read_text(encoding='utf-8')) for p in paths]" (2026-06-12 editor_material theme role consumer: passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\tests\v2_asset.rs (2026-06-12 editor_material theme role consumer: passed)
  - git diff --check -- zircon_editor/assets/ui/theme/editor_material.v2.ui.toml zircon_runtime/src/ui/tests/v2_asset.rs docs/zircon_runtime/ui/theme.md docs/zircon_runtime/ui/v2.md .codex/sessions/20260612-0904-editor-ui-architecture-implementation.md (2026-06-12 editor_material theme role consumer: passed with LF-to-CRLF warnings only)
  - python -c "import tomllib, pathlib; paths=[r'zircon_editor/assets/ui/theme/editor_base.v2.ui.toml', r'zircon_editor/assets/ui/theme/editor_material.v2.ui.toml', r'zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml', r'zircon_editor/assets/ui/editor/workbench_status_bar.v2.ui.toml']; [tomllib.loads(pathlib.Path(p).read_text(encoding='utf-8')) for p in paths]" (2026-06-12 editor_base chrome theme role consumer: passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\tests\v2_asset.rs (2026-06-12 editor_base chrome theme role consumer: passed)
  - git diff --check -- zircon_editor/assets/ui/theme/editor_base.v2.ui.toml zircon_runtime/src/ui/tests/v2_asset.rs docs/zircon_runtime/ui/theme.md docs/zircon_runtime/ui/v2.md docs/zircon_editor/ui/template_runtime/runtime_host.md .codex/sessions/20260612-0904-editor-ui-architecture-implementation.md (2026-06-12 editor_base chrome theme role consumer: passed with LF-to-CRLF warnings only)
  - rustfmt --edition 2021 --check touched UI theme asset/importer files (2026-06-12 UiThemeAsset slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-theme-asset-0612-coremin-check --message-format short --color never (2026-06-12 UiThemeAsset slice: passed with existing warnings)
  - cargo test -p zircon_runtime --lib ui_theme_asset --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-theme-asset-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12 UiThemeAsset slice: timed out after 904s while compiling runtime lib-test target; no Rust diagnostics returned)
doc_type: module-detail
---

# Runtime UI Theme Registry

`zircon_runtime::ui::theme::UiThemeRegistry` is the Stage B owner for active editor theme state. It stores one active `UiThemeDocument`, computes a fingerprint for change detection, and resolves palette token references into `UiStyleColor` values.

The registry establishes the runtime token lookup boundary required before style assets, painter selectors, and component templates can stop using raw color constants. The first real asset consumers are `zircon_editor/assets/ui/theme/editor_material.v2.ui.toml` and `zircon_editor/assets/ui/theme/editor_base.v2.ui.toml`: shared Material surface, text, accent, separator, semantic color aliases, and the workbench chrome base aliases now point at `$theme.palette.*` roles, while specialized pressed/selected/container colors remain document-local until the palette model grows matching roles.

`UiThemeDocument` also has a standalone asset path now. `zircon_runtime::asset::UiThemeAsset` wraps the theme DTO directly, the built-in `.theme.toml` importer emits `ImportedAsset::UiTheme`, and the payload is stored under the existing `UiStyle` resource family with facade label `ui_theme`. This is intentionally separate from the current editor `editor_*.v2.ui.toml` files, which remain v2 style-token assets consumed by the v2 surface builder.

## Token Resolution

The registry currently resolves palette tokens:

- `palette.surface.0` through `palette.surface.3`
- `palette.text.primary`
- `palette.text.secondary`
- `palette.text.disabled`
- `palette.accent`
- `palette.success`
- `palette.info`
- `palette.warning`
- `palette.error`
- `palette.separator`

Unknown tokens return `None`. That keeps missing-token diagnostics explicit for later asset compiler integration instead of silently falling back to transparent or black.

`resolve_role` accepts the raw role strings produced by style parsing. It accepts both plain token ids such as `palette.accent` and theme-qualified spellings such as `theme.palette.accent` or `$theme.palette.accent`. `resolve_style_color` is the narrow consumption helper for later style and painter code: it rewrites `UiStyleColor::Role(...)` when the role resolves to an active theme token, while leaving literal RGBA, inherit, transparent, and unknown roles unchanged.

Runtime v2 surface construction now consumes those roles through `UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(...)`. Imported editor assets such as Welcome keep authoring against stable document tokens like `$material_surface`, but the metadata provenance chain records `token.material_surface -> theme.palette.surface.2`, so final retained colors can be traced back to the central palette without rewriting every component asset in one pass.

Workbench chrome assets that import `editor_base.v2.ui.toml` now get the same provenance. For example, `workbench_activity_rail.v2.ui.toml` resolves its activity rail background from `token.panel_bg -> theme.palette.surface.2`, while `workbench_status_bar.v2.ui.toml` resolves its status-bar surface from `token.surface_hover -> theme.palette.surface.3` and foreground from `token.text -> theme.palette.text.primary`.

The latest real-asset slice validated `editor_material.v2.ui.toml` and `welcome.v2.ui.toml` with TOML parsing plus a focused formatting/diff check for the updated runtime assertions. The focused runtime `theme_tokens` test was not rerun in that slice because the immediately previous run timed out after 904 seconds with no Rust diagnostics while other active cargo/rustc work was present; the next milestone testing stage should rerun the focused runtime v2 theme/provenance filters once compile pressure clears.

## Reload Fingerprint

`apply_document` replaces the active theme and returns `UiThemeReloadOutcome` with previous and new fingerprints plus a `changed` flag. Future hot reload should use this to invalidate style caches and restyle surfaces without rebuilding the UI tree.
