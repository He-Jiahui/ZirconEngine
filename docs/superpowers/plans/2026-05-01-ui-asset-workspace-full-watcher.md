# UI Asset Workspace Full Watcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M5 UI Asset Editor workspace behavior by adding project-level `.ui.toml` watching, deterministic hot reload, external conflict actions, stale import diagnostics, and dependency refresh after external effects.

**Architecture:** Keep the pure `UiAssetEditorSession` responsible for document/source/preview state, and keep workspace facts in `zircon_editor::ui::host::asset_editor_sessions`. A real watcher feeds changed asset ids into the same deterministic refresh pipeline used by tests, conflict commands, save, promotion, undo, and redo. Dirty sessions never get overwritten; imported asset failures become stale import diagnostics while last-good previews remain usable.

**Tech Stack:** Rust 2021 workspace, `notify` workspace dependency, `crossbeam-channel`, TOML UI assets, `zircon_editor` host/session modules, targeted Cargo validation on Windows PowerShell.

---

## Execution Notes

- Work in the existing `main` checkout. Do not create a worktree or feature branch.
- Do not create commits unless the user explicitly requests them.
- Use `apply_patch` for manual source edits.
- Preserve unrelated dirty worktree changes.
- Follow `.codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md` milestone 5 and `docs/superpowers/specs/2026-05-01-ui-asset-workspace-full-watcher-design.md`.
- During implementation slices, add tests and docs as needed, but defer Cargo build/test commands to the milestone testing stages unless a compile blocker requires an earlier scoped check.
- Use a shared target directory when running Cargo, for example `$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"`, and avoid concurrent Cargo writers to that directory.

## Target File Structure

### New Files

- `zircon_editor/src/ui/host/asset_editor_sessions/workspace_state.rs`
  - Owns `UiAssetWorkspaceEntry`, `UiAssetExternalConflict`, `UiAssetStaleImportDiagnostic`, `UiAssetDiffSnapshot`, source hash helpers, conflict summaries, stale import formatting, and entry constructors.
- `zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs`
  - Owns deterministic hot reload, direct session conflict detection, imported dependency refresh, stale import update/clear, conflict actions, diff snapshots, and dependent refresh after external effects.
- `zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs`
  - Owns project-level watcher lifecycle, changed path normalization, event coalescing, and non-blocking drain into the refresh pipeline.
- `zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs`
  - Host-level tests for M5 workspace watcher, conflict, stale import, and external-effect refresh behavior.

### Modified Files

- `zircon_editor/Cargo.toml`
  - Add `notify.workspace = true` to editor dependencies.
- `zircon_editor/src/ui/host/editor_ui_host.rs`
  - Add watcher state field.
- `zircon_editor/src/ui/host/project_access.rs`
  - Start/restart watcher after project open/save establishes a project root.
- `zircon_editor/src/ui/host/asset_editor_sessions/mod.rs`
  - Register new modules and re-export workspace state types.
- `zircon_editor/src/ui/host/asset_editor_sessions/editing.rs`
  - Remove inline `UiAssetWorkspaceEntry`, use workspace state module, and route external-effect notifications through refresh.
- `zircon_editor/src/ui/host/asset_editor_sessions/open.rs`
  - Create workspace entries with disk baseline metadata.
- `zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs`
  - Restore workspace entries with disk baseline metadata; merge workspace conflict/stale info into reflection and presentation.
- `zircon_editor/src/ui/host/asset_editor_sessions/save.rs`
  - Update disk baseline after save and notify dependent refresh.
- `zircon_editor/src/ui/host/asset_editor_sessions/imports.rs`
  - Add import loading helpers that can return per-reference errors for stale diagnostics.
- `zircon_editor/src/ui/host/asset_editor_sessions/hydration.rs`
  - Preserve existing hydration behavior, but make it callable from refresh without clearing stale state incorrectly.
- `zircon_editor/src/ui/host/asset_editor_sessions/editing/navigation.rs`
  - After undo/redo external effects, notify dependent refresh using affected asset ids.
- `zircon_editor/src/ui/host/asset_editor_sessions/editing/node_ops.rs`
  - After direct promote writes, update dependent refresh using affected asset ids.
- `zircon_editor/src/ui/host/editor_manager_asset_editor.rs`
  - Add manager forwarding methods for conflict actions and watcher polling.
- `zircon_editor/src/ui/asset_editor/contract.rs`
  - Add M5 reflection fields.
- `zircon_editor/src/ui/asset_editor/presentation.rs`
  - Add M5 pane presentation fields.
- `zircon_editor/src/ui/asset_editor/session/presentation_state.rs`
  - Initialize new fields to defaults from pure session state; host lifecycle overlays workspace values.
- `zircon_editor/src/ui/slint_host/ui/tests.rs`
  - Struct literal uses `..Default`, so only update if compile requires explicit fields.
- `zircon_editor/src/tests/host/manager/mod.rs`
  - Add `mod ui_asset_workspace_watcher;`.
- `docs/editor-and-tooling/ui-asset-editor-host-session.md`
  - Document M5 workspace watcher, refresh, conflicts, stale imports, external effects, and validation.

## Milestone 1: Workspace State And Presentation Contracts

- Goal: Make workspace facts explicit without changing reload behavior yet.
- In-scope behaviors: disk baseline, source hash, external conflict record, stale import diagnostics, diff snapshot DTO, reflection/presentation fields, workspace entry constructors.
- Dependencies: existing `UiAssetEditorSession` open/save/source dirty behavior.
- Lightweight checks: Rust syntax/type check may be deferred until Milestone 1 testing stage.
- Exit evidence: M5 DTOs and workspace metadata are present and existing open/save tests still pass in testing stage.

### Implementation Slices

- [ ] Add `workspace_state.rs` with these public-in-module data structures:

```rust
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::ui::asset_editor::UiAssetEditorSession;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalConflict {
    pub(crate) asset_id: String,
    pub(crate) source_path: PathBuf,
    pub(crate) baseline_hash: u64,
    pub(crate) local_hash: u64,
    pub(crate) external_hash: u64,
    pub(crate) local_source: String,
    pub(crate) external_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetStaleImportDiagnostic {
    pub(crate) reference: String,
    pub(crate) message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiAssetDiffSnapshot {
    pub asset_id: String,
    pub baseline_hash: u64,
    pub local_hash: u64,
    pub external_hash: u64,
    pub local_source: String,
    pub external_source: String,
    pub summary: String,
}

pub(crate) struct UiAssetWorkspaceEntry {
    pub(crate) source_path: PathBuf,
    pub(crate) session: UiAssetEditorSession,
    pub(crate) disk_source: String,
    pub(crate) disk_source_hash: u64,
    pub(crate) conflict: Option<UiAssetExternalConflict>,
    pub(crate) stale_imports: BTreeMap<String, UiAssetStaleImportDiagnostic>,
    pub(crate) diff_snapshot: Option<UiAssetDiffSnapshot>,
}

pub(crate) fn ui_asset_source_hash(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}
```

- [ ] Implement `UiAssetWorkspaceEntry::new(source_path, source, session)` so `disk_source` and `disk_source_hash` are initialized from the opened source and conflict/stale/diff fields are empty.

```rust
impl UiAssetWorkspaceEntry {
    pub(crate) fn new(
        source_path: PathBuf,
        source: String,
        session: UiAssetEditorSession,
    ) -> Self {
        let disk_source_hash = ui_asset_source_hash(&source);
        Self {
            source_path,
            session,
            disk_source: source,
            disk_source_hash,
            conflict: None,
            stale_imports: BTreeMap::new(),
            diff_snapshot: None,
        }
    }

    pub(crate) fn update_disk_baseline(&mut self, source: String) {
        self.disk_source_hash = ui_asset_source_hash(&source);
        self.disk_source = source;
    }

    pub(crate) fn has_external_conflict(&self) -> bool {
        self.conflict.is_some()
    }

    pub(crate) fn external_conflict_summary(&self) -> String {
        self.conflict
            .as_ref()
            .map(|conflict| {
                format!(
                    "External change detected for {} (local {}, external {})",
                    conflict.asset_id, conflict.local_hash, conflict.external_hash
                )
            })
            .unwrap_or_default()
    }

    pub(crate) fn stale_import_items(&self) -> Vec<String> {
        self.stale_imports
            .values()
            .map(|diagnostic| format!("{}: {}", diagnostic.reference, diagnostic.message))
            .collect()
    }
}
```

- [ ] Move `UiAssetWorkspaceEntry` ownership out of `editing.rs` into `workspace_state.rs`, and update imports in `mod.rs`, `open.rs`, and `lifecycle.rs` to call `UiAssetWorkspaceEntry::new(source_path, source.clone(), session)`.
- [ ] Add M5 fields to `UiAssetEditorReflectionModel` in `contract.rs`:

```rust
pub has_external_conflict: bool,
pub external_conflict_summary: String,
pub stale_import_items: Vec<String>,
pub can_reload_from_disk: bool,
pub can_keep_local_and_save: bool,
pub can_open_diff_snapshot: bool,
```

- [ ] Initialize those fields in `UiAssetEditorReflectionModel::new` and add a builder method:

```rust
pub fn with_workspace_state(
    mut self,
    has_external_conflict: bool,
    external_conflict_summary: impl Into<String>,
    stale_import_items: Vec<String>,
) -> Self {
    self.has_external_conflict = has_external_conflict;
    self.external_conflict_summary = external_conflict_summary.into();
    self.stale_import_items = stale_import_items;
    self.can_reload_from_disk = has_external_conflict;
    self.can_keep_local_and_save = has_external_conflict;
    self.can_open_diff_snapshot = has_external_conflict;
    self
}
```

- [ ] Add the same M5 fields to `UiAssetEditorPanePresentation` in `presentation.rs`, initialize via `Default`, and set them from `reflection` in `presentation_state.rs`.
- [ ] In `asset_editor_sessions/lifecycle.rs`, overlay workspace state after obtaining `entry.session.reflection_model()` and `entry.session.pane_presentation()`. Do not put workspace conflict state into `UiAssetEditorSession`.

### Testing Stage

- [ ] Run focused existing tests that should remain behaviorally unchanged:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib editor_manager_opens_and_saves_ui_asset_editor_sessions --locked -- --nocapture
```

Expected: PASS. If it fails, fix only baseline constructor/field propagation issues before advancing.

- [ ] Run a scoped check if the focused test does not compile enough surface:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo check -p zircon_editor --lib --locked
```

Expected: PASS or only failures directly caused by fields planned in later milestones. Do not advance with unknown type errors in touched files.

## Milestone 2: Deterministic Refresh And Conflict Actions

- Goal: Implement hot reload semantics through explicit refresh APIs before wiring the watcher.
- In-scope behaviors: direct clean reload, dirty conflict, reload from disk, keep local and save, diff snapshot, imported dependency refresh, stale import preservation/recovery.
- Dependencies: Milestone 1 workspace state and existing import hydration.
- Lightweight checks: defer Cargo commands to testing stage unless Rust type failures block implementation.
- Exit evidence: deterministic tests can drive every M5 policy without filesystem watcher timing.

### Implementation Slices

- [ ] Add `refresh.rs` with host methods and implement each body in the same slice. The refresh method must normalize and deduplicate changed ids, process direct session assets, process import dependents, then sync affected instances after releasing `ui_asset_sessions`.

```rust
impl EditorUiHost {
    pub(super) fn refresh_ui_asset_workspace_for_changes(
        &self,
        changed_asset_ids: impl IntoIterator<Item = String>,
    ) -> Result<(), EditorError> {
        let changed_asset_ids = normalize_ui_asset_change_set(changed_asset_ids);
        if changed_asset_ids.is_empty() {
            return Ok(());
        }
        let sync_instances = self.apply_ui_asset_workspace_changes(&changed_asset_ids)?;
        for instance_id in sync_instances {
            self.sync_ui_asset_editor_instance(&instance_id)?;
        }
        Ok(())
    }

    pub fn reload_ui_asset_editor_from_disk(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        let (source_path, route) = {
            let sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            (entry.source_path.clone(), entry.session.route().clone())
        };
        let source = std::fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let session = rebuild_ui_asset_session_from_source(route, source.clone())?;
        {
            let mut sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session = session;
            entry.update_disk_baseline(source);
            entry.conflict = None;
            entry.diff_snapshot = None;
            entry.stale_imports.clear();
        }
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(true)
    }

    pub fn keep_ui_asset_editor_local_and_save(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.save_ui_asset_editor(instance_id)
    }

    pub fn open_ui_asset_editor_diff_snapshot(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<UiAssetDiffSnapshot>, EditorError> {
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        if entry.diff_snapshot.is_none() {
            entry.diff_snapshot = entry.conflict.as_ref().map(UiAssetDiffSnapshot::from);
        }
        Ok(entry.diff_snapshot.clone())
    }
}
```

- [ ] Add the direct-session conflict helper:

```rust
fn build_external_conflict(
    asset_id: String,
    source_path: PathBuf,
    baseline_hash: u64,
    local_source: String,
    external_source: String,
) -> UiAssetExternalConflict {
    UiAssetExternalConflict {
        asset_id,
        source_path,
        baseline_hash,
        local_hash: ui_asset_source_hash(&local_source),
        external_hash: ui_asset_source_hash(&external_source),
        local_source,
        external_source,
    }
}
```

- [ ] Add a private helper to rebuild an entry session from source:

```rust
fn rebuild_ui_asset_session_from_source(
    route: UiAssetEditorRoute,
    source: String,
) -> Result<UiAssetEditorSession, EditorError> {
    let preview_size = preview_size_for_preset(route.preview_preset);
    UiAssetEditorSession::from_source(route, source, preview_size)
        .map_err(|error| EditorError::UiAsset(error.to_string()))
}
```

- [ ] Add import dependency detection. Start minimal and exact: a session depends on a changed asset if `entry.session.import_references()` contains the normalized asset id in widgets or styles. Nested imports are handled by reusing `collect_ui_asset_import_document` during hydration.
- [ ] Make import refresh collect errors per reference. Add a helper in `imports.rs`:

```rust
pub(super) fn try_collect_ui_asset_import_document(
    &self,
    reference: &str,
    expected_kind: UiAssetKind,
    widget_docs: &mut BTreeMap<String, UiAssetDocument>,
    style_docs: &mut BTreeMap<String, UiAssetDocument>,
    visited: &mut BTreeSet<String>,
) -> Result<(), String> {
    self.collect_ui_asset_import_document(reference, expected_kind, widget_docs, style_docs, visited)
        .map_err(|error| error.to_string())
}
```

- [ ] When import refresh fails for a dependent session, insert `UiAssetStaleImportDiagnostic { reference, message }`, keep the existing session compiler imports/preview unchanged, and sync presentation.
- [ ] When import refresh succeeds, call `entry.session.replace_imports(widget_docs, style_docs)`, clear stale diagnostics for those refs, and sync presentation.
- [ ] Update `save_ui_asset_editor` to call `entry.update_disk_baseline(saved.clone())` after a successful `fs::write`, clear any conflict/diff for that entry, and refresh dependents for the saved `asset_id`.
- [ ] Add manager forwards in `editor_manager_asset_editor.rs`:

```rust
pub fn reload_ui_asset_editor_from_disk(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
    self.host.reload_ui_asset_editor_from_disk(instance_id)
}

pub fn keep_ui_asset_editor_local_and_save(&self, instance_id: &ViewInstanceId) -> Result<String, EditorError> {
    self.host.keep_ui_asset_editor_local_and_save(instance_id)
}

pub fn open_ui_asset_editor_diff_snapshot(&self, instance_id: &ViewInstanceId) -> Result<Option<UiAssetDiffSnapshot>, EditorError> {
    self.host.open_ui_asset_editor_diff_snapshot(instance_id)
}

pub fn refresh_ui_asset_workspace_for_changes(&self, changed_asset_ids: impl IntoIterator<Item = String>) -> Result<(), EditorError> {
    self.host.refresh_ui_asset_workspace_for_changes(changed_asset_ids)
}
```

### Test Code To Add

- [ ] Create `ui_asset_workspace_watcher.rs` and add tests for deterministic refresh:

```rust
#[test]
fn editor_manager_refreshes_clean_ui_asset_session_from_external_file_change() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_hot_reload_clean");
    let ui_asset_path = unique_temp_dir("zircon_editor_asset_hot_reload_clean_file").join("test.ui.toml");
    std::fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME).unwrap();
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();

    let changed = SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "External");
    write_ui_asset(&ui_asset_path, &changed);
    manager
        .refresh_ui_asset_workspace_for_changes(vec![ui_asset_path.to_string_lossy().to_string()])
        .expect("refresh external change");

    let pane = manager.ui_asset_editor_pane_presentation(&instance_id).unwrap();
    assert!(pane.source_text.contains("External"));
    assert!(!pane.source_dirty);
    assert!(!pane.has_external_conflict);
}
```

- [ ] Add dirty conflict test:

```rust
#[test]
fn editor_manager_marks_dirty_ui_asset_session_conflicted_without_overwriting_local_source() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_hot_reload_conflict");
    let ui_asset_path = unique_temp_dir("zircon_editor_asset_hot_reload_conflict_file").join("test.ui.toml");
    std::fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME).unwrap();
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    manager.update_ui_asset_editor_source(&instance_id, SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "Local")).unwrap();

    write_ui_asset(&ui_asset_path, &SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "External"));
    manager.refresh_ui_asset_workspace_for_changes(vec![ui_asset_path.to_string_lossy().to_string()]).unwrap();

    let pane = manager.ui_asset_editor_pane_presentation(&instance_id).unwrap();
    assert!(pane.source_text.contains("Local"));
    assert!(!pane.source_text.contains("External"));
    assert!(pane.has_external_conflict);
    assert!(pane.can_reload_from_disk);
    assert!(pane.can_keep_local_and_save);
}
```

- [ ] Add reload, keep-local, and diff snapshot tests using the same setup. Assert reload accepts external text, keep-local writes local text to disk, and diff snapshot contains both source bodies and distinct hashes.
- [ ] Add imported style stale/recovery test by creating a project root with `assets/ui/layouts/editor.ui.toml` importing `res://ui/theme/shared_theme.ui.toml`, opening by id, corrupting the style file, refreshing with that asset id, asserting stale import item and last preview availability, then restoring valid style and asserting stale list clears.

### Testing Stage

- [ ] Run deterministic M5 tests:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked -- --nocapture
```

Expected: PASS. If stale import tests fail because existing import hydration treats all errors as fatal, fix `refresh.rs` to catch errors before calling the existing fatal hydration path for watched import changes.

- [ ] Run existing related host session tests:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib ui_asset_session_preview --locked -- --nocapture
```

Expected: PASS.

## Milestone 3: Real Project Watcher Lifecycle

- Goal: Add real file watching that feeds the already-tested refresh pipeline.
- In-scope behaviors: watcher start/restart on project open/save, non-blocking event collection, `.ui.toml` filtering, path-to-asset normalization, explicit drain API.
- Dependencies: Milestone 2 refresh pipeline.
- Lightweight checks: defer Cargo commands to testing stage.
- Exit evidence: watcher compiles, can be started for a project, and deterministic drain method refreshes sessions from queued watcher events.

### Implementation Slices

- [ ] Add `notify.workspace = true` to `zircon_editor/Cargo.toml` dependencies.
- [ ] Add watcher state to `EditorUiHost`:

```rust
pub(super) ui_asset_workspace_watcher: Mutex<Option<UiAssetWorkspaceWatcher>>,
```

Initialize it to `None` in `EditorUiHost::new`.

- [ ] Implement `watcher.rs` with a non-blocking state object:

```rust
use std::path::{Path, PathBuf};

use crossbeam_channel::{unbounded, Receiver};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub(crate) struct UiAssetWorkspaceWatcher {
    project_root: PathBuf,
    assets_root: PathBuf,
    receiver: Receiver<PathBuf>,
    _watcher: RecommendedWatcher,
}
```

- [ ] In `UiAssetWorkspaceWatcher::start(project_root)`, watch `project_root/assets` recursively. The notify callback should send changed paths into the channel and ignore send failures.
- [ ] Add `drain_changed_asset_ids(&self) -> Vec<String>` that drains all queued paths, filters extension `.toml` plus file name ending `.ui.toml`, converts `project_root/assets/foo/bar.ui.toml` to `res://foo/bar.ui.toml`, sorts, and dedups.
- [ ] Add host methods:

```rust
pub(super) fn restart_ui_asset_workspace_watcher(&self) -> Result<(), EditorError>;
pub fn poll_ui_asset_workspace_watcher(&self) -> Result<Vec<String>, EditorError>;
```

`poll_ui_asset_workspace_watcher` should drain ids, call `refresh_ui_asset_workspace_for_changes(ids.clone())`, and return the ids for tests/diagnostics.

- [ ] Call `restart_ui_asset_workspace_watcher` after `open_project` and `save_project` successfully establish the runtime asset manager project root.
- [ ] Do not make watcher startup fatal for project open. If `notify` fails, store no watcher and return `Ok(())` from restart after recording a host diagnostic if an existing diagnostics mechanism is readily available; otherwise, leave manual refresh APIs functional and document the limitation.
- [ ] Add manager forward:

```rust
pub fn poll_ui_asset_workspace_watcher(&self) -> Result<Vec<String>, EditorError> {
    self.host.poll_ui_asset_workspace_watcher()
}
```

### Test Code To Add

- [ ] Add a focused watcher normalization/drain test. Prefer not to rely on OS event timing. If direct injection is needed, add a `#[cfg(test)]` helper on `UiAssetWorkspaceWatcher` to construct from queued paths without starting notify.
- [ ] Add an integration-style test that opens a project, opens a UI asset by id, writes the file, and calls `poll_ui_asset_workspace_watcher`. If this proves flaky on Windows, mark the OS-event part as a narrow best-effort test and keep deterministic refresh tests as acceptance authority.

### Testing Stage

- [ ] Run watcher tests:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked -- --nocapture
```

Expected: PASS. If OS event timing is flaky, keep only deterministic queued-path watcher tests in the default suite and document manual OS watcher verification separately.

## Milestone 4: External Effects Refresh And Direct Promote Convergence

- Goal: Ensure cross-asset writes/removes from promote, undo, redo, and replay refresh dependent sessions through the M5 pipeline.
- In-scope behaviors: affected asset id extraction, refresh after `UpsertAssetSource`, `RestoreAssetSource`, and `RemoveAssetSource`, direct promote paths notifying dependents, stale diagnostics after removal, recovery after redo/upsert.
- Dependencies: Milestone 2 refresh pipeline.
- Lightweight checks: defer Cargo commands to testing stage.
- Exit evidence: existing promote tests still pass and new dependent refresh tests prove external effects notify open dependents.

### Implementation Slices

- [ ] Add helper in `editing.rs` or `refresh.rs`:

```rust
fn ui_asset_external_effect_asset_id(effect: &UiAssetEditorExternalEffect) -> &str {
    match effect {
        UiAssetEditorExternalEffect::UpsertAssetSource { asset_id, .. }
        | UiAssetEditorExternalEffect::RestoreAssetSource { asset_id, .. }
        | UiAssetEditorExternalEffect::RemoveAssetSource { asset_id } => asset_id,
    }
}
```

- [ ] In undo/redo host navigation paths, collect affected asset ids after applying external effects, then call `refresh_ui_asset_workspace_for_changes` once with the collected ids.
- [ ] In direct widget/theme promote host paths, call `refresh_ui_asset_workspace_for_changes(vec![target_asset_id])` after successful file write/import. Avoid duplicating import refresh logic.
- [ ] In `apply_ui_asset_editor_external_effect`, after write/remove/import, return or expose the normalized affected asset id so callers can refresh dependents without recomputing path-specific behavior.

### Test Code To Add

- [ ] Extend host manager promotion tests or add tests in `ui_asset_workspace_watcher.rs`:

Add these imports at the top of `ui_asset_workspace_watcher.rs` if the file does not already have them:

```rust
use zircon_runtime::scene::DefaultLevelManager;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::project::EditorProjectDocument;

use super::support::*;
```

```rust
#[test]
fn editor_manager_refreshes_dependents_after_theme_external_effect_undo_and_redo() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_effect_refresh");
    let project_root = unique_temp_dir("zircon_editor_asset_effect_refresh_project");
    let layout_path = project_root.join("assets").join("ui").join("layouts").join("editor.ui.toml");
    let theme_path = project_root.join("assets").join("ui").join("themes").join("editor_theme.ui.toml");
    std::fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(theme_path.parent().unwrap()).unwrap();

    let source = DETACH_THEME_UI_LAYOUT_ASSET.replace(
        "res://ui/theme/shared_theme.ui.toml",
        "res://ui/themes/editor_theme.ui.toml",
    );
    write_ui_asset(&layout_path, &source);
    write_ui_asset(&theme_path, IMPORTED_THEME_COLLISION_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME).unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();
    write_ui_asset(&layout_path, &source);
    write_ui_asset(&theme_path, IMPORTED_THEME_COLLISION_ASSET);
    manager.open_project(&project_root).expect("open project");
    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("open dependent layout");

    std::fs::remove_file(&theme_path).expect("remove imported theme");
    manager
        .refresh_ui_asset_workspace_for_changes(vec!["res://ui/themes/editor_theme.ui.toml".to_string()])
        .expect("refresh removed theme");
    let stale = manager.ui_asset_editor_pane_presentation(&instance_id).unwrap();
    assert!(stale.preview_available);
    assert!(stale
        .stale_import_items
        .iter()
        .any(|item| item.contains("res://ui/themes/editor_theme.ui.toml")));

    write_ui_asset(&theme_path, IMPORTED_THEME_COLLISION_ASSET);
    manager
        .refresh_ui_asset_workspace_for_changes(vec!["res://ui/themes/editor_theme.ui.toml".to_string()])
        .expect("refresh restored theme");
    let recovered = manager.ui_asset_editor_pane_presentation(&instance_id).unwrap();
    assert!(recovered.preview_available);
    assert!(recovered.stale_import_items.is_empty());
}
```

Use existing fixtures from `ui_asset_reference_and_promotion.rs` where possible rather than inventing a second promotion format.

### Testing Stage

- [ ] Run promotion and replay tests:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib ui_asset_reference_and_promotion --locked -- --nocapture
cargo test -p zircon_editor --lib ui_asset_replay --locked -- --nocapture
cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked -- --nocapture
```

Expected: PASS. If an upper-layer promotion test fails, check refresh and import hydration before changing promotion document transforms.

## Milestone 5: Documentation And Final Validation

- Goal: Document the completed M5 behavior and run focused acceptance before handing off.
- In-scope behaviors: docs header maintenance, behavior explanation, test evidence, targeted compile/test validation, remaining risk reporting.
- Dependencies: Milestones 1-4.
- Exit evidence: docs updated and validation commands recorded.

### Implementation Slices

- [ ] Update `docs/editor-and-tooling/ui-asset-editor-host-session.md` frontmatter:
  - Add `zircon_editor/src/ui/host/asset_editor_sessions/workspace_state.rs`.
  - Add `zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs`.
  - Add `zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs`.
  - Add `zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs`.
  - Add `docs/superpowers/specs/2026-05-01-ui-asset-workspace-full-watcher-design.md` and this plan to `plan_sources`.
- [ ] Add a section named `UI Asset Workspace Hot Reload And Conflicts` describing:
  - workspace entry disk baseline;
  - project watcher lifecycle;
  - deterministic refresh API;
  - clean direct reload;
  - dirty conflict preservation;
  - reload/keep-local/diff conflict actions;
  - stale import diagnostics;
  - external-effect dependent refresh.
- [ ] Keep docs factual and tied to implemented APIs. M7 schema migration, M12 invalidation graph/cache, and the M16 artifact envelope/package manifest are accepted foundations, but this M5 watcher lane must not claim their remaining future work such as editor read-only migration UX, cross-process cache storage, resolver/runtime-loader/editor resource UX, or broad workspace green.

### Testing Stage

- [ ] Run focused UI asset workspace tests:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked -- --nocapture
```

- [ ] Run neighboring editor host suites:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo test -p zircon_editor --lib ui_asset_session_preview --locked -- --nocapture
cargo test -p zircon_editor --lib ui_asset_reference_and_promotion --locked -- --nocapture
cargo test -p zircon_editor --lib ui_asset_replay --locked -- --nocapture
```

- [ ] Run scoped compile check:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-shared"
cargo check -p zircon_editor --lib --locked
```

- [ ] Run diff hygiene:

```powershell
git diff --check
```

- [ ] If all focused commands pass, record the commands and outputs in the final response. If broad validation is requested after this slice, use `.opencode/skills/zircon-dev/scripts/validate-matrix.ps1` with an explicit target dir.

## Acceptance Criteria

- Clean open UI asset sessions hot reload from changed disk source without becoming dirty.
- Dirty open UI asset sessions enter external conflict and preserve local source text.
- Reload from disk, keep local and save, and diff snapshot APIs are available through `EditorManager`.
- Imported widget/style changes refresh dependent sessions when valid.
- Invalid or deleted imported widget/style assets produce stale import diagnostics and preserve last-good preview/session state.
- Fixing a stale import clears the diagnostic and refreshes dependent sessions.
- Promote/extract undo/redo external effects refresh dependent sessions or mark stale imports through the same refresh pipeline.
- Project watcher is real, non-blocking, project-scoped, and feeds the deterministic refresh path.
- `docs/editor-and-tooling/ui-asset-editor-host-session.md` documents the new M5 behavior and does not overclaim later milestones.
