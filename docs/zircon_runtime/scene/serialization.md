---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/reflect/mod.rs
  - zircon_runtime/src/scene/reflect/conversion.rs
  - zircon_runtime/src/scene/reflect/json_document
  - zircon_runtime/src/scene/dynamic_scene/document
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/validation.rs
  - zircon_runtime/src/scene/dynamic_scene/error.rs
  - zircon_runtime_interface/src/serialization
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/project/retired_asset_ref_migration
implementation_files:
  - zircon_runtime/src/scene/reflect/json_document/mod.rs
  - zircon_runtime/src/scene/reflect/json_document/document.rs
  - zircon_runtime/src/scene/reflect/json_document/error.rs
  - zircon_runtime/src/scene/reflect/json_document/migration.rs
  - zircon_runtime/src/scene/reflect/json_document/read.rs
  - zircon_runtime/src/scene/reflect/json_document/schema.rs
  - zircon_runtime/src/scene/reflect/json_document/write.rs
  - zircon_runtime/src/scene/dynamic_scene/document/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document/migration/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document/migration/project_world.rs
  - zircon_runtime/src/scene/dynamic_scene/document/read.rs
  - zircon_runtime/src/scene/dynamic_scene/document/schema.rs
  - zircon_runtime/src/scene/dynamic_scene/document/write.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/validation.rs
  - zircon_runtime/src/scene/dynamic_scene/error.rs
  - zircon_runtime/src/scene/dynamic_scene/session/construction/serialization.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot/scene_document.rs
  - zircon_runtime_interface/src/project/retired_asset_ref_migration/migrate.rs
plan_sources:
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation/versioned_json.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/scene_patch_document.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_core.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_runtime/src/scene/tests/component_structure/dynamic_scene_owner_tree.rs
  - zircon_runtime/tests/plan11_scene_serialization_contract.rs
  - tests/fixtures/serialization/scene-reflection/v0/reflected-value.json
  - tests/fixtures/serialization/scene-dynamic/v0/dynamic-scene.json
doc_type: module-detail
---

# Scene Versioned Serialization

## Ownership

Scene reflected JSON is owned by `scene::reflect::json_document`; dynamic-scene text persistence is owned by `scene::dynamic_scene::document`. Both owners use `zircon_runtime_interface::serialization` for the shared `$zircon` envelope, migration chain, typed load/write errors, and canonical text writer. Root modules remain structural and do not contain parsing or migration behavior.

The public reflected JSON API accepts and returns text documents. It no longer exposes the retired unversioned `serde_json::Value` conversion contract. `DynamicScene::from_versioned_json` and `DynamicScene::to_versioned_json_pretty` likewise read and write only the versioned envelope.

## Schemas and migrations

- Reflected JSON uses schema `zircon.scene.reflected-json`, current version 1.
- Dynamic scene uses schema `zircon.scene.dynamic-scene`, current version 2.
- Reflected JSON declares one explicit 0-to-1 migration. Dynamic scene declares 0-to-1 for the original envelope payload and 1-to-2 for the M2.2 inner-version removal. The shared loader rejects missing, duplicate, out-of-order, mismatched, or future headers before current payload decoding.

The reflected JSON migration treats every unwrapped v0 value as arbitrary JSON and unconditionally wraps it in `ReflectedValue::Json`. It never guesses tagged shapes as typed reflection variants. The exact retired `{ "uuid", "url" }` asset-reference walker is owned once by `zircon_runtime_interface::project`; both reflected and dynamic-scene migration call that owner directly. The UUID becomes `AssetRef.guid`; a required `res://` locator becomes an `assets/`-relative `path_hint`; its fragment becomes `sub`. Other objects are traversed without aliases or heuristic field renaming.

Dynamic-scene v0 also accepts the retired project-document `{ "world": ... }` data long enough to perform the one-way migration. Serialized world maps are projected through pure `serde_json::Value` transformation; no old Rust DTO is deserialized or retained. The 0-to-1 step materializes the historical v1 payload and applies the exact legacy asset-reference rewrite. The 1-to-2 step first requires the historical inner `format_version` to exist and equal 1, then removes it; malformed v1 envelopes fail as typed migration errors instead of being widened into a compatibility read. No runtime type alias, compatibility module, fallback writer, or current inner-version field survives the cutover.

## Header authority and canonical output

`DynamicScene::ensure_supported` checks its private payload-header state, whose schema id and version are established by the versioned persistence owner. Runtime-session slots serialize every embedded scene as a complete `$zircon` envelope and validate that header before payload decoding. Plan 11 M2.2 deleted `DynamicScene.format_version`, `DYNAMIC_SCENE_FORMAT_VERSION`, and their root re-exports; current payloads therefore have exactly one schema authority at `$zircon.header.schema_version`. Current-version payload decoding denies unknown fields, so restoring the retired inner field fails as a typed payload-decode error instead of silently preserving a compatibility path. Runtime-session summaries derive their reported scene version from the private header.

Successful saves recursively order JSON object keys, use shortest finite float text, pretty-print, and end with exactly one newline. Therefore load, migrate, save, reload, and resave produce the same value and byte-identical current-version text. Unsupported formats, non-finite payloads, malformed documents, schema mismatches, future versions, migration failures, and payload decode failures remain typed.

## Current validation

The Plan 11 contracts cover v0 tag-shape preservation, pure-value project-world migration, real v0 dynamic-scene byte-idempotence, current payloads with no inner version, embedded scene envelope authority, and future embedded-header rejection before payload decode. Validation results for the M2.2 hard cut are recorded in `docs/plans/zircon_editor/editor/11/2026-07-14-dynamic-scene-version-shell-hard-cut.md`; external package failures are routed to their owning plans instead of being converted into scene compatibility code.
