---
related_code:
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/update.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/compute.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/picking.rs
  - zircon_runtime_interface/src/ui/text.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/pipeline/mod.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_counters.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_report.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report.rs
  - zircon_runtime_interface/src/ui/pipeline/dirty_reason.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/schema/legacy_template.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tests/ecs_projection.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/pipeline_report.rs
  - zircon_runtime/src/ui/tests/asset_contract_spine.rs
  - zircon_runtime/src/ui/tests/mod.rs
  - zircon_editor/src/ui/asset_editor/palette/instantiate.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/ui/asset_editor/document_diff.rs
  - zircon_editor/src/ui/asset_editor/tree/tree_editing.rs
  - zircon_runtime_interface/src/tests/mod.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_ecs_projection_contracts.rs
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
implementation_files:
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/update.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/compute.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/picking.rs
  - zircon_runtime_interface/src/ui/text.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/pipeline/mod.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_counters.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_report.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report.rs
  - zircon_runtime_interface/src/ui/pipeline/dirty_reason.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/schema/legacy_template.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_editor/src/ui/asset_editor/palette/instantiate.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/ui/asset_editor/document_diff.rs
  - zircon_editor/src/ui/asset_editor/tree/tree_editing.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - zircon_runtime_interface/src/tests/ui_ecs_projection_contracts.rs
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
plan_sources:
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md
  - user: 2026-05-08 continue M1 Contract Spine implementation
  - user: 2026-05-08 include M3 tab/directional navigation in M2 focus milestone
tests:
  - zircon_runtime_interface/src/tests/mod.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - planned: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1
  - target: cargo test -p zircon_runtime_interface --lib pipeline_contracts --locked --offline --jobs 1
  - target: cargo test -p zircon_runtime_interface --lib ui_ecs_projection_contracts --locked --offline --jobs 1
  - target: cargo test -p zircon_runtime --lib ecs_projection --locked --offline --jobs 1
  - target: cargo test -p zircon_runtime --lib pipeline_report --locked --offline --jobs 1
  - zircon_runtime/src/ui/tests/asset_contract_spine.rs
  - target: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked
  - target: cargo check -p zircon_runtime_interface --locked
  - target: cargo test -p zircon_runtime --lib ui_asset_compiler_preserves_m1_contract_sections_from_source_nodes --locked
  - target: cargo test -p zircon_runtime --lib focus_navigation --locked
  - 2026-05-17 scrollbar-widget-contract-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-scrollbar-contract --message-format short --color never (passed, 6 tests)
  - 2026-05-17 typed-style-contract-validation: WSL cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 (passed)
  - 2026-05-20 binding-update-contract-validation: cargo test -p zircon_runtime_interface --lib ui_binding_update_contract_represents_attribute_state_and_ecs_domains --locked --jobs 1 --message-format short --color never (passed, 1 test)
  - 2026-05-20 binding-update-contract-spine-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20 binding-result-contract-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
  - 2026-05-20 widget-binding-result-contract-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/pointer/result.rs zircon_runtime_interface/src/ui/dispatch/navigation/result.rs zircon_runtime_interface/src/tests/ui_contract_spine.rs (passed)
  - 2026-05-20 widget-binding-result-contract-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20 widget-binding-result-contract-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
  - 2026-05-20 scrollbar-runtime-state-contract-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20 scrollbar-runtime-state-contract-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
  - 2026-05-23 popup-tooltip-owner-contract-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/input/event.rs zircon_runtime_interface/src/ui/dispatch/input/effect.rs zircon_runtime_interface/src/tests/contracts.rs (passed)
  - 2026-05-23 drag-metrics-contract-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/component/drag.rs zircon_runtime_interface/src/ui/component/mod.rs zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs zircon_runtime_interface/src/tests/contracts.rs (passed)
  - 2026-05-23 shared-drag-metrics-contract-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/input/result.rs zircon_runtime_interface/src/tests/contracts.rs (passed after formatting contracts.rs)
  - 2026-05-23 layout-engine-taffy-tree-stats-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/layout/engine.rs zircon_runtime_interface/src/ui/layout/mod.rs zircon_runtime_interface/src/tests/layout_engine_contracts.rs (passed)
  - 2026-05-23 layout-engine-taffy-tree-stats-contract-validation: cargo test -p zircon_runtime_interface layout_engine_contracts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-tree-stats-20260523 --message-format short --color never (passed, 7 tests)
doc_type: module-detail
---

# UI Contract Spine

## Purpose

This document records the M1 Contract Spine for the Bevy UI/Text/Widgets/Focus/A11y completion plan. The work is contract-only inside `zircon_runtime_interface::ui`; it defines neutral DTOs that later runtime and editor milestones can consume without moving focus, picking, widget behavior, text editing, render extraction, or accessibility authority into the editor.

M1 does not implement widget reducers, text shaping, AccessKit bridging, or editor styling. M2/M3 now consume the focus/navigation DTOs for runtime-owned focus mutation and contract-aware navigation, while later milestones still own picking convergence, headless widgets, text shaping/editing completion, accessibility snapshots, and editor styling.

## Contract Families

`ui::focus` defines `UiInputFocus`, `UiFocusVisible`, `UiFocusChangeEvent`, `UiFocusedInput`, and `UiFocusContract`. The state distinguishes the focused node, previous focus, pending autofocus, focus-ring visibility reason, focus-change reason, and focused-input bubbling route.

`ui::navigation` defines `UiTabIndex`, `UiNavigationGroup`, `UiNavigationGroupId`, `UiDirectionalNavigation`, and `UiNavigationContract`. These DTOs let runtime M3 represent tab order, modal trap groups, nested navigation groups, manual directional overrides, and blocked edges.

`ui::picking` defines `UiPickPolicy`, `UiPickMode`, `UiPointerCapture`, and `UiPointerCaptureKind`. The policy intentionally uses one neutral vocabulary for pointer, hover, focus, accessibility, capture, and text-hit eligibility so Zircon does not inherit Bevy's split between UI picking and focus policy.

`ui::binding` defines both legacy native event-binding DTOs and the newer binding-update report vocabulary. `UiBindingSource`, `UiBindingTarget`, `UiBindingUpdate`, and `UiBindingUpdateReport` let retained attributes, runtime state, runtime component state, widget behavior, accessibility actions, host projection, and the future runtime UI ECS bridge describe the same value movement and dirty domains without moving execution into the interface crate.

`ui::ecs` defines the M5 runtime UI ECS projection vocabulary. `UiEcsProjectionSnapshot` is a read-only projection of retained `UiSurface` facts into stable node records, dirty domains, interaction state, render command counts, and hit-entry counts. It intentionally does not create a second source of truth: runtime retained nodes, component state, focus/capture/hover facts, and dirty flags remain authoritative, while the projection gives future Bevy-like systems and diagnostics a queryable snapshot for layout, text, input, picking, a11y, and render scheduling.

`UiSurfaceFrame` and `UiSurfaceDebugSnapshot` carry the ECS projection as defaulted fields so old serialized diagnostics remain readable and new editor/runtime tools can observe projection facts through the same surface boundary as layout, render, hit testing, focus, and pipeline reports.

The projection contract also includes `UiEcsProjectionDelta`, `UiEcsProjectionNodeChange`, `UiEcsProjectionChangeKind`, `UiEcsProjectionChangeReason`, and `UiEcsProjectionDeltaTotals`. These DTOs are the M5 change-detection vocabulary: they compare two read-only snapshots, classify added/removed/updated nodes, attach schedule-visible dirty domains, and summarize which layout/text/input/picking/a11y/render domains need attention without making the interface crate responsible for executing systems.

`UiEcsProjectionScheduleMask` is the bridge from projection change detection to the M7 pipeline vocabulary. Snapshot and delta DTOs carry a defaulted mask and can recompute it from their contained dirty domains, deriving ordered `UiPipelineStage` requirements and `UiPipelineDirtyReason` values. Legacy payloads that lack the carried mask keep deserializing with an empty default, while `schedule_mask()` and `recompute_schedule_mask()` let consumers recover the stage requirements from nodes or changes. `UiEcsProjectionScheduleImpact` adds the queryable detail layer: snapshots and deltas carry defaulted impact rows and expose helpers that group affected node ids by required pipeline stage and report the stage-specific dirty reasons driving each row. Legacy payloads without the carried rows deserialize with an empty list and can call `recompute_schedule_impacts()` to recover them from nodes or changes. This keeps the aggregate mask cheap while giving diagnostics enough evidence to show which nodes made TextMeasure, Layout, Picking, A11yExtract, RenderExtract, or BatchPrepare necessary.

The delta contract now includes fast-path structure classifiers. `UiEcsProjectionNodeChange::changes_component_structure()` distinguishes added/removed nodes and parent/child/path/component changes from frame-only, interaction-only, or render-only changes. `UiEcsProjectionDelta::interaction_only()` identifies hover/focus/press/capture deltas that should not rebuild retained component structure. The corresponding totals count component-structure, interaction, and render-only changes so future schedule runners and diagnostics can keep pointer interaction on a narrow path. Actual systems, change clearing, and mutation authority remain in `zircon_runtime`.

Those fast-path classifiers now have node-id query helpers as well. `component_structure_change_node_ids()`, `interaction_change_node_ids()`, `interaction_only_change_node_ids()`, and `render_only_change_node_ids()` expose the concrete affected nodes for schedule probes without forcing callers to duplicate the classifier predicates.

Delta rows can also be queried directly. `change(node_id)`, `changes_by_kind(kind)`, `node_ids_by_change_kind(kind)`, `changes_requiring_stage(stage)`, and `changes_in_dirty_domain(domain)` provide the Bevy-like change-detection shape future runtime systems need without making callers duplicate `UiEcsDirtyDomains -> UiPipelineStage` expansion or scan totals as a proxy for concrete rows.

`UiEcsDirtyDomainImpact` is the domain-facing companion to schedule impacts. Snapshot and delta payloads carry defaulted rows grouping affected node ids by `UiEcsDirtyDomainKind`, and recompute helpers can restore the rows from projected nodes or changes when legacy diagnostics omit the field. This lets host diagnostics answer which nodes dirtied text, layout, accessibility, render, picking, input, style, or visible-range domains before those domains are expanded into pipeline stages.

Snapshot and delta DTOs expose schedule/domain query helpers over those recomputed rows. `node_ids_requiring_stage(stage)` gives systems and diagnostics the sorted nodes that require one active `UiPipelineStage`, while `node_ids_in_dirty_domain(domain)` gives the raw dirty-domain nodes before stage expansion. The private grouping logic lives in `ui::ecs::compute`, keeping the public DTO file focused on data contracts and query entry points.

The projection DTOs also expose unified derived-field freshness helpers. `derived_fields_are_fresh()` verifies that carried totals, schedule masks, stage impacts, and dirty-domain impacts still match their source nodes or changes. `recompute_derived_fields()` and `with_recomputed_derived_fields()` normalize those derived fields together, giving editor importers and archived diagnostic readers one safe entry point instead of several separate recompute calls.

`ui::layout::engine` defines the neutral layout backend route report used by runtime and editor diagnostics. `UiLayoutEngineSelectionReport` records per-container Taffy/Zircon selection, fallback and unsupported counts, fallback-reason summaries, and default-compatible Taffy tree-build metrics (`UiLayoutEngineTaffyTreeBuildStats`) so M2 can measure the current transient tree pass before replacing it with a surface-level cache.

`ui::dispatch::input::UiInputDispatchResult`, `ui::dispatch::pointer::UiPointerDispatchResult`, and `ui::dispatch::navigation::UiNavigationDispatchResult` carry defaulted `binding_reports` next to component events, host requests, and legacy route results. This keeps the result DTOs backward-compatible while allowing runtime dispatch paths such as accessibility SetValue and default widget reducers to return binding update evidence without string-only diagnostics. `UiPointerComponentEvent`, `UiComponentEventReport`, and `UiInputDispatchResult` also carry optional defaulted `UiDragMetrics` payloads so pointer-owned component events and selection-only input routes can report drag phase, start/current points, delta, and distance without changing the existing `UiComponentEvent` variants. Popup and tooltip input/effect DTOs also carry a defaulted optional owner id so runtime can validate explicit popup/tooltip owners without making old serialized host events invalid.

`ui::accessibility` defines neutral a11y roles, states, text-selection state, actions, nodes, diagnostics, `UiAccessibilityTreeSnapshot`, and `UiAccessibilityContract`. The snapshot is AccessKit-independent: runtime M9 may map it to AccessKit, but the shared contract stores node ids, paths, role/name/description, bounds, state, TextInput caret/selection offsets, actions, label links, tooltip text, focused node, and diagnostics such as missing accessible names. The action vocabulary keeps whole-field `SetValue`, selected-range `ReplaceSelectedText`, selection-only `SetTextSelection`, explicit open-state `Expand`/`Collapse`, popup/dialog/tooltip `Dismiss`, and scroll-offset `ScrollTo` payloads separate, so platform bridges can preserve text-edit, popup/disclosure, cancel, and scroll requests without leaking AccessKit types into the interface crate.

`ui::widget` defines the headless widget event vocabulary requested by M1: `Activate`, `ValueChange`, `TextEditChange`, `OpenChanged`, and `SelectionChanged`. `UiWidgetContract` carries defaulted template-facing widget metadata such as disabled, checked, value, label-for, tooltip, value/checked/open aliases, and scrollbar ownership fields. `UiWidgetBehavior::Scrollbar` and `ScrollbarThumb` mirror Bevy's headless scrollbar split: the track targets a scrollable container through `scroll_target`, may declare `scroll_axis`, and records `min_thumb_extent` for later thumb layout, while the thumb is a passive child marker. These widget events intentionally remain separate from `UiComponentEventKind` in M1 because no concrete component-envelope variant or reducer bridge exists yet.

`ui::component::drag` defines both drag/drop payload provenance and pointer-drag metrics. `UiDragPhase` plus `UiDragMetrics` are neutral DTOs shared by Range, ScrollbarThumb, TextInput pointer selection, and future editor drag affordances: component events still own semantic value changes when such events exist, while input-result drag metrics describe selection-only pointer gestures that do not have a component event to decorate.

`ui::text` defines `UiTextEdit` and `UiTextCursorStyle` around the existing `UiEditableTextState` and `UiTextEditAction` render-surface text DTOs. It records the edit source and before/after state while leaving actual editing behavior and shaping to later runtime text milestones.

`ui::style` defines shared typed style values that must cross runtime/editor boundaries without Rust trait objects or host-local string enums. The first covered family is Material Button style: `ButtonVariant`, `ButtonColor`, `ButtonSize`, `ButtonIconPlacement`, `ButtonInteractionState`, `UiStyleColor`, `StyleDimension`, `UiResolvedElementStyle`, and `ResolvedButtonStyle`. These DTOs let runtime style resolution and editor retained painting agree on Button/IconButton variant, tone, size, icon placement, state, disabled/loading flags, and resolved element colors/border/radius while keeping authored `.ui.toml/.zui` values as the source input.

`ui::surface::render` now exposes `UiRenderExtractKind` and `UiRenderStats` beside the existing `UiRenderExtract`. The existing extract shape remains compatible with legacy `{ tree_id, list }` JSON because the new classification and stats DTOs are separate rather than required fields.

`ui::pipeline` defines the M7 UI schedule/report spine. The required order is now `InputCollect -> Focus -> WidgetBehavior -> TextMeasure -> Layout -> PostLayout -> Picking -> A11yExtract -> RenderExtract -> BatchPrepare`, matching the Bevy-aligned plan while keeping Zircon-owned stage names. `UiPipelineStageReport` records elapsed time, dirty reasons, counters, skipped state, and notes for each stage; `UiPipelineFrameReport` aggregates ordered stage reports and exposes helpers for missing required stages and pointer-move fast-path checks. Legacy serialized stage names such as `focus_interaction`, `content_measure`, and `hit_grid` remain deserializable for archived diagnostics, but they are no longer required stages in the active schedule contract. `UiSurfaceFrame` and `UiSurfaceDebugSnapshot` now carry a defaulted `pipeline_report`, so runtime/editor diagnostics can consume the same ordered schedule DTO as contract tests. The current runtime bridge derives that report from `UiSurfaceRebuildReport`: layout, post-layout arranged-tree rebuild, picking-grid rebuild, and render extract receive measured rebuild timing/counters, while input/focus/widget/text/a11y/batch stages are retained as ordered skipped stages until their owners emit direct timing.

`ui::template::UiTemplateNode` has defaulted `focus`, `navigation`, `picking`, `a11y`, and `widget` sections. `ui::template::asset::UiNodeDefinition` mirrors those sections as optional source-authoring tables so old `.ui.toml` files continue to deserialize with no authored contract sections, while new source assets can opt into structured M1 contracts with `[root.focus]`, `[root.navigation]`, `[root.picking]`, `[root.a11y]`, and `[root.widget]` tables. Runtime asset compilation copies native-node sections into compiled `UiTemplateNode` defaults and treats component-instance sections as explicit root contract overrides. Legacy-template migration only emits optional source sections when the old compiled template node had a non-default contract value, preventing default migrated instance nodes from erasing component-root contracts.

`ui::tree::UiTreeNode` now carries defaulted `focus: UiFocusContract` and `navigation: UiNavigationContract` fields in the retained runtime tree. These fields are the runtime consumption point for compiled template contract sections, avoiding a second focus/navigation schema under `zircon_runtime`.

`ui::surface::UiFocusState` now stores runtime focus facts that need to be visible to hosts and tests: previous focus, pending autofocus, focus-visible policy, focus-change events, and focused-input route records. `captured`, `pressed`, and `hovered` remain on the same state object so cleanup can clear focus/capture/hover coherently when nodes are hidden, disabled, or despawned.

## M2/M3 Runtime Consumption

Runtime focus is source-aware. Programmatic calls record `Programmatic`, autofocus records `Autofocus`, pointer focus records `Input` with pointer-interaction visibility, and navigation records `Navigation` with keyboard-navigation visibility. Accepted `enabled`, `visible`, `visibility`, and `focusable` mutations reconcile focus and can emit `Disabled` or `Hidden`; node-pool detach emits `Despawned`.

Focused input bubbling uses `UiFocusedInput` rather than host-local state. Keyboard, navigation, text, and IME dispatch store the focused node, bubble route, accepted handler/owner, and acceptance result in `UiFocusState::focused_inputs`.

Contract-aware navigation uses `UiNavigationContract` fields on `UiTreeNode`: tab indices and group order define Next/Previous, non-modal groups contribute ordering, modal groups trap traversal, manual directional targets can point at a node or group, blocked edges suppress movement, and spatial fallback selects the nearest candidate by node center when no manual override exists. Runtime filters manual directional targets through the current modal group so authored overrides cannot escape an active dialog trap.

## Boundary Rules

The interface crate owns declarations and narrow helpers only. Runtime code must still own focus cleanup, tab/directional traversal, picking hit-grid policy application, widget behavior, text editing mutation, render extraction, a11y snapshot generation, and platform bridge mapping.

Editor code may consume these DTOs for authoring and presentation, but it must not become the source of truth for runtime focus, picking, widget, text, render, or a11y semantics.

## Test Coverage

`zircon_runtime_interface/src/tests/ui_contract_spine.rs` covers serde roundtrips and default compatibility for the new focus/navigation/picking, a11y snapshot/diagnostics/text-selection state, a11y action spellings including `expand` and `collapse`, widget/text/cursor, scrollbar target fields, render kind/stats, compiled-template sections, and source `.ui.toml` section contracts. It also verifies that legacy TOML without new sections still deserializes with safe defaults. `zircon_runtime/src/ui/tests/asset_contract_spine.rs` covers the runtime compiler path that preserves authored source sections through component expansion.

The M3 binding-update contract slice is covered by `ui_binding_update_contract_represents_attribute_state_and_ecs_domains`, which roundtrips `UiBindingUpdate`, verifies retained-attribute, runtime-state, component-state, accessibility-action, widget-alias, and runtime-ECS source or target kinds, checks report dirty-domain aggregation, and verifies `UiInputDispatchResult`, `UiPointerDispatchResult`, and `UiNavigationDispatchResult` binding reports roundtrip while legacy results default the field to empty.

`zircon_runtime_interface/src/tests/contracts.rs` covers the popup/tooltip owner extension by constructing owner-bearing `UiPopupInputEvent`, `UiTooltipTimerInputEvent`, `UiDispatchEffect::Popup`, and `UiDispatchEffect::Tooltip` payloads, then deserializing legacy JSON with the `owner` field removed and verifying the field defaults to `None`. It also covers pointer drag metrics by roundtripping a `UiPointerComponentEvent`, `UiInputDispatchResult`, and `UiComponentEventReport` with `UiDragMetrics`, then removing the `drag` field from JSON and verifying legacy payloads default to `None`.

`zircon_runtime_interface/src/tests/layout_engine_contracts.rs` covers the layout route report contract. It verifies Taffy/Zircon backend family selection, fallback and unsupported aggregation, deserialization recomputation from `selections`, and `UiLayoutEngineTaffyTreeBuildStats` roundtrip/default behavior for the tree-build baseline used by runtime M2 diagnostics.

`zircon_runtime_interface/src/tests/pipeline_contracts.rs` covers the M7 schedule/report spine. It pins the required stage order, stage string spellings, legacy stage-name deserialization, frame-report total recomputation, dirty reason aggregation, stage counters, and repeated pointer-move fast-path invariants. `zircon_runtime/src/ui/tests/pipeline_report.rs` covers the runtime bridge by asserting that `UiSurface::surface_frame()` and `UiSurface::debug_snapshot()` expose the required ordered report, that layout/post-layout/picking/render counters are sourced from the authoritative rebuild facts, and that render-only rebuilds do not dirty layout or picking stages.

`zircon_runtime_interface/src/tests/ui_ecs_projection_contracts.rs` covers the M5 UI ECS projection contract. It verifies dirty-domain derivation from `UiDirtyFlags`, snapshot total recomputation, serde roundtrip, legacy default behavior, delta change classification, component-structure fast paths, schedule mask derivation, carried-mask recomputation, carried-impact recomputation, dirty-domain impact recomputation, and stage/domain impact grouping. `zircon_runtime/src/ui/tests/ecs_projection.rs` covers the runtime bridge from `UiSurface` to `UiEcsProjectionSnapshot`, including node identity, retained component/control metadata, effective disabled inheritance, focus/hover/press/capture facts, interaction-only fast-path classification, dirty domains, render command counts, hit-entry counts, schedule masks, carried text-dirty schedule impacts, and runtime dirty-domain impact helper rows.
