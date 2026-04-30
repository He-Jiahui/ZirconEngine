# Slint Fence UI TOML Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep editor `.slint` sources out of the active UI authority path, then replace the broken generated-Slint host seams with `.ui.toml` and Rust-owned Runtime UI contracts in small slices.

**Architecture:** deleted Slint copies are disposable and non-authoritative. The active tree must keep `zircon_editor/ui/**/*.slint` absent, no build/test/runtime/doc reader may depend on `temp/slint-migration/**`, and the replacement path is `TOML -> UiSurface -> host projection` through Rust-owned `host_contract` DTOs.

**Tech Stack:** Rust workspace on `main`, root Slint generation removed, Runtime UI assets in `.ui.toml`, editor host code under `zircon_editor/src/ui`, Rust-owned host contracts under `zircon_editor/src/ui/slint_host/host_contract/**`, docs under `docs/ui-and-layout`.

---

### Task 1: Fence All Active Slint Sources

**Files:**
- Remove from active authority: `zircon_editor/ui/**/*.slint`
- Do not restore or read deleted Slint copies for implementation authority
- Modify: `.codex/sessions/20260429-2236-ui-cutover-single-milestone.md`
- Modify: `docs/ui-and-layout/runtime-ui-component-showcase.md`

- [x] **Step 1: Remove every active Slint source from the editor UI authority path**

The original fence moved active sources out of `zircon_editor/ui/<relative>.slint`; current follow-up treats any deleted or temporary copy as non-authoritative and disposable. Do not restore or read these files for implementation authority.

- [x] **Step 2: Verify the fence**

Run: `Get-ChildItem -Path zircon_editor/ui -Filter *.slint -Recurse`

Expected: no active `zircon_editor/ui/**/*.slint` remains, and current source guards do not require or read any temporary Slint copy.

Run: `cargo test -p zircon_editor --lib editor_host_sources_do_not_depend_on_deleted_slint_trees --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never`

Expected: source guard passes without reading any deleted Slint copy.

- [x] **Step 3: Record the expected compile red state**

Update the active session note and Runtime UI docs to state that the move intentionally breaks Slint build seams until replacements land. Record these immediate red seams:

```text
zircon_editor/build.rs -> compile_slint_ui("ui/workbench.slint")
zircon_editor/src/ui/slint_host/mod.rs -> slint::include_modules!()
zircon_editor/src/tests/host/** -> include_str!(... "/ui/workbench.slint") source guards
zircon_editor/tests/integration_contracts/workbench_slint_shell.rs -> reads ui/workbench.slint
```

### Task 2: Replace the Root Build Seam

**Files:**
- Modify: `zircon_editor/build.rs`
- Modify: `zircon_editor/src/ui/slint_host/mod.rs`
- Test: `zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs`

- [x] **Step 1: Convert root Slint build guards into migration-fence guards**

Replace tests that assert contents of `ui/workbench.slint` with guards that assert the file is absent and that editor host sources do not read deleted Slint trees, generated Slint includes, Slint build staging, or compatibility aliases.

- [x] **Step 2: Stop compiling active `ui/workbench.slint` and remove generated root include staging**

`zircon_editor/build.rs` no longer compiles an active `ui/workbench.slint`, no longer stages deleted Slint sources into `OUT_DIR`, and no longer calls `slint_build`. `zircon_editor/src/ui/slint_host/mod.rs` no longer calls `slint::include_modules!()`. Former generated users such as `UiHostWindow`, `HostWindowPresentationData`, `FrameRect`, `PaneData`, host surface DTOs, and generated globals such as `UiHostContext` / `PaneSurfaceHostContext` are now backed by Rust-owned host contracts under `zircon_editor/src/ui/slint_host/host_contract/**`.

- [x] **Step 3: Run the focused red/green check for the build seam**

Run after replacement code exists: `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir target\codex-slint-fence-validation`

Expected: source guards describe the new fenced state and no longer require `zircon_editor/ui/workbench.slint`.

Latest evidence: `cargo test -p zircon_editor --lib editor_host_source_guard_rejects_hyphenated_generated_build_dependency --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` first failed before `slint-build` was added to the source guard marker list, then passed after the marker was added. `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 6 tests / 0 failed / 837 filtered out. `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed.

### Task 3: Backfill Rust-Owned Host DTOs From Former Slint DTOs

**Files:**
- Modify: `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`
- Modify/Create: `zircon_editor/src/ui/template_runtime/*`
- Modify/Create: `zircon_editor/src/ui/host/*`
- Reference: `zircon_editor/src/ui/slint_host/host_contract/**`
- Reference: `zircon_editor/assets/ui/editor/**/*.ui.toml`
- Reference: current Rust projection/conversion modules under `zircon_editor/src/ui/slint_host/ui/**`

- [x] **Step 1: Inventory generated DTO usage**

Search: `slint_ui::FrameRect|slint_ui::PaneData|slint_ui::HostWindow|UiHostWindow|ModelRc<slint_ui::` under `zircon_editor/src/ui`.

Expected: a finite list of generated DTO constructors to replace with Rust-owned DTOs.

Current inventory: no active generated Slint include remains under `zircon_editor/src`. The compatibility alias `as slint_ui` is now rejected by `editor_host_sources_do_not_depend_on_deleted_slint_trees`, and `apply_presentation.rs`, `pane_data_conversion/**`, and `template_node_conversion.rs` import the Rust-owned surface as `host_contract` instead. The remaining `to_slint_*` helper names are local conversion labels over Rust-owned DTOs rather than generated source dependencies.

- [x] **Step 2: Move DTO authority into Rust host/projection modules**

For each generated Slint DTO still used by Rust, either reuse the existing `host_window::*` Rust struct or create a focused Rust-owned DTO beside the producer that owns it. Do not recreate a Slint-specific mirror type.

- [x] **Step 3: Validate DTO replacement scope**

Run: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-slint-fence-validation`

Expected during early slices: failures should shrink toward missing window/backend implementation, not missing `.slint` files or generated DTO names.

Latest evidence: `cargo test -p zircon_editor --lib editor_host_sources_do_not_depend_on_deleted_slint_trees --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` first failed on `as slint_ui` in the presentation and pane conversion modules, then passed after the alias was renamed to `host_contract`. The follow-up generated-build guard also went RED/GREEN for `slint-build = { workspace = true }`. `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 6 tests / 0 failed / 837 filtered out, `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed, and `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 27 tests / 0 failed. The integration-contract checks now assert Rust-owned host contracts and `.ui.toml` assets directly instead of reading active or deleted Slint source paths.

### Task 4: Backfill UI TOML Assets For Former Slint Business Surfaces

**Files:**
- Create/Modify: `zircon_editor/assets/ui/editor/**/*.ui.toml`
- Modify: `zircon_editor/src/ui/template_runtime/builtin/*`
- Reference: current `.ui.toml` assets and Rust-owned host projection; deleted Slint copies are not implementation authority.

- [x] **Step 1: Migrate one former Slint surface at a time**

Prefer this order because existing assets already cover some chrome: activity rail, menu chrome, page chrome, dock headers, status bar, floating header, pane fallback, welcome pane, UI asset editor panes.

- [x] **Step 2: Add source guards per migrated surface**

Each migrated surface gets a test that asserts business labels/control ids/layout nodes come from `.ui.toml` or Runtime UI projection, not from a restored `.slint` file.

Module Plugins slice evidence: `module_plugins_body.ui.toml` is registered as `pane.module_plugins.body`, `ModulePluginsPaneBody` is registered as a builtin component, `ModulePluginsPaneBody/FocusModulePlugins` is projected through the runtime binding table, and `module_plugin_list_slot` is declared as the stable hybrid slot. Focused checks passed with `cargo test -p zircon_editor --lib builtin_pane_body_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` (2 tests / 0 failed), `cargo test -p zircon_editor --lib builtin_hybrid_pane_body_documents_declare_stable_native_slot_names --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` (1 test / 0 failed), and `cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` (1 test / 0 failed).

- [x] **Step 3: Run the milestone testing stage**

Run focused editor Runtime UI checks first, then expand only when the build seam is no longer red:

```powershell
cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1
cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir target\codex-slint-fence-validation
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-slint-fence-validation
```

Expected: focused tests pass for the migrated surface; broader editor check passes only after all generated Slint dependencies have replacements.

Final evidence: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed; `cargo test -p zircon_editor --doc --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed with 0 doctests; `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture` passed 6 tests / 0 failed; `cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture` passed 3 tests / 0 failed; and `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture` passed 16 tests / 0 failed.

Workspace validation then passed with `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir E:\cargo-targets\zircon-ui-cutover-move-first`: `cargo build --workspace --locked` OK and `cargo test --workspace --locked` OK. Final source searches kept the hard fence intact: active `zircon_editor/ui/**/*.slint` count is 0, preserved migration copies under `temp/slint-migration/zircon_editor/ui` count 43, and remaining editor source `slint` hits are test names or absence guards rather than generated-source readers or local `to_slint_*` conversion helpers.

### Task 5: Preserve No-Slint Fence Equivalents For Former Slint Layout/Split Tasks

**Files:**
- Modify: `zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs`
- Reference: `zircon_editor/assets/ui/editor/**/*.ui.toml`
- Reference only: `temp/slint-migration/zircon_editor/ui/**`

- [x] **Step 1: Translate obsolete Slint move/split expectations into active absence guards**

The old move-first Task 4/5 Slint-path work is intentionally obsolete under the hard fence. The active equivalent is to keep `zircon_editor/ui/**/*.slint` absent, reject deleted-tree source dependencies, and ensure replacement editor UI authority comes from `.ui.toml` assets and Rust-owned host contracts rather than restored Slint domain files.

- [x] **Step 2: Re-run the focused no-Slint guards**

Latest evidence: `cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 3 tests / 0 failed / 841 filtered out, and `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 6 tests / 0 failed / 838 filtered out.

### Task 6: Add Runtime UI Input Dispatch Acceptance

**Files:**
- Modify: `zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs`
- Modify: `zircon_runtime/src/tests/ui_boundary/runtime_host.rs`

- [x] **Step 1: Add acceptance for pointer and navigation dispatch through `RuntimeUiManager`**

The focused test first targeted missing manager methods and the shared `UiSurface` focus/capture result. During implementation the button enum was corrected to the existing `UiPointerButton::Primary` variant.

- [x] **Step 2: Forward dispatch through the owned shared surface**

`RuntimeUiManager::dispatch_pointer_event(...)` now forwards to `UiSurface::dispatch_pointer_event(...)`, and `RuntimeUiManager::dispatch_navigation_event(...)` forwards to `UiSurface::dispatch_navigation_event(...)`. The manager remains the crate-local runtime host facade while shared surface dispatch stays the implementation authority.

- [x] **Step 3: Validate runtime dispatch and boundary coverage**

Latest evidence: `cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed 1 test / 0 failed / 1190 filtered out; `cargo test -p zircon_runtime --lib ui_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed 17 tests / 0 failed / 1174 filtered out; `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed; and `rustfmt --edition 2021 --check zircon_runtime\src\ui\runtime_ui\runtime_ui_manager.rs zircon_runtime\src\tests\ui_boundary\runtime_host.rs zircon_editor\src\tests\host\slint_window\generic_host_layout_paths.rs` passed after formatting.

### Task 7: Expand Runtime Graphics Fixture Acceptance

**Files:**
- Modify: `zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs`

- [x] **Step 1: Cover all builtin runtime UI fixtures through the screen-space UI pass**

The feature-gated acceptance now submits `HudOverlay`, `PauseMenu`, `SettingsDialog`, and `InventoryList` through `RuntimeUiManager::build_frame()` into `WgpuRenderFramework::submit_runtime_frame(...)`, then checks that each fixture contributes a non-trivial UI command list and reaches either quad or text payload output.

- [x] **Step 2: Validate the feature-gated graphics fixture path and text contract**

Latest evidence: `cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture` passed 1 test / 0 failed / 1195 filtered out after waiting for an artifact lock; `cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture` passed 7 tests / 0 failed.

### Self-Review

- Spec coverage: the plan covers the selected all-`.slint` fence, the immediate red build seams, DTO replacement, `.ui.toml` backfill, no-Slint equivalents for obsolete Slint layout tasks, Runtime UI dispatch acceptance, Runtime UI graphics fixture acceptance, docs/session updates, and validation gates.
- Placeholder scan: no `TBD` or undefined task remains; early red compile state is explicitly intentional.
- Type consistency: generated Slint names are treated as replacement targets, not new desired contracts.
