---
related_code:
  - zircon_runtime_interface/src/ui/binding/model/update.rs
  - zircon_runtime_interface/src/ui/binding/model/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime_interface/src/tests/contracts.rs
implementation_files:
  - zircon_runtime_interface/src/ui/binding/model/update.rs
  - zircon_runtime_interface/src/ui/binding/model/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
plan_sources:
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
tests:
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - 2026-05-20: cargo test -p zircon_runtime_interface --lib ui_binding_update_contract_represents_attribute_state_and_ecs_domains --locked --jobs 1 --message-format short --color never (passed, 1 test)
  - 2026-05-20: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
  - 2026-05-20 widget-result-propagation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20 widget-result-propagation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
  - 2026-05-20 scrollbar-runtime-state-target: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20 scrollbar-runtime-state-target: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
doc_type: module-detail
---

# UI Binding Update Contract

`zircon_runtime_interface::ui::binding` now owns the neutral binding-update DTOs used by the UI/Text/Input/A11y convergence plan. The contract describes a value movement without choosing the concrete runtime executor: where the update came from, where it is applied, the previous and next value, the update status, and the dirty domains that downstream schedules should observe.

This is intentionally separate from component data-source descriptors. Component data binding describes authoring or host data sources such as inspector rows. `UiBindingUpdate` describes a concrete runtime update that can be produced by retained attributes, runtime component state, widget behavior, accessibility actions, component events, host projection, or a future runtime ECS bridge.

## Contract Shape

`UiBindingSource` identifies the origin by `UiBindingSourceKind`, optional `UiNodeId`, optional property, and optional path. Runtime ECS sources use `path` so the interface can describe future ECS-backed facts without importing runtime scene or editor types.

`UiBindingTarget` mirrors that shape for destinations. Current target kinds cover retained attributes, runtime state, component-state values, component-state flags, runtime ECS targets, widget aliases, and host projection. Runtime state is used for surface-owned facts that are not reflected metadata, such as a scrollable node's `scroll_offset`.

`UiBindingDirtyDomain` is the schedule-facing vocabulary. It maps the existing `UiDirtyFlags` domains into binding reports and adds `Accessibility`, `Interaction`, and `Schedule` for later milestones that need a11y extraction, widget interaction, or ECS schedule invalidation without extending the old tree dirty flags.

`UiBindingUpdateReport` aggregates updates and recomputes applied/unchanged/rejected counts plus the union of dirty domains. It is a DTO, not an executor; runtime modules decide when to attach reports to property mutation, widget reducers, accessibility actions, and ECS projection.

`UiInputDispatchResult`, `UiPointerDispatchResult`, and `UiNavigationDispatchResult` now have a defaulted `binding_reports` array plus `record_binding_report(...)`. This lets input, widget, pointer, navigation, and accessibility dispatchers return structured binding evidence next to component events and host requests without changing old serialized results that do not contain the field.

## Current Boundary

This slice is the M3 contract foundation. It does not replace `UiSurface::mutate_property`, `UiSurfaceComponentStateStore`, or widget reducers. Those systems now emit the same update vocabulary incrementally while preserving their existing behavior. Accessibility SetValue dispatch attaches its mutation binding report to the shared input dispatch result, default pointer/navigation/widget actions attach the same reports to their legacy result surfaces, and Scrollbar `scroll_target` page scrolls report `WidgetBehavior -> RuntimeState(scroll_offset)` updates for the target scroll container.

## Validation

On 2026-05-20 the focused contract test `ui_binding_update_contract_represents_attribute_state_and_ecs_domains` passed. It covers serde roundtrip, default status, source and target classification, report counts, dirty-domain aggregation, the serialized `component_state_value` target spelling, dispatch-result binding report roundtrip, and legacy dispatch JSON missing `binding_reports`. The later widget-result propagation slice extended that coverage to pointer and navigation dispatch result roundtrips plus legacy JSON defaulting. The Scrollbar slice extended the same test to `UiBindingTargetKind::RuntimeState`. The full `ui_contract_spine` filter passed after this slice, covering 7 interface contract tests, and the broader `contracts` filter passed 91 tests after the dispatch-result fields were added.
