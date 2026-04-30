# UI Asset Workspace Full Watcher Design

## Context

The UI asset milestone audit found that the document model, editor session, save path, import hydration, promotion, and replay effects are already present. The missing M5 behaviors are workspace-grade change handling: file watching, hot reload, external conflicts, stale import diagnostics, explicit conflict actions, and dependency refresh after cross-asset operations.

The user selected the full watcher direction. This design keeps the watcher real while making the behavior testable through a deterministic refresh API. The watcher should never own parsing, conflict policy, or session mutation rules; it only reports changed `.ui.toml` paths into the host workspace refresh layer.

## Approved Direction

Implement M5 as a project-level UI asset workspace watcher plus a deterministic refresh pipeline.

Runtime behavior:

- watch the active project `assets/` tree for `.ui.toml` file changes;
- normalize changed paths to `res://...` asset ids;
- route changes through the same refresh function used by tests and manual commands;
- refresh clean sessions automatically;
- preserve dirty sessions by entering external conflict state;
- refresh imported widget/style dependencies or mark stale imports without destroying the last valid preview;
- update dependent sessions after promote/extract external effects.

## Scope

In scope:

- `zircon_editor` UI asset host session state and orchestration.
- `UiAssetWorkspaceEntry` disk baseline, conflict, stale import, and diff snapshot state.
- Project-scoped watcher lifecycle for `assets/**/*.ui.toml`.
- Deterministic host APIs for polling watcher changes and applying explicit conflict actions.
- Editor reflection/presentation additions for conflict and stale import visibility.
- Host manager tests covering reload, conflict, diff snapshot, keep-local save, stale import, import recovery, and external-effect dependent refresh.
- Documentation update in `docs/editor-and-tooling/ui-asset-editor-host-session.md`.

Out of scope:

- Schema migration and read-only version policy from M7.
- Incremental compiler/invalidation graph from M12.
- UI compiled artifact manifests/package validation from M16.
- Full visual diff UI in Slint. This slice exposes diff snapshot data through Rust APIs/presentation; a separate UI-polish milestone can consume it.
- Watching non-UI asset dependencies such as media, fonts, localization catalogs, or packaged artifacts.

## Architecture

### Workspace Entry State

`UiAssetWorkspaceEntry` should become the host-owned record for an open UI asset file. It keeps the session plus disk/workspace metadata that the pure `UiAssetEditorSession` should not own.

Required fields:

- `source_path`: canonical source file path.
- `session`: existing `UiAssetEditorSession`.
- `disk_source`: latest source text accepted from disk by open, save, reload, or clean hot reload.
- `disk_source_hash`: stable hash of `disk_source` for cheap change checks.
- `conflict`: optional external conflict record with asset id, path, disk source, local source, and hashes.
- `stale_imports`: normalized import diagnostics for imports that failed to reload.
- `diff_snapshot`: last generated diff snapshot for conflict review.

The session remains the document/source/preview authority. The entry owns only workspace facts: what disk last said, whether local source diverged, and what imported assets failed to refresh.

### Watcher Layer

Add a small watcher module under `zircon_editor/src/ui/host/asset_editor_sessions/` so all UI asset workspace logic stays together.

The watcher owns:

- project root and watched `assets/` path;
- a channel of raw filesystem events;
- normalization from changed path to `res://...` asset id;
- event coalescing for repeated writes;
- a public host method that drains pending changes and calls the refresh pipeline.

The watcher must not hold the `ui_asset_sessions` lock while blocking on filesystem events. It should only enqueue paths, then the host drains and applies them from normal editor update points or explicit tests.

### Refresh Pipeline

The central method should accept a list of changed normalized asset ids and process all open UI asset sessions.

For a direct session asset change:

- If the session is not dirty and there is no conflict, reload from disk, replace session source, update disk baseline, clear stale diagnostics for the asset, rehydrate imports, and sync the view instance.
- If the session has local modifications, store `ExternalConflict` and keep the current source buffer unchanged.
- If the disk source is unchanged from the stored baseline, ignore the event.

For an imported asset change:

- Re-read the changed import and dependent nested imports.
- If parsing/kind validation succeeds, replace imports in every session that depends on the changed asset, recompile preview, and clear the stale diagnostic for that import.
- If parsing/kind validation fails, preserve the session document/preview and record a stale import diagnostic that names the reference and parse/kind error.

For deleted imports:

- Preserve the last valid compiled state.
- Record a stale import diagnostic for the missing reference.

### Conflict Actions

Expose explicit host actions through `EditorManager` and `EditorUiHost`:

- `reload_ui_asset_editor_from_disk(instance_id)`: discard local source, load disk source, rebuild session, update baseline, clear conflict and diff snapshot.
- `keep_ui_asset_editor_local_and_save(instance_id)`: write local canonical source over disk, update baseline, clear conflict, import asset, refresh dependents.
- `open_ui_asset_editor_diff_snapshot(instance_id)`: return a structured snapshot containing asset id, local source, external source, baseline hash, local hash, external hash, and simple line ranges or unified text.

These names can be shortened if existing naming conventions require it, but the behavior should remain explicit.

### Presentation And Reflection

Add minimal M5 state to editor-facing models:

- `has_external_conflict`.
- `external_conflict_summary`.
- `stale_import_items`.
- `can_reload_from_disk`.
- `can_keep_local_and_save`.
- `can_open_diff_snapshot`.

This keeps Slint changes optional. Existing tests can assert host reflection/presentation without requiring a polished conflict pane.

### External Effects And Dependency Refresh

Promotion/extract currently writes sources and imports assets. After any external effect writes, restores, or removes a UI asset source, the host must notify the refresh pipeline with the affected asset id. Dependent sessions should either refresh their imports or record stale import diagnostics if the new source is invalid or missing.

Undo/redo external effects should use the same notification path. Direct promote host paths should stop bypassing the shared effect handling behavior where practical; if a direct write remains, it must still call the refresh pipeline afterward.

## Error Handling

- Watcher startup failures should not prevent opening the editor. They should be reported through host diagnostics and leave deterministic manual refresh APIs usable.
- File events for paths outside project `assets/` should be ignored.
- Unknown path-to-asset mappings should be ignored rather than panicking.
- Parse/kind errors for direct dirty sessions should create conflict only if disk changed; they must not overwrite local source.
- Parse/kind errors for imported assets should become stale import diagnostics, not direct session diagnostics that block editing unrelated local source.
- Save failures should leave the conflict state intact.
- `Keep Local And Save` should not clear conflict until file write and asset import succeed.

## Testing And Acceptance

Focused tests should be added under `zircon_editor/src/tests/host/manager/` because M5 is host workspace orchestration.

Required tests:

- clean open session receives external file change and refreshes source, preview, inspector, title/dirty state;
- dirty session receives external file change and enters conflict without overwriting source buffer;
- `Reload From Disk` resolves conflict by accepting external source;
- `Keep Local And Save` resolves conflict by writing local source and updating disk baseline;
- `Open Diff Snapshot` returns local/external/baseline hashes and source bodies;
- imported widget/style change refreshes dependent session preview/reflection;
- invalid imported widget/style change records stale import diagnostic while preserving last valid preview;
- fixing the imported file clears stale diagnostic and refreshes dependent session;
- promote/undo/redo external effects refresh dependent sessions or mark them stale after asset removal.

Validation stage should run targeted editor tests first, then `cargo check -p zircon_editor --lib --locked` using the repository target-dir policy. Broader workspace validation belongs to final milestone acceptance if the targeted stage passes.

## Documentation

Update `docs/editor-and-tooling/ui-asset-editor-host-session.md` because it already owns UI Asset Editor host session behavior. The update should document:

- workspace entry disk baseline and conflict ownership;
- watcher lifecycle and deterministic refresh API;
- conflict actions;
- stale import semantics;
- external effect dependency refresh;
- tests and validation commands used for this slice.

## Review Gate

Implementation should follow a layered plan: state/data definitions first, refresh pipeline second, watcher lifecycle third, manager/presentation APIs fourth, tests/docs/validation last. Do not start M7/M12/M16 work while implementing this M5 slice.
