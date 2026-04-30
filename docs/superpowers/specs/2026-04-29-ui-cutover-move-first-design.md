# UI Cutover Move-First Design

## Context

The active roadmap requires two final standards before the cutover can be claimed complete.

- Editor host: business tree, layout formulas, event semantics, hit-test, and route results must come from `TOML -> UiSurface -> host projection`; Slint must remain only a generic host/bootstrap layer.
- Runtime UI: shared UI assets must support HUD, pause menu, settings dialog, and virtualized inventory/list fixtures through the engine render/input path.

Current source already contains major parts of the runtime side: visual `UiRenderExtract`, `UiRenderCommandKind`, resolved style/text/image/opacity fields, screen-space UI rendering, fixture `.ui.toml` assets, and a crate-private `RuntimeUiManager`. The remaining risk is structural: large or mixed editor host files can still hide business authority, and final acceptance needs to prove the editor and runtime paths are both using the shared contracts.

The user approved the aggressive policy: move files first, then fix the compile and behavior breakage.

## Approved Direction

Use a move-first cutover.

The first implementation slice should reshape the file tree aggressively before behavior fixes. It should prefer pure moves, import rewiring, module re-exports, and source guard updates over incremental compatibility. Compile failures from the move are expected and should drive the repair order.

No long-term backward-compatibility path is allowed. Temporary aliases are allowed only inside the same repair slice and must be removed before final acceptance.

## Scope

This milestone owns the overlapping runtime graphics and runtime UI showcase scope. Active session notes are coordination inputs, not blockers.

In scope:

- Editor Slint host file movement and import repair.
- Oversized/mixed Slint and Rust test file decomposition when touched.
- Generic host boundary source guards after the move.
- Runtime UI manager, fixture, visual extract, and input/acceptance gaps.
- Graphics screen-space UI pass acceptance and any needed renderer contract repair.
- Documentation updates under `docs/ui-and-layout`.
- Final cleanup of old file paths, unused Slint primitives, control-specific glue, and legacy geometry seams that still serve the old hand-written shell.

Out of scope for this milestone:

- World-space UI.
- Full visual theme editor.
- Rich text and deep IME support.
- A second UI renderer that bypasses `UiRenderExtract`.
- Preserving old workbench-specific file paths as public API.

## Architecture

### Ownership

- `zircon_runtime::ui` owns template assets, shared UI trees, layout, hit-test, focus, input dispatch, runtime UI fixtures, and `UiRenderExtract`.
- `zircon_runtime::graphics` owns screen-space UI rendering and consumes only the shared draw list and asset/font references. It must not learn editor docking, workbench, pane, or menu semantics.
- `zircon_editor` owns editor state and host projection. Its Slint files should consume projected data and transparent input frames only.
- Docs and tests must describe the shared contract, not the old Slint business tree.

### Target Editor Slint Shape

The current `zircon_editor/ui/workbench.slint` may remain only as the first-move build entry while imports are repaired. It should export `UiHostWindow` from generic host files rather than importing from a workbench-owned component folder. By final cleanup, the build entry must either be renamed to a generic host file or reduced to a guarded no-business shim that is not treated as public API.

Target layout:

- `zircon_editor/ui/host/window/`: host bootstrap, context, root presentation DTOs, scene assembly, scaffold, resize layer, drag overlay, and native-window switching.
- `zircon_editor/ui/host/surface/`: menu chrome, page chrome, side/document/bottom dock surfaces, status bar, floating-window layer, and native floating-window surface.
- `zircon_editor/ui/host/primitives/`: generic visual primitives still needed by `TemplatePane` or transparent host inputs.
- `zircon_editor/ui/template/`: `TemplatePane`, template node DTOs, collection-field rows, and template popup rendering.
- `zircon_editor/ui/pane/`: pane DTOs, pane surface, pane host context, pane content, and pane-specific native slot files.
- `zircon_editor/ui/assets/`: asset-browser and asset-reference Slint files.

The first move may keep component names stable to reduce generated ABI churn. The path authority changes first; component and DTO names are tightened only where a source guard or compile failure requires it.

### Target Rust Editor Shape

Host projection and Slint adapter files should follow the same boundary. The move-first slice should rename the Rust projection folder before deeper behavior repair.

- `zircon_editor/src/ui/layouts/windows/workbench_host_window/` should move to `zircon_editor/src/ui/layouts/windows/host_window/` during the structural move.
- Any old Rust module path is temporary repair scaffolding only. It must be removed before final acceptance unless the build system still needs a one-line re-export that contains no behavior.
- Root files in that tree should stay structural. If a file already mixes chrome projection, pane projection, floating projection, and layout recovery, new code should be moved into folder-backed children before behavior fixes are added.
- Large tests such as Slint host conversion tests should be split by acceptance domain: generic host boundary, pane projection, floating-window projection, drawer/source projection, and runtime UI fixture integration.

### Runtime UI Shape

The runtime path stays crate-private unless an acceptance test needs a public engine-facing facade.

- `RuntimeUiManager` remains under `zircon_runtime::ui` and loads fixture assets through `UiAssetLoader -> UiDocumentCompiler -> UiTemplateSurfaceBuilder -> UiSurface`.
- Runtime fixtures remain under `zircon_runtime/assets/ui/runtime/fixtures/` and must all produce non-trivial shared surfaces.
- Runtime input acceptance should enter through the same shared surface/pointer/navigation concepts used by editor host projection, not through a graphics-specific or fixture-specific route table.

### Graphics Shape

The screen-space UI pass remains renderer-owned.

- `UiRenderExtract` is the only graphics input for UI.
- The pass records quads, borders, images/icons, text batches, clips/scissors, opacity, and z ordering from shared commands.
- Renderer tests should validate output and stats without depending on editor host file paths.

## Move-First Execution Model

1. Publish coordination state for absorbing overlapping sessions.
2. Move editor Slint files to the target folders and update imports/exports.
3. Move or split oversized/mixed tests and supporting Rust files touched by the move.
4. Run narrow compile checks to expose broken imports, generated Slint ABI failures, and stale path assumptions.
5. Fix compile failures without restoring old paths as permanent compatibility.
6. Restore source guards so they assert the new folder layout and the absence of business authority in Slint.
7. Re-run editor host suites, then runtime UI suites, then graphics/render fixture acceptance.
8. Delete temporary aliases, unused legacy primitives, and old glue once all gates pass.

## Data Flow

Editor flow:

`editor .ui.toml assets -> shared UiSurface -> Rust host projection -> TemplatePaneNodeData and transparent input frames -> generic Slint host -> UiHostContext callbacks -> shared pointer/router dispatch`

Runtime flow:

`runtime .ui.toml fixtures -> RuntimeUiManager -> UiSurface -> UiRenderExtract -> ViewportRenderFrame.ui -> screen-space UI renderer -> captured frame/stats acceptance`

Both flows share the same render contract. Neither path should reinterpret `.ui.toml` visual fields in a host-specific way after `UiSurface` has produced the render extract or projection payload.

## Error Handling

- Missing fixture assets must return typed runtime UI manager errors rather than panic.
- Invalid template/layout contracts must fail at loader/compiler/builder boundaries before rendering.
- Slint import failures after moves are fixed by updating imports and exports, not by duplicating old files.
- Renderer failures are diagnosed from shared draw-list commands and UI submission stats before changing graphics internals.
- If an upper-layer editor acceptance test fails, first check shared support layers: template build, layout, projection, pointer route, render extract, then Slint host forwarding.

## Testing And Acceptance

Editor acceptance:

- Source guards confirm `workbench.slint` is bootstrap-only and imports generic host paths.
- Source guards confirm menu, popup, status bar, floating headers, activity rails, dock headers, pane chrome, and remaining business visuals are projected `TemplatePane` nodes or transparent input frames.
- Slint host tests cover root shell, drawer, document dock, floating windows, menu/popup, dialog/transient overlay, pane content, scroll/list/tree input, tab drag, resize, and native floating mode.

Runtime acceptance:

- All four fixtures build into shared surfaces: HUD overlay, pause menu, settings dialog, inventory/list.
- Runtime visual tests cover quad, border, text, image/icon, clip/scissor, opacity, z order, wrapped text, and clipped text.
- Runtime input tests cover mouse, keyboard/focus, gamepad-style navigation through the shared navigation dispatcher, scroll, popup dismiss, dialog default/cancel, capture/block/passthrough, and focus fallback.
- Graphics integration tests prove UI extracts submit through the renderer and produce visible captured output or stable render-list/stat golden evidence.

Performance acceptance:

- Static UI updates should not force full layout for unrelated nodes.
- Long list scrolling should not require rendering all offscreen rows.
- Screen-space UI pass should expose stable command/draw/text stats so regressions can be detected without screenshot-only evidence.

Final gate:

- `cargo build --workspace --locked --verbose`
- `cargo test --workspace --locked --verbose`
- Focused editor/runtime/graphics tests with isolated target dirs when broad commands are blocked by the Windows environment.
- Targeted `rustfmt --edition 2021 --check` for touched Rust files.
- Targeted `git diff --check` for touched files.

## Documentation

Update `docs/ui-and-layout/shared-ui-template-runtime.md` with:

- New file ownership and moved paths.
- Runtime fixture and graphics pass evidence.
- Removed legacy paths and source guards.
- Commands used for acceptance.

Update `docs/ui-and-layout/index.md` if the navigation summary changes.

## Reference Alignment

- Fyrox separates UI widgets/resources from renderer ownership, which supports keeping `zircon_runtime::ui` and `zircon_runtime::graphics` separate.
- Godot keeps GUI controls in separate files under `scene/gui`, which supports splitting large editor Slint host surfaces by component family.
- Unity Graphics keeps screen-space UI as an explicit render pass, which matches the current `ScreenSpaceUiRenderer` boundary.
- Slint compiler separates parser, object tree, passes, and generator folders, which supports keeping root Slint/Rust files as structural wiring rather than behavior holders.

## Review Gate

Implementation should not start until this design is reviewed. After review, the next artifact is a detailed implementation plan with ordered move, compile-repair, test-repair, cleanup, and acceptance steps.
