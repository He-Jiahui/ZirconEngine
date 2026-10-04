---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
  - docs/plans/optimize/zircon_editor/130-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration/overlay_lifecycle.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/layout/tab_drop.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/asset_browser_content.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/command_palette.rs
---

# Editor917 Overlay, Layout, Asset Browser, and Palette Test Contract Repairs

## 计划完成列表

| Scope | Completed source change | Current evidence | Acceptance |
| --- | --- | --- | --- |
| Plugin overlay registration/retirement | Check the rejected invalid registration against the existing `SceneMode` error, and retain the successful contribution handle for both idempotent retirement assertions. | Terminal v27 reported two missing handle diagnostics and one unused rejected handle; source contract checked against the host's typed registration result. | implemented_pending_validation |
| Drag-to-drawer route | Read the mode through `active_activity_window_drawers()` rather than the removed root `drawers` field; preserve the event, dirty-effect, and auto-hide assertions. | Terminal v27 reported one stale layout field; existing window-layout getter inspected. | implemented_pending_validation |
| Browser visual-list fixture | Materialize the immutable visible-asset generation once into a test-local vector, clone its small base for repeated rows, and install one new generation after all append operations. | Terminal v27 reported an invalid `&generation` iterator and derived unknown item type; the generation exposes `iter()` and `From<Vec<_>>`, not `push`. | implemented_pending_validation |
| Command palette anchor fixture | Query the public surface's template metadata by control ID, as adjacent notification/menu tests do, without accessing private bridge internals; preserve the `Surface` anchor and absent positional attributes checks. | Terminal v27 reported two private member accesses; the existing `surface()` accessor and neighboring tests were checked. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/tests/editor_event/runtime/extensions_registration/overlay_lifecycle.rs` | `7573164758CAD8C3253B25A8767EB14A065C02E648BEA006306F14A4F71107FA` |
| `zircon_editor/src/tests/host/retained_callback_dispatch/layout/tab_drop.rs` | `3736BDE5A396ADBE6FBF618CA46D77CD05CF8E1BF313622880F2094BC4A7E0C7` |
| `zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/asset_browser_content.rs` | `11F598E8277AD39E6074C9938ED24C6FF775E048DA454FC646DE5BF6C1ECC56E` |
| `zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/command_palette.rs` | `B83B4EB488B9BF16809C1E2B3716593CD3DE9CA61361415E2E985CEC60EB066D` |

## Managed gate

All four clean test owners were repaired after v28 was submitted; that batch
terminated during Cargo metadata input resolution without compiling any
source. Exact Rustfmt checks pass, but no current-source Rust test, ignored
Release benchmark, allocator check, or product p50/p95/p99 performance gate
has run for these changes. Keep them in a later grouped managed validation
wave rather than invoking Cargo separately per fixture.
