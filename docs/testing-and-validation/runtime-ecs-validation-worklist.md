---
status: in_progress
---

# Runtime Ecs Validation Worklist

## ECS semantics and validation gates

Component/entity identity and generations remain world-local. StorageType Table/SparseSet, observer lifecycle, deferred commands, event/message resources and change-tick wrap behavior retain their original gates. Missing/despawned command targets require an explicit tested outcome, not silent loss.

Transferred-descriptor admission owns preflight, conflicts, staged IDs and final publication in component/registry/transferred.rs; it shares the original registry storage and six original tests. ArchetypeTopologySnapshot compares canonical indexes and ordered ID/signature/entity rows, excluding diagnostic counters and membership history. Sparse locators use their storage owner and retain pending product/profile acceptance.

### Inventory and pending validation

The retained inventories distinguish the 2026-08-01 source69 snapshot from the 2026-08-28 hard-cut source77 snapshot; the separate source76 index record retains its own provenance. Test count10 remains a declared target. They are not fresh scans. Pending entity/observer/command/messages/change_tick/ecs gates and exact behavior names remain visible; no completed static source status closes them.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Retained source status identities

| Contract | Responsibility / evidence boundary |
|---|---|
| `runtime_08_15_archetype_topology_equality_receipt_static_passed_cargo_deferred` | Compare canonical indexes and ordered structural rows, excluding diagnostics/history. |
| `runtime_08_15_component_registry_transfer_owner_split_static_passed_cargo_deferred` | Preflight, conflicts, staged IDs and final publication stay one transaction. |
| `runtime_08_15_scene_property_entry_component_owner_split_static_passed_cargo_profile_deferred` | Domain property enumeration preserves ordering and instance identity. |
| `runtime_08_60_sparse_component_locator_source_complete_cargo_product_profile_pending` | Sparse locators retain bounded page ownership and pending product/profile gates. |

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `zircon_runtime/src/scene/ecs/component/registry/transferred.rs`, `zircon_runtime/src/scene/ecs/component/registry.rs`, `src/scene/ecs/storage_type.rs`.
- `src/scene/ecs/entity/slot.rs`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `command_queue_on_despawned_entity_target_is_reported_not_silently_dropped`, `change_tick_comparison_survives_wraparound`, `tick_window_clamps_stale_ticks`, `ecs_kernel_data_boundary`.

### Recorded targets

These retained targets belong to the original source snapshots, not a new measurement. The source dates/identities below keep distinct inventories separate.

- `component_storage_private_reexport_anchors = 9/9`; source 2026-08-01:L90.
- `missing_behavior_test_anchors = []`; source 2026-08-01:L90.
- `component_identity_anchors = 18/18`; source 2026-08-01:L90.
- `resource_identity_anchors = 12/12`; source 2026-08-01:L90.
- `runtime_08_guard_anchors = 21/21`; source 2026-08-01:L90.
- `pending_cargo_gate_anchors = 6/6`; source 2026-08-01:L90.
- `entity_lifecycle_anchors = 10/10`; source 2026-08-01:L90.
- `deferred_command_anchors = 11/11`; source 2026-08-01:L90.
- `expected_source_file_count = 77`; source 2026-08-28:L88.
- `behavior_test_anchor_count = 16`; source 2026-08-01:L90.
- `expected_test_file_count = 10`; source 2026-08-28:L88.
- `event_message_anchors = 12/12`; source 2026-08-01:L90.
- `change_tick_anchors = 6/6`; source 2026-08-01:L90.
- `archetype_anchors = 15/15`; source 2026-08-01:L90.
- `observer_anchors = 8/8`; source 2026-08-01:L90.
- `storage_anchors = 9/9`; source 2026-08-01:L90.
- `doc_anchors = 13/13`; source 2026-08-01:L90.

### Historical command identities

These are exact original command identities, not current executable recipes. Actual execution needs explicit finite package/features/target/profile admission.

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter change_tick`, `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter observer`, `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter messages`, `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter command`.
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ecs`, `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter entity`.

### Contract details

The following original contract labels remain in scope; they do not establish new validation:

- `mirror_docs_guard_present = true`.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `08-ecs-kernel-data-alignment.md`, SHA-256 `8e38409d30af35709ec5800a36800d73c7c3ccd167d8dd8192214305e85ad2be`, selected source lines 82-325.
