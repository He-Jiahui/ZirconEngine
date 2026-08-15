---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/tests/editor_design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/src/core/settings/mod.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/core/settings/tests/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/typography.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/paint.rs
  - zircon_editor/Cargo.toml
implementation_files:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/src/core/settings/mod.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/core/settings/tests/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/typography.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - 'cargo fmt -p zircon_editor -p zircon_runtime_interface --check'
  - 'cargo check -p zircon_runtime_interface --locked --target-dir D:\cargo-targets\zircon-editor-text-preferences-0702'
  - 'cargo check -p zircon_editor --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-preferences-0702'
  - 'cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-preferences-0702'
  - 'cargo test -p zircon_runtime_interface editor_design_tokens --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-preferences-0702'
  - 'cargo test -p zircon_editor retained_text_font_request_uses_global_preferences --lib --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-preferences-0702'
  - 'cargo test -p zircon_editor asset_browser_utility_tab_label_uses_ui_text_style --lib --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-preferences-0702'
  - 'cargo test -p zircon_editor retained_ui_runtime_family_resolves_from_preferences_without_platform_paths --lib --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-preferences-0702'
  - 'cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --lib --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-preferences-0702 -- --ignored'
  - 'rustfmt zircon_editor/src/core/settings/mod.rs zircon_editor/src/core/settings/defaults.rs zircon_editor/src/core/settings/io.rs zircon_editor/src/core/settings/tests.rs zircon_editor/src/ui/retained_host/host_contract/paint_text/raster/tests.rs'
  - 'cargo check -p zircon_editor --tests --no-default-features --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-preferences-0704 --message-format short --color never'
doc_type: module-detail
---

# Retained Host Text Preferences

## Purpose

The retained editor host must not decide product typography by naming a platform font inside a control painter. Text style now flows from editor design tokens into a host-level preference object, and the font resolver treats concrete font family names as user/theme preferences rather than control code.

This keeps the small-component work aligned with the larger editor layout plan: button labels, utility tabs, lists, status chips, popups, drawers, and future windows all consume the same typography preference route before they are assembled into larger surfaces.

## Data Flow

`EditorDesignTokens::workbench_dark()` owns the default typography tokens:

- `ui_family = "system-ui"`
- `ui_strong_family = "system-ui"`
- `code_family = "monospace"`
- size, weight, and line-height values shared with the theme document

`zircon_editor/assets/ui/editor/theme/editor_tokens.zui` mirrors those defaults for authored theme data. The retained host projects the tokens through `project_host_text_preferences(...)` and installs them with `apply_host_text_preferences(...)` before creating the native window.

`utility_tab_text_role` is part of the same typography token group. It defaults to `ui`, can be changed to `code` through the typed `editor.appearance.design_tokens` user setting, and is projected into `HostTextPreferences` rather than being hardcoded inside the Asset Browser utility-tab component.

`paint_text/font.rs` resolves the current `HostTextPreferences` through `fontdb`. Logical families such as `system-ui` and `monospace` map to system sans/mono queries; explicit user-selected names map to `Family::Name`. Resolution produces a bounded two-generation, three-face snapshot shared by measurement, retained rasterization, and draw-list conversion. When a requested system face is unavailable, the retained host uses face 0 of Runtime Text's packaged `ZirconDefaultComposite-subset.ttc` and publishes its private Runtime fallback alias to both the logical matcher and glyphon's backend database, so CPU retention, Runtime measurement, and GPU drawing address the same bytes without shadowing an explicitly selected system `Fira Mono` face. The global cache retains two snapshots and thread-local lookup keeps only a weak reference, so inactive render threads cannot extend that cache lifetime.

## Runtime Text Boundary

Button label and retained text width measurement call `zircon_runtime::ui::surface::measure_text_size(...)` through the retained-host helper, using the same resolved runtime font family as the retained raster and GPU chrome draw-list paths. Runtime Text discovers system faces first, then registers the same packaged `Fira Mono` face under a private backend alias, so headless measurement can resolve the retained fallback before GPU UI startup loads `res://fonts/default.font.toml` without changing an explicit system-family selection. Each draw-list captures the three resolved face identities once, while a non-text list performs no font capture. This keeps layout measurement and drawing on the runtime text interface instead of reintroducing editor-local character-width estimates or per-command font-cache synchronization.

Asset Browser utility tabs no longer force `UiTextRunPaintStyle::code` in the component. They use the ordinary UI text style by default, so changing the global UI family changes those labels with the rest of the editor shell. Code/mono remains available only through the global `utility_tab_text_role` preference lane, not through a local font-family override.

## Owner Rules

The current owner split is:

- `zircon_runtime_interface/src/ui/design_tokens.rs`: public editor typography token shape and theme projection.
- `paint_theme/typography.rs`: retained-host preference storage and token projection.
- `paint_text/font.rs`: bounded face-set resolution, cache key, Runtime Text family projection, and measurement helper.
- `paint_text/raster.rs`: glyph cache identity includes the resolved font request key.
- `template_buttons/content/metrics.rs`: button label style and measurement consumer.

No root painter branch, compatibility module, old path re-export, or control-specific font family table is introduced by this slice.

## Evidence

The 2026-07-02 verification refreshed the editor screenshots in `docs/tests/editor`, not in any Cargo target directory.

| Artifact | Modified | SHA256 |
| --- | --- | --- |
| `editor-window-m3-asset-browser-900x620.png` | 2026-07-02 03:22:27 +08:00 | `CB35E99D3D049F2455FBB57A9CB104C52563C6EB054A726BAF9E64E5108CDFDF` |
| `editor-window-m3-asset-browser-utility-tabs-ui-preference-crop-20260702.png` | 2026-07-02 03:25:22 +08:00 | `C97A03DAF57EDBA2AF6B71E59FCC6E9862BFF6ACD27A0E41484669BD3355E1ED` |
| `editor-window-m3-asset-browser-utility-tabs-ui-preference-crop-3x-20260702.png` | 2026-07-02 03:25:33 +08:00 | `D4D1B49CDCBA69AF80F5CD3102C48353B2B154AE46D5CE37D45B10CEDCE47ADE` |

The target scan found no matching editor screenshot artifacts under `E:\Git\ZirconEngine\target` or `D:\cargo-targets\zircon-editor-text-preferences-0702`.

The 2026-07-03 S15.4eb follow-up added the global `utility_tab_text_role` route to the runtime-interface token shape, theme document, retained-host projection, and button label style selection. Verification passed touched-file rustfmt, static concrete-font-name scan for the touched paths, `cargo check -p zircon_runtime_interface --lib`, `cargo check -p zircon_editor --lib --no-default-features`, `cargo check -p zircon_runtime_interface --tests`, and `cargo check -p zircon_editor --tests --no-default-features` on `E:\cargo-targets\zircon-editor-segmented-metrics-0703b`.

Fresh screenshot evidence was attempted but did not complete: `cargo test -p zircon_editor --lib capture_workbench_component_slate_atlas_visual_artifact --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-segmented-metrics-0703b -- --ignored --exact --test-threads=1 --nocapture` timed out after 20 minutes while compiling/linking and its matching target processes were stopped. Current editor PNG modified times therefore remain `2026-07-03 21:04:27` for `editor-components-workbench-slate-atlas-900x620.png`, `2026-07-03 21:01:14` for `editor-window-m3-asset-browser-900x620.png`, and `2026-07-03 21:01:21` for `editor-window-m3-asset-browser-list-900x620.png`; these are not accepted as proof for S15.4eb visual completion.

The former private appearance TOML route has since been retired. Current persistence stores `EditorDesignTokens`, including `utility_tab_text_role`, through the typed settings registry and canonical `zircon.editor.settings` envelope. `core/settings/tests/mod.rs` mounts current-shell round-trip and retired-format rejection coverage; the historical 2026-07-04 private-preference commands above are retained only as dated evidence and are not replay guidance.
