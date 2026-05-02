# UI Runtime Interface Big Cutover Design

## Goal

Unify three in-flight lanes into one hard cutover:

- Runtime-interface M2 editor cutover.
- M18 runtime binding semantics.
- M21 action policy and M14 localization runtime foundations.

The result should make `zircon_runtime_interface::ui` the canonical neutral UI contract namespace used by runtime, editor, app, and future plugin consumers. Runtime remains the implementation owner for compiling, validating, and executing UI assets; editor remains the authoring/projection owner. The cutover removes duplicate UI DTO identities that currently block focused M18 and M21/M14 lib-test execution.

## Current Evidence

- Runtime interface audit classifies `zircon_app` as converged, and `zircon_runtime` / `zircon_editor` as needing refactor because of large production files, not missing module owners.
- `zircon_runtime_interface::ui` currently uses `#[path = "../../../zircon_runtime/src/ui/..." ]` to include runtime UI modules. This creates a shared consumer namespace but still leaves runtime-local `crate::ui::*` and interface `zircon_runtime_interface::ui::*` as distinct Rust type identities in mixed test and editor paths.
- M18 has landed runtime-owned `zircon_runtime/src/ui/template/asset/binding/` with public integration evidence, but runtime lib-test filters are blocked by unrelated UI DTO identity mismatches.
- M21/M14 have landed runtime-owned `action_policy/` and `localization/` foundations, but their focused lib-test filters hit the same DTO identity blocker before test execution.
- Runtime-interface M2 has a minimal `EditorRuntimeClient` and app/editor startup path. Its remaining work is a technical cutover: separate neutral contracts from concrete runtime services.

## Architecture Decision

`zircon_runtime_interface` becomes the canonical owner for neutral UI DTO declarations that cross crate, dynamic-library, editor, app, or plugin boundaries. `zircon_runtime` must import those DTOs rather than defining parallel public DTO identities for cross-boundary seams. Concrete runtime behavior remains in `zircon_runtime`.

This means:

- Neutral contract modules move to real `zircon_runtime_interface/src/ui/**` files instead of path-including runtime modules.
- Runtime-owned algorithms may stay in `zircon_runtime/src/ui/**`, but their public inputs/outputs at shared seams use interface types.
- Editor imports neutral UI asset, binding, action policy, localization, component, dispatch, event, layout, surface, template, and tree DTOs from `zircon_runtime_interface::ui`.
- Runtime compiler/package/action/localization behavior remains under `zircon_runtime::ui::template::asset::*` as the semantic authority.
- The old path-inclusion approach is removed in the touched UI contract namespace rather than preserved as a compatibility bridge.

## Boundary Ownership

- `zircon_runtime_interface`: ABI-safe handles, buffers, status, runtime/plugin API tables, and neutral UI DTO declarations. It must not depend on `zircon_runtime`, `zircon_editor`, Slint, wgpu, or plugin implementation crates.
- `zircon_runtime`: runtime module lifecycle, asset compiler, package validation, binding validation, action policy validation, localization collection, runtime UI execution, graphics integration, and conversions from interface DTOs into runtime-only behavior where necessary.
- `zircon_editor`: authoring state, inspector/projection, session replay, Slint host glue, and `EditorRuntimeClient` use of interface handles/contracts.
- `zircon_app`: process host and runtime cdylib loader. It does not regain optional runtime/editor implementation ownership.

## Cutover Shape

The cutover is layered bottom-up:

1. Freeze the canonical UI contract inventory.
   - Inventory every neutral type currently consumed as `zircon_runtime_interface::ui::*` and every runtime-local `crate::ui::*` type that appears in DTO identity compile errors.
   - Classify each item as interface DTO, runtime implementation behavior, editor authoring state, or graphics/plugin implementation detail.

2. Materialize interface-owned UI modules.
   - Create folder-backed `zircon_runtime_interface/src/ui/{binding,component,dispatch,event_ui,layout,surface,template,tree}/` modules as needed.
   - Place M18/M21/M14 asset contract declarations under `zircon_runtime_interface/src/ui/template/asset/{binding,action_policy,localization}/` so package reports and editor projections share one DTO identity.
   - Put declarations and serde-compatible contract helpers there.
   - Keep roots structural and avoid path-include bridges.

3. Rewire runtime shared seams.
   - Update runtime public/cross-boundary UI surfaces and tests to consume interface DTO types.
   - Keep runtime behavior files in runtime modules, but accept or emit interface DTOs at package/report/extract/event boundaries.
   - Fix M18/M21/M14 test blockers by removing mixed interface/runtime DTO identities, not by adding conversion shims.

4. Rewire editor source imports.
   - Replace neutral UI imports from `zircon_runtime` with `zircon_runtime_interface`.
   - Keep concrete runtime service access behind `EditorRuntimeClient` or later serialized runtime commands.
   - Do not broaden editor work into unrelated resource UX, graphics/plugin renderer, or Slint source restoration.

5. Remove migration residue.
   - Delete path-include wiring from `zircon_runtime_interface/src/ui/mod.rs` once the equivalent real modules exist.
   - Search for old direct runtime neutral DTO imports in editor and mixed runtime/interface DTO use in runtime tests.
   - Do not keep `pub use`, shim, compat, facade, bridge, or legacy modules just to preserve old paths.

## M18 Binding Semantics Integration

M18 stays runtime-owned semantically. The interface cutover only changes where neutral declaration types live.

- `UiBindingRef`, target assignments, binding expressions, binding diagnostics, and binding report DTOs are interface contracts when authored, serialized, projected, or returned across editor/runtime/plugin seams.
- Runtime validation remains in `zircon_runtime::ui::template::asset::binding::validation` or a focused runtime behavior module that consumes interface DTOs.
- Editor binding inspector consumes interface diagnostics and schema DTOs. It does not invent a separate editor binding semantic model.

## M21 Action Policy Integration

M21 remains a runtime validation surface.

- Side-effect classes, host policy DTOs, action policy diagnostics, and report rows live under `zircon_runtime_interface::ui::template::asset::action_policy` when they appear in package reports or editor policy inspectors.
- Runtime validation chooses the policy result during package/compile validation.
- Editor policy inspector is a later productization consumer; it must consume interface report data instead of runtime implementation types.

## M14 Localization Integration

M14 remains a runtime asset/package foundation.

- Localized text refs, text direction, localization dependencies, diagnostics, and extraction candidates live under `zircon_runtime_interface::ui::template::asset::localization` when serialized in assets, package reports, or editor preview/productization surfaces.
- Runtime collection/validation owns fallback and dependency rules.
- Editor locale preview/extraction UI remains later productization and consumes the interface report shape.

## Runtime-Interface M2 Integration

The editor dependency cutover advances only after neutral DTO identity is stable.

- Keep `zircon_runtime_interface` as the editor's contract dependency.
- Keep `zircon_runtime` out of editor for neutral UI data.
- Concrete runtime world, module, asset loading, and renderer services remain inaccessible except through `EditorRuntimeClient` handles, serialized commands, or future capability APIs.
- Do not remove `zircon_runtime` from `zircon_editor/Cargo.toml` until imports prove the remaining uses are concrete runtime service dependencies that have either been routed or deliberately deferred.

## Reference Alignment

- Current `zirconEngine` plans lead: fixed root packages are `zircon_app`, `zircon_runtime`, and `zircon_editor`; shared neutral contracts belong in a framework/interface layer rather than editor or graphics implementation code.
- Fyrox and Godot-style editor/runtime separation supports keeping authoring state in editor while runtime owns asset/package validation.
- Bevy-style data-oriented contracts support explicit shared DTO crates and systems that consume data rather than concrete editor/runtime objects.
- Slint remains a toolkit integration detail; no Slint objects cross the runtime-interface boundary.

## Validation Plan

Milestone testing must be staged and serial with `--locked`:

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

If graphics/plugin `ViewportRenderFrame` drift appears during broad runtime or workspace validation, record it as unrelated unless the cutover touched that source path. Do not claim workspace green without fresh workspace validation.

## Acceptance Criteria

- `zircon_runtime_interface::ui` contains real interface-owned modules for the touched neutral UI contract surface, not path-included runtime modules.
- Runtime, editor, and runtime tests use one type identity for `UiRenderExtract`, UI event/layout/surface DTOs, and M18/M21/M14 report DTOs at shared seams.
- M18 and M21/M14 focused tests no longer fail before execution due to mixed interface/runtime DTO identities.
- Editor neutral UI imports are routed through `zircon_runtime_interface` unless a remaining concrete runtime service dependency is explicitly documented.
- Docs and active session notes record the cutover evidence and any deferred concrete runtime-service dependencies.
- No migration-only compatibility shim, bridge, facade, or old-path re-export remains in the touched UI contract namespace.

## Out Of Scope

- Full graphics/plugin renderer cutover.
- Slint source restoration or new `.slint` business authority.
- Full M5/M6/M24 editor recovery/designer implementation.
- Full M22 parity fixture implementation.
- Removing every `zircon_editor` dependency on `zircon_runtime` if remaining imports are concrete runtime service behavior outside neutral UI DTOs; those must be documented and routed through later client APIs.

## Risks

- Moving all neutral UI DTOs at once can expose many imports. Mitigation: inventory first, then migrate one folder-backed contract family at a time while keeping the single canonical destination.
- Some runtime modules may mix declarations and behavior in the same file. Mitigation: move declarations to interface and leave behavior in runtime, splitting large files when the touched file already carries multiple responsibilities.
- Interface crate must stay dependency-light. Mitigation: reject any move that pulls Slint, wgpu, editor, runtime, or plugin implementation crates into `zircon_runtime_interface`.
