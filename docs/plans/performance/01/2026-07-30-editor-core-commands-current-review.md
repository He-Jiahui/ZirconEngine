# Editor core commands current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Primary owner: Editor08. EditorUI08 co-owns reflection/menu materialization and visible palette rows; Editor12 co-owns extension registration publication.
- Accounting: keep `zircon_editor/src/core/commands/**` in `pending.md`. Do not add it to `review.md` before current-source managed Cargo, deterministic scale counters and F4 keyboard/menu/palette product evidence are GREEN.
- Code disposition: no Rust source was changed. Seven tracked modified files and the untracked `keymap/tests.rs` subtree were reviewed and preserved at their current contents.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/commands/**` | 17/17 | 3,888 | 21 | `fdf2d0cf7c077d2fb23aadcf3d99cb10b8d85720615c7ce67ab79556e3e48571` |

The fingerprint streams each workspace-relative path, a zero byte, the file's raw bytes and a zero byte in sorted path order into SHA256. All 17 files were read in full. Production reachability was followed through `EditorContext`, command projection/dispatch, extension registration, full reflection/workbench model construction, retained-host recompute and command-palette edit/window actions.

## Per-file review

| file | current-source performance result |
|---|---|
| `asset_write_target.rs` | Two small owned argument names, created only with descriptor metadata. No per-query work. |
| `contribution.rs` | Deterministic BTree membership plus descriptor insertion; repeated id clones and validation are structural-registration costs owned by PERF-MVP-079/538. |
| `defaults.rs` | Built-in descriptors and chords are constructed once for a registry owner. String formatting/parsing here is startup work, not an input hot path. |
| `descriptor.rs` | `is_enabled` directly evaluates stored predicates/capabilities without materializing `effective_when`; builder sort/dedup remains construction-only. PERF-MVP-062's source fix is current. |
| `document_kind.rs` | Streaming identifier validation and a small enum-like string owner. No repeated collection or lock. |
| `eval_snapshot_handle.rs` | Every `snapshot()` clones the full `CommandEvalCtx`, including `BTreeSet<String>` capabilities, under an `RwLock`. Palette query edits and UI/menu dispatch call it directly. |
| `key_chord.rs` | Keyboard dispatch now derives a borrowed normalized key and Copy FNV signature; normal input lookup creates no owned normalized chord. Display writes directly to the formatter. |
| `keymap.rs` | A generation-built `HashMap<signature, candidate indices>` makes input lookup near O(1)+collision candidates. Base/override map cloning and conflict materialization occur only when settings change or diagnostics are requested. |
| `keymap/tests.rs` | Eight tests cover settings layers, conflict semantics, 10K bindings, fallback keys and a one-million-event storm; none ran in this pass. |
| `menu_model.rs` | Owned menu DTOs are appropriate publication values, but their complete reconstruction cost depends on `menu.rs`. |
| `menu.rs` | `menu_bar_model` invokes `menu_model` for seven labels; each invocation scans all commands and enabled rows allocate label/id/shortcut DTOs. Full reflection/workbench recompute holds the registry mutex while doing this. |
| `mod.rs` | Mounts and re-exports only. |
| `palette.rs` | Catalog entries/search documents/enablement slots are immutable per registry generation and windows retain only handles. Each query still scans every entry; a substring pass can be followed by a subsequence pass, empty queries evaluate all entries, and UI conversion clones the visible window. |
| `play_mode_predicate.rs` | Constant-time predicate matching only. |
| `registry_handle.rs` | One `Mutex<EditorCommandRegistry>` serializes reads, mutation and projections. Short dispatch lookups release it promptly, but palette scans and full workbench model builds hold it across O(N) or O(7N) work. |
| `registry.rs` | Palette catalog caching and direct enablement slots are current. Registration advances generation per command and headless route/name uniqueness scans existing descriptors; extension staging also clones the complete registry, so batch/finalize publication remains open. |
| `when.rs` | Recursive predicates evaluate borrowed data; capability lookup is BTreeSet membership. `WhenClause::all` sort/dedup is construction-time. The snapshot's owned capability set is the remaining query clone cost. |

## Current bottlenecks and corrected tasks

### PERF-MVP-074: source implementation present, dynamic gate missing

The 2026-07-22 claim that every keyboard event creates an owned normalized key and linearly scans all bindings is no longer current. `EditorKeyboardChordInput` writes a borrowed normalized signature, and `EditorKeymap::resolve_keyboard_input` probes `signature_index` before exact candidate matching. The current source includes 10K-binding and one-million-event tests, but they have not been executed through the managed current-source gate. Keep PERF-MVP-074 open only for collision/probe/allocation counters, settings-generation rebuild cost, behavior parity and F4 keyboard evidence; do not design a second index.

### PERF-MVP-211: full catalog query and context clone remain

The old descriptor-id BTree lookup is also gone: catalog generation stores enablement slots aligned with entries. The remaining query cost is still linear in command count for every edit and window request. A non-empty query can scan each search document twice, a blank query evaluates all enablement slots, MRU membership/rank performs bounded 32-entry scans, and `CommandEvalSnapshotHandle::snapshot` first clones the capability set. The registry mutex is held for the complete query, after which only 8 visible plus 4 overscan rows are converted to `UiValue`.

Editor08 should publish a generation-owned normalized/token index and retain prefix-query state or a bounded top-K candidate frontier. Command evaluation state should be an immutable shared generation/compact dependency bitset so a query does not clone capability strings. Preserve complete match counts, deterministic score/MRU/order, current 12-row projection, exact collision checks and no stale-generation apply. Do not move this latency-sensitive UI query to an unbounded private worker queue.

### PERF-MVP-076/099: menu projection amplifies full reflection

`WorkbenchViewModel::build_with_context` calls `menu_bar_model`; the latter performs seven full command scans and allocates every projected menu row. Both retained recompute and reflection refresh hold `EditorCommandRegistryHandle`'s mutex while building the model, so command reads/mutations are serialized behind broad UI projection. This is not a separate menu-only defect: event-time full reflection and recompute are already the P0 owner.

EditorUI08/Editor08 should compile menu descriptors into top-level buckets once per registry generation, publish immutable row metadata, and evaluate only dynamic enablement fields from a compact context generation. Full model/reflection work must run after cloning a stable registry/menu handle and releasing the mutex. Stable domain generation must build zero menu rows; changed context should patch affected enabled states rather than reconstructing all labels/ids/shortcuts.

### PERF-MVP-079/538 and PERF-MVP-062

Extension registration still clones the full command registry and registry insertion independently checks duplicate command, route and commandlet name before advancing generation. Preserve failure atomicity but stage a complete batch, validate/build command/route/name/menu/palette indexes once and publish one generation. Direct `is_enabled` evaluation and streaming validators already satisfy PERF-MVP-062 at source level; only managed behavior/performance acceptance remains.

## Acceptance plan

- Keymap: bindings `1/100/10K`, events `1/1M`, signature collisions `0/1/100%`, settings updates `1/1K`. Record owned key/String allocations, hash probes, exact candidate comparisons, rebuild allocations/p95 and input-thread lock time. Stable dispatch requires zero owned key allocation and work near O(1)+actual collision candidates.
- Palette: commands `1/100/10K/100K`, opens `1/100`, edits `1/1K`, query lengths `0/1/32`, windows `0/12/504`, capabilities `0/100/10K`. Record context clone bytes, registry lock wait/hold, document visits, substring/subsequence comparisons, enablement evaluations, candidate handles, retained rows, `UiValue` clone bytes and p50/p95. Stable catalog build is zero; visible projection is at most 12 rows; query work must be driven by indexed/incremental candidates rather than all commands.
- Menu/reflection: commands/menu rows `1/100/10K`, editor events `1/1K`, unchanged/enablement/structural generations. Record command scans, row/string builds, registry lock hold, workbench/reflection builds and frame p95. Require one menu index build per structural generation, zero stable rebuild, changed enablement near affected rows and no broad projection while holding the registry mutex.
- Registration: commands/extensions `1/100/10K`, duplicate route/name/id at first/middle/last, failed batches `1/100`. Record descriptor visits, registry clones, index builds, generations and rollback bytes. Require one validate/index/publish per successful batch and zero published change on failure.
- Preserve keyboard alias/dead/unidentified/released behavior, command ordering, when/headless semantics, MRU ranking, total match count, deep paging, menu projection ownership and extension rollback. Run current-source managed command tests plus F4 open/search/keyboard/menu interaction before promotion.

## Reference check

- Unreal `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandList.cpp` builds one `FInputChord`, asks `FInputBindingManager` for the command in each active context, then performs a direct command-to-action map lookup. This supports Zircon's current signature index and the remaining goal of compact context-aware candidate evaluation instead of a full binding or catalog scan.
- Godot `dev/godot/editor/settings/editor_command_palette.cpp` scans every command, constructs an owned candidate vector, sorts it and materializes up to 300 rows on each text change. Zircon's generation-owned normalized documents and 12-row window are already a stronger baseline; Godot is a useful warning against restoring full candidate/row materialization, not the target algorithm for 10K-100K commands.

## Static gates executed

- Read all current 17/17 Rust files and the listed production caller chains.
- `rustfmt --edition 2021 --check` passed for all 17 files.
- `git diff --check -- zircon_editor/src/core/commands` passed. Existing diffs remain in seven tracked files and one untracked test subtree; no file was rewritten.
- No managed Cargo, allocator/RSS scale run, WPR F4 product trace or independent dynamic review ran. RenderDoc is not applicable to this CPU/editor-command slice. The module remains pending.
