---
related_code:
  - zircon_editor/src/ui/preferences.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/src/ui/retained_host/asset_control_ids.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/palette_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/typography.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/tokens.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/
  - zircon_editor/src/tests/host/retained_menu_pointer/appearance_visual_screenshot.rs
implementation_files:
  - zircon_editor/src/ui/preferences.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/src/ui/retained_host/asset_control_ids.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/palette_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/typography.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/tokens.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/appearance_visual_screenshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/tab_like.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_table_row/colors/background.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/separators.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_button_glyphs/
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdown_glyphs/
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields/
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_rows/
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15b-host-control-metrics-single-source.md
  - docs/plans/zircon_editor/editor_layout/15c-retained-palette-single-source.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib host_control_metrics_match_unreal_slate_baseline --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib button_label_frame_keeps_raster_guard_for_short_actions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib template_buttons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_table_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib workbench_chrome --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_shell_panels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_icon_buttons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_fields --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_dropdowns --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_axis_value_fields --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_inspector_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_activation_semantics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib template_segmented --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo test -p zircon_editor capture_workbench_component_slate_atlas_visual_artifact --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - D:\cargo-targets\zircon-editor-components-0625\debug\deps\zircon_editor-820618fe5427109a.exe tests::host::retained_menu_pointer::visual_screenshot::capture_workbench_component_slate_atlas_visual_artifact --ignored --exact --nocapture --test-threads=1
  - S15.1 chrome old-metric-alias source scan
  - S15.1 touched production debt scan
  - S15.6 retained palette handwritten RGBA scan
  - S15.6 touched production debt scan
  - paint-theme model/token ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - cargo fmt -p zircon_editor --check (2026-07-02 passed after retained appearance palette preference slice)
  - cargo build -p zircon_app --bin zircon_editor --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-0702-rerun --message-format short --color never (2026-07-02 latest rerun passed with existing warnings)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-0702-rerun --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-07-02 passed 1/1, refreshed docs/tests/editor/editor-window-m3-asset-browser-900x620.png)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-0702-rerun --message-format short --color never (2026-07-02 latest rerun passed with existing warnings)
  - cargo test -p zircon_editor host_control_metrics_project_from_editor_design_tokens --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-toolbar-0702 --message-format short --color never -- --test-threads=1 --nocapture (2026-07-02 passed 1/1)
  - cargo test -p zircon_editor asset_browser_toolbar_search_field_ignores_legacy_declared_chrome --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-toolbar-0702 --message-format short --color never -- --test-threads=1 --nocapture (2026-07-02 passed 1/1)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-toolbar-0702 --message-format short --color never (2026-07-02 passed with existing warnings)
  - cargo build -p zircon_app --bin zircon_editor --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-toolbar-0702 --message-format short --color never (2026-07-02 passed with existing warnings)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-toolbar-0702 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-07-02 passed 1/1, refreshed docs/tests/editor/editor-window-m3-asset-browser-900x620.png)
  - rustfmt --check zircon_editor/src/ui/preferences.rs (2026-07-03 passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-prefs-0703 --message-format short --color never with CARGO_INCREMENTAL=0 (2026-07-03 passed with existing warnings)
  - cargo test -p zircon_editor --lib appearance_preferences --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-prefs-0703 --message-format short --color never -- --nocapture --test-threads=1 with CARGO_INCREMENTAL=0 (2026-07-03 passed 8/8)
  - D:\cargo-targets\zircon-editor-appearance-prefs-0703\debug\deps\zircon_editor-34f3a39d0731014c.exe tests::host::retained_menu_pointer::visual_screenshot::capture_m3_gui_acceptance_visual_artifacts --ignored --exact --nocapture --test-threads=1 (2026-07-03 passed 1/1, refreshed docs/tests/editor M3 PNGs)
  - preferences.rs concrete font/code-style scan (2026-07-03 no Deng/Segoe/Fira/Cascadia/Consolas/Microsoft YaHei/Arial/Helvetica/font-family/UiTextRunPaintStyle::code matches)
  - rustfmt --check zircon_editor/src/ui/preferences.rs zircon_editor/src/ui/retained_host/app.rs (2026-07-03 passed after startup appearance load path)
  - cargo test -p zircon_editor --lib appearance_preferences --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-prefs-0703 --message-format short --color never -- --nocapture --test-threads=1 with CARGO_INCREMENTAL=0 (2026-07-03 passed 11/11 after startup appearance load path)
  - D:\cargo-targets\zircon-editor-appearance-prefs-0703\debug\deps\zircon_editor-34f3a39d0731014c.exe tests::host::retained_menu_pointer::visual_screenshot::capture_m3_gui_acceptance_visual_artifacts --ignored --exact --nocapture --test-threads=1 (2026-07-03 passed 1/1, refreshed docs/tests/editor M3 PNGs at 08:17)
  - preferences/app concrete font/code-style scan (2026-07-03 no Deng/Segoe/Fira/Cascadia/Consolas/Microsoft YaHei/Arial/Helvetica/font-family/UiTextRunPaintStyle::code matches)
  - cargo check -p zircon_editor --tests --no-default-features --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-appearance-global-0704 --message-format short --color never with CARGO_INCREMENTAL=0 (2026-07-04 passed with existing warnings)
  - cargo test -p zircon_editor tests::host::retained_menu_pointer::appearance_visual_screenshot::capture_global_appearance_preferences_component_visual_artifact --lib --no-default-features --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-appearance-global-0704 --message-format short --color never -- --ignored --exact --test-threads=1 --nocapture with CARGO_INCREMENTAL=0 (2026-07-04 passed 1/1, refreshed docs/tests/editor/editor-components-global-appearance-preferences-900x360.png)
  - cargo build -p zircon_app --bin zircon_editor --features target-editor-host --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-appearance-global-0704 --message-format short --color never with CARGO_INCREMENTAL=0 (2026-07-04 passed with existing warnings)
doc_type: module-detail
---

# Paint Theme

`paint_theme.rs` is the retained-host software paint theme entry. It stays as a structural module that re-exports the palette model and the active palette tokens used by Workbench fallback paint, template-node drawing, diagnostics, and primitive replay.

`paint_theme/model.rs` owns the `HostMaterialPalette` data shape. `paint_theme/palette_projection.rs` owns the central-token-to-retained-host projection. `paint_theme/tokens.rs` exposes the active `PALETTE` as the default projection for the current editor shell instead of owning a second handwritten RGBA table.

`paint_theme/metrics.rs` owns the retained-host chrome control metrics derived from the Unreal Slate baseline used by S15: 4 px control radius, 1 px borders, 10/8/14 font sizes, 1.2 line-height ratio, button padding, pressed offset, row/gap sizing, selection indicator width, and `text_clip_guard`. Template button content uses this guard when measuring short labels so raster rounding cannot clip the last glyph.

The 2026-06-25 S15.1 hard cutover moved chrome atomic consumers onto `METRICS` directly: button content/glyph/text, text fields, dropdown labels/chevrons, icon-button radius and border width, axis value fields, and inspector row primitives no longer own local font, radius, inset, or chevron constants. The same slice added `retained_host/asset_control_ids.rs` so asset dispatch source and asset surface action/control-id normalization are shared by retained-host activation, text-input dispatch, and asset control callbacks instead of being duplicated in multiple leaves.

The 2026-06-25 S15.6 palette cutover extended `EditorPaletteTokens` with retained-host semantic surface, state, separator, popup, track, focus, shadow, and semantic-container roles, mirrored those names in `editor_tokens.zui`, and made `PALETTE` come from `DEFAULT_HOST_PALETTE` in `palette_projection.rs`. Workbench style selector palettes now consume `PALETTE` roles instead of local handwritten RGBA values; the previous retained-host drift in border, primary text, muted text, disabled text, and error colors intentionally converges to the central workbench token values.

The 2026-07-02 retained appearance preference follow-up makes that palette projection runtime-installable. `EditorAppearancePreferences` now exposes replacement hooks for typography, palette, control, density, and state-role tokens; retained-host startup installs both host text preferences and the current host palette from the same design-token source. `palette_projection.rs` owns the `HostMaterialPalette` projection plus the current host palette lock, and TextField/Search is the first style-selector family to consume `current_host_palette()` instead of production `PALETTE` constants. This keeps font family, color theme, and style density switchable through a single preference entry while leaving the actual preference UI and persisted settings for a later slice.

The later 2026-07-02 toolbar preference-route pass extends the same appearance entry to retained-host control metrics and button chrome. `metrics.rs` keeps the Slate baseline as the default but adds `project_host_metrics(...)`, `apply_host_metrics_from_tokens(...)`, and `current_host_metrics()` so radius, border width, body/caption/title font sizes, line height, gaps, row height, and input/button geometry come from `EditorDesignTokens`. Retained-host startup now installs metrics, palette, and text preferences together. Button and Search/TextField consumers read the current metric/palette owners instead of writing concrete font families, component-local color themes, or toolbar-specific RGB values.

The 2026-07-03 Asset Browser utility-tab follow-up keeps that same global route for selected tab-like buttons. Preview/References/Metadata/Plugins no longer get a filled selected pill from the local button selector; they keep transparent surface plus shared primary text and let the underline use `current_host_palette().accent`. `template_buttons/surface.rs` reads underline height and tab inset from `current_host_metrics()`, while selected toolbar chips still keep a low-emphasis framed surface. This changes selection style without adding a concrete font family, local RGB table, or component-owned theme override.

The 2026-07-03 preference persistence foundation keeps that route global instead of binding fonts in controls. `EditorAppearancePreferencesDocument` is the versioned TOML shape for the active appearance profile plus the full `EditorDesignTokens` payload, and `EditorAppearancePreferenceStore` owns string/path load and save. The default document still uses only logical font families from `EditorTypographyTokens` (`system-ui` and `monospace`); a user-selected concrete font can be stored later as a global token value without touching button, table, tab, or label owners. Unsupported document versions parse and then fall back to the current default tokens rather than partially applying an unknown style payload.

The follow-up startup path consumes that persisted shape without introducing a component-local font policy. `editor_startup_appearance_preferences()` reads the optional `ZIRCON_EDITOR_APPEARANCE_PREFERENCES` path and `run_editor_with_startup_request(...)` installs the loaded tokens before constructing the retained host window. Missing, empty, invalid, or unreadable preference files fall back to the default logical-family token set and emit a warning, so a bad user preference cannot strand editor startup. This mirrors the UE `FAppStyle` application-wide style entry: the startup path chooses the active style document, while controls keep reading the current host text, palette, and metric projections.

The 2026-07-04 S15.4el follow-up adds `apply_host_appearance_from_tokens(...)` as the single retained-host appearance application route. Startup and visual tests now install host metrics, palette, and `HostTextPreferences` through that helper instead of calling each projection separately. `HostTextPreferences` continues to carry logical/requested font-family values; concrete user-selected fonts belong in the global preferences payload, not in component defaults. Component-level screenshot evidence was refreshed at `docs/tests/editor/editor-components-global-appearance-preferences-900x360.png`, modified `2026-07-04 14:41:56 +08:00`,38596 bytes,SHA256 `F2A997A1922828220F3197E8B918D4F7BCF38DE731585C4AF042F0CF28E3BE0F`;repo target and external target same-name scans found no matching screenshot, and the external target contained no PNG artifacts.

The 2026-06-26 S15.6d/S15.6e command-button passes keep prominent Workbench command styling in `style_selector/workbench_button/command.rs`. Compile and asset import controls retain accent text and glyph color, but their surface and border come from the muted Workbench palette ladder instead of authored accent fill. This keeps command emphasis available without reintroducing a second color table or large cyan blocks in the module toolbar and Asset Browser command row.

The 2026-06-26 S15.4l/S15.6f table-row selected indicator pass extends the same metric ownership to complex list rows: `selection_indicator_width` is the single retained-host width for the Workbench table selected marker. Selected table rows now use the low-contrast pressed surface for their fill and reserve teal for the 2 px left indicator, so authored selected backgrounds cannot reintroduce full-width cyan row fills.

The 2026-06-26 component-noise follow-up keeps that same token ownership but tunes the Workbench state palette toward a lower-contrast Slate-like surface model. Selected, focused, checked, and open states now resolve their fill to `surface_selected` while keeping primary text, with accent/focus retained for thin borders or underlines. The segmented-control style selector also defaults undeclared selected segments to no rectangular border plus a 2 px underline, so compound controls can move away from full cyan blocks before the larger page-tab/window chrome pass.

The 2026-06-26 tab-like button follow-up moves page-tab, dock-tab, Asset Browser tab-like, and Workbench top module-tab classification into `style_selector/workbench_button/tab_like.rs`. Those controls now resolve through the Workbench button style path even when authored without an explicit button variant, use low-contrast surface/text colors for selected states, and rely on `template_buttons/surface.rs` to paint only a 2 px accent underline. Active Workbench chrome separators use the normal border color instead of promoting the whole container edge to the focus ring. The module-tab whitelist deliberately excludes command buttons such as Save, Browse, Compile, Diff, Simulate, and More so command affordances can be tuned independently.

S15.6 closeout passed the runtime-interface token test, editor lib/test check, the focused retained palette projection and chrome background tests, the editor build, and the M3 screenshot harness using external output under `D:\cargo-targets\zircon-editor-components-0625`. The component atlas cargo wrapper timed out before reporting a result, so it is not counted as passed; the already compiled test binary was then run directly and passed, refreshing `docs/tests/editor/editor-components-workbench-slate-atlas-900x620.png`.

The 2026-06-20 model/token split reduced `paint_theme.rs` from 57 lines to a 4-line structural entry. `model.rs` is 28 lines and owns the palette field schema; `tokens.rs` is 30 lines and owns the concrete RGBA values. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming palette struct fields and token values no longer live in `paint_theme.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.

The 2026-06-25 S15 component pass verifies the metrics owner with `host_control_metrics_match_unreal_slate_baseline` and verifies the button raster guard with `button_label_frame_keeps_raster_guard_for_short_actions`; both use external Cargo output under `D:\cargo-targets\zircon-editor-components-0625`.

S15.1 closeout also refreshed the full M3 retained-host screenshot harness and component atlas under `docs/tests/editor/`; build artifacts remained in `D:\cargo-targets\zircon-editor-components-0625`, not in the repository `target` directory.
