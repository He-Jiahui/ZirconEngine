---
related_code:
  - zircon_editor/src/ui/preferences/mod.rs
  - zircon_editor/src/ui/preferences/appearance.rs
  - zircon_editor/src/ui/preferences/persistence.rs
  - zircon_editor/src/ui/preferences/startup.rs
  - zircon_editor/src/ui/preferences/typography_migration.rs
  - zircon_editor/src/ui/preferences/tests
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/ui/skin/preset.rs
implementation_files:
  - zircon_editor/src/ui/preferences/mod.rs
  - zircon_editor/src/ui/preferences/appearance.rs
  - zircon_editor/src/ui/preferences/persistence.rs
  - zircon_editor/src/ui/preferences/startup.rs
  - zircon_editor/src/ui/preferences/typography_migration.rs
  - zircon_editor/src/ui/preferences/tests
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
plan_sources:
  - user: 2026-07-02 do not hardcode fonts; allow global engine preferences for fonts, colors, themes, and styles
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/tests/test_editor11_appearance_preferences_version_shell_contract.py (2026-07-18 shared version-shell hard cut: passed, 4/4)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/preferences/appearance.rs zircon_editor/src/ui/preferences/persistence.rs zircon_editor/src/ui/preferences/startup.rs zircon_editor/src/ui/preferences/tests/persistence.rs (2026-07-18: passed)
  - cargo fmt -p zircon_runtime -p zircon_editor -p zircon_runtime_interface --check (2026-07-02 retained appearance preference font-family hardening: passed)
  - cargo test -p zircon_runtime --lib text_prepare_report --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-preferences-0702 --message-format short --color never -- --test-threads=1 --nocapture (2026-07-02 retained appearance preference font-family hardening: passed, 1/1)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-preferences-0702 --message-format short --color never (2026-07-02 retained appearance preference font-family hardening: passed after cargo clean -p zircon_runtime_interface; existing warnings)
  - cargo build -p zircon_app --bin zircon_editor --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-preferences-0702 --message-format short --color never (2026-07-02 retained appearance preference font-family hardening: passed; existing warnings)
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-appearance-preferences-0702 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-07-02 retained appearance preference font-family hardening: passed, 1/1; refreshed docs/tests/editor screenshot evidence)
  - concrete family scan for DengXian, Segoe UI, Cascadia Mono, Consolas, Fira Sans, Fira Mono, and Inter/Roboto over zircon_editor/src, zircon_runtime_interface/src, and zircon_runtime/src/graphics/scene/scene_renderer/ui (2026-07-02 result NO_CONCRETE_FONT_FAMILY_MATCHES_IN_SCANNED_CODE)
  - target screenshot scan across repo target and D:\cargo-targets\zircon-editor-appearance-preferences-0702 for refreshed M3 screenshot/crop names (2026-07-02 result no matching target screenshots)
doc_type: module-detail
---

# Editor Appearance Preferences

## Purpose

`zircon_editor/src/ui/preferences/` is the retained editor owner for global appearance preferences. The thin `mod.rs` exposes only the startup entry point; `appearance.rs`, `persistence.rs`, `startup.rs`, and `typography_migration.rs` own the DTO, storage, resolution, and migration responsibilities separately. It gives the editor one startup path for design tokens before those tokens are projected into retained host rendering. Fonts, color theme, and style packs should be selected here or through a future persisted preference model, not by individual controls or startup code.

The current implementation stores `EditorDesignTokens` in `EditorAppearancePreferences`. The default is still the workbench dark token set, while the versioned preference store can replace the source without changing control renderers. New writes use the shared canonical JSON `$zircon` envelope with schema id `zircon.editor.appearance-preferences`; there is no second version field inside the payload.

## Related Files

`zircon_editor/src/ui/mod.rs` exposes the module inside the editor UI crate. `zircon_editor/src/ui/retained_host/app.rs` consumes the preference object during retained-host startup and projects its typography tokens into `HostTextPreferences`.

`zircon_runtime_interface/src/ui/design_tokens.rs` remains the shared token contract. `zircon_runtime_interface/src/ui/skin/preset.rs` now uses logical typography defaults rather than concrete platform or product font families. `paint_text/font.rs` owns fontdb resolution and fallback font bytes, but it must preserve the requested logical or user family as the runtime family visible to measurement.

## Behavior Model

Controls choose semantic text intent such as UI, strong UI, or code. They do not choose concrete font families. The appearance preference owns the design token set, and the retained host text bridge converts those tokens into runtime text preferences.

`EditorAppearancePreferences::with_typography(...)` replaces the typography token block while retaining the rest of the design tokens. `EditorAppearancePreferences::from_design_tokens(...)` replaces the whole token set. These constructors stay deliberately small so persistence remains an outer document/store concern; theme catalogs and an in-editor preference UI have not landed yet.

Legacy version-1 and version-2 TOML documents are read only as unwrapped schema-v0 migration input. `preferences/typography_migration.rs` upgrades version-1 point-as-pixel defaults from 10/8.5/14 to the central Unreal-compatible logical sizes 13.33/10.67/18.67. It migrates only fields still equal to the old defaults, so explicit project or user font-size choices are preserved; legacy version 2 keeps its logical-pixel values. After migration, `EditorAppearancePreferencesLoad::migrated_from` reaches startup policy so the editor records that the file should be resaved in the canonical shell. The store never emits TOML or dual-writes the old embedded version.

## Design and Rationale

The module exists to keep component standardization bottom-up. Primitive controls, toolbar chips, utility tabs, lists, and drawers should stay reusable and adaptive; if they write `DengXian`, `Segoe UI`, `Fira Mono`, or another concrete family directly, user preferences cannot switch fonts globally.

The same boundary will apply to colors and style packs. Component renderers should consume semantic tokens and roles. Preference loading should decide which token set is active.

## Edge Cases and Constraints

Embedded font assets remain allowed as a rendering fallback when fontdb cannot load the requested face. They are not a user-visible font policy and must not rewrite the runtime family name to the fallback asset name.

Unknown future `$zircon` schema versions fail closed before payload decode. Startup may fall back to current defaults after reporting the typed load error, but the persistence layer never treats a future document as current data and never partially applies it.

Screenshots under `docs/tests/editor` are evidence that the harness refreshed the rendered state at a concrete time. They are not an automated pass for final text quality. The 2026-07-02 utility-tab crop still needs visual follow-up before the editor text stack can be called polished.

## Test Coverage

The slice added focused preference tests for default logical typography, global typography replacement, full token replacement, version-1 typography migration, and preservation of custom sizes. On 2026-07-10 the fresh editor test binary ran the preference filter with 12 passed, 0 failed, and one ignored screenshot export; component and Workbench captures were then refreshed under `docs/tests/editor`. On 2026-07-17 all 12 behavior tests were preserved while moving them into `preferences/tests/{tokens,persistence,startup}.rs`; managed job `7471e8a6feb040cf9f9eddb0fdd9c291` passed 12/12 with exit code 0, while the one ignored item remains the explicit screenshot exporter. The 2026-07-18 hard-cut slice adds a real legacy-v1 TOML fixture, v0-to-v1 migration/resave idempotence, current shell round-trip, future-shell rejection, and path load metadata coverage. Its static structure contract passes 4/4 and exact rustfmt passes; Cargo remains pending until Coordinator01 permits a source-bound compile gate.

## Follow-up

The next layer is a real preferences UI, followed by runtime FontDatabase/FontFace DTO wiring, GPU draw-list family/style propagation, and window-level visual QA for text sharpness.
