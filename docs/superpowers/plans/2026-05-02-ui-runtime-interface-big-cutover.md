# UI Runtime Interface Big Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `zircon_runtime_interface::ui` the canonical owner for neutral UI DTO contracts and remove the mixed runtime/interface UI type identities blocking M18, M21, and M14 focused runtime tests.

**Architecture:** `zircon_runtime_interface` owns the neutral UI declaration tree. `zircon_runtime` keeps concrete UI behavior, validation, compilation, layout extraction, event routing, and runtime services while importing interface DTOs directly. `zircon_editor` consumes interface UI contracts and only keeps concrete runtime dependencies where the remaining API is a real runtime service dependency.

**Tech Stack:** Rust 2021, Cargo workspace, Serde/TOML DTOs, `zircon_runtime_interface`, `zircon_runtime::ui`, runtime UI asset compiler/tests, and editor UI host docs.

---

## Execution Boundary

- Stay on `main`; do not create worktrees or feature branches.
- Do not restore or edit `zircon_editor/ui/**/*.slint` or `temp/slint-migration/**`.
- Do not broaden into the active graphics/plugin renderer execution cutover. Touch graphics UI renderer files only to canonicalize UI DTO imports required by this cutover.
- Do not preserve the old `zircon_runtime_interface::ui` path-include bridge.
- Do not add migration-only `pub use`, shim, compatibility, bridge, facade, alias, or legacy modules to keep old DTO paths alive.
- During implementation slices, write production code, unit-test code, and docs first. Run Cargo build/test commands only in the named testing stages unless a syntax/type blocker requires an earlier scoped check.
- Before every Cargo build/test stage, check free space on the target drive and avoid concurrent Cargo writers against `E:\cargo-targets\zircon-ui-interface-big-cutover`.

## Current Baseline

- `zircon_runtime_interface/src/ui/mod.rs` currently path-includes runtime UI source files from `zircon_runtime/src/ui/**`.
- Runtime-local `crate::ui::*` DTOs and interface `zircon_runtime_interface::ui::*` DTOs compile as distinct type identities.
- M18 binding semantics code and public integration tests have landed, but filtered runtime lib tests fail before execution due to unrelated mixed UI DTO identities.
- M21 action-policy and M14 localization runtime foundations have landed, but their focused lib-test filters hit the same compile blocker.
- Known blocker files that mix `crate::ui::*` DTO imports with interface DTO call sites are:
  - `zircon_runtime/src/asset/tests/assets/font.rs`
  - `zircon_runtime/src/graphics/tests/render_framework_bridge.rs`
  - `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs`
  - `zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs`
  - `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs`
  - `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs`
- `zircon_editor` still has direct `zircon_runtime::ui` imports. The 2026-05-02 Milestone 4 audit found 128 `zircon_runtime::ui` hits in `zircon_editor/src`, alongside 428 `zircon_runtime_interface::ui` hits. The lower-layer tree/surface identity has now converged: `UiSurface.tree` stores the interface-owned `UiTree`, tree DTOs are imported from `zircon_runtime_interface::ui::tree`, and runtime tree behavior is exposed through `zircon_runtime::ui::tree::UiRuntimeTree*Ext` traits. Remaining runtime imports are dominated by concrete services and behavior APIs such as `UiSurface`, `UiPointerDispatcher`, `UiAssetLoader`, `UiDocumentCompiler`, `UiTemplateSurfaceBuilder`, `UiTemplateBuildError`, `UiComponentDescriptorRegistry`, `UiAssetDocumentRuntimeExt`, and runtime tree behavior extension traits. Do not mechanically rewrite those behavior imports as DTO ownership fixes.

## File Structure

### Interface Contract Files To Create Or Materialize

- Modify: `zircon_runtime_interface/src/ui/mod.rs` so it contains structural `pub mod` declarations only.
- Create: `zircon_runtime_interface/src/ui/binding/mod.rs`.
- Create: `zircon_runtime_interface/src/ui/binding/model/mod.rs`.
- Create: `zircon_runtime_interface/src/ui/binding/model/binding_call.rs`.
- Create: `zircon_runtime_interface/src/ui/binding/model/binding_value.rs`.
- Create: `zircon_runtime_interface/src/ui/binding/model/event_binding.rs`.
- Create: `zircon_runtime_interface/src/ui/binding/model/event_kind.rs`.
- Create: `zircon_runtime_interface/src/ui/binding/model/event_path.rs`.
- Create: `zircon_runtime_interface/src/ui/binding/model/parse_error.rs`.
- Create: `zircon_runtime_interface/src/ui/binding/model/parser.rs` if `UiEventBinding::parse` remains an interface helper.
- Create: `zircon_runtime_interface/src/ui/component/mod.rs`.
- Create: `zircon_runtime_interface/src/ui/component/category.rs`.
- Create: `zircon_runtime_interface/src/ui/component/drag.rs`.
- Create: `zircon_runtime_interface/src/ui/component/event.rs`.
- Create: `zircon_runtime_interface/src/ui/component/state.rs`.
- Create: `zircon_runtime_interface/src/ui/component/validation.rs`.
- Create: `zircon_runtime_interface/src/ui/component/value.rs`.
- Create: `zircon_runtime_interface/src/ui/component/descriptor/mod.rs`.
- Create interface-owned descriptor declaration files matching `zircon_runtime/src/ui/component/descriptor/{component_descriptor,default_node_template,fallback_policy,host_capability,option_descriptor,palette_metadata,prop_schema,render_capability,slot_schema}.rs`.
- Create: `zircon_runtime_interface/src/ui/component/data_binding/mod.rs`.
- Create interface-owned data-binding declaration files matching `zircon_runtime/src/ui/component/data_binding/{adapter_error,adapter_result,binding_target,data_source,event_envelope,projection_patch}.rs`.
- Create: `zircon_runtime_interface/src/ui/dispatch/mod.rs`.
- Create interface-owned dispatch DTO files matching `zircon_runtime/src/ui/dispatch/{navigation,pointer}/{context,effect,invocation,result}.rs` and `zircon_runtime/src/ui/dispatch/pointer/event.rs`.
- Create: `zircon_runtime_interface/src/ui/event_ui/mod.rs`.
- Create: `zircon_runtime_interface/src/ui/event_ui/control.rs`.
- Create: `zircon_runtime_interface/src/ui/event_ui/reflection.rs`.
- Create: `zircon_runtime_interface/src/ui/event_ui/codec.rs` if it remains a pure binding-codec helper.
- Create: `zircon_runtime_interface/src/ui/layout/mod.rs`.
- Create: `zircon_runtime_interface/src/ui/layout/constraints.rs`.
- Create: `zircon_runtime_interface/src/ui/layout/geometry.rs`.
- Create: `zircon_runtime_interface/src/ui/layout/scroll.rs`.
- Create: `zircon_runtime_interface/src/ui/layout/virtualization.rs`.
- Create: `zircon_runtime_interface/src/ui/surface/mod.rs`.
- Create interface-owned surface DTO files matching `zircon_runtime/src/ui/surface/{focus_state,navigation_state}.rs`.
- Create interface-owned navigation/pointer DTO files matching `zircon_runtime/src/ui/surface/{navigation,pointer}/**/*.rs`.
- Create: `zircon_runtime_interface/src/ui/surface/render/mod.rs`.
- Create interface-owned render declaration files matching `zircon_runtime/src/ui/surface/render/{command,command_kind,list,resolved_style,typography,visual_asset_ref}.rs`.
- Create: `zircon_runtime_interface/src/ui/surface/render/extract.rs` with only `UiRenderExtract` data declaration.
- Create: `zircon_runtime_interface/src/ui/surface/render/text_layout.rs` with only `UiResolvedTextLayout` and `UiResolvedTextLine` data declarations.
- Create: `zircon_runtime_interface/src/ui/template/mod.rs`.
- Create: `zircon_runtime_interface/src/ui/template/document.rs`.
- Create: `zircon_runtime_interface/src/ui/template/asset/mod.rs`.
- Create: `zircon_runtime_interface/src/ui/template/asset/binding/mod.rs`.
- Create interface-owned M18 contract files matching `zircon_runtime/src/ui/template/asset/binding/{diagnostic,expression,target}.rs`.
- Create: `zircon_runtime_interface/src/ui/template/asset/action_policy/mod.rs`.
- Create interface-owned M21 contract files matching `zircon_runtime/src/ui/template/asset/action_policy/{diagnostic,host_policy,report,side_effect_class}.rs`.
- Create: `zircon_runtime_interface/src/ui/template/asset/localization/mod.rs`.
- Create interface-owned M14 contract files matching `zircon_runtime/src/ui/template/asset/localization/{diagnostic,localized_text_ref,report,text_direction}.rs`.
- Create interface-owned asset contract files for shared package/report DTOs matching `zircon_runtime/src/ui/template/asset/compiler/package/{artifact,cache_record,header,manifest,package_manifest,profile,report}.rs`.
- Create interface-owned asset contract files matching `zircon_runtime/src/ui/template/asset/{component_contract,invalidation,resource_ref,schema}` declaration/report files that appear in package reports or editor-facing diagnostics.
- Create: `zircon_runtime_interface/src/ui/tree/mod.rs`.
- Create: `zircon_runtime_interface/src/ui/tree/node/mod.rs`.
- Create interface-owned tree DTO files matching `zircon_runtime/src/ui/tree/node/{dirty_flags,input_policy,layout_cache,template_node_metadata,tree_error,tree_node,ui_tree}.rs` only after moving behavior extension methods out of the declarations.

### Runtime Behavior Files To Modify

- Modify: `zircon_runtime/src/ui/mod.rs` to keep runtime module ownership structural and remove duplicated neutral DTO declarations from runtime-owned files.
- Modify: `zircon_runtime/src/ui/binding/mod.rs` and `zircon_runtime/src/ui/binding/router.rs` to import interface binding DTOs directly.
- Modify: `zircon_runtime/src/ui/component/descriptor/validation.rs`, `zircon_runtime/src/ui/component/catalog/registry.rs`, and `zircon_runtime/src/ui/component/catalog/palette_view.rs` to consume interface component DTOs.
- Keep `zircon_runtime/src/ui/component/catalog/editor_showcase.rs` runtime-owned as catalog data, importing interface descriptors.
- Modify: `zircon_runtime/src/ui/dispatch/{navigation,pointer}/dispatcher.rs` to consume interface dispatch DTOs and runtime `UiTree` behavior.
- Keep `zircon_runtime/src/ui/event_ui/manager/**` runtime-owned as event manager behavior, importing interface control/reflection DTOs.
- Modify: `zircon_runtime/src/ui/layout/pass/**` to consume interface layout DTOs while keeping layout pass behavior in runtime.
- Modify: `zircon_runtime/src/ui/surface/render/extract.rs` so it exposes a runtime behavior function such as `extract_ui_render_tree(tree: &UiTree) -> zircon_runtime_interface::ui::surface::UiRenderExtract` instead of defining the DTO locally.
- Modify: `zircon_runtime/src/ui/surface/render/text_layout.rs` so it keeps `layout_text(...)` behavior and returns interface `UiResolvedTextLayout` / `UiResolvedTextLine`.
- Modify: `zircon_runtime/src/ui/surface/{surface.rs,render/resolve.rs,render/node_visual_data.rs}` to import interface surface/layout/tree DTOs directly.
- Modify: `zircon_runtime/src/ui/template/asset/binding/validation.rs` to keep validation in runtime while importing M18 contract DTOs from `zircon_runtime_interface::ui::template::asset::binding`.
- Modify: `zircon_runtime/src/ui/template/asset/action_policy/validate.rs` to keep policy validation in runtime while importing M21 contract DTOs from `zircon_runtime_interface::ui::template::asset::action_policy`.
- Modify: `zircon_runtime/src/ui/template/asset/localization/collect.rs` to keep localization collection in runtime while importing M14 contract DTOs from `zircon_runtime_interface::ui::template::asset::localization`.
- Modify: `zircon_runtime/src/ui/template/asset/compiler/package/**` to use interface package/report DTOs for package validation output.
- Modify: `zircon_runtime/src/ui/runtime_ui/{public_frame,runtime_ui_manager,runtime_ui_fixture}.rs` to use interface UI render/event/layout DTOs at shared seams.

### Known Mismatch Files To Update

- Modify: `zircon_runtime/src/asset/tests/assets/font.rs` to import `UiTextRenderMode` from `zircon_runtime_interface::ui::surface`.
- Modify: `zircon_runtime/src/graphics/tests/render_framework_bridge.rs` to import `UiNodeId`, `UiTreeId`, `UiFrame`, `UiRenderExtract`, `UiRenderList`, `UiRenderCommand`, `UiRenderCommandKind`, `UiResolvedStyle`, `UiTextAlign`, `UiTextRenderMode`, and `UiTextWrap` from `zircon_runtime_interface::ui::*`.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs` test module to use interface `UiNodeId`, `UiTreeId`, `UiRenderExtract`, `UiRenderList`, `UiResolvedStyle`, `UiResolvedTextLayout`, `UiResolvedTextLine`, `UiTextAlign`, `UiTextRenderMode`, `UiTextWrap`, and `UiFrame`.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs` tests to import `UiTextAlign`, `UiTextRenderMode`, and `UiTextWrap` from `zircon_runtime_interface::ui::surface`.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs` tests to import `UiTextAlign` and `UiTextWrap` from `zircon_runtime_interface::ui::surface`.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs` tests to import `UiTextAlign` and `UiTextWrap` from `zircon_runtime_interface::ui::surface`.

### Tests And Docs To Modify

- Modify: `zircon_runtime_interface/src/tests/contracts.rs` with interface UI contract construction tests and a source guard rejecting `#[path =` in `zircon_runtime_interface/src/ui/mod.rs`.
- Modify: `zircon_runtime/src/tests/ui_boundary/mod.rs` and add a focused structural test if runtime still has source references to `zircon_runtime_interface/src/ui` path-include residue.
- Create: `docs/zircon_runtime_interface/ui/mod.md` as the source-path mirror module document for the interface UI contract tree.
- Update: `docs/engine-architecture/runtime-interface-cdylib-loader.md` to replace the Milestone 2 path-include wording with real interface-owned UI modules.
- Update: `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` to list interface-owned M18/M21/M14 contract paths and runtime-owned validation behavior paths.
- Update: `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` to list interface-owned component/package/report contract paths and runtime-owned validation/catalog paths.
- Update: `docs/editor-and-tooling/ui-asset-editor-host-session.md` with the residual editor dependency status: neutral UI contracts are interface-owned; concrete runtime services remain linked until capability APIs replace them.
- Update: `.codex/sessions/20260502-0805-ui-runtime-interface-big-cutover.md` after every material scope, blocker, or validation change.

## Milestone 1: Materialize Interface-Owned UI Contracts

**Goal:** Replace runtime source path-inclusion with real `zircon_runtime_interface/src/ui/**` modules that compile without depending on `zircon_runtime`, `zircon_editor`, Slint, wgpu, or plugin implementation crates.

**In-scope behaviors:** Interface crate can construct and serialize representative UI layout, surface render, event/control, binding, component descriptor, template document, action-policy, localization, and package-report DTOs from its own source tree.

**Dependencies:** Current `zircon_runtime_interface` ABI crate exists and already depends on `serde`, `serde_json`, `toml`, `glam`, `uuid`, and `thiserror`.

**Implementation slices:**

- [x] Replace `zircon_runtime_interface/src/ui/mod.rs` path attributes with structural module declarations for `binding`, `component`, `dispatch`, `event_ui`, `layout`, `surface`, `template`, and `tree`.
- [x] Copy or move low-risk declaration files into the matching `zircon_runtime_interface/src/ui/**` paths listed in File Structure.
- [x] For mixed files, split declarations into interface and leave behavior in runtime. Required mixed splits are `surface/render/extract.rs`, `surface/render/text_layout.rs`, `template/asset/document.rs`, and `tree/node/ui_tree.rs` if behavior methods exceed declaration helpers.
- [x] Keep pure contract helpers in interface when callers need them on the DTO type itself, such as binding expression parsing, side-effect class inference, host policy allow checks, localized text reference validation, source-path formatting, and value-kind conversion.
- [x] Do not move runtime event manager, component registry/editor-showcase data, dispatchers, layout pass behavior, surface orchestration, tree behavior extension files, template loaders, schema migrators, compilers, or package validation behavior into interface.
- [x] Add interface contract tests in `zircon_runtime_interface/src/tests/contracts.rs` that instantiate representative DTOs from each created UI family.
- [x] Add a `zircon_runtime_interface/src/tests/contracts.rs` source guard that reads `zircon_runtime_interface/src/ui/mod.rs` and fails if it contains `#[path =` or `zircon_runtime/src/ui`.
- [x] Create `docs/zircon_runtime_interface/ui/mod.md` with the required machine-readable header covering the new interface files, plan source, and validation commands.

**Testing stage:**

- Check target-drive space: `Get-PSDrive -Name E`.
- Run `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- Run `cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- Run `cargo tree -p zircon_runtime_interface --locked` and confirm no `zircon_runtime`, `zircon_editor`, `slint`, or `wgpu` dependency appears.
- Debug and correct any interface dependency leak or compile failure before advancing.

**Exit evidence:** Interface UI modules compile from real files; interface tests pass; `ui/mod.rs` no longer path-includes runtime source; dependency tree stays implementation-free.

## Milestone 2: Rewire Runtime UI Behavior To Interface DTOs

**Goal:** Make runtime UI behavior consume interface DTOs directly and remove duplicate runtime-owned neutral DTO declarations for shared UI seams.

**In-scope behaviors:** Runtime compilation, binding validation, action policy validation, localization collection, layout extraction, surface rendering extraction, event routing, and package report assembly continue to work while using interface DTO identities.

**Dependencies:** Milestone 1 interface modules compile and expose all contract DTOs required by runtime behavior.

**Implementation slices:**

- [ ] Replace `crate::ui::binding::*`, `crate::ui::component::*`, `crate::ui::event_ui::*`, `crate::ui::layout::*`, `crate::ui::surface::*`, `crate::ui::template::*`, and `crate::ui::tree::*` imports in runtime behavior files with direct `zircon_runtime_interface::ui::*` imports when the symbol is a moved neutral DTO.
- [ ] Remove duplicate DTO declarations from runtime files after their runtime behavior imports interface DTOs.
- [ ] Keep runtime module roots structural. Do not add runtime `pub use` re-exports solely to preserve old neutral DTO paths.
- [x] Convert `UiRenderExtract::from_tree(tree)` call sites to runtime-owned `extract_ui_render_tree(tree)` behavior in `zircon_runtime/src/ui/surface/render/extract.rs`; the stale editor node/view projection call sites now use the runtime free function.
- [ ] Keep `layout_text(...)`, dispatchers, event managers, component registry behavior, tree mutation/query behavior, and asset compiler behavior in `zircon_runtime` while using interface DTO types.
- [ ] Rewire `zircon_runtime/src/ui/template/asset/binding/validation.rs` so M18 validation consumes interface binding expression, target, diagnostic, and report DTOs.
- [ ] Rewire `zircon_runtime/src/ui/template/asset/action_policy/validate.rs` so M21 validation consumes interface policy, side-effect, diagnostic, and report DTOs.
- [ ] Rewire `zircon_runtime/src/ui/template/asset/localization/collect.rs` so M14 collection consumes interface localization reference, direction, diagnostic, dependency, candidate, and report DTOs.
- [ ] Rewire `zircon_runtime/src/ui/template/asset/compiler/package/**` so package validation reports use interface package/action/localization/invalidation/resource DTOs at shared seams.
- [ ] Update runtime unit tests under `zircon_runtime/src/ui/tests/**` to import interface DTOs where they construct shared UI contract data.
- [ ] Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` and `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` with the split between interface DTO ownership and runtime behavior ownership.
- [ ] Update `.codex/sessions/20260502-0805-ui-runtime-interface-big-cutover.md` with touched runtime modules and any compile blockers.

**Testing stage:**

- Check target-drive space: `Get-PSDrive -Name E`.
- Run `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- Run `cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- If `cargo check` exposes an upper-layer graphics or editor failure, diagnose the lowest shared UI DTO import or behavior split first before editing the upper layer.
- Debug and correct runtime compile failures before advancing.

**Exit evidence:** Runtime library type-checks with interface DTOs; M18 public integration tests still pass; runtime no longer defines duplicate shared DTO identities for touched UI seams.

## Milestone 3: Unblock Known Runtime Lib-Test DTO Mismatches

**Goal:** Remove the concrete compile blocker that prevents M18, M21, and M14 filtered runtime lib tests from executing.

**In-scope behaviors:** Asset-font test expectations and graphics UI renderer test helpers construct the same interface DTO types accepted by production renderer paths.

**Dependencies:** Runtime behavior accepts interface DTOs from Milestone 2.

**Implementation slices:**

- [ ] Update `zircon_runtime/src/asset/tests/assets/font.rs` to import `UiTextRenderMode` from `zircon_runtime_interface::ui::surface`.
- [ ] Update `zircon_runtime/src/graphics/tests/render_framework_bridge.rs` to construct interface `UiRenderExtract` and related event/layout/surface DTOs.
- [ ] Update `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs` tests to use interface event/layout/surface DTO imports and remove `crate::ui::surface::UiResolvedStyle` fully-qualified references.
- [ ] Update `zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs` tests to use interface text enum DTOs.
- [ ] Update `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs` tests to use interface text enum DTOs.
- [ ] Update `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs` tests to use interface text enum DTOs.
- [ ] Search the touched runtime test and graphics files for `crate::ui::surface`, `crate::ui::layout`, and `crate::ui::event_ui`; remaining matches must be runtime behavior, not shared DTO construction.
- [ ] Update `.codex/sessions/20260502-0805-ui-runtime-interface-big-cutover.md` with exact files and new focused-test status.

**Testing stage:**

- Check target-drive space: `Get-PSDrive -Name E`.
- Run `cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib asset_action_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib asset_localization --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- Debug and correct the lowest shared UI DTO import if any filtered command still fails before executing its filtered tests.

**Exit evidence:** The previously blocked focused lib-test filters execute. Passing tests or remaining failures are recorded with exact output and no broad workspace-green claim.

## Milestone 4: Editor Import Audit And Documentation Cutover

**Goal:** Prove editor neutral UI contract usage routes through `zircon_runtime_interface` and document concrete runtime service dependencies that remain outside this UI DTO cutover.

**In-scope behaviors:** Editor UI code type-checks after the interface UI module hard cutover; docs clearly state which editor/runtime dependencies remain concrete services rather than neutral UI DTOs.

**Dependencies:** Milestones 1-3 have stabilized interface and runtime DTO identities.

**Implementation slices:**

- [x] Search `zircon_editor/src` for `zircon_runtime::ui` and `use zircon_runtime::ui`; the audit found 128 hits and classified the remaining imports instead of rewriting mixed runtime behavior sites mechanically.
- [x] Search `zircon_editor/src` for `zircon_runtime_interface::ui` and confirm editor UI paths already use interface DTOs broadly where neutral UI data is needed; the audit found 428 hits.
- [x] Inventory remaining `zircon_editor` imports of `zircon_runtime` into concrete runtime services and deferred mixed DTO/behavior sites in `docs/editor-and-tooling/ui-asset-editor-host-session.md`.
- [x] Do not remove `zircon_runtime` from `zircon_editor/Cargo.toml`; the inventory proves concrete runtime service imports remain deliberate.
- [x] Update `docs/engine-architecture/runtime-interface-cdylib-loader.md` so it no longer says UI contracts are path-included from runtime source and records the residual editor service dependency split.
- [x] Update `.codex/sessions/20260502-0805-ui-runtime-interface-big-cutover.md` with editor dependency status and the tree DTO/runtime behavior extension split.
- [x] Follow-up lower-layer cutover: runtime `UiSurface.tree` now owns the interface `UiTree` DTO directly. Editor files that construct `UiSurface` import `UiTree`, `UiTreeNode`, `UiInputPolicy`, and `UiTreeError` from `zircon_runtime_interface::ui::tree`, while files that call tree mutation/query behavior import the specific runtime extension traits from `zircon_runtime::ui::tree`.

**Testing stage:**

- Check target-drive space: `Get-PSDrive -Name E`.
- Run `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`.
- Run `cargo tree -p zircon_editor --locked` and record whether `zircon_runtime` remains through deliberate concrete runtime service imports.
- Debug and correct any editor neutral UI import failure before advancing.

**Exit evidence:** Editor library type-checks or any current lower-layer blocker is recorded; direct neutral tree DTO imports from runtime are removed, other neutral UI imports from runtime are either removed or explicitly recorded as behavior/service dependencies, and residual runtime dependency status is documented without claiming full runtime-interface completion while concrete services and runtime tree behavior extension traits remain.

## Milestone 5: Hard-Cut Residue Scan And Acceptance Records

**Goal:** Verify the touched UI contract namespace has no migration bridge residue and record evidence for the M18/M21/M14 unblock.

**In-scope behaviors:** Source tree has no path-include bridge in interface UI, no known stale mixed DTO imports in blocker files, and docs/session state are synchronized.

**Dependencies:** Milestones 1-4 pass their scoped testing stages or have exact unrelated blockers recorded.

**Implementation slices:**

- [ ] Search `zircon_runtime_interface/src/ui` for `#[path =`, `zircon_runtime/src/ui`, `compat`, `shim`, `facade`, `bridge`, and `legacy`; any live hit in the touched namespace blocks acceptance unless it is historical test text.
- [ ] Search runtime known mismatch files for `crate::ui::surface`, `crate::ui::layout`, and `crate::ui::event_ui`; any remaining shared DTO construction must be converted to interface imports.
- [ ] Search `zircon_editor/src` for `zircon_runtime::ui`; any remaining neutral UI import must be converted or explicitly recorded as not a UI DTO.
- [ ] Run `rustfmt --edition 2021 --check` on all touched Rust files.
- [ ] Run `git diff --check -- docs/superpowers/specs/2026-05-02-ui-runtime-interface-big-cutover-design.md docs/superpowers/plans/2026-05-02-ui-runtime-interface-big-cutover.md docs/zircon_runtime_interface/ui/mod.md docs/engine-architecture/runtime-interface-cdylib-loader.md docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md docs/editor-and-tooling/ui-asset-editor-host-session.md .codex/sessions/20260502-0805-ui-runtime-interface-big-cutover.md`.
- [ ] Update the active session note with all passing commands, remaining failures, and whether the older M18/M21/M14 notes can be archived or left blocked for a non-UI reason.

**Testing stage:**

- Check target-drive space: `Get-PSDrive -Name E`.
- Re-run the final focused command set:
  - `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo test -p zircon_runtime --lib asset_action_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo test -p zircon_runtime --lib asset_localization --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never`
  - `cargo tree -p zircon_editor --locked`
- If a broad runtime or workspace check exposes graphics/plugin `ViewportRenderFrame` drift, record it as unrelated unless this cutover touched that source path.

**Exit evidence:** All scoped UI-interface commands have fresh recorded output; docs and session note list implementation files, tests, and residual risks; no workspace-wide success is claimed without running workspace validation.

## Acceptance Checklist

- [ ] `zircon_runtime_interface::ui` uses real source modules and no `#[path = "../../../zircon_runtime/src/ui/..." ]` includes.
- [ ] Runtime shared seams use interface DTO identities for surface render extracts, UI event/layout/surface contracts, and M18/M21/M14 report DTOs.
- [ ] M18, M21, M14, and package-validation focused lib-test filters no longer fail before execution because of mixed UI DTO identities.
- [ ] Editor neutral UI imports route through `zircon_runtime_interface`.
- [ ] Remaining editor `zircon_runtime` dependency, if any, is documented as concrete runtime services or non-UI interface follow-up.
- [ ] Documentation headers list new interface files, runtime implementation files, plan sources, and tests.
- [ ] No migration-only compatibility shim, bridge, facade, legacy alias, or old-path re-export remains in the touched UI contract namespace.
