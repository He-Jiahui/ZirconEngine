---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/163/2026-08-26-divider-label-borrow.md
  - docs/plans/optimize/zircon_editor/237/2026-08-28-borrowed-command-palette-window-query.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/120/2026-08-26-circular-progress-topology-front-hit.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-single-pass-native-load-state-classification.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/divider/labels/text/capacity_tests.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions/borrowed_window_request_tests.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_material_feedback/circular_progress/pixels/topology_front_hit_tests.rs
  - zircon_editor/src/tests/editing/authoring_world.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_load_state/optimization_tests.rs
  - zircon_editor/src/core/extension/toolkit/tests/lifecycle.rs
---

# Editor918 Benchmark, Authoring, and Toolkit Fixture Contract Repairs

## 计划完成列表

| Scope | Completed source change | Evidence | Acceptance |
| --- | --- | --- | --- |
| Borrowed divider label | Construct `Rc<String>` option fallbacks explicitly in both fixture rows instead of using unsupported `&str.into()` conversion. | Two v27 type errors; string option field is currently `Rc<String>`. | implemented_pending_validation |
| Borrowed command-palette query | Check the borrowed query pointer against the actual request's byte-slice pointer range, preserving the zero-copy contract and ignored 31-pair Release benchmark. | One v27 missing `String::as_ptr_range` error; byte-slice range stays within the original request allocation. | implemented_pending_validation |
| Authored document tab layout | Import the existing `DOCUMENT_TAB_GAP` constant from the document-tabs owner for the no-overlap assertion. | One v27 missing constant; production already uses the same layout constant. | implemented_pending_validation |
| Circular-progress cache | Give the synthetic topology fixture its current `target_size_bits`, using the size-to-target mapping of the production default; retain front-hit and LRU assertions and ignored performance benchmark. | One v27 missing struct field; production default target is `size as f32`. | implemented_pending_validation |
| Authoring-world gateway | Make the facade mutable before its existing test-only gateway replacement; preserve stale-generation and serialized-access assertions. | One v27 borrow mutability error. | implemented_pending_validation |
| Native plugin status performance fixture | Pass the existing borrowed diagnostic iterator through `inspect` directly, without moving `String` values from borrowed items. | One v27 iterator item-type error; visits still increment once per inspected item. | implemented_pending_validation |
| Document toolkit lifecycle | Supply the fixture's existing successful reference validator when constructing one direct `FixtureToolkit` record. | One v27 missing field; matches `FixtureToolkit::new`'s default validator. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/divider/labels/text/capacity_tests.rs` | `6259345DC87C600E4ECD515F777330705A0CB751DDCBED81F96C39A0DB766BDD` |
| `zircon_editor/src/ui/retained_host/app/command_palette_actions/borrowed_window_request_tests.rs` | `8F8530034CFC52A31932C697F7F4B8F8F826105B1B502320BFEADB48F70CF524` |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests/mod.rs` | `BCA993B83054F646B7F6E36C3453BE526A6AB67FEB3ADC03535D6198669414C0` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_material_feedback/circular_progress/pixels/topology_front_hit_tests.rs` | `745D4F852E597DD75C2BBA85CD6A5A88BCE38FB6A412057B65A3758FA9B6B896` |
| `zircon_editor/src/tests/editing/authoring_world.rs` | `64A4EA6EA0F5F5A408109CA8FA5757A621056F3945F671437119ACCDE864088D` |
| `zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_load_state/optimization_tests.rs` | `206CC18E8853B0FE1274E29C1AE5E9F25A5A0D7FF5B7422D01E40DF1AB9AA502` |
| `zircon_editor/src/core/extension/toolkit/tests/lifecycle.rs` | `BD2CFCB437F2D3287B708A871C7BD532D064746B6809666EFE8D1F240ADA5C07` |

## Managed gate

The one-time coordinator admission check encountered an unrelated running
blocking Cargo lease. No second Cargo wave was queued or watched. These seven
post-v28 clean owners require a later grouped source-bound Rust library test,
then their original ignored Release thresholds, allocation budgets, and
product percentile measurements. Rustfmt and local diff checks are the only
completed checks; neither old compiler diagnostics nor static checks establish
passing tests or performance.
