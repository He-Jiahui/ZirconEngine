---
related_code:
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
  - zircon_runtime/src/script/vm/plugin/vm_state_blob.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_instance.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/state_migration.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
implementation_files:
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_runtime/src/script/vm/plugin/vm_state_blob.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_instance.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
plan_sources:
  - user: 2026-07-13 implement the complete engine plugin architecture plan
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/script/vm/plugin/vm_state_blob.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/state_migration.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
doc_type: module-detail
---

# ZrVM State Migration Integration

## Plugin boundary

The ZrVM language plugin re-exports the shared Script VM M5 contracts instead of defining a plugin-local snapshot or reflection schema. Public backend-facing types include `VmStateBlob`, `VmStateTypeIdentity`, `VmStateObject`, `VmStateFieldValue`, `VmStateSchema`, `VmStateTypeSchema`, and `VmStateMigrationError`. Field metadata is always `ReflectTypeRegistration`; there is no second ZrVM-only field description.

The feature-gated ZrVM instance adapter is owned only by `zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs` and uses one hard-cut v3 lifecycle protocol: `saveState` returns JSON for the complete `VmStateBlob`, and `restoreState` receives JSON for the complete blob. `VmStateBlob::from_json` validates reflected snapshots with non-empty type tables; `to_json` preserves the producer schema version, type identities, hashes, and payload as one envelope. Null or absent `saveState` produces the V3 default blob. Raw payload-only lifecycle strings and reflected V2 `field_name` payloads are no longer accepted.

## Reflected and opaque modes

The shared `VmPluginInstance::state_schema` hook defaults to `None`:

- `None` means the backend owns opaque bytes. The coordinator transfers the versioned blob without field migration.
- `Some(VmStateSchema)` opts into reflected migration. The payload must then be a reflected `VmStateObject` array and its types must be declared by the blob's authoritative identity table.

The single feature-gated real ZrVM adapter queries an optional `stateSchema` lifecycle export. A missing or null export selects opaque mode. A string export must be JSON for `VmStateSchema`; decode failures are typed `SchemaDecode` errors, and a valid schema activates the same reflected stable-ID/default migration used by every backend. The source `saveState` blob must carry the corresponding old type table, while `stateSchema` describes the new generation. The external build now exists under `E:/Git/zr_vm/build`; production-adapter acceptance remains a managed validation concern rather than evidence implied by this document.

## Reload behavior

On a reflected reload, the host coordinator migrates values before invoking the new instance's `restoreState`. `ReflectFieldId` is the only field lookup key; a current-name rename retains its stable identity key and needs no rename map. Defaults fill newly added field IDs, removed IDs are discarded, and missing required fields fail with a typed error. Old `renames` schema members are rejected by `deny_unknown_fields`.

Any schema, migration, or restore failure deactivates the new instance, removes its generation registrations, reactivates the old instance with its saved capability/source/root context, restores the old state, and reinstates the exact pre-reload callback and extension-registration snapshot. The slot returns to the old manifest and generation only when old activation and state restoration succeed. Plugins must make `activate` safe for rollback re-entry and register host interfaces under the owner generation supplied by `VmPluginHostContext`.

## Validation status

The current V3 source adds focused coverage for round-trip, stable-ID rename preservation, legacy `field_name`/`renames` rejection, duplicate-ID admission, migration defaults, rollback, and the feature-gated real backend's V3 schema projection. Scoped rustfmt and diff checks pass. The exact Windows `script::vm` and `backend-zr-vm` release gates are managed validation work and remain pending until their current-source tickets complete.

Earlier Plugins 08 M4 evidence predates the V3 field-identity cut and is not promoted as acceptance for the current source. Full runtime details are in [`../../zircon_runtime/script/vm/state_migration.md`](../../zircon_runtime/script/vm/state_migration.md).
