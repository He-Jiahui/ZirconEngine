# UI Foundation M10/M12/M13 Design

## Summary

This spec covers the next UI asset foundation batch from `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`: M13 widget descriptor registry/capabilities, M10 component public contract/API/private boundary, and M12 incremental compile/invalidation/performance.

The batch stays inside shared Runtime UI asset/template/component authority. `zircon_runtime::ui` owns the reusable data model, validation, descriptor registry, component contract, invalidation graph, compiler cache keys, and diagnostics. `zircon_editor` consumes runtime-owned registry views and authoring DTOs for the UI Asset Editor palette and related authoring projection. This batch deliberately avoids graphics/GI/VG, Runtime UI showcase data-source work, physics/animation, workspace watcher work, and unrelated editor host cleanup.

The implementation order is M13 first, then M10, then M12. M13 establishes descriptor authority. M10 builds component public contracts and privacy validation on top of descriptor/asset authority. M12 then models invalidation/cache dependencies after descriptor and contract revisions are explicit.

## Goals

- Formalize existing `UiComponentDescriptor` and `UiComponentDescriptorRegistry` as the V1 widget descriptor authority instead of adding a parallel registry.
- Add host capabilities, render capabilities, palette metadata, default node templates, fallback policy, registry revisioning, and descriptor validation.
- Make the UI Asset Editor palette descriptor-driven while preserving separate local-component and imported-widget-reference palette sources.
- Extend UI asset component definitions with public contracts: `api_version`, params, slots, public parts, root class policy, focus contract, and binding contract.
- Enforce closed component privacy by default. External documents may target only a reference root or explicitly exported public parts.
- Add an asset/template-owned `UiInvalidationGraph` and compiler cache path that models parse, shape, import, descriptor, contract, selector, style, layout, render, interaction, and projection invalidation.
- Keep `UiDirtyFlags` as lower-level runtime tree state and map asset/compiler invalidation into those flags rather than replacing them.
- Preserve milestone-first validation: implementation slices can add tests and docs, while compile/build/unit-test execution belongs to each milestone testing stage.

## Non-Goals

- Do not implement full theme/token governance M11 beyond the invalidation hooks needed by M12.
- Do not implement localization M14, media/font/resource refs M15, compiled artifact/package validation M16, action safety/host policy M21, or runtime/editor dual-host parity M22 in this batch.
- Do not change Runtime UI showcase retained-state or real data-source adapter ownership.
- Do not touch graphics/GI/VG, physics/animation, render plugin, or unrelated editor watcher/chrome areas.
- Do not introduce compatibility shims or a second widget registry that must coexist with `UiComponentDescriptorRegistry`.

## Current Baseline

- `zircon_runtime/src/ui/component/descriptor.rs` defines component descriptors, prop schema, slot schema, option descriptors, events, drop policy, and schema lookup helpers.
- `zircon_runtime/src/ui/component/catalog.rs` owns the current `UiComponentDescriptorRegistry` and the `editor_showcase()` built-in descriptor set.
- `zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs` already stores a `UiComponentDescriptorRegistry` and defaults to `editor_showcase()`.
- `zircon_runtime/src/ui/template/asset/compiler/component_props.rs` applies descriptor defaults and validates typed component props.
- `zircon_runtime/src/ui/template/asset/document.rs` already models local component params, slots, `style_scope`, tree nodes, reference nodes, bindings, style rules, and tree authority helpers.
- `zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs` validates named slot fills and applies param defaults for local/imported components.
- `zircon_runtime/src/ui/tree/node/dirty_flags.rs` contains runtime tree dirty flags, including `visible_range`.
- `zircon_runtime/src/ui/surface/surface.rs` still rebuilds hit-test and render extract after layout.
- `zircon_editor/src/ui/asset_editor/tree/tree_editing.rs` still hardcodes native palette entries and owns palette insertion types beside broader tree editing behavior.

## Reference Evidence

- Slint LSP palette support builds palette entries from compiler document/type registry state and lazily caches the result, rather than hardcoding every authoring entry in the editor UI. Relevant local reference files: `dev/slint/tools/lsp/preview/ui/palette.rs`.
- Bevy uses explicit change-detection filters such as `Changed<T>`, `Added<T>`, and changed resources to narrow recompute and UI style updates. Relevant local reference files: `dev/bevy/examples/ecs/change_detection.rs` and `dev/bevy/examples/ui/widgets/standard_widgets.rs`.
- Fyrox editor inspector flows build property UI through registered editor definitions and inspector context, which supports descriptor-driven authoring instead of hardcoded per-widget UI. Relevant local reference file: `dev/Fyrox/project-manager/src/settings.rs`.

The Zircon divergence is intentional: the UI asset system is document/template-first rather than ECS-first, so invalidation is modeled as asset/compiler stages that later map into runtime tree dirty flags. Descriptor and contract authority remain in `zircon_runtime::ui` so editor and runtime consumers share one schema surface.

## Architecture

### Owner Boundary

The owning crate/module is `zircon_runtime::ui` as a supporting runtime module under the current converged `zircon_runtime` package. The editor may consume runtime DTOs and registry views, but must not own the canonical widget descriptor, component contract, or invalidation rules.

The public surfaces are descriptors, contracts, validation reports, compile cache keys, and authoring DTOs. No upper layer should reach into concrete compiler internals when a registry view or contract report can carry the data.

### M13 Descriptor Authority

`UiComponentDescriptor` remains the descriptor root. The implementation should reorganize descriptor-related declarations into a folder-backed module and add these concepts:

- `UiHostCapability`: host capability enum covering `editor`, `runtime`, `text_input`, `pointer_input`, `keyboard_navigation`, `gamepad_navigation`, `image_render`, `canvas_render`, and `virtualized_layout`.
- `UiRenderCapability`: render capability enum for visual output requirements such as text, image, icon/vector, clipping, scroll, and virtualized layout.
- `UiPaletteMetadata`: display/category/icon/sort/default-template metadata used to build UI Asset Editor palette entries.
- `UiDefaultNodeTemplate`: descriptor-owned default authored node shape used for quick insert and drag/drop insertion.
- `UiWidgetFallbackPolicy`: editor preview fallback and runtime fallback/reject policy when host capabilities are missing.
- Descriptor validation that rejects duplicate prop/state/slot names, missing prop schema for default props, invalid capability/fallback combinations, missing palette templates for palette-visible widgets, and invalid slot/default-node references.
- Registry revisioning so M12 cache keys can include descriptor changes.

The registry remains deterministic and category-aware. The UI Asset Editor palette consumes descriptors filtered by host capability, then appends local component entries and imported widget reference entries from the current document/import set.

### M10 Component Public Contract

`UiComponentDefinition` gains a formal public contract while preserving the existing `params` and `slots` fields as authored contract data.

The V1 contract includes:

- `api_version`: semantic version string, defaulting to `1.0.0`, parsed into `UiComponentApiVersion` for validation.
- public params and slots: existing `params` and `slots` remain the source, with typed validation expanded as needed.
- public parts: explicit `part -> node_id` exports. No node is public by default.
- root class policy: the root may accept appended classes by default, but internal class/style targeting remains closed unless a public part is exported.
- focus contract: root focusability, optional initial focus target, and named public navigation targets.
- binding contract: public action/event names and payload schema placeholders that are validated as structure, not as full M18 expression semantics.

Closed component privacy is the default. Parent documents can reference the component root, pass params, fill slots, append allowed root classes, and use declared public parts. Parent documents cannot directly target private internal `node_id` or `control_id` values through selectors, binding targets, focus targets, or source outline deep links.

API compatibility is semantic-version based:

- Patch changes are compatible when public names and required/default semantics do not change.
- Minor changes are compatible when they add optional params, optional slots, public parts, or public binding outputs.
- Major mismatches are rejected.

### M12 Invalidation And Compiler Cache

`UiInvalidationGraph` belongs under runtime UI asset/template ownership. It models invalidation stages as a reusable graph, not editor-specific heuristics.

Required stages:

- `SourceParse`
- `DocumentShape`
- `ImportGraph`
- `DescriptorRegistry`
- `ComponentContract`
- `SelectorMatch`
- `StyleValue`
- `Layout`
- `Render`
- `Interaction`
- `Projection`

Compiler cache keys include document fingerprints, registered widget/style import fingerprints, descriptor registry revision, and component contract revisions. Cache outcomes must report whether a compile was reused, rebuilt, and which invalidation stages were responsible.

`UiDirtyFlags` remain the lower-level runtime surface/tree flags. M12 adds a mapping from invalidation stages to runtime dirty consequences. For example, `Layout` maps to layout/hit-test/render dirty state, `Render` maps to render dirty state, `Interaction` maps to input-related dirty state, and `Projection` remains editor-only authoring projection state.

Large-document diagnostics belong to the invalidation/performance layer. Thresholds should be named constants in the local invalidation diagnostics module, not scattered numeric literals.

## Module Shape

Descriptor and catalog work should be folder-backed before adding new responsibilities:

- `zircon_runtime/src/ui/component/descriptor/` owns descriptor declarations and descriptor validation helpers.
- `zircon_runtime/src/ui/component/catalog/` owns registry storage, built-in descriptor construction, registry validation, registry views, and revisioning.
- `zircon_runtime/src/ui/template/asset/component_contract/` owns public component contract declarations and validation.
- `zircon_runtime/src/ui/template/asset/invalidation/` owns invalidation stages, graph, impacts, fingerprints, diagnostics, and reports.
- `zircon_runtime/src/ui/template/asset/compiler/cache/` owns compile cache keys, entries, and cache outcomes.
- `zircon_editor/src/ui/asset_editor/palette/` owns palette entries, descriptor-driven palette construction, default-node instantiation, placement, and insertion helpers.

Existing `mod.rs` and crate root files stay structural. If a file starts mixing declarations, parsing, validation, and behavior families, split before adding the next responsibility.

## Error Handling

Prefer structured error/report types over matching strings in tests or editor code.

Expected additions include:

- Descriptor validation errors for invalid capability, palette, fallback, prop, state, or slot declarations.
- Component contract errors for invalid semantic version strings, private target access, unknown public parts, and API incompatibility.
- Invalidation/cache reports for cache hits, rebuild causes, missing fingerprints, large-document diagnostics, and unsupported incremental paths.

Existing `UiAssetError::InvalidDocument` can remain the compiler-facing error envelope when a new public error would be too invasive, but lower-level validators should expose typed reasons for tests and future editor diagnostics.

## Testing Design

M13 focused tests:

- descriptor validation rejects duplicate/missing/invalid declarations.
- host capability filtering hides unsupported descriptors and reports missing capabilities.
- descriptor-driven palette builds runtime native entries without hardcoded editor widget lists.
- default node template instantiation produces stable `UiNodeDefinition` values.

M10 focused tests:

- component contract defaults to closed privacy and `1.0.0`.
- external selectors/bindings/focus targets reject private imported component internals.
- public part exports allow the corresponding structured selector or target.
- reference API version accepts compatible patch/minor and rejects major mismatch.

M12 focused tests:

- invalidation graph maps document/descriptor/contract/style/layout changes to expected stages.
- compiler cache reuses identical document/import/registry/contract fingerprints.
- descriptor or contract revision changes invalidate the cache.
- large-document diagnostics flag non-virtualized large scroll/list shapes and repeated broad selector work.

Compile/build/unit-test execution belongs to each milestone testing stage. Implementation slices may add test code and docs before that stage.

## Documentation Updates

Implementation must update or create docs under `docs/` with machine-readable headers.

Required documentation work:

- Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` with descriptor/contract/invalidation links and validation evidence.
- Create or update a dedicated UI foundation detail document such as `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` to explain M13/M10/M12 behavior.
- Update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md` after implementation and validation evidence exists.
- Keep `docs/superpowers/plans/2026-05-01-ui-foundation-m10-m12-m13.md` and this spec listed as plan sources in affected module docs.

## Acceptance Criteria

- M13: runtime descriptor registry validates descriptors, exposes capability-filtered palette metadata, records registry revision, and editor palette native entries come from descriptors.
- M10: component public contracts parse, validate, and reject private-boundary/API-version violations before expansion leaks internals.
- M12: invalidation graph and compile cache report hit/rebuild causes using document/import/descriptor/contract fingerprints and map accepted stages to runtime dirty impacts.
- Docs record owner modules, implementation files, tests, validation commands, and remaining gaps.
- Active-session boundaries are respected: no graphics/GI/VG, Runtime UI showcase data-source, physics/animation, or watcher changes are included.
