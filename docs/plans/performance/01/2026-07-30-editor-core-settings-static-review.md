# Editor core settings current-source static review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Accounting: keep `zircon_editor/src/core/settings` in `pending.md`; do not add it to `review.md` before current-source managed Cargo, scale counters and F0/F4 product evidence are GREEN.
- Code disposition: no Rust source was changed. The complete settings directory is external untracked work and was preserved.

## Exact scope

| module | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/settings` | 9/9 | 1,625 | 12 | `0d1911d42dd9bd86cab24f7220cfa94d35e06b3f965cb5b671555fd90e9439fc` |

The exact files are `defaults.rs`, `definition.rs`, `io.rs`, `keymap_overrides.rs`, `mod.rs`, `page.rs`, `registry.rs`, `scope.rs` and `tests.rs`. All nine were reread at their current contents while standardizing this record to the repository-wide path-plus-raw-content fingerprint. Consumer tracing covered `ui/host/editor_manager.rs`, retained-host startup, viewport controller construction/accessors, project document load, workbench state/snapshot projection and the settings/keymap tests. The behavior findings below did not drift.

## Confirmed performance boundary

1. `SceneViewportController::set_project_snap_step` executes on the editor command path. It parses a static key, clones the complete `SettingsRegistry`, changes one value, then calls `SettingsStore::save_from`; that method clones the complete persistent BTreeMap, serializes the complete settings document to a `String`, creates a temporary file, writes it, calls `sync_all`, renames and syncs the parent directory. A one-float edit therefore blocks the caller and scales with the whole registry/file and filesystem latency.
2. Settings ownership is split. `EditorManager::new` independently calls `settings_registry_at_startup`; retained-host `run_editor_with_config` obtains design tokens through another call to that function; the viewport constructs its own default registry and reloads user/project files when a project opens. Product tracing must prove the exact launch count, but the source already proves there is no single authority or shared load generation.
3. `SettingsRegistry` keeps `changes: Vec<SettingChange>` without a capacity or retention policy. Only tests call `drain_changes`; no production consumer was found. Successful command-palette MRU changes append a cloned change indefinitely during a long editor session even though the MRU value itself is bounded to 32 entries.
4. Generic `set` clones the `SettingSchema`, inserts/clones the key and emits a new revision/change without comparing the existing scoped value. `replace_persistent_layer` deep-clones the previous complete layer, builds a changed-key set, then clones each change both into the internal journal and the returned result.
5. `chrome_settings()` resolves three snap values for workbench projection. Each lookup reparses a static `&str` into a newly allocated `SettingsKey(String)`, looks up the definition and probes session/project/user BTreeMaps. Current consumers include the complete editor-state snapshot builder and a public viewport-state accessor, so stable snapshots repeat work whose inputs only change with a settings generation.
6. `decode_current_document` parses the complete source into `serde_json::Value` only to detect `$zircon`, then calls the generic versioned reader on the same bytes. This remains assigned to `PERF-MVP-570` / Editor11 rather than creating another serialization authority.

## Plan and acceptance

- `PERF-MVP-590` / Editor17 + Editor05 + Runtime11: one in-memory settings authority; typed single-key submissions; bounded per-scope/key latest-generation coalescing; shared asynchronous atomic persistence; explicit flush/shutdown ticket and read-your-write semantics. UI caller filesystem wall must be zero and a single-key update must not deep-clone the full registry/layer/document.
- `PERF-MVP-591` / Editor17 + Editor05: one shared immutable generation snapshot; compile built-in keys into typed slot IDs at registration; bounded cursor/delta changes; no-op set produces no revision or event; all startup, retained host and viewport consumers share the same load/decode generation.
- Matrix: definitions/keys `1/1k/100k`, values `0/1KiB/1MiB`, changes/palette events `1/1k/1M`, snapshots `60/120 Hz`, filesystem latency `0/10ms/2s`, consumers/writers `1/16`. Record authority count, file reads/parse passes, full clone bytes, key/String alloc, BTree probes, journal/queue entries+bytes+age, writes/fsync, caller wall, RSS and p50/p95.

## Reference check

- Godot `dev/godot/editor/settings/editor_settings_dialog.cpp` restarts a one-shot timer whenever settings change, sets its wait time to 1.5 seconds, and saves only on timeout. This supports burst coalescing; Zircon additionally needs an explicit bounded worker/flush authority so the timer callback itself does not block the editor thread.

## Static gates executed

- Read 9/9 exact settings files and the listed consumer paths.
- `rustfmt --check --edition 2021` passed all nine current files.
- No managed Cargo, allocation/filesystem fault benchmark, WPR product trace or F0/F4 launch/viewport trace ran. RenderDoc is not applicable to this non-rendering settings slice. The module remains pending.
