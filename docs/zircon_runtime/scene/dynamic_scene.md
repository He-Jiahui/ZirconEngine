---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document.rs
  - zircon_runtime/src/scene/dynamic_scene/entity.rs
  - zircon_runtime/src/scene/dynamic_scene/error.rs
  - zircon_runtime/src/scene/dynamic_scene/patch.rs
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/session.rs
  - zircon_runtime/src/scene/dynamic_scene/session/error.rs
  - zircon_runtime/src/scene/dynamic_scene/session/io.rs
  - zircon_runtime/src/scene/dynamic_scene/session/manifest.rs
  - zircon_runtime/src/scene/dynamic_scene/session/merge.rs
  - zircon_runtime/src/scene/dynamic_scene/session/metadata.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_status.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports.rs
  - zircon_runtime/src/scene/dynamic_scene/session/retention.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_id.rs
  - zircon_runtime/src/scene/dynamic_scene/session/statistics.rs
  - zircon_runtime/src/scene/dynamic_scene/value.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/reflect_resource.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
implementation_files:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document.rs
  - zircon_runtime/src/scene/dynamic_scene/entity.rs
  - zircon_runtime/src/scene/dynamic_scene/error.rs
  - zircon_runtime/src/scene/dynamic_scene/patch.rs
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/session.rs
  - zircon_runtime/src/scene/dynamic_scene/session/error.rs
  - zircon_runtime/src/scene/dynamic_scene/session/io.rs
  - zircon_runtime/src/scene/dynamic_scene/session/manifest.rs
  - zircon_runtime/src/scene/dynamic_scene/session/merge.rs
  - zircon_runtime/src/scene/dynamic_scene/session/metadata.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_status.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports.rs
  - zircon_runtime/src/scene/dynamic_scene/session/retention.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_id.rs
  - zircon_runtime/src/scene/dynamic_scene/session/statistics.rs
  - zircon_runtime/src/scene/dynamic_scene/value.rs
plan_sources:
  - user: 2026-05-16 Bevy-grade ECS/reflect/scene/transform completion request
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - dev/bevy/crates/bevy_scene/src/scene.rs
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/mod.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/capture.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/load.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/path_management.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/persistence.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/queries.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/retention.rs
  - rustfmt --edition 2021 zircon_runtime/src/scene/dynamic_scene/session.rs zircon_runtime/src/scene/dynamic_scene/session/error.rs zircon_runtime/src/scene/dynamic_scene/session/io.rs zircon_runtime/src/scene/dynamic_scene/session/manifest.rs zircon_runtime/src/scene/dynamic_scene/session/retention.rs zircon_runtime/src/scene/dynamic_scene/mod.rs zircon_runtime/src/scene/mod.rs zircon_runtime/src/scene/tests/dynamic_scene.rs zircon_runtime/src/scene/tests/dynamic_scene_session/queries.rs zircon_runtime/src/scene/tests/mod.rs
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib scene::tests::dynamic_scene --locked --jobs 1 --message-format short
doc_type: module-detail
---

# Dynamic Scene Serialization

`zircon_runtime::scene::dynamic_scene` is the runtime-side dynamic scene layer for milestone M9 of the Bevy-grade ECS/reflect/scene plan. It sits above the fixed `World` scene records and the reflection registry, giving Zircon a serializable scene artifact that can be captured from one world and spawned into another with entity remapping.

The design follows the shape of Bevy's `bevy_scene` APIs rather than replacing Zircon's existing project save/load path. In Bevy, `DynamicScene` is reflection-driven scene data, `Scene` is a concrete world snapshot, and `ScenePatch` applies reflected scene data onto a world. Zircon keeps its fixed `NodeRecord` world boundary intact, then adds reflected component/resource payloads around it.

## Public Types

- `DynamicScene` is a versioned serializable snapshot. It owns `entities` and `resources`, and exposes `from_world` plus `spawn_into`.
- `DynamicEntity` stores the source entity id, its fixed `NodeRecord`, and reflected components found through the `TypeRegistry`.
- `DynamicComponent` stores a component type path, whether it is plugin-owned, and its serializable reflected fields.
- `DynamicResource` stores a resource type path and its serializable reflected fields.
- `EntityRemap` records old scene ids to target world ids. It preserves ids when available and allocates the next free id when the target world already contains a source id.
- `ScenePatch` wraps a `DynamicScene` and applies it to a target `World`.
- `RuntimeSessionArchive` is a versioned runtime session file with one or more `RuntimeSessionSlot` entries.
- `RuntimeSessionArchiveManifest` is the read-only index view of an archive, exposing sorted slot summaries plus latest/oldest update selectors without restoring worlds.
- `RuntimeSessionArchiveMergePolicy` selects whether archive imports reject duplicate slot ids, keep existing slots, or replace existing slots.
- `RuntimeSessionArchiveMergeReport` records inserted, replaced, and skipped slot ids after an archive import.
- `RuntimeSessionArchiveRetentionPolicy` configures runtime session archive pruning, including maximum retained slots and protected slot ids.
- `RuntimeSessionArchivePruneReport` records retained and removed slot ids after retention pruning.
- `RuntimeSessionArchivePathStatus` reports whether an archive path is missing, available with a validated manifest, or present but invalid with the load error.
- `RuntimeSessionArchiveStatistics` is the read-only diagnostic summary of an archive's slot count, entity/resource totals, per-slot maxima, update timestamp range, and untimed slot count.
- `RuntimeSessionLevelRestoreReport` reports a slot-to-level restore, including the restored slot id, level metadata, and entity count.
- `RuntimeSessionSlot` stores a slot id, runtime metadata, and a `DynamicScene` snapshot.
- `RuntimeSessionSlotDiffReport` reports whether a slot snapshot matches a target `World` or `LevelSystem` and records slot/target entity and resource counts.
- `RuntimeSessionSlotSummary` stores the slot id, runtime metadata, scene format version, entity count, and resource count.
- `RuntimeSessionMetadata` carries runtime-level project root, asset URI, display name, optional timestamps, and normalized tags without adding editor authoring state.
- `RuntimeSessionArchiveError` reports session I/O, JSON parse, dynamic-scene, archive version, duplicate-slot, missing-slot, and empty-slot failures.
- `DynamicSceneError` reports format, duplicate id, missing parent, entity id exhaustion, world mutation, reflected value conversion, and reflection errors.

## Versioned Documents

`DynamicScene::from_versioned_json` is the migration boundary for runtime scene JSON. It accepts both the new `DynamicScene` document shape and the current legacy project document shape that stores a serialized `World` under `world`.

When the parser sees a top-level `world` field, it deserializes the legacy world and immediately exports it through `DynamicScene::from_world`. New documents are parsed directly as `DynamicScene` and checked against `DYNAMIC_SCENE_FORMAT_VERSION`.

`DynamicScene::to_versioned_json_pretty` writes the new document shape. This keeps migration one-way: legacy files can load into the dynamic scene model, while new saves should be emitted as reflected dynamic scene documents instead of preserving the old `ProjectDocument { world }` wrapper.

`DynamicScene::ensure_supported` is the shared runtime validation boundary for direct dynamic-scene documents and session archive slots. It checks the dynamic scene format version and duplicate source entities before a scene is accepted by load, archive, manifest, apply, or restore paths.

## Runtime Session Archives

`RuntimeSessionArchive` is the runtime-owned save/session persistence boundary for dynamic scene data. It stores a stable archive format version and a canonical set of named slots sorted by slot id at construction, manifest projection, load, and pretty-JSON write time. Each `RuntimeSessionSlot` embeds a `DynamicScene` snapshot, so session persistence reuses the reflected component/resource capture path instead of inventing a gameplay save schema inside scene/project I/O.

Archive validation checks both the archive format and every embedded dynamic scene. It also requires slot ids in archive files and manually constructed slots to be canonical: non-empty and already trimmed. Capture helpers normalize user-provided ids before creating a slot, while `from_versioned_json`, `from_slots`, `push_slot`, and `upsert_slot` reject persisted or hand-built ids such as `" autosave "`. `push_slot` and `upsert_slot` also validate the embedded `DynamicScene` before mutating the archive, so manual insertion cannot park unsupported scene payloads until a later manifest/save/apply call. Slot metadata tags are normalized by trimming whitespace, dropping empty tags, sorting, and deduplicating before manifest projection or JSON write. A session JSON document with an unsupported slot scene version, duplicate source entities, duplicate slot ids, or non-canonical slot ids is rejected during `from_versioned_json`/`load_from_path`, before higher layers can present its manifest or attempt to restore it.

The archive API covers the runtime workflows that higher layers need first:

- capture a `World` or `LevelSystem` into a named slot,
- inspect, filter, or choose from a manifest of sorted slot ids, metadata, entity counts, and resource counts,
- read slot ids, existence, or one slot summary directly from an archive path without restoring worlds,
- filter manifest slot summaries directly from an archive path by normalized tag or display-name substring,
- choose latest/oldest updated slot ids directly from an archive path, globally or inside a tag bucket,
- inspect archive statistics for runtime diagnostics and retention decisions,
- upsert, copy, extract or atomically save a standalone one-slot archive, single-slot import, rename, update metadata, touch update timestamps, or remove slots in memory,
- copy, export one slot as a standalone archive, import one slot from a loaded archive or source archive path, rename, update metadata, touch update timestamps, remove slots, or apply retention pruning directly on an archive path through rollback-protected atomic save,
- preview retention pruning directly from an archive path without mutating the file,
- preview archive merge reports without mutating loaded archives or persisted archive paths,
- merge another session archive with an explicit duplicate-slot policy,
- prune old slots through a retention policy while protecting selected slots,
- write/read the archive as pretty JSON,
- load-or-create an archive path and atomically capture a world or level slot back to that path,
- apply a slot into an existing `World` or live `LevelSystem` with entity remapping,
- apply or restore a named slot directly from an archive path,
- restore a slot into an empty world while preserving free source entity ids,
- restore a slot directly into a `LevelSystem`, replacing the world, resetting cached runtime state, and writing slot metadata back to level metadata,
- diff a slot against a `World` or `LevelSystem` without mutating the target.

Session metadata mirrors runtime level metadata (`project_root`, `asset_uri`, `display_name`) and adds optional millisecond timestamps plus tags. Tags use a deterministic archive-local form so a runtime save selector can compare manifests and pretty JSON without caller-side cleanup. Session metadata intentionally does not serialize selection, viewport, inspector, gizmo, or other editor-authoring state.

`RuntimeSessionArchive::manifest` returns the lightweight runtime selection surface for save/load flows. The manifest keeps the archive format version and one `RuntimeSessionSlotSummary` per sorted slot, including normalized metadata, the embedded dynamic scene format version, and counts for captured entities/resources. This lets runtime code list slots and present basic session facts without applying a slot to a world. `RuntimeSessionArchive::load_manifest_from_path` applies the same archive load validation and then returns the manifest directly, so save selectors can read a persisted session index without open-coding `load_from_path(...).manifest()`. For narrower persisted-index reads, `slot_ids_from_path`, `contains_slot_from_path`, and `slot_summary_from_path` load the same validated manifest and return sorted slot ids, slot existence, or a cloned summary without writing the archive file or restoring worlds. `RuntimeSessionArchive::statistics_from_path` applies the same archive load validation and returns aggregate archive statistics without restoring worlds. `RuntimeSessionArchive::inspect_path` adds a non-throwing path probe for startup/recent-session flows: missing files return `RuntimeSessionArchivePathStatus::Missing`, readable valid archives return `Available { manifest }`, and present but unreadable or invalid files return `Invalid { error }`. `RuntimeSessionArchiveManifest::slots_with_tag` and `slots_matching_display_name` provide read-only filtering over that manifest, keeping save selector search logic on normalized summaries instead of requiring callers to restore worlds. `slots_with_tag_from_path` and `slots_matching_display_name_from_path` apply those same manifest filtering rules directly to a persisted archive path and return cloned summaries without writing the archive file. `latest_updated_slot` and `oldest_updated_slot` choose summaries by `updated_at_unix_millis` without world restore; missing timestamps are treated as `0`, and equal timestamps use the slot id as the deterministic tie-breaker. `latest_updated_slot_with_tag` and `oldest_updated_slot_with_tag` apply the same ordering after normalized tag filtering, so runtime code can choose the newest manual save or oldest autosave bucket without restoring worlds. `RuntimeSessionArchive::latest_updated_slot_id`, `oldest_updated_slot_id`, `latest_updated_slot_id_with_tag`, and `oldest_updated_slot_id_with_tag` expose these rules after archive validation for callers that only need the chosen id. `latest_updated_slot_id_from_path`, `oldest_updated_slot_id_from_path`, `latest_updated_slot_id_with_tag_from_path`, and `oldest_updated_slot_id_with_tag_from_path` apply those same selector rules directly to a persisted archive path without mutating the file or restoring a world.

The manifest types and query/selection helpers live in `zircon_runtime/src/scene/dynamic_scene/session/manifest.rs`, archive-specific errors live in `session/error.rs`, file I/O lives in `session/io.rs`, retention policy/report plus pruning algorithms live in `session/retention.rs`, slot data and slot-to-world/level behavior live in `session/slot.rs`, metadata normalization lives in `session/metadata.rs`, and small report/selection support declarations live in `session/reports.rs`, `session/merge.rs`, `session/statistics.rs`, `session/path_status.rs`, and `session/slot_id.rs`. `session.rs` re-exports the public session surface while remaining the archive persistence, archive-owned mutation, apply/restore, validation, and public facade owner.

`RuntimeSessionArchive::statistics` returns the lightweight runtime diagnostic surface for session archives. It validates the archive before summarizing, then reports slot count, total and maximum entity/resource counts, the earliest/latest `updated_at_unix_millis`, and how many slots lack update timestamps. The statistics view is intentionally aggregate-only: it helps runtime code make retention, telemetry, or diagnostics decisions without exposing authoring state or restoring worlds.

`RuntimeSessionArchive::copy_slot` and `RuntimeSessionArchive::copy_slot_with_metadata` are the runtime "save as" path for session slots. Copying clones the source slot's `DynamicScene` snapshot, normalizes the requested destination id, rejects collisions through the same `push_slot` path as manual inserts, and either preserves source metadata or stores a normalized metadata override.

`RuntimeSessionArchive::import_slot_from_archive` and `RuntimeSessionArchive::import_slot_from_archive_with_metadata` are the one-slot import path between already loaded session archives. They validate both archives, clone only the requested incoming slot, normalize the destination id, and then insert through the same archive-owned `push_slot` path used by copy/save-as. The default import preserves source slot metadata; the metadata override variant stores normalized caller-provided metadata while keeping the incoming `DynamicScene` snapshot unchanged.

`RuntimeSessionArchive::single_slot_archive` extracts one named slot into a new standalone archive while preserving the cloned slot id, metadata, and `DynamicScene` snapshot. `save_single_slot_archive_to_path_atomically` applies that extraction to an already loaded archive and writes the standalone one-slot archive through the rollback-protected atomic save path. This is the runtime export/share path for one save slot without forcing callers to manually clone slots, rebuild archive invariants, or first round-trip through a source archive file.

`RuntimeSessionArchive::merge_archive` is the runtime import path for combining session archives. It validates both archives before import and supports three explicit duplicate-slot policies: `RejectConflicts`, `KeepExisting`, and `ReplaceExisting`. Reject mode scans conflicts before mutation so a failed import cannot partially insert earlier non-conflicting slots. Keep-existing and replace-existing modes return `RuntimeSessionArchiveMergeReport` lists for inserted, skipped, and replaced slot ids. `preview_merge_archive` runs the same validation and policy/report calculation without mutating either loaded archive, letting save/load selectors show an import summary before committing the merge.

`RuntimeSessionArchive::prune_slots` is the runtime retention path for autosave/quicksave style archives. The policy can set `max_slots` and protect named slots from removal. Non-protected slots are kept by newest `updated_at_unix_millis`; slots without an update timestamp are treated as the oldest candidates. Protected slots are allowed to exceed `max_slots`, because explicit user or runtime pins should not be silently deleted by an automatic cleanup pass. The prune report records final retained ids and removed ids in canonical slot order. `RuntimeSessionArchive::prune_slots_with_tag` applies the same policy to one normalized metadata tag bucket, so an autosave cleanup can trim only `autosave` slots while preserving manual saves or other session categories. `preview_prune_slots` and `preview_prune_slots_with_tag` return the same report without mutating the archive, letting runtime save selectors show a cleanup preview before committing deletion.

`RuntimeSessionArchive::rename_slot`, `update_slot_metadata`, and `touch_slot` are the runtime save-selector mutation surface. Renaming normalizes the requested slot id, rejects collisions, and keeps the slot list sorted. Metadata updates normalize tags before storage, so UI/editor-host code does not need a separate cleanup pass before saving the archive back to disk. `touch_slot` updates only `updated_at_unix_millis`, preserving display names, tags, and source metadata for quicksave/autosave paths that need to refresh ordering without replacing the slot snapshot. Direct mutable slot access stays private: public callers can inspect slots through `slot`/`slots`, but slot mutation must go through archive-owned methods so duplicate-id, ordering, and metadata normalization invariants cannot be bypassed.

`RuntimeSessionArchive::diff_slot_with_world` and `RuntimeSessionArchive::diff_slot_with_level` are read-only preview helpers for overwrite/load decisions. They export the target through the same `DynamicScene::from_world` capture path used by archive slots, compare the slot snapshot with that target scene, and return `RuntimeSessionSlotDiffReport` with match status plus slot/target entity and resource counts. The helpers do not apply, restore, or mutate the world.

`RuntimeSessionArchive::to_versioned_json_pretty` also writes canonical slot order and normalized slot metadata tags. This matters for manually constructed archives as well as archives built through helper APIs: the serialized document is deterministic even if the in-memory `slots` vector was not already sorted.

`RuntimeSessionArchive::save_to_path_atomically` writes the same canonical pretty JSON through a same-directory temporary file before committing it to the requested path. The method creates the parent directory, writes the temporary payload first, then moves a pre-existing target file to a same-directory backup before renaming the temporary file into place. A successful commit removes the backup; a failed commit removes the temporary file and attempts to restore the backup to the original target path. Non-file targets are rejected before the old target is moved. This gives runtime session persistence a safer rollback-protected default for autosave/quicksave writes than direct target-file overwrite while keeping `save_to_path` available for simple callers.

`RuntimeSessionArchive::load_or_empty_from_path` is the startup helper for runtime session persistence. A missing archive path produces an empty archive, while a present but unreadable or invalid archive still returns an error. `capture_world_slot_to_path_atomically` and `capture_level_slot_to_path_atomically` build on that rule for quicksave/autosave flows: load an existing archive or create an empty one, capture or upsert the requested slot, save through the rollback-protected atomic path, and return the updated manifest. These helpers deliberately do not overwrite corrupted archive files, so startup/recent-session code can surface invalid-session diagnostics instead of silently replacing user data.

`RuntimeSessionArchive::restore_slot_from_path_to_empty_world`, `restore_slot_from_path_into_level`, `apply_slot_from_path_to_world`, and `apply_slot_from_path_to_level` are the path-level load side of the same session workflow. They load and validate the archive from disk, resolve the named slot through the existing archive-owned slot lookup, then delegate to the same restore/apply logic used by in-memory archives. Missing archive files, malformed archives, unsupported embedded scene versions, and missing slot ids all remain ordinary `RuntimeSessionArchiveError` results; the helper layer does not add fallback slot selection or gameplay save-schema inference.

`RuntimeSessionArchive::diff_slot_from_path_with_world` and `diff_slot_from_path_with_level` are the path-level read-only preview helpers. They load and validate the archive from disk, resolve the named slot, then delegate to the same in-memory diff logic without mutating the target world or level.

`RuntimeSessionArchive::rename_slot_at_path_atomically`, `update_slot_metadata_at_path_atomically`, `touch_slot_at_path_atomically`, and `remove_slot_at_path_atomically` are the path-level save-selector mutation side. Each helper loads and validates the existing archive file, applies exactly one archive-owned slot mutation, saves through the same rollback-protected atomic path, and returns the updated manifest. Missing archive files, invalid archives, duplicate rename targets, and missing slot ids are surfaced as ordinary `RuntimeSessionArchiveError` values before a replacement file is committed.

`RuntimeSessionArchive::prune_slots_at_path_atomically` and `prune_slots_with_tag_at_path_atomically` are the path-level retention commit helpers. They load and validate the existing archive file, reuse the same in-memory retention algorithms as `prune_slots` and `prune_slots_with_tag`, save the pruned archive atomically, and return the retained/removed slot report. `preview_prune_slots_from_path` and `preview_prune_slots_with_tag_from_path` load and validate the same persisted archive but return the retained/removed report without writing the archive file, so save selectors can show a dry-run cleanup report before calling the atomic commit helpers.

`RuntimeSessionArchive::copy_slot_at_path_atomically`, `copy_slot_with_metadata_at_path_atomically`, `save_single_slot_archive_to_path_atomically`, `save_single_slot_archive_from_path_atomically`, `import_slot_from_archive_at_path_atomically`, `import_slot_from_archive_with_metadata_at_path_atomically`, `import_slot_from_archive_path_at_path_atomically`, `import_slot_from_archive_path_with_metadata_at_path_atomically`, `merge_archive_at_path_atomically`, and `merge_archive_from_path_at_path_atomically` are the save-as/import/export commit helpers. Copying loads one archive file, duplicates a slot through the archive-owned copy semantics, saves atomically, and returns the updated manifest. Loaded-archive single-slot export extracts the named slot from `self`, saves the standalone one-slot archive atomically to a target path, and returns the standalone manifest without mutating the loaded source archive. Path single-slot export loads the source archive path, applies the same extraction, saves that archive atomically to a distinct target path, and returns the standalone manifest without mutating the source archive file. Loaded-incoming single-slot import accepts an already loaded incoming archive, clones only one source slot into the target archive path, saves atomically, and returns the updated manifest. Source-path single-slot import explicitly loads and validates the incoming archive path first, then delegates to the same loaded-incoming import path so destination id normalization, optional metadata override, target rollback protection, and source-file immutability stay identical. Loaded-incoming merge accepts an already loaded incoming archive, reuses the existing duplicate-slot merge policy, saves the target archive atomically, and returns the merge report. Source-path merge explicitly loads and validates the incoming archive path first, then delegates to the same loaded-incoming merge path so reject/keep/replace policy behavior, merge reports, target rollback protection, and source-file immutability stay identical. `preview_merge_archive_at_path` and `preview_merge_archive_from_path_at_path` load the same target/source archive shapes but return the merge report without writing either archive path, so UI and runtime save selectors can display pending inserts, replacements, skips, or conflict errors before committing. The loaded-incoming variants intentionally do not read a second path; the source-path variants make cross-path coordination explicit at the API boundary.

`RuntimeSessionArchive::restore_slot_into_level` is the runtime-level load path. It restores the slot into a fresh `World`, replaces the `LevelSystem` world through `replace_world_and_reset_runtime_state`, then updates `LevelMetadata` from the slot metadata. Resetting cached physics, animation, and script-start state is part of the contract so stale per-frame state from the previous world cannot survive a session load.

`RuntimeSessionArchive::apply_slot_to_level` is the runtime-level instancing path. It applies the slot's dynamic scene into the current `LevelSystem` world with the same entity remapping semantics as `DynamicScene::spawn_into`, preserving existing level entities and returning the `EntityRemap` for the newly spawned scene content.

## Capture Flow

`DynamicScene::from_world` reads the sorted scene nodes from `World::node_records`, expands them back into `NodeRecord` values, and fills the fixed fields that `SceneNode` does not expose directly by asking the world for active state, render layer mask, and mobility.

For every entity, the dynamic scene exporter iterates `world.type_registry().iter()` and keeps registrations that are:

- components,
- marked serializable,
- backed by a `ReflectComponent` adapter,
- present on the entity.

The adapter's `read_fields` result is filtered to schema fields marked serializable. This keeps the serialized payload bound to the reflection schema instead of dumping arbitrary adapter output.

Resources use the same rule with `ReflectResource`: serializable resource registrations with an adapter are captured only when the adapter reports that the resource exists in the source world.

## Spawn And Patch Flow

`DynamicScene::spawn_into` performs four ordered steps:

1. Validate the format version and duplicate source ids.
2. Build an `EntityRemap` by preserving each source id when the target world does not already contain it, otherwise walking to the next free id that is not already reserved by this scene spawn.
3. Insert remapped `NodeRecord` values into the target world, remapping parent ids and joint connected entities.
4. Apply reflected components and resources.

Plugin-owned dynamic components are applied as complete JSON objects built from all serialized reflected fields. This intentionally preserves read-only plugin fields such as labels or authored metadata, because attaching a full dynamic component is a scene instantiation operation, not an editor property write.

Non-plugin fixed components and reflected resources are applied through their reflection adapters. Only fields that still exist in the target schema and are both serializable and editable are written. Read-only fixed data is expected to be carried by `NodeRecord` when it belongs to the core scene model.

## Entity Reference Remapping

Entity ids inside reflected values are remapped when applying a scene:

- `ReflectedValue::Entity(Some(id))` maps through `EntityRemap`.
- nested `List` and `Map` values are remapped recursively.
- `Json` values recursively remap objects shaped as `{ "entity": <id-or-null> }`.
- resource references are left unchanged.

This is the first step toward Bevy-style scene instancing semantics: serialized entity references follow the instantiated entity ids instead of pointing back at the original source world.

## Current Boundaries

This module does not yet serialize component type descriptors, create missing reflected resources, or integrate with asset hot-reload and asynchronous scene spawning. Those remain follow-up milestones after M9. The current layer is intentionally scoped to runtime reflection data that the target world already knows how to interpret.

## Validation

M9 added `zircon_runtime/src/scene/tests/dynamic_scene.rs` with coverage for:

- serializing a world with a plugin-owned reflected dynamic component,
- deserializing the dynamic scene with `serde_json`,
- spawning into a target world where source entity ids collide and verifying the remap,
- preserving dynamic component JSON including read-only fields,
- applying a `ScenePatch` that writes reflected resource fields through a resource adapter.
- migrating a legacy `ProjectDocument { world }` JSON payload into the new versioned dynamic scene shape, then reloading the new JSON document.
- roundtripping a `RuntimeSessionArchive` slot, preserving runtime metadata, excluding authoring tokens, and restoring the slot into an empty world.
- rejecting duplicate runtime session slot ids.
- rejecting runtime session archives whose embedded dynamic scene version is unsupported.
- rejecting unsupported embedded dynamic scene payloads during direct slot push/upsert.
- rejecting runtime session archives with non-canonical persisted slot ids.
- serializing manually constructed runtime session archives in canonical slot order.
- normalizing runtime session metadata tags for manifest and pretty JSON output.
- renaming runtime session slots, preserving sorted slot ids, and updating normalized slot metadata.
- touching runtime session slot update timestamps without replacing slot metadata.
- copying runtime session slots for save-as flows, including destination id normalization, duplicate rejection, missing-source errors, and metadata override normalization.
- merging runtime session archives with reject/keep/replace duplicate-slot policies and a merge report.
- previewing runtime session archive merge reports without mutating loaded archives or persisted archive files.
- pruning runtime session archives by newest update timestamp while protecting selected slots and reporting retained/removed ids.
- pruning runtime session archives inside one normalized tag bucket without deleting slots outside that bucket.
- previewing runtime session retention cleanup reports without mutating the archive.
- saving runtime session archives through a same-directory temporary file before committing the target path, with same-directory backup rollback when replacing an existing archive.
- summarizing runtime session archive statistics without restoring worlds.
- filtering runtime session manifests by normalized tags and display-name substring without restoring worlds.
- loading a runtime session manifest directly from an archive path without manually projecting a full archive in the caller.
- reading runtime session slot id lists, existence, and one cloned slot summary directly from archive paths without mutating the archive.
- filtering runtime session manifest summaries directly from archive paths by normalized tag and display-name substring.
- selecting latest/oldest updated runtime session slot ids directly from an archive path, including normalized tag bucket selection, without mutating the archive.
- probing a runtime session archive path without treating a missing file as a corrupted archive.
- loading or creating a runtime session archive path, atomically capturing world/level slots into it, and rejecting invalid existing archives without overwrite.
- restoring or applying a named runtime session slot directly from an archive path while preserving the existing in-memory archive semantics.
- renaming, updating metadata, touching update timestamps, and removing slots directly through archive paths while preserving atomic save and manifest return semantics.
- pruning global or tagged retention buckets directly through archive paths while preserving atomic save and retained/removed report semantics.
- previewing global or tagged retention buckets directly from archive paths without mutating the persisted archive.
- copying slots, exporting one loaded slot or path slot as a standalone archive, importing one slot from a loaded incoming archive or source archive path, and merging loaded incoming archives or source archive paths directly into archive paths while preserving atomic save, manifest, and merge-report semantics.
- selecting latest and oldest updated runtime session slots without restoring worlds.
- selecting latest and oldest updated runtime session slots within a normalized tag bucket without restoring worlds.
- keeping runtime session manifest/query, archive error, file I/O, retention, metadata, slot behavior, reports, merge policy, path status, statistics, and slot-id validation folder-backed under `session/*` while preserving public re-exports.
- guarding the runtime session mutation surface so mutable slot access is not public.
- diffing runtime session slots against current worlds before overwrite/load decisions.
- summarizing sorted runtime session slots through the archive manifest.
- replacing an existing runtime session slot through upsert/capture and observing the updated manifest summary.
- restoring a runtime session slot directly into a `LevelSystem`, replacing stale world contents, updating metadata, and clearing cached runtime state.
- applying a runtime session slot into a live `LevelSystem`, preserving existing entities and remapping colliding source ids.

Validation commands for this milestone are:

```powershell
cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short
cargo test -p zircon_runtime --lib scene::tests::dynamic_scene --locked --jobs 1 --message-format short
```

Latest local evidence:

- `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short` passed on Windows after M9 implementation.
- After the versioned legacy migration slice, the same scoped check reached crate compilation but failed in an unrelated active asset/texture edit: `zircon_runtime/src/asset/assets/texture/texture_asset.rs:7:49` imports `TextureArrayLayout` from `asset::assets::texture`, where that symbol is not currently exported. This file is owned by active asset-image/texture sessions, so M9 did not modify it.
- After the active asset/texture export issue was resolved by its owning session, `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short` passed again.
- `cargo test -p zircon_runtime --lib scene::tests::dynamic_scene --locked --jobs 1 --message-format short` passed: 3 passed, 0 failed, 1434 filtered out. Earlier focused test attempts did not reach Zircon test execution because they stopped during dependency compilation while other workspace Cargo validations were active.
- Runtime session archive implementation added `RuntimeSessionArchive`, `RuntimeSessionSlot`, `RuntimeSessionMetadata`, and `RuntimeSessionArchiveError` on 2026-06-18. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new archive tests is deferred under the current implementation-first direction.
- Runtime session archive manifest implementation added `RuntimeSessionArchiveManifest` and `RuntimeSessionSlotSummary` on 2026-06-18. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new manifest tests is deferred under the current implementation-first direction.
- Runtime session archive level restore implementation added `RuntimeSessionLevelRestoreReport`, `RuntimeSessionArchive::restore_slot_into_level`, `RuntimeSessionSlot::restore_into_level`, and `LevelSystem::replace_world_and_reset_runtime_state` on 2026-06-18. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new level restore test is deferred under the current implementation-first direction.
- Runtime session archive level apply implementation added `RuntimeSessionArchive::apply_slot_to_level` and `RuntimeSessionSlot::apply_to_level` on 2026-06-18. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new live-level apply test is deferred under the current implementation-first direction.
- Runtime session archive embedded-scene validation added `DynamicScene::ensure_supported` as the shared validation boundary and makes `RuntimeSessionArchive::ensure_supported` reject unsupported slot scene versions on 2026-06-18. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new embedded-scene validation test is deferred under the current implementation-first direction.
- Runtime session archive slot write validation added embedded `DynamicScene::ensure_supported` checks to `push_slot` and `upsert_slot` on 2026-06-18 so direct slot writes fail before mutating the archive when a slot carries an unsupported scene payload. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new slot-write validation test is deferred under the current implementation-first direction.
- Runtime session archive canonical slot validation added `RuntimeSessionArchiveError::NonCanonicalSlotId` on 2026-06-18. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new canonical-slot validation test is deferred under the current implementation-first direction.
- Runtime session archive deterministic write ordering updated `RuntimeSessionArchive::to_versioned_json_pretty` on 2026-06-18 so manually constructed archives serialize slots in canonical order. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new deterministic serialization test is deferred under the current implementation-first direction.
- Runtime session archive metadata tag normalization added deterministic tag cleanup on 2026-06-18 so manifest and pretty JSON outputs trim, drop empty, sort, and deduplicate tags. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new metadata-tag normalization test is deferred under the current implementation-first direction.
- Runtime session archive slot management added `rename_slot` and `update_slot_metadata` on 2026-06-18 so runtime save selectors can rename slots and update normalized metadata without restoring world contents. Mutable slot lookup remains private to keep archive invariants centralized. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new slot-management test is deferred under the current implementation-first direction.
- Runtime session archive mutation-surface guard on 2026-06-18 keeps `slot_mut` private and adds a source-level behavior anchor so future public mutation must go through archive-owned methods. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new mutation-surface guard is deferred under the current implementation-first direction.
- Runtime session archive diff preview added `RuntimeSessionSlotDiffReport`, `diff_slot_with_world`, and `diff_slot_with_level` on 2026-06-18 so runtime code can compare a slot against current world/level contents without mutation. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new diff-preview test is deferred under the current implementation-first direction.
- Runtime session archive slot copy added `copy_slot` and `copy_slot_with_metadata` on 2026-06-18 so runtime save selectors can duplicate a slot or create a save-as slot with metadata override while preserving archive-owned id, duplicate, ordering, and metadata normalization invariants. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new slot-copy test is deferred under the current implementation-first direction.
- Runtime session archive merge/import added `RuntimeSessionArchiveMergePolicy`, `RuntimeSessionArchiveMergeReport`, and `merge_archive` on 2026-06-18 so runtime code can combine session archives under explicit reject/keep/replace duplicate-slot policy. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new merge-policy test is deferred under the current implementation-first direction.
- Runtime session archive retention pruning added `RuntimeSessionArchiveRetentionPolicy`, `RuntimeSessionArchivePruneReport`, and `prune_slots` on 2026-06-18 so runtime code can cap session slot count while protecting named slots and reporting retained/removed ids. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new retention-prune test is deferred under the current implementation-first direction.
- Runtime session archive statistics added `RuntimeSessionArchiveStatistics` and `statistics` on 2026-06-18 so runtime diagnostics can read aggregate slot/entity/resource/timestamp facts without applying or restoring slot worlds. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new statistics test is deferred under the current implementation-first direction.
- Runtime session archive manifest filters added `slots_with_tag` and `slots_matching_display_name` on 2026-06-18 so runtime save selectors can search normalized manifest summaries without applying or restoring slot worlds. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new manifest-filter test is deferred under the current implementation-first direction.
- Runtime session archive slot selection added `latest_updated_slot`, `oldest_updated_slot`, `latest_updated_slot_id`, and `oldest_updated_slot_id` on 2026-06-18 so runtime save/load selectors can choose default slots by update time without applying or restoring slot worlds. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new slot-selection test is deferred under the current implementation-first direction.
- Runtime session archive slot touch added `touch_slot` on 2026-06-18 so quicksave/autosave flows can refresh a slot's update timestamp without replacing display names, tags, metadata, or world snapshots. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new slot-touch test is deferred under the current implementation-first direction.
- Runtime session archive tagged slot selection added `latest_updated_slot_with_tag`, `oldest_updated_slot_with_tag`, `latest_updated_slot_id_with_tag`, and `oldest_updated_slot_id_with_tag` on 2026-06-18 so runtime save/load selectors can choose default slots inside normalized tag buckets without applying or restoring slot worlds. Rust formatting passed for the touched dynamic scene files; Cargo execution for the new tagged-slot-selection test is deferred under the current implementation-first direction.
- Runtime session archive module split moved `RuntimeSessionArchiveManifest`, `RuntimeSessionSlotSummary`, manifest query/selection helpers, `RuntimeSessionArchiveError`, retention policy/report, and prune algorithms into folder-backed session children on 2026-06-18 while preserving the public `scene::dynamic_scene` and `scene` exports. Rust formatting passed for the touched dynamic scene files; Cargo execution is deferred under the current implementation-first direction.
- Runtime session archive tagged retention pruning added `RuntimeSessionArchive::prune_slots_with_tag` on 2026-06-18 so runtime autosave/quicksave cleanup can prune one normalized tag bucket while retaining slots outside the bucket. Rust formatting passed for the touched dynamic scene files; `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short` passed with existing warnings only. Cargo execution for the new `dynamic_scene_session` behavior anchors remains deferred under the current implementation-first direction.
- Runtime session archive retention preview added `preview_prune_slots` and `preview_prune_slots_with_tag` on 2026-06-18 so save selectors can inspect retained/removed slot reports before mutating archives. Rust formatting passed for the touched dynamic scene files; `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short` passed with existing warnings only. Cargo execution for the new `dynamic_scene_session` behavior anchors remains deferred under the current implementation-first direction.
- Runtime session archive atomic save added `save_to_path_atomically` and moved archive file I/O into `session/io.rs` on 2026-06-18 so session persistence can write canonical JSON through a same-directory temporary file before committing the target path. Rust formatting passed for the touched dynamic scene files; focused `dynamic_scene_session` behavior anchors cover reloadability and temporary-file cleanup.
- Runtime session archive atomic save existing-target rollback on 2026-06-18 moves an existing archive file to a same-directory backup before committing the temporary archive, removes the backup on success, and attempts to restore the backup if the commit rename fails. Focused `dynamic_scene_session` behavior anchors cover replacing an existing archive and leaving no `.tmp` or `.bak` artifacts behind.
- Runtime session archive manifest path loading added `RuntimeSessionArchive::load_manifest_from_path` on 2026-06-18 so save selectors can load a validated persisted archive index without manually opening and projecting the full archive.
- Runtime session archive path inspection added `RuntimeSessionArchivePathStatus` and `RuntimeSessionArchive::inspect_path` on 2026-06-18 so startup and recent-session flows can distinguish missing archive files from present-but-invalid archives while still receiving a validated manifest for available files.
- Runtime session archive capture-to-path helpers added `load_or_empty_from_path`, `capture_world_slot_to_path_atomically`, and `capture_level_slot_to_path_atomically` on 2026-06-18 so runtime quicksave/autosave flows can create or update archive files through the same validation and rollback-protected atomic-save path without overwriting corrupted archives.
- Runtime session archive load-from-path helpers added `restore_slot_from_path_to_empty_world`, `restore_slot_from_path_into_level`, `apply_slot_from_path_to_world`, and `apply_slot_from_path_to_level` on 2026-06-18 so runtime load/instantiate flows can use archive paths directly while preserving validated archive load, missing-slot errors, level metadata restore, and entity remapping semantics.
- Runtime session archive path-management helpers added `rename_slot_at_path_atomically`, `update_slot_metadata_at_path_atomically`, `touch_slot_at_path_atomically`, and `remove_slot_at_path_atomically` on 2026-06-18 so runtime save selectors can mutate persisted slot metadata and ids through the validated archive load plus rollback-protected atomic-save path.
- Runtime session archive declaration split moved metadata, slot behavior, merge policy/report, path status, slot reports, slot-id validation, and archive statistics into focused folder-backed session children on 2026-06-18. `session.rs` dropped from the near-1000-line risk zone while preserving the same public `scene::dynamic_scene` and `scene` exports; `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-session-archive-tests-0618 --message-format short --color never` passed with existing warnings only.
- Runtime session archive path-retention helpers added `prune_slots_at_path_atomically` and `prune_slots_with_tag_at_path_atomically` on 2026-06-18 so autosave/quicksave cleanup can mutate persisted archive files through the validated archive load plus rollback-protected atomic-save path while returning the same retained/removed slot report as in-memory pruning.
- Runtime session archive path-copy/import helpers added `copy_slot_at_path_atomically`, `copy_slot_with_metadata_at_path_atomically`, and `merge_archive_at_path_atomically` on 2026-06-18 so save-as and import flows can mutate persisted archive files through validated load plus rollback-protected atomic-save while returning updated manifest or merge-report evidence.
- Runtime session archive behavior anchors moved from the near-threshold single `dynamic_scene_session.rs` file into the folder-backed `scene/tests/dynamic_scene_session/` subtree on 2026-06-18. The split keeps capture, load, path-management, persistence, and retention tests under separate owner files while preserving the `mod dynamic_scene_session;` public test entry.
- Runtime session archive path-query helpers added `statistics_from_path`, `diff_slot_from_path_with_world`, and `diff_slot_from_path_with_level` on 2026-06-18 so runtime diagnostics and overwrite previews can query persisted archive files through validated load without mutating worlds or archive files.
- Runtime session archive path-retention preview helpers added `preview_prune_slots_from_path` and `preview_prune_slots_with_tag_from_path` on 2026-06-19 so autosave/quicksave cleanup can dry-run persisted archive retention without writing the archive file. Rust formatting, conflict marker, trailing whitespace, and scoped diff checks passed for touched files. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-session-archive-tests-0619 --message-format short --color never` and the same command with `--no-default-features` are currently blocked by external render post-process drift (`execute_*` post-process resources missing `UVec2`, plus a GPU post-process call arity mismatch); no Cargo pass is claimed for this slice.
- Runtime session archive path-slot selection helpers added `latest_updated_slot_id_from_path`, `oldest_updated_slot_id_from_path`, `latest_updated_slot_id_with_tag_from_path`, and `oldest_updated_slot_id_with_tag_from_path` on 2026-06-19 so save/load selectors can choose persisted slots directly from a validated archive path without duplicating manifest projection code. Rust formatting, conflict marker, trailing whitespace, and scoped diff checks passed for touched files. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-session-archive-tests-0619 --message-format short --color never` is still blocked by external render post-process drift (`execute_*` post-process resources missing `UVec2`); no Cargo pass is claimed for this slice.
- Runtime session archive path manifest-filter helpers added `slots_with_tag_from_path` and `slots_matching_display_name_from_path` on 2026-06-19 so save/load selectors can filter persisted slot summaries directly from a validated archive path without duplicating manifest projection code. Rust formatting, conflict marker, trailing whitespace, and scoped diff checks passed for touched files. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-session-archive-tests-0619 --message-format short --color never` is still blocked by external render post-process drift (`execute_*` post-process resources missing `UVec2` plus a GPU post-process call arity mismatch); no Cargo pass is claimed for this slice.
- Runtime session archive path slot-summary helpers added `slot_ids_from_path`, `contains_slot_from_path`, and `slot_summary_from_path` on 2026-06-19 so save/load selectors can read persisted slot ids, existence, or one cloned summary from a validated archive path without duplicating manifest projection code. Rust formatting, conflict marker, trailing whitespace, and scoped diff checks passed for touched files. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-session-archive-tests-0619 --message-format short --color never` passed with existing warnings only; focused behavior tests remain deferred under the implementation-first direction.
- Runtime session archive path single-slot import helpers added `import_slot_from_archive`, `import_slot_from_archive_with_metadata`, `import_slot_from_archive_at_path_atomically`, and `import_slot_from_archive_with_metadata_at_path_atomically` on 2026-06-19 so save/load selectors can import one slot from an already loaded incoming archive into an in-memory archive or persisted archive path with optional metadata override. Rust formatting, conflict marker, trailing whitespace, and scoped Cargo checks passed for touched files; focused behavior tests remain deferred under the implementation-first direction.
- Runtime session archive single-slot standalone export helpers added `single_slot_archive`, `single_slot_archive_from_path`, and `save_single_slot_archive_from_path_atomically` on 2026-06-19 so save/share flows can extract one named slot into a validated standalone archive and atomically persist it to a distinct path without mutating the source archive. Rust formatting and scoped Cargo checks passed for touched files; focused behavior tests remain deferred under the implementation-first direction.
- Runtime session archive loaded single-slot save-to-path helper added `save_single_slot_archive_to_path_atomically` on 2026-06-19 so save/share flows can atomically persist one named slot from an already loaded archive as a standalone archive without mutating the source archive. Rust formatting and scoped Cargo checks passed for touched files; focused behavior tests remain deferred under the implementation-first direction.
- Runtime session archive source-path single-slot import helpers added `import_slot_from_archive_path_at_path_atomically` and `import_slot_from_archive_path_with_metadata_at_path_atomically` on 2026-06-19 so save/load flows can import one slot from a validated source archive path into a target archive path while preserving source-file immutability and target atomic-save semantics. Rust formatting and scoped Cargo checks passed for touched files; focused behavior tests remain deferred under the implementation-first direction.
- Runtime session archive source-path merge helper added `merge_archive_from_path_at_path_atomically` on 2026-06-19 so import flows can merge a validated source archive path into a target archive path through the existing duplicate-slot policy/report semantics while preserving source-file immutability and target atomic-save rollback. Rust formatting and scoped Cargo checks passed for touched files; focused behavior tests remain deferred under the implementation-first direction.
- Runtime session archive merge-preview helpers added `preview_merge_archive`, `preview_merge_archive_at_path`, and `preview_merge_archive_from_path_at_path` on 2026-06-19 so save/load selectors can inspect inserted/replaced/skipped merge reports before mutating loaded archives or persisted archive files. Rust formatting and scoped Cargo checks passed for touched files; focused behavior tests remain deferred under the implementation-first direction.
