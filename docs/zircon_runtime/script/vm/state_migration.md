---
related_code:
  - zircon_runtime/src/script/vm/plugin/vm_state_blob.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_instance.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/state_migration.rs
  - zircon_runtime/src/script/vm/backend/vm_error.rs
  - zircon_runtime/src/script/vm/backend/mock_vm_backend.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/field_info.rs
implementation_files:
  - zircon_runtime/src/script/vm/plugin/vm_state_blob.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_instance.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/backend/vm_error.rs
  - zircon_runtime/src/script/vm/backend/mock_vm_backend.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/mod.rs
plan_sources:
  - user: 2026-07-13 implement the complete engine plugin architecture plan
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/script/vm/plugin/vm_state_blob.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/state_migration.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/tests/lifecycle_failures.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
doc_type: module-detail
---

# Script VM State Migration

## Purpose and ownership

The Script VM state-migration module owns the runtime-neutral snapshot, schema, field migration, and hot-reload rollback contracts used by VM language plugins. The shared runtime owns these contracts because migration and rollback must behave identically for mock, ZrVM, and future VM backends. A backend may publish a schema through `VmPluginInstance::state_schema`; it does not get a private reflection model or a separate reload transaction.

This implementation closes the runtime-neutral Plugins 08 M5 contract while preserving the M4 validation boundary. The feature-gated real ZrVM adapters now consume an optional `stateSchema` JSON export, but their external library, collector, and schema-export execution cannot be validated until the M4 ZrVM build environment is available.

## `VmStateBlob` v2

`VmStateBlob` contains three fields:

- `schema_version`: the producer's state-schema version. `VM_STATE_SCHEMA_VERSION_V2` is the default.
- `types`: the authoritative `VmStateTypeIdentity` table. Each entry contains one `ReflectTypePath` and the producer's `u32` type hash.
- `payload`: backend bytes. Reflected migration encodes a JSON array of `VmStateObject`; each object carries its type path and ordered `ReflectFieldValue` entries.

`from_reflected_objects`, `reflected_objects`, and `validate_reflected` are the centralized reflected-payload boundary. Construction and decoding both enforce unique type identities, declared object types, and unique field names per object; encode and decode failures become `VmStateMigrationError` rather than stringly backend failures. `from_json` / `to_json` define the cross-language lifecycle envelope for a complete versioned blob. `from_payload` remains available to runtime-neutral opaque backends and produces a v2 blob with an empty type table. Such a blob may be restored unchanged when the destination returns `state_schema = None`; a destination that opts into reflected migration must provide reflected payload and authoritative type identities.

The migration path rejects duplicate source type identities and any payload object whose type path is absent from the source type table. This prevents a payload from silently claiming a type not described by its schema metadata.

## One reflection model

`VmStateTypeSchema` embeds `zircon_runtime_interface::reflect::ReflectTypeRegistration`. Serializable destination fields, field defaults, the type path, and serialization eligibility all come directly from that registration. M5 deliberately removed the temporary duplicate field-schema DTO, so Inspector, replication, VM type registration, and state migration consume the same `ReflectTypeRegistration` / `ReflectFieldInfo` model.

The migration-only additions are limited to:

- the destination `type_hash` written into the new identity table as revision metadata;
- `VmStateFieldRename { from, to }` declarations for historical field names;
- the destination `schema_version`.

Target registrations must be serializable. Duplicate target type paths, duplicate serializable field names, duplicate rename sources or destinations, and rename destinations absent from the serializable target field set are typed errors. M5 carries the source hash and rewrites it to the target hash; it does not use hash equality as a migration gate because explicit field migration is selected by the stable fully qualified type path.

## Field migration algorithm

`migrate_vm_state_blob` performs a deterministic transaction:

1. Validate and index the source identity table.
2. Validate and index serializable target registrations by fully qualified type path.
3. Decode reflected objects and require each object type to be declared by the source table and present in the target schema.
4. Reject duplicate source fields.
5. Visit serializable destination fields in `ReflectTypeInfo` order. A field with the current name wins; otherwise its declared historical name is consumed.
6. If neither source name exists, clone `ReflectFieldInfo::default_value`. A field without a source value or default returns `MissingRequiredField`.
7. Drop source fields not present in the destination and emit objects with the destination type path, destination schema version, and destination type hashes.

The algorithm is value-preserving rather than coercive: M5 does not invent numeric or container conversions. A future conversion layer must be explicit and typed instead of weakening the current deterministic behavior.

## Hot-reload transaction and rollback

For a preserve-state reload, `HotReloadCoordinator` snapshots the current host-interface generation, saves the current state, and retains the current generation's exact `VmPluginHostContext`. It then deactivates the current instance, loads and activates the next generation, requests its optional schema, migrates when a schema is present, and restores the resulting blob. Only after successful restoration does it discard the old generation's host-interface registrations and publish the next slot generation.

Schema discovery, migration, or restore failure invokes one rollback path:

1. Deactivate the next instance.
2. Discard every next-generation host-interface registration.
3. Temporarily remove current-generation descriptors so rollback activation cannot collide with existing IDs.
4. Invoke the old instance with its original capability set, package source, roots, and owner generation.
5. Restore the saved old state after successful reactivation.
6. Replace the callback table and all four extension channels with the exact pre-reload registration snapshot, discarding any provisional reactivation registrations.
7. Restore the old instance into the slot as `Active`; if reactivation or state restoration fails, retain it as `Failed`.

When new-instance cleanup, old-instance reactivation, and old-state restoration all succeed, the original typed migration error is returned. If rollback cleanup fails, `VmError::Operation` includes the primary error plus new deactivation, old activation, and old restoration outcomes so the secondary failure is not hidden. The coordinator never holds its slot-table mutex across backend lifecycle calls.

## Validation and reference evidence

Windows validation used fixed toolchain `1.94.1-x86_64-pc-windows-msvc`, a coordinator-managed retained target, `--locked --offline --jobs 1`, and the `core-min,script,net-contracts` runtime feature set. The focused migration group passed 4/4; the complete `script::vm` domain passed 86/86. The default ZrVM plugin package passed 11/11 plus an empty doctest set.

The design follows Godot's rule that extension reload must retain explicit class/instance ownership and clean registration state (`dev/godot/core/extension/gdextension.cpp`), Bevy's use of dynamic reflection metadata for field-wise application (`dev/bevy/crates/bevy_reflect/src`), and Fyrox's explicit versioned visitor model. Zircon intentionally keeps rollback in the host coordinator and treats backend state bytes as opaque unless the backend opts into the shared reflection schema.
