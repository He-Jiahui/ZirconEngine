# Slint Material Retained Editor Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reproduce `dev/material-rust-template/material-1.0` Material styling and behavior inside Zircon Editor retained UI without linking Slint or making `.slint` the Editor business UI source.

**Architecture:** The migration is evidence-first and lower-layer-first. M0 records every Slint Material export and the retained owner; M1 ports foundation tokens; later milestones implement state layer/ripple/elevation, primitive controls, composite surfaces, and Editor adoption through existing `.ui.toml` / `.zui` / runtime UI / retained host seams.

**Tech Stack:** Rust Cargo workspace, `zircon_editor` retained UI assets, UI v2 TOML/ZUI, `zircon_runtime` UI descriptors/layout/event routing, `zircon_runtime_interface` UI DTOs, repository-local Slint Material template under `dev/material-rust-template/material-1.0`.

---

## File Responsibility Map

- `docs/superpowers/specs/2026-05-20-slint-material-retained-editor-migration-design.md`: approved design and scope boundary.
- `docs/ui-and-layout/slint-material-retained-editor-migration.md`: M0 source inventory, export mapping, retained owner, milestone placement, and intentional divergence.
- `docs/ui-and-layout/index.md`: UI/Layout index link for the new mapping doc.
- `.codex/sessions/20260520-slint-material-retained-editor-migration.md`: live cross-session coordination note while active.
- `zircon_editor/assets/ui/theme/editor_material.v2.ui.toml`: M1 Slint Material foundation token landing zone.
- `zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs`: M0/M1/M2 static boundary tests for source coverage, docs coverage, token coverage, metadata coverage, and no direct Editor Slint dependency.
- `zircon_editor/src/tests/ui/boundary/mod.rs`: test module registration only.
- `zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs`: retained host metadata carrier for state-layer/ripple facts.
- `zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs`: retained native painter state-layer and static ripple command helper.
- `zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs`: template-node surface, elevation, state priority, and state-layer integration.
- `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs`: projection of source-compatible state-layer/ripple attributes into the host contract.
- `zircon_runtime/src/ui/style.rs`: runtime bool fallback priority for Material button interaction state.
- `zircon_runtime/src/ui/tests/material_button_style.rs`: focused runtime style priority coverage.
- `zircon_editor/src/tests/host/retained_window/native_material_painter.rs`: focused retained painter pixel coverage for priority, ripple, and elevation.

## Milestone 0: Evidence, Fence, And Documentation

**Goal:** Freeze the Slint Material source inventory and retained migration rules before behavior code changes.

**In-scope behaviors:** docs/spec/plan creation, `material.slint` export coverage, no direct `zircon_editor` Slint dependency, docs index registration, coordination note.

**Dependencies:** Existing retained UI docs, Material UI token/component audit, current `zircon_editor/Cargo.toml`, and local `dev/material-rust-template/material-1.0` source tree.

**Implementation slices:**

- [x] Create the design spec under `docs/superpowers/specs/2026-05-20-slint-material-retained-editor-migration-design.md`.
- [x] Create this milestone plan under `docs/superpowers/plans/2026-05-20-slint-material-retained-editor-migration.md`.
- [x] Create `.codex/sessions/20260520-slint-material-retained-editor-migration.md` with current scope and coordination warnings.
- [x] Create `docs/ui-and-layout/slint-material-retained-editor-migration.md` with YAML frontmatter, source inventory, export mapping, retained owner, milestone placement, tests, and divergence notes.
- [x] Add the mapping doc to `docs/ui-and-layout/index.md`.
- [x] Add `zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs`.
- [x] Register the new boundary test module in `zircon_editor/src/tests/ui/boundary/mod.rs`.

**Testing stage:**

- [x] Run `rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs`.
- [x] Run `cargo metadata --locked --no-deps --format-version 1` as the lightweight Cargo gate for this docs/test/theme slice.
- [x] Run focused `git diff --check -- docs/superpowers/specs/2026-05-20-slint-material-retained-editor-migration-design.md docs/superpowers/plans/2026-05-20-slint-material-retained-editor-migration.md docs/ui-and-layout/slint-material-retained-editor-migration.md docs/ui-and-layout/index.md zircon_editor/src/tests/ui/boundary/mod.rs zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs .codex/sessions/20260520-slint-material-retained-editor-migration.md`.

**Lightweight checks:** The boundary test is static and can be compiled/run later in the milestone testing stage; no broad `zircon_editor` Cargo test is required during M0 implementation.

**Exit evidence:** M0 exits when the mapping doc covers all exports in `material.slint`, the docs name the no-Slint Editor fence, and focused static checks above pass or record only unrelated line-ending warnings.

## Milestone 1: Foundation Token Convergence

**Goal:** Add Slint Material foundation token names and values to `editor_material.v2.ui.toml` while preserving existing compact Editor selectors.

**In-scope behaviors:** palette/scheme role tokens, metric ladder, icon sizes, padding/spacing/radius ladder, typography scale, animation timing/easing names, state-layer opacities, disabled opacity, modal background, shadow/elevation roles, and source-trace docs/tests.

**Dependencies:** M0 mapping doc and `dev/material-rust-template/material-1.0/ui/styling/**` plus `state_layer.slint` and `elevation.slint`.

**Implementation slices:**

- [x] Extend `[tokens]` in `zircon_editor/assets/ui/theme/editor_material.v2.ui.toml` with Slint Material palette roles prefixed `slint_material_*` and stable aliases where existing `material_color_*` roles already exist.
- [x] Add metric ladder tokens such as `slint_material_size_32`, `slint_material_padding_16`, `slint_material_spacing_8`, and `slint_material_border_radius_28`.
- [x] Add typography tokens for `display_*`, `headline_*`, `title_*`, `label_*`, and `body_*` font sizes and weights.
- [x] Add animation tokens for emphasized, standard, ripple, and opacity durations/easing names.
- [x] Add elevation shadow offset/blur/alpha tokens for levels 1 through 5 in light and dark variants.
- [x] Update `docs/ui-and-layout/slint-material-retained-editor-migration.md` with M1 token landing details and the focused validation plan.
- [x] Extend `zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs` to assert M1 token coverage and selected exact source-derived values.

**Testing stage:**

- [x] Run `rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs`.
- [x] Run `cargo metadata --locked --no-deps --format-version 1`.
- [x] Run focused `git diff --check -- docs/ui-and-layout/slint-material-retained-editor-migration.md docs/ui-and-layout/index.md zircon_editor/assets/ui/theme/editor_material.v2.ui.toml zircon_editor/src/tests/ui/boundary/mod.rs zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs .codex/sessions/20260520-slint-material-retained-editor-migration.md`.

**Lightweight checks:** TOML parsing is covered by the boundary test source and can be executed during the milestone testing stage; no full workspace build is required for the token-only slice.

**Exit evidence:** M1 exits when source-derived token groups are present in `editor_material.v2.ui.toml`, exact values are documented and statically checked, and no direct Slint dependency appears in `zircon_editor/Cargo.toml`.

## Milestone 2: State Layer, Ripple, And Elevation Behavior

**Goal:** Translate `state_layer.slint` and `elevation.slint` into retained runtime/editor behavior.

**Implementation slices:**

- [x] Define retained state-layer priority: disabled, focus, pressed, hover, drag, and default.
- [x] Preserve keyboard activation metadata equivalent to `FocusTouchArea.enter_pressed`; runtime event routing remains the owner for setting the flag.
- [x] Represent ripple as retained press metadata and painter-supported static pulse; animated expansion is recorded as a later motion-layer gap.
- [x] Add elevation painter support through retained host paint commands, not Slint imports.
- [x] Update migration docs and retained host module docs with M2 behavior, source facts, implementation files, and tests.
- [x] Add focused runtime/editor tests for state priority, projection metadata, static ripple pixels, and elevation shadow.

**Testing stage:**

- [x] Run `rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs zircon_editor/src/tests/host/retained_window/native_material_painter.rs zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs zircon_runtime/src/ui/style.rs zircon_runtime/src/ui/tests/material_button_style.rs`.
- [x] Run `cargo metadata --locked --no-deps --format-version 1`.
- [x] Run focused `cargo check -p zircon_runtime --lib --locked --jobs 1`; passed with one existing dead-code warning for `World::entity_ids_matching_query_archetypes`.
- [x] Run focused `git diff --check -- docs/ui-and-layout/slint-material-retained-editor-migration.md docs/zircon_editor/ui/retained_host/host_contract/data/template_nodes.md docs/superpowers/plans/2026-05-20-slint-material-retained-editor-migration.md zircon_editor/assets/ui/theme/editor_material.v2.ui.toml zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs zircon_editor/src/tests/host/retained_window/native_material_painter.rs zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/painter/mod.rs zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs zircon_runtime/src/ui/style.rs zircon_runtime/src/ui/tests/material_button_style.rs .codex/sessions/20260520-slint-material-retained-editor-migration.md`; no whitespace errors, only CRLF normalization warnings.
- [ ] `cargo check -p zircon_editor --lib --locked --jobs 1` is blocked outside this slice by `zircon_runtime/src/core/framework/render/material/validation_error.rs` adding `RenderMaterialValidationError::MissingRequiredProperty` without the matching arm in `zircon_editor/src/ui/material_editor/projection.rs`.
- [ ] `cargo test -p zircon_runtime --lib material_button_style --locked --jobs 1` is blocked outside this slice before reaching the focused tests by `zircon_runtime/src/graphics/tests/renderer_data_asset.rs` using `BTreeMap<AssetReference, ...>` while the currently dirty `zircon_runtime_interface::resource::AssetReference` lacks `Ord`.

**Exit evidence:** M2 exits when retained metadata, painter support, source-symbol docs, and focused tests exist and the lightweight gates above either pass or record only unrelated line-ending warnings. Full workspace green remains reserved for M6 or the explicit milestone testing stage after unrelated dirty lanes settle.

## Milestone 3: Core Controls

**Goal:** Port Slint Material primitive controls into retained descriptors, `.zui` prototypes, events, and painter projection.

**Implementation slices:**

- [ ] BaseButton and button variants from `base_button.slint`, `filled_button.slint`, `outline_button.slint`, `text_button.slint`, `tonal_button.slint`, `elevated_button.slint`, and icon/FAB variants.
- [ ] TextField and SearchBar from `text_field.slint` and `search_bar.slint`.
- [ ] Checkbox, RadioButton, Switch, Slider, ProgressIndicator, ListTile, MenuItem, and material text/icon primitives.

**Testing stage:** Focused runtime component catalog/event/layout tests plus editor Material Lab/projection/native painter tests.

## Milestone 4: Composite Surfaces, Navigation, And Overlays

**Goal:** Port composite Material structures while keeping retained surface ownership.

**Implementation slices:**

- [ ] App bars, bottom app bar, cards, drawer/modal drawer, dialog/fullscreen dialog, menu/popup menu, bottom sheet, snackbar, tooltip.
- [ ] Navigation bar/drawer/rail, tab bar/secondary tab bar, list view, scroll view, grid/horizontal/vertical layout helpers.
- [ ] Date/time picker popups and modal/portal behavior through retained overlay surfaces.

**Testing stage:** Runtime popup/modal/event/focus tests, editor Material Lab shell/projection tests, painter/visual capture where applicable.

## Milestone 5: Editor Shell Adoption

**Goal:** Replace approximate retained Editor Material controls with completed Slint-derived retained components without moving Editor business UI back to Slint.

**Implementation slices:**

- [ ] Apply completed retained components to workbench shell, Project Overview, Asset Browser, Inspector controls, toolbar/menu surfaces, and Material Lab.
- [ ] Keep `.ui.toml` and runtime projection as structure/event truth.
- [ ] Remove any temporary duplicated style paths introduced during earlier milestones.

**Testing stage:** Focused editor host, template assets, Material Lab, native painter, pointer routing, and visual capture gates.

## Milestone 6: Full Validation And Acceptance

**Goal:** Prove the retained migration is complete and no Slint business-source regression exists.

**Testing stage:**

- [ ] `cargo fmt --all --check`
- [ ] `cargo test -p zircon_editor --lib material_component_lab --locked --jobs 1`
- [ ] `cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1`
- [ ] `cargo test -p zircon_editor --lib template_assets --locked --jobs 1`
- [ ] `cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1`
- [ ] `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1`
- [ ] `cargo test -p zircon_runtime --lib material_layout --locked --jobs 1`
- [ ] Static guard: `zircon_editor` still has no direct Slint dependency and no `@material` or generated Slint UI source path in retained Editor UI.

Record failures, debug lower shared support first, and do not claim workspace-wide green when unrelated dirty-session compile blockers remain.
