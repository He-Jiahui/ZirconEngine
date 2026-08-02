---
related_code:
  - zircon_editor/src/core/settings/mod.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/core/settings/registry.rs
  - zircon_editor/src/core/settings/tests.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/ui/skin/preset.rs
implementation_files:
  - zircon_editor/src/core/settings/mod.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/core/settings/registry.rs
  - zircon_editor/src/core/settings/tests.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
plan_sources:
  - user: 2026-07-02 do not hardcode fonts; allow global engine preferences for fonts, colors, themes, and styles
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - 'zircon_editor/src/core/settings/tests.rs (current settings registry, version-shell, startup fallback, and retired-format coverage)'
  - 'cargo fmt -p zircon_runtime -p zircon_editor -p zircon_runtime_interface --check (2026-07-02 retained appearance preference font-family hardening: passed)'
  - 'cargo test -p zircon_runtime --lib text_prepare_report --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-preferences-0702 --message-format short --color never -- --test-threads=1 --nocapture (2026-07-02 retained appearance preference font-family hardening: passed, 1/1)'
  - 'cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-preferences-0702 --message-format short --color never (2026-07-02 retained appearance preference font-family hardening: passed after cargo clean -p zircon_runtime_interface; existing warnings)'
  - 'cargo build -p zircon_app --bin zircon_editor --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-preferences-0702 --message-format short --color never (2026-07-02 retained appearance preference font-family hardening: passed; existing warnings)'
  - 'cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-preferences-0702 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-07-02 retained appearance preference font-family hardening: passed, 1/1; refreshed docs/tests/editor screenshot evidence)'
  - 'concrete family scan for DengXian, Segoe UI, Cascadia Mono, Consolas, Fira Sans, Fira Mono, and Inter/Roboto over zircon_editor/src, zircon_runtime_interface/src, and zircon_runtime/src/graphics/scene/scene_renderer/ui (2026-07-02 result NO_CONCRETE_FONT_FAMILY_MATCHES_IN_SCANNED_CODE)'
  - 'target screenshot scan across repo target and D:\cargo-targets\zircon-editor-appearance-preferences-0702 for refreshed M3 screenshot/crop names (2026-07-02 result no matching target screenshots)'
doc_type: module-detail
---

# Editor Appearance Preferences

## Purpose

`zircon_editor/src/core/settings/` is the editor owner for global appearance settings. `defaults.rs` registers the typed `editor.appearance.design_tokens` user setting and resolves it at startup, `io.rs` owns the canonical versioned settings document and user/project paths, and `registry.rs` owns layer precedence. The retained host receives the resolved design tokens before constructing its native window. Fonts, color theme, and style packs belong in this settings route, not in individual controls or startup branches.

The current implementation stores `EditorDesignTokens` as a typed `SettingValue::DesignTokens`. The default is still the workbench dark token set. `SettingsStore` writes the shared canonical JSON `$zircon` envelope with schema id `zircon.editor.settings`; there is no private appearance document or second version field.

## Related Files

`zircon_editor/src/ui/retained_host/app.rs` calls `editor_design_tokens_at_startup()` and projects the resolved tokens into retained-host metrics, palette, and `HostTextPreferences`. `zircon_editor/src/ui/mod.rs` no longer mounts a private preferences module.

`zircon_runtime_interface/src/ui/design_tokens.rs` remains the shared token contract. `zircon_runtime_interface/src/ui/skin/preset.rs` now uses logical typography defaults rather than concrete platform or product font families. `paint_text/font.rs` owns fontdb resolution and fallback font bytes, but it must preserve the requested logical or user family as the runtime family visible to measurement.

## Behavior Model

Controls choose semantic text intent such as UI, strong UI, or code. They do not choose concrete font families. The settings registry owns the design-token value, and the retained-host text bridge converts those tokens into runtime text preferences.

The registry validates replacement values against `SettingSchema::DesignTokens` and resolves User/Project/Session precedence without introducing a second appearance store. Theme catalogs and an in-editor preference UI have not landed yet.

Legacy private appearance TOML and schema version zero are retired. `SettingsStore` accepts only the current `zircon.editor.settings` envelope and rejects legacy or future payloads before mutating the registry. Startup logs the typed load error and retains registered defaults; it never partially applies, migrates, emits, or dual-writes the retired format.

## Design and Rationale

The module exists to keep component standardization bottom-up. Primitive controls, toolbar chips, utility tabs, lists, and drawers should stay reusable and adaptive; if they write `DengXian`, `Segoe UI`, `Fira Mono`, or another concrete family directly, user preferences cannot switch fonts globally.

The same boundary will apply to colors and style packs. Component renderers should consume semantic tokens and roles. Preference loading should decide which token set is active.

## Edge Cases and Constraints

Embedded font assets remain allowed as a rendering fallback when fontdb cannot load the requested face. They are not a user-visible font policy and must not rewrite the runtime family name to the fallback asset name.

Unknown future `$zircon` schema versions fail closed before payload decode. Startup may fall back to current defaults after reporting the typed load error, but the persistence layer never treats a future document as current data and never partially applies it.

Screenshots under `docs/tests/editor` are evidence that the harness refreshed the rendered state at a concrete time. They are not an automated pass for final text quality. The 2026-07-02 utility-tab crop still needs visual follow-up before the editor text stack can be called polished.

## Test Coverage

Historical private-preference validation is retained in repository history, but it is not current replay guidance. Current coverage lives in `core/settings/tests.rs`, including registered design-token defaults, current-shell round trip at the planned user/project paths, startup fallback, atomic layer application, and rejection of retired formats.

## Follow-up

The next layer is a real preferences UI, followed by runtime FontDatabase/FontFace DTO wiring, GPU draw-list family/style propagation, and window-level visual QA for text sharpness.
