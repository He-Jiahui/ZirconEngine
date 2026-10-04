---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/273/2026-08-29-deferred-progress-alias-probes.md
  - docs/plans/optimize/zircon_editor/137/2026-08-26-activity-descriptor-capacity.md
  - docs/plans/optimize/zircon_editor/168-editor-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-current-source-review.md
  - docs/plans/optimize/zircon_editor/130-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/button_style/deferred_progress_tests.rs
  - zircon_editor/src/ui/workbench/reflection/activity_descriptors/capacity_tests.rs
  - zircon_editor/src/core/play/live_link.rs
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
---

# Editor909 Nested Test Owner Import Repairs

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Deferred-progress alias probe and ignored benchmark | Import the five existing private attribute/alias helpers from the nested test's direct `button_style` parent instead of its grandparent. | v27 reported one grouped unresolved-import diagnostic; the 1,024-attribute, 8,192-check performance fixture and assertions remain unchanged. | implemented_pending_validation |
| Activity descriptor capacity tests | Import `ViewDescriptorId` from its current workbench-view owner instead of a parent-module import that no longer exists. | v27 reported one unresolved name; size/capacity and ignored Release assertions remain unchanged. | implemented_pending_validation |
| Play gateway identity guard | Import the existing detached gateway test stand-in from `core::gateway`, leaving play identity replacement/detachment behavior unchanged. | v27 reported one stale `super` import in the clean test module. | implemented_pending_validation |
| Remote command route gate | Import the existing `MenuAction` from `core::editor_event`, keeping remote policy and binding assertions. | v27 reported one stale workbench event import. Exact Rustfmt and diff checks pass on all four owners. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/button_style/deferred_progress_tests.rs` | `7DA8CCF0E674D7C72CBB7018F4C18AECB9B04350791A560269594E148623649F` |
| `zircon_editor/src/ui/workbench/reflection/activity_descriptors/capacity_tests.rs` | `57207FFC3D60BFB5829074287F0F6658377CECDE5E494C86932A55B4C51E7AFA` |
| `zircon_editor/src/core/play/live_link.rs` | `79DEC4EB7174D053DE9FF1F6ADBEDF584D765669518874F43821E532C6770D87` |
| `zircon_editor/src/tests/workbench/reflection/action_dispatch.rs` | `5953C4AB4C85FAC8207A2185BB6912EE352911F2BE602E4789D7B23D7F172E5C` |

## Managed gate

All four repairs postdate v28 admission and are not claimed as checked by
that in-flight source snapshot. A later grouped Runtime/Editor managed Rust
test and explicit ignored Release filters must qualify affected tests and
benchmarks. No allocation or product p50/p95/p99 gate is accepted.
