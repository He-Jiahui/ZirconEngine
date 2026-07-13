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
- Dynamic scene uses schema `zircon.scene.dynamic-scene`, current version 1.
- Each owner declares one explicit 0-to-1 migration step and relies on the shared loader to reject missing, duplicate, out-of-order, mismatched, or future headers before current payload decoding.

The reflected JSON migration treats every unwrapped v0 value as arbitrary JSON and unconditionally wraps it in `ReflectedValue::Json`. It never guesses tagged shapes as typed reflection variants. The exact retired `{ "uuid", "url" }` asset-reference walker is owned once by `zircon_runtime_interface::project`; both reflected and dynamic-scene migration call that owner directly. The UUID becomes `AssetRef.guid`; a required `res://` locator becomes an `assets/`-relative `path_hint`; its fragment becomes `sub`. Other objects are traversed without aliases or heuristic field renaming.

Dynamic-scene v0 also accepts the retired project-document `{ "world": ... }` data long enough to perform the one-way migration. Serialized world maps are projected through pure `serde_json::Value` transformation; no old Rust DTO is deserialized or retained. The migration materializes the current scene payload and applies the same exact legacy asset-reference rewrite. No runtime type alias, compatibility module, or fallback writer survives the cutover.

## Header authority and canonical output

`DynamicScene::ensure_supported` checks its private payload-header state, whose schema id and version are established by the versioned persistence owner. Runtime-session slots serialize every embedded scene as a complete `$zircon` envelope and validate that header before payload decoding. The inner `format_version` field remains dual-written only until Plan 11 M2.2 removes it; writers normalize it to 1 and it is not the support authority.

Successful saves recursively order JSON object keys, use shortest finite float text, pretty-print, and end with exactly one newline. Therefore load, migrate, save, reload, and resave produce the same value and byte-identical current-version text. Unsupported formats, non-finite payloads, malformed documents, schema mismatches, future versions, migration failures, and payload decode failures remain typed.

## Current validation

The managed `zircon_runtime_interface` package gate passed, including canonical-writer contracts, and the target-server runtime library check passes. The standalone Plan 11 integration binary passes 5/5: v0 tag-shape preservation, pure-value project-world migration, real v0 dynamic-scene byte-idempotence, embedded scene envelope/inner-version normalization, and future embedded-header rejection before payload decode. The package command still exits after compiling that binary because existing package binaries import graphics, script, or dynamic-api modules while those features are disabled; the compiled integration binary was therefore executed directly for focused evidence. The managed default-feature build remains blocked before this crate by the `wgpu-hal` Windows 0.54/0.62 split recorded under Runtime 01. None of these external failures is converted into scene compatibility code.
