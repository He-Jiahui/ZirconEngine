---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/runtime_ui
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/mod.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/layout/pass
  - zircon_runtime/src/ui/layout/pass/pipeline.rs
  - zircon_runtime/src/ui/layout/pass/layout_tree.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime/src/ui/tree/node/scroll.rs
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/route_authority.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_capture.rs
  - zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs
  - zircon_runtime/src/ui/surface/pointer
  - zircon_runtime/src/ui/surface/navigation
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/ui/surface/render/collection_rows/table.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/pipeline.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/event_ui/mod.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/v2/mod.rs
  - zircon_runtime/src/ui/v2/cache.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/tests/ui_v2_contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime_interface/src/ui/v2/mod.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
implementation_files:
  - docs/zircon_runtime/ui/architecture.md
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/route_authority.rs
  - zircon_runtime/src/ui/surface/input/navigation.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_capture.rs
  - zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/table.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/layout/pass/pipeline.rs
  - zircon_runtime/src/ui/layout/pass/layout_tree.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/mod.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/tree/node/scroll.rs
  - zircon_runtime/src/ui/v2/cache.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/tests/ui_v2_contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/pipeline.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
plan_sources:
  - user: 2026-06-13 runtime architecture implementation request
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/ui-and-layout/shared-ui-template-runtime.md
  - CLAUDE.md
tests:
  - runtime_09_m0_ui_architecture_static_passed
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_ui_architecture_doc_records_current_boundaries
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_ui_architecture_baselines_match_current_source_scan
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_v2_verdict_matches_runtime_and_interface_modules
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_ui_input_events_route_through_single_dispatch_authority
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_taffy_layout_pass_order_uses_bridge_authority
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_virtualization_scroll_boundary_records_invalidation_authority
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_template_pipeline_boundary_records_compile_instance_validate_authority
  - zircon_runtime::ui::tests::scroll_virtualization::virtualized_list_only_materializes_visible_window
  - zircon_runtime::ui::tests::scroll_virtualization::scroll_offset_invalidates_virtualization_window
  - zircon_runtime::ui::tests::scroll_virtualization::non_virtualized_scroll_offset_keeps_full_window_dirty_domain
  - zircon_runtime::ui::tests::template_pipeline::template_validate_rejects_unknown_component_contract
  - zircon_runtime::ui::tests::template_pipeline::template_instance_failure_surfaces_loader_error
  - zircon_runtime::ui::tests::template_pipeline::compiled_template_artifact_stays_binary_leaf_dto_not_generated_source
  - ui_architecture_boundary targeted audit
  - docs/zircon_runtime/ui/v2.md
  - docs/zircon_runtime/ui/dispatch/input_manager.md
  - docs/zircon_runtime/ui/layout/pass.md
  - docs/zircon_runtime/ui/surface/default_interactions.md
  - docs/zircon_runtime/ui/template/pipeline.md
  - docs/zircon_runtime/ui/template/asset/dependency_index.md
doc_type: module-detail
---

# Runtime UI Architecture M0 Boundary

runtime_09_m0_ui_architecture_static_passed

本文件完成 Runtime 09 的 M0.1 模块边界图与 M0.2 v2 双代裁决，并记录 M1.1 输入路由单点权威化、M1.2 导航回复、pointer 回复、pointer capture fallback、table row-label fallback、template component-name fallback、property visibility flag、responsive MUI visibility flag、accessibility open-state fallback、layout engine backend name 与 surface default interaction fallback 命名收敛、M2.1 Taffy 桥接与 layout pass 顺序权威化、M2.2 virtualization/scroll 边界声明、M3.1 template compile/instance/validate pipeline 边界切片。当前 UI 生产代码的 legacy 命名桶已清零；完整 UI behavior filters 仍必须等待 owner 空窗或重新协调。

## Owner Verdict

`zircon_runtime::ui` owns runtime-only UI behavior: layout passes, dispatch, render extraction inputs, text/layout engines, template compilation, surface/tree mutation, runtime v2 prototype loading, v2 style resolution, and v2 surface construction.

`zircon_runtime_interface::ui` owns neutral UI contract DTOs. Its `ui::v2` surface is the stable data schema layer for arenas, asset records, compiled graphs, repeat expansion records, and style DTOs. It must not own runtime mutation, route ordering, layout pass execution, cache invalidation, or render extraction.

`zircon_editor::ui` owns editor workbench authoring and retained-host consumption. It can consume v2 assets and runtime projection APIs, but it cannot define runtime UI route authority, layout backend ownership, or template compilation behavior.

## Module Boundary Map

Current scan baseline:

- `ui/` top-level entries: 17 = 15 directories plus `module.rs` and `style.rs`.
- `surface/` entries: 20 in the current worktree scan.
- Full UI-tree `legacy` hits: `ui_legacy_hits=54`.
- Production UI `legacy` hits/files after excluding tests and fixtures: `ui_legacy_production_hits=0` / `ui_legacy_production_files=0`.
- Production UI `taffy` hits/files after excluding tests and fixtures: `ui_taffy_production_hits=173` / `ui_taffy_production_files=9`.

| Module | Runtime owner | Boundary note |
|---|---|---|
| `module.rs` | Runtime UI module declaration | Module descriptor/config wiring only. |
| `runtime_ui/` | Runtime preview and fixture manager | Crate-private runtime UI manager path; consumes v2 cache/building and surface projection. |
| `layout/` | Constraints, pass sequence, scroll, style mapping, Taffy bridge, virtualization | Owns layout execution and backend adaptation. M2.1 makes `taffy_bridge/compute.rs` the only Taffy tree-build/compute owner, `pass/pipeline.rs` the authoritative pass-order owner, and leaves `style_mapping.rs` as a DTO adapter rather than a layout backend executor. M2.2 makes `layout/scroll.rs::UiScrollVirtualizationPlan` / `plan_scrollable_virtual_window(...)` the owner for scroll offset clamping, viewport/content invalidation, and virtual-window dirty decisions while `layout/virtualization.rs` remains pure window math. |
| `surface/` | Retained surface state and runtime interaction state | Owns arranged output, hit testing, focus, popup stack, input state, component state, property mutation, default interactions, reflection snapshots, render collection data, timeline, and diagnostics. |
| `dispatch/` | UI route manager | Owns the route authority entry point documented by `UiInputManager`: capture, popup, preview, target, bubble, focus, default action. |
| `template/` | Asset/build/instance/loader/validate/pipeline boundary | Owns template compilation, validation, dependency indexing, hot reload coordination, and old template migration surfaces. M3.1 records `UiTemplateRuntimePipeline` as the thin `load -> validate -> instance -> build` owner boundary while v2 remains the replacement mainline. |
| `v2/` | Runtime v2 loader, compiler, prototype cache, style resolver, surface builder, surface tree | Runtime implementation of the v2 schema. It consumes `zircon_runtime_interface::ui::v2` DTOs and creates runtime surfaces. |
| `component/` | Component descriptor catalog and state reducers | Owns runtime catalog metadata and component state reduction, including editor shell component entries that are still runtime projection data. |
| `tree/` | Runtime tree extensions and hit-test helpers | Utility owner for UI node tree traversal and hit-test integration. |
| `binding/` | Binding event router and update report | Runtime binding/event bridge into surface mutation reports. |
| `event_ui/` | UI event manager | Runtime UI event manager layer. `UiBindingCodec` is an interface-owned contract type after Runtime 10 M2.1. |
| `style.rs` and `theme/` | Typed style fields and active theme registry | Owns runtime style resolution inputs and theme token registry over v2 resolved style DTOs. |
| `text/` | Crate-private text layout support | Runtime internal helper; broader text stack ownership remains Runtime 01 M2. |
| `accessibility/` | Runtime accessibility extraction | Reads surface state into accessibility output; Runtime 09 M1.2 names the open-state compatibility list as `fallback_properties`, leaving remaining accessibility behavior as runtime-owned extraction rather than legacy migration debt. |
| `icon_atlas/` | Runtime icon atlas support | Leaf runtime UI support module. |
| `tests/` | Runtime UI test tree | Not a production owner; excluded from production debt file counts above. |

Dependency direction:

```text
template asset/build -> v2 cache/compiler -> v2 surface_tree -> surface
component catalog/state -> v2 surface_tree + surface default interactions
surface -> layout pass -> arranged output
surface input/focus/popup state -> dispatch route authority
binding/event_ui -> surface mutation reports and route results
runtime_ui -> v2 prototype cache + surface + theme
interface::ui::v2 DTOs -> runtime::ui::v2 implementation
```

Runtime 10 M2.1 removes the remaining runtime-local UI contract duplicates instead of adding compatibility aliases. `UiBindingCodec` now lives only under `zircon_runtime_interface::ui::event_ui`, and `UiAssetSchemaVersionPolicy` plus schema-version constants live only under `zircon_runtime_interface::ui::template::asset::schema`; runtime template/schema code imports those interface contracts directly and keeps behavior in `UiAssetSchemaMigrator`.

No M0 blocker-level owner inversion was found in the scanned module graph. The remaining work is not "unknown ownership"; it is explicit cleanup of debt-bearing areas:

1. UI legacy naming and migration terms: production UI source now contains no `legacy` scan hits after Runtime 09 M1.2 cutovers. `runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending` removed the navigation reply variable from this bucket by renaming the local route reply to `routed_reply`; `runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending` removed the pointer reply local naming debt by using `routed_result`; `runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending` removed the pointer capture fallback API wording debt by using `has_pointer_capture_or_unindexed_fallback_for_owner`; `runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending` removed the render table row-label fallback wording debt by using `split_row_label_table_text`; `runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending` removed the template interaction inference wording debt by using `component_name_interaction_fallback`; `runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending` removed the property mutation visibility-transition wording debt by using `state_visible_flag`; `runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending` removed the responsive MUI visibility DTO wording debt by using the same `state_visible_flag` semantic name; `runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending` removed the accessibility open-state fallback wording debt by using `fallback_properties`; `runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending` removed the layout-engine public backend wording debt by hard-cutting to `UiLayoutEngineBackend::Zircon`, `UiLayoutEngineCapability::zircon()`, and `zircon_selected_count`; `runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending` removed the final surface default interaction fallback wording debt by using `fallback_properties` in `default_open_boolean_value(...)`.
2. Taffy backend exposure: 9 production files currently mention `taffy`. `runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending` hardens the intended owner shape: `taffy_bridge/compute.rs` owns Taffy tree construction and `compute_layout`, `pass/taffy_arrange.rs` owns eligibility/fallback/recursive arrange only, and `style_mapping.rs` remains `runtime_09_m2_1_style_mapping_remains_taffy_dto_adapter`.
3. Virtualization and scroll cache invalidation: `runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending` records `UiScrollVirtualizationPlan` as the layout-owned authority for scroll offset, viewport/content extent, and visible-range invalidation. Tree scrolling and layout arrange consume that planner; render, hit grid, and editor code do not compute virtual windows.
4. Template generation and migration: `runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending` records `UiTemplateRuntimePipeline`, `UI_TEMPLATE_RUNTIME_PIPELINE_STAGES`, `UiTemplateRuntimePipelineError`, template failure-path anchors, and the binary DTO generated policy. Old recursive template paths still coexist with v2 runtime paths as migration/test surfaces.

## M1.1 Input Route Authority

runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending

The normalized runtime UI input path is now:

```text
platform/window input -> UiInputEvent -> UiSurface::dispatch_input_event
  -> surface/input/dispatch.rs -> leaf dispatchers
  -> route_authority diagnostic note
  -> component/focus/popup/default-action result
```

`surface/input/dispatch.rs` is the single exit for normalized `UiInputEvent` dispatch. It collects pointer, navigation, keyboard, text, IME, analog, mouse-motion, drag-drop, popup, tooltip/typeahead/submenu/toast timer, and accessibility leaf results, then calls `annotate_authoritative_input_dispatch`.

`surface/input/route_authority.rs` consumes `zircon_runtime::ui::dispatch::UI_INPUT_ROUTE_ORDER` and records a diagnostic note shaped as `route_authority=runtime_09_m1_1_ui_input_route_authority;policy=...;stages=...`. The stage list is derived from the route policy while preserving the manager-owned order: pointer capture, popup stack, preview tunnel, direct target, bubble path, focus path, default action.

Bypass owner verdict:

- `UiSurface::dispatch_input_event`, `UiSurface::dispatch_input_event_with_manager`, `RuntimeUiManager::dispatch_input_event`, and window-pump normalization are the normalized route authority path.
- `UiSurface::dispatch_pointer_event*` and `RuntimeUiManager::dispatch_pointer_event` remain direct pointer leaf helpers for existing low-level callers/tests.
- `UiSurface::dispatch_navigation_event` and `RuntimeUiManager::dispatch_navigation_event` remain direct navigation leaf helpers for existing low-level callers/tests.
- These direct pointer/navigation entry points are not unowned bypasses; they are recorded as `runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers` until callers can migrate to normalized `UiInputEvent` dispatch or the helpers are retired.

## M1.2 Pointer Reply Naming

runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending

The direct pointer route helpers now name the dispatch result as `routed_result` in `surface/input/pointer.rs` and `surface/input/pointer_reply.rs`. This is a local naming cutover only: route order, focus/popup capture behavior, diagnostics, and returned `UiInputDispatchResult` data flow are unchanged.

`runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt` keeps both files from reintroducing `legacy` wording while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Pointer Capture Fallback Naming

runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending

The high-precision pointer effect path now calls `has_pointer_capture_or_unindexed_fallback_for_owner` in `surface/input/effect/focus_pointer.rs`, backed by `surface/input/state/pointer_capture.rs`. The behavior is unchanged: indexed pointer captures are preferred, and the unindexed fallback marker remains available for older single-pointer capture state while `surface.focus.captured` still identifies the owner.

`runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt` keeps the old `has_legacy_or_indexed_pointer_capture_for_owner` wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Table Row Label Fallback Naming

runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending

The table collection-row renderer now calls `split_row_label_table_text` in `surface/render/collection_rows/table.rs` when explicit `cells` / `columns` / `options` data is absent and the row label must be split into display cells. The behavior is unchanged: explicit cell arrays still win, and row-label fallback splitting still follows the same whitespace token cases.

`runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt` keeps the old `split_legacy_table_text` wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Template Component-Name Fallback Naming

runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending

The template build interaction inference now calls `component_name_interaction_fallback` in `template/build/interaction.rs` when a node has no authored `input_*` metadata and no binding-derived input capability. The behavior is unchanged: explicit input metadata remains authoritative, binding event capabilities still infer input flags first, and the component-name fallback still only promotes `Button`, `IconButton`, and `TextField`.

`runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt` keeps the old `legacy_component_interaction_fallback` and `legacy_interactive` wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Property Visibility Flag Naming

runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending

The property mutation visibility transition helper now calls the `UiVisibility::effective(...)` boolean input `state_visible_flag` in `surface/property_mutation.rs`. The behavior is unchanged: `visibility_transition_dirty(...)` still compares the current and next effective layout occupancy using the same retained `UiStateFlags::visible` value, and only marks layout dirty when occupancy changes.

`runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt` keeps the old `legacy_visible` local wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Responsive MUI Visibility Flag Naming

runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending

The responsive MUI layout pre-pass now names the resolved authored `visible` boolean as `state_visible_flag` in `layout/pass/responsive_mui.rs`. The behavior is unchanged: `display`, `visibility`, and authored `visible` still resolve before measurement, and the pre-pass still writes the same `UiStateFlags::visible` value and dirty domains before full or incremental layout solving.

`runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt` keeps the old `legacy_visible` DTO wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Accessibility Open-State Fallback Naming

runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending

The accessibility snapshot extractor now names the retained/component-state open-state compatibility list `fallback_properties` in `accessibility/extract.rs`. The behavior is unchanged: authored `open_property` remains authoritative, then the same component-state property is read, then retained/component-state alternatives `expanded`, `popup_open`, and `open` are checked before true runtime expanded/popup flags.

`runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt` keeps the old `legacy_properties` and `legacy_property` helper wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Layout Engine Backend Name Cutover

runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending

The layout engine contract now names the runtime fallback backend directly as `UiLayoutEngineBackend::Zircon`. The public capability constructor is `UiLayoutEngineCapability::zircon()`, and route reports expose `zircon_selected_count`. `zircon_runtime/src/ui/layout/pass/engine.rs` consumes those names directly when recording Taffy-native routes, explicit fallbacks, and Zircon-owned routes.

`runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt` keeps the old `LegacyZircon`, `legacy_zircon`, and `legacy_selected_count` API names from returning. This is a hard cutover, not a compatibility alias layer.

## M1.2 Surface Default Interaction Fallback Naming

runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending

The surface default interaction open-state helper now names its compatibility property list `fallback_properties` in `surface/surface/default_interactions.rs`. `default_open_boolean_value(...)` still resolves authored metadata first, then the same component-state property, then retained/component-state fallback aliases, and finally the canonical runtime open flag before applying the supplied default.

`runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt` keeps the old `legacy_properties` and `legacy_property` local wording from returning. This closes the Runtime 09 production UI `legacy` scan bucket without changing default interaction behavior.

## M2.1 Layout Backend Authority

runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending

`zircon_runtime::ui::layout::taffy_bridge` is now folder-backed. `taffy_bridge/compute.rs` owns Taffy tree construction, fractional rounding policy, `TaffyTree::new()`, and `compute_layout` through `compute_taffy_child_frames(...)`. `layout/pass/taffy_arrange.rs` no longer imports `taffy::`, creates a `TaffyTree`, or computes layout directly; it builds `TaffyChildLayoutInput` records, delegates backend computation to the bridge, then records native/fallback selection and recurses into `arrange_node(...)`.

`UI_LAYOUT_PASS_ORDER` lives in `layout/pass/pipeline.rs` and is consumed by both full and incremental layout. The order is: responsive style resolution, measurement, backend selection, Taffy bridge arrangement, Zircon fallback arrangement, clip and virtual-window propagation, selection report. Incremental layout now measures all selected layout roots before arranging them, matching the same phase split as full layout.

runtime_09_m2_1_style_mapping_remains_taffy_dto_adapter

`layout/style_mapping.rs` still mentions Taffy because it converts neutral Zircon `UiLayoutStyle` / container DTOs into Taffy style DTOs. That file is not a backend executor and does not own tree construction or pass sequencing. This explicit adapter verdict replaces the older "only `taffy_bridge.rs` may mention Taffy" wording with the stronger runtime boundary: Taffy compute is only reachable through `compute_taffy_child_frames`, while style DTO conversion remains isolated.

## M2.2 Virtualization Scroll Boundary

runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending

`layout/scroll.rs` now owns `UiScrollVirtualizationPlan` and `plan_scrollable_virtual_window(...)`. The planner clamps the requested scroll offset against the current content/viewport extents, computes the scroll state's `offset` / `viewport_extent` / `content_extent`, projects the virtual window, and reports whether `visible_range` must be invalidated. `layout/virtualization.rs::compute_virtual_list_window(...)` stays pure arithmetic over offset, viewport extent, item extent, item count, and overscan.

Invalidation timing is now explicit:

- Scroll offset changes recompute the virtual window. `tree/node/scroll.rs::set_scroll_offset(...)` consumes the planner, marks layout/hit/render/input dirty, and ORs `node.dirty.visible_range |= plan.visible_range_changed` so an earlier visible-range dirty mark cannot be cleared by a later scroll operation.
- Viewport and content changes are settled in `layout/pass/arrange.rs`, which consumes the same planner after measurement has produced `content_size` and the parent frame has produced `viewport_extent`. The resulting full or virtual window is cached in `node.layout_cache.virtual_window`.
- Data changes that alter child count or measured content extent flow through layout dirtiness into the arrange pass. Virtualized scroll containers mark `visible_range` when the planned window, viewport extent, or content extent changes. Non-virtualized scroll containers keep a full-window cache and do not report `visible_range` changes for ordinary scroll offset movement.

Behavior coverage is recorded by `scroll_virtualization.rs`: `virtualized_list_only_materializes_visible_window`, `scroll_offset_invalidates_virtualization_window`, and `non_virtualized_scroll_offset_keeps_full_window_dirty_domain`. Package behavior filters are still deferred by the current implementation-first request, but the tests are present and wired into `ui/tests/mod.rs`.

## M3.1 Template Pipeline Boundary

runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending

The old recursive template path now has an explicit runtime entry in `UiTemplateRuntimePipeline`. `UI_TEMPLATE_RUNTIME_PIPELINE_STAGES` fixes the phase order as:

```text
load -> validate -> instance -> build
```

The stage owners are intentionally narrow. `UiTemplateLoader` owns TOML parse and file IO, `UiTemplateValidator` owns structural contract validation, `UiTemplateInstance::from_validated_document(...)` owns already-validated expansion, and `UiTemplateSurfaceBuilder` owns lazy `UiSurface` construction. `UiTemplateRuntimePipelineError` keeps those phases observable as `Load`, `Validate`, `Instance`, and `Build`, so callers and tests can tell which boundary rejected the document.

The M3.1 failure-path anchors are present in `template_pipeline.rs`: `template_validate_rejects_unknown_component_contract`, `template_instance_failure_surfaces_loader_error`, and `compiled_template_artifact_stays_binary_leaf_dto_not_generated_source`. Broader package behavior execution remains deferred by the current implementation-first request.

Generated output policy is tied to Runtime 02 M4. `UiRuntimeCompiledAssetArtifact::generated_policy()` records `runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source`; current compiled template package output is a binary/TOML DTO payload and does not require a generated-source marker. If a future template compiler writes source files, the first line must be `// @generated <generator> - do not edit by hand`, and the generated file may only contain leaf DTO/table/adaptor material, not runtime validation, loading, expansion, or surface mutation behavior.

## V2 Verdict

v2-replacement-mainline

The v2 UI path is the replacement mainline for authored runtime/editor UI assets, not dead code and not a second unconstrained runtime contract.

Runtime 10 M2.2 now mirrors this verdict through `runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending`: interface `ui/v2` owns the v2 asset/compiled/style DTOs, runtime `ui/v2` consumes those DTOs for loader/cache/compiler/instancer behavior, and `UiComponentApiVersion` remains the interface-owned component contract version. Runtime validation continues to report `UiComponentContractDiagnosticCode::ApiMismatch` via `actual.is_compatible_with(required)`; the current Runtime 10 structural mirror records `ui_v2_contract_sync_anchors = 9/9` and keeps `v2-replacement-mainline` mutually linked with Runtime 09.

The source-profile split is:

- `.zui` is the production component asset suffix for imported project/editor component assets.
- `.v2.ui.toml` remains valid for v2 view/style/runtime fixture/editor chrome assets that are loaded through the v2 cache path, but it is not the general production component importer path.
- recursive `UiTemplateNode` and old template document paths are legacy/migration/test-only surfaces until M3 proves their remaining owners and deletion conditions.

Current evidence:

- `zircon_runtime_interface::ui::v2` contains the neutral v2 DTO schema.
- `zircon_runtime::ui::v2` contains the runtime loader/compiler/cache/style/surface-builder/surface-tree implementation.
- Runtime UI fixture/manager docs describe `UiV2PrototypeStoreFileCache -> UiV2SurfaceBuilder -> surface_tree -> UiSurface`.
- Editor view/chrome assets and runtime fixtures already consume v2 assets.
- Asset importer/plugin registry docs enforce `.zui` for production component assets while keeping explicit v2 fixture/view paths.

Migration route:

1. Keep `zircon_runtime_interface::ui::v2` as the neutral contract layer.
2. Keep `zircon_runtime::ui::v2` as the sole runtime implementation layer for v2 cache, compilation, style resolution, and surface construction.
3. Keep editor usage on asset/cache/projection APIs; editor code may not grow a parallel runtime builder.
4. Delete or isolate old recursive template paths only after M3 records owner files, generated-file rules, fixture exceptions, and failure-path tests.

Deletion conditions for non-v2 runtime paths:

- production project/editor component importers do not accept old recursive `.ui.toml` component documents;
- component catalog and editor shell projection use v2 assets or `.zui` component assets;
- runtime fixtures and preview manager have no fallback through `UiTemplateTreeBuilder` or old `UiTemplateSurfaceBuilder`;
- migration fixtures are named and isolated from production asset registration;
- template compile/instance/validate failure paths have explicit tests and generated output markers where they write files.

## Static Acceptance

This Runtime 09 record contains the M0 documentation/status pass, the M1.1 normalized input route authority note, the M1.2 local navigation, pointer reply, pointer-capture fallback, table row-label fallback, template component-name fallback, property visibility flag, responsive MUI visibility flag, accessibility open-state fallback, layout engine backend name, and surface default interaction fallback cutovers, the M2.1 Taffy bridge/pass-order authority cutover, the M2.2 virtualization/scroll boundary implementation, and the M3.1 template pipeline/generated-policy boundary. Package Cargo was run only as focused static/type checks in this lane; full UI behavior filters are still deferred per the current implementation-first request. The accepted static evidence is:

- current owner map covers all 17 UI top-level entries;
- current `surface/` scan is recorded as 20 entries rather than the stale 2026-06-12 value;
- `legacy` full-tree and production-file baselines are recorded separately after the Runtime 09 M1.2 navigation, pointer reply, pointer-capture fallback, table row-label fallback, template component-name fallback, property visibility flag, responsive MUI visibility flag, accessibility open-state fallback, layout engine backend name, and surface default interaction fallback cutovers;
- `taffy` production-file baseline is refreshed after M2.1 to record the bridge-directory and pass-order owner shape;
- `UiScrollVirtualizationPlan` and `plan_scrollable_virtual_window(...)` are recorded as the Runtime 09 M2.2 owner boundary for scroll offset, viewport/content extent, and virtual-window invalidation;
- `UiTemplateRuntimePipeline`, `UI_TEMPLATE_RUNTIME_PIPELINE_STAGES`, and `UiTemplateRuntimePipelineError` are recorded as the Runtime 09 M3.1 template load/validate/instance/build boundary;
- `runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source` records that current compiled template artifacts are binary DTO payloads rather than generated source; future generated source must use `// @generated <generator> - do not edit by hand`;
- v2 is explicitly classified as replacement mainline with a source-profile split and deletion conditions.
- `runtime_absorption::ui_architecture` now guards the module count, baseline scan values, v2 runtime/interface module shape, the route authority note, the direct pointer/navigation owner verdict, the navigation reply rename, the pointer reply rename, the pointer-capture fallback rename, the table row-label fallback rename, the template component-name fallback rename, the property visibility flag rename, the responsive MUI visibility flag rename, the accessibility open-state fallback rename, the layout engine backend name cutover, the surface default interaction fallback rename, the Taffy bridge/pass-order authority, the virtualization/scroll invalidation planner, the template pipeline/generated-policy boundary, and the plan/index anchors.
- `ui_architecture_boundary` mirrors the same static facts: `expected_source_file_count = 52`, `expected_ui_entry_count = 17`, `expected_surface_entry_count = 20`, `legacy_full_hits = 54`, `expected_legacy_full_hits = 54`, `legacy_production_hits = 0`, `expected_legacy_production_hits = 0`, `legacy_production_file_count = 0`, `expected_legacy_production_file_count = 0`, `taffy_production_hits = 173`, `expected_taffy_production_hits = 173`, `taffy_production_file_count = 9`, `expected_taffy_production_file_count = 9`, `runtime_v2_anchor_count = 10`, `interface_v2_anchor_count = 9`, `guard_anchor_count = 19`, `cargo_gate_anchor_count = 7`, `doc_anchor_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts` keeps this document aligned with Runtime 09, the runtime index, the M0 review, runtime-interface convergence, and the Python audit. This is static structure evidence only.
- `runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation` keeps Runtime 09 on the `ui/input/naming_boundary/layout/template` owner/Cargo gate until editor UI owner coordination and the declared Cargo filters provide real evidence.
