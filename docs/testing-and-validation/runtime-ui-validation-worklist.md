---
status: in_progress
---

# Runtime Ui Validation Worklist

## UI authority and validation work

Runtime UI implements shared interface DTO and editor layout contracts; it does not create a second design language. Taffy tree construction/computation belongs only to taffy_bridge/compute.rs. Style mapping adapts DTOs, and full/incremental layout consumes UI_LAYOUT_PASS_ORDER.

### 切片 2.2 virtualization / scroll 边界声明

The virtualization/scroll boundary owns retained-child geometry windowing and invalidation on data/viewport/offset changes. retained_virtual_list_only_arranges_visible_window does not prove bounded instance materialization. Preserve scroll_offset_invalidates_virtualization_window and non_virtualized_scroll_offset_keeps_full_window_dirty_domain.

Rebuild, property transactions, pointer state, template actions, resource registration and Winit event-domain packaging keep their separate source owners. Source-only split statuses do not change event order, focus/pointer semantics or product acceptance.

### Recorded UI inventory

The 2026-09-10 snapshot retains source49, UI entry23, raw surface45, legacy full15/production0/files0, Taffy production254/files16, Runtime V2 anchors10, interface V2 anchors9, guard anchors19, Cargo anchors7 and doc anchors61. Missing doc/Cargo anchors and risks are empty in that dated snapshot, and mirror_docs_guard_present is true; these are retained source targets, not fresh measurements. The 2026-09-30 source-backed correction surface_source_module_entry_count = 43 excludes navigation/pointer directories without Rust source; it does not rewrite the raw45 historical snapshot. Legacy production debt is restricted to the declared retired vocabulary. ui/input/naming_boundary/layout/template Cargo gates and product/profile validation remain pending.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Retained source status identities

| Contract | Responsibility / evidence boundary |
|---|---|
| `runtime_09_15_ui_asset_surface_node_resource_owner_split_static_passed_cargo_profile_deferred` | Runtime registration remains tolerant; strict compile-time schema diagnostics stay separate. |
| `runtime_09_15_ui_pointer_component_state_owner_split_static_passed_cargo_profile_deferred` | Pointer state/invalidation remains separate from route/event envelope construction. |
| `runtime_09_15_ui_pointer_template_action_owner_split_static_passed_cargo_profile_deferred` | Action projection consumes existing tree/bindings/state without adding a dispatcher. |
| `runtime_09_15_ui_surface_incremental_rebuild_owner_split_static_passed_cargo_profile_deferred` | Dirty layout/rebuild belongs to the child; full rebuild and render extract remain in the parent. |
| `runtime_09_15_ui_surface_property_transaction_owner_split_static_passed_cargo_profile_deferred` | Validation, mutation and invalidation share the original surface transaction authority. |
| `runtime_09_15_winit_translation_domain_owner_split_static_passed_cargo_product_profile_deferred` | Keyboard/pointer/IME/window packaging preserves event order and normalization. |

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `pointer_component_events/template_action.rs`, `docs/crates/zircon_runtime/ui/`, `ui/layout/virtualization.rs`.
- `zircon_runtime/src/ui`, `ui/layout/scroll.rs`, `incremental.rs`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `runtime_09_taffy_layout_pass_order_uses_bridge_authority`, `routed_reply`.

### Historical command identities

These are exact original command identities, not current executable recipes. Actual execution needs explicit finite package/features/target/profile admission.

- `cargo test -p zircon_runtime --lib template --locked -- --nocapture`, `cargo test -p zircon_runtime --lib layout --locked -- --nocapture`, `cargo test -p zircon_runtime --lib input --locked -- --nocapture`, `cargo test -p zircon_runtime --lib naming_boundary --locked`.
- `cargo test -p zircon_runtime --lib ui --locked`, `cargo check -p zircon_runtime --lib --locked`.

### Contract details

The following original contract labels remain in scope; they do not establish new validation:

- `TaffyTree::new()`.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `09-ui-subsystem-architecture.md`, SHA-256 `6b7c6e18f36ee98dafb0f0d44e06a4bf67db37edefbb5af501aeef63c1833fbf`, selected source lines 62-376.
