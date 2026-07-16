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

The ZrVM language plugin re-exports the shared Script VM M5 contracts instead of defining a plugin-local snapshot or reflection schema. Public backend-facing types include `VmStateBlob`, `VmStateTypeIdentity`, `VmStateObject`, `VmStateSchema`, `VmStateTypeSchema`, `VmStateFieldRename`, and `VmStateMigrationError`. Field metadata is always `ReflectTypeRegistration`; there is no second ZrVM-only field description.

The feature-gated ZrVM instance adapter is owned only by `zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs` and uses one hard-cut v2 lifecycle protocol: `saveState` returns JSON for the complete `VmStateBlob`, and `restoreState` receives JSON for the complete blob. `VmStateBlob::from_json` validates reflected snapshots with non-empty type tables; `to_json` preserves the schema version, type identities, hashes, and payload as one envelope. Null or absent `saveState` still produces the v2 default blob. Raw payload-only lifecycle strings are no longer accepted.

## Reflected and opaque modes

The shared `VmPluginInstance::state_schema` hook defaults to `None`:

- `None` means the backend owns opaque bytes. The coordinator transfers the v2 blob without field migration.
- `Some(VmStateSchema)` opts into reflected migration. The payload must then be a reflected `VmStateObject` array and its types must be declared by the blob's authoritative identity table.

The single feature-gated real ZrVM adapter queries an optional `stateSchema` lifecycle export. A missing or null export selects opaque mode. A string export must be JSON for `VmStateSchema`; decode failures are typed `SchemaDecode` errors, and a valid schema activates the same reflected rename/default migration used by every backend. The source `saveState` blob must carry the corresponding old type table, while `stateSchema` describes the new generation. The external build now exists under `E:/Git/zr_vm/build`; production-adapter acceptance and its coordinator commit remain owned by Plugins 08 M4 rather than this docs-only G7 batch.

## Reload behavior

On a reflected reload, the host coordinator migrates values before invoking the new instance's `restoreState`. Current field names take precedence, rename mappings preserve historical values, defaults fill newly added fields, removed fields are discarded, and missing required fields fail with a typed error.

Any schema, migration, or restore failure deactivates the new instance, removes its generation registrations, reactivates the old instance with its saved capability/source/root context, restores the old state, and reinstates the exact pre-reload callback and extension-registration snapshot. The slot returns to the old manifest and generation only when old activation and state restoration succeed. Plugins must make `activate` safe for rollback re-entry and register host interfaces under the owner generation supplied by `VmPluginHostContext`.

## Validation status

The runtime-neutral M5 acceptance names pass exactly as planned: `state_blob_round_trips_with_schema`, `schema_change_migrates_fields`, and `migration_failure_rolls_back_old_module`. Two additional tests enforce the source type table. The complete Windows `script::vm` suite passes 86/86, and the default `zircon_plugin_zr_vm_language_runtime` package passes 11/11 with doctests 0/0 under fixed toolchain `1.94.1-x86_64-pc-windows-msvc`, `--locked --offline --jobs 1`.

The Plugins 08 M4 output record reports a managed Windows `backend-zr-vm` real-backend run with 15/15 passing and doc-tests 0 failures, plus a passing default-feature package matrix. This Frameworks 06 G7 batch does not rerun or promote that foreign milestone; it only records the current single-owner path while Plugins 08 retains responsibility for committing its exact source and evidence manifest. Full runtime details are in [`../../zircon_runtime/script/vm/state_migration.md`](../../zircon_runtime/script/vm/state_migration.md).
