# Editor core project current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Primary owners: Editor10 for authority/session/product orchestration, Runtime04 for the prepared project and manifest generation, Runtime11 for budgeted filesystem work, and Editor14 for Welcome job admission.
- Accounting: keep `zircon_editor/src/core/project/**` in `pending.md`. Do not add it to `review.md` before current-source managed Cargo, deterministic I/O/allocation counters and F0/F4 product evidence are GREEN.
- Code disposition: no Rust source was changed. Existing tracked modifications and the untracked current `project_probe.rs` were reviewed and preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/project/**` | 18/18 | 1,690 | 23 | `b9f52b7d6dc7b20b062138b48b68d0d71c24a8c2d08d4997a46befd958a3febd` |

The fingerprint streams each workspace-relative path, a zero byte, the file's raw bytes and a zero byte in sorted path order into SHA256. All 18 files were read in full. Production reachability was followed through startup creation/open/recent/session restoration, the retained Welcome probe, Runtime prepared-project activation, project save and asset-watcher refresh.

## Per-file review

| file | current-source performance result |
|---|---|
| `authority.rs` | Open returns one prepared `ProjectManager`, removing the old Authority/Runtime/Editor reopen chain. Creation is synchronous and transactional: it repeats target validation, writes every owned template entry with per-entry `create_dir_all`, then reloads and saves the manifest before opening it again. Legacy session migration allocates and probes an externally sized recent list before typed decode or the normal eight-entry limit. |
| `created_project.rs` | Owns and transfers the prepared manager rather than reopening by path. Root and summary are still duplicated beside the manager for presentation, but creation is not a steady-frame path. |
| `error.rs` | Typed error/source propagation only. It preserves I/O and rollback evidence and does not own a queue, scan or retry loop. |
| `filesystem.rs` | Canonicalization and link/reparse rejection perform metadata work over existing ancestors. The former duplicate post-canonical walk is gone; the remaining security walk is required and must be counted rather than bypassed. |
| `mod.rs` | Module declaration and narrow exports only. |
| `new_project_draft.rs` | Validation trims owned draft strings, validates the name and builds a target path. Welcome now runs this in a background job; create deliberately rechecks it at the transaction boundary. |
| `new_project_template.rs` | Small enum-to-pack-id mapping only. |
| `opened_project.rs` | Wraps the one prepared `ProjectManager` and exposes borrowed root/summary or ownership transfer. This is the current generation-sharing boundary. |
| `project_probe.rs` | Carries only canonical root and cloned manifest summary. Because it omits a content identity/prepared generation, the successful background result cannot currently be promoted by the click/open path. |
| `recent_project_entry.rs` | Presentation row owns path and summary strings plus scalar time/validation. Projection cost is acceptable only after the persisted list is bounded. |
| `recent_project_validation.rs` | Compact result enum; no work is performed here. |
| `stored_recent_project_entry.rs` | Persisted row DTO. It has no local size validation; the session decoder must enforce the entry limit before migration I/O. |
| `stored_startup_session.rs` | Persisted session DTO with an unconstrained `Vec`. Normal remember flow truncates to eight, but hostile/legacy decode can bypass that bound until authority sanitization is added. |
| `tests/boundary.rs` | Static ownership guard rejects UI dependencies and retired template generation. It does not measure I/O or allocation counts. |
| `tests/directory_transaction.rs` | Four rollback/backup fault tests protect the transaction boundary that template streaming must preserve. No scale or syscall counter is present. |
| `tests/mod.rs` | Test module wiring and temporary-root helper only. |
| `tests/recent_projects.rs` | Four tests cover summary persistence, dedup/newest ordering, dynamic validation and legacy migration. Missing cases are oversized/malformed session pre-cap behavior and unchanged-generation I/O counts. |
| `tests/template_creation.rs` | Thirteen tests cover template contents, canonical publication, settings, non-ASCII paths, generation ownership, unsafe/non-empty targets, read-only probe and rollback. They do not prove single parse/encode, shared template bytes, unique-parent creation or off-main-thread execution. |

## Corrected and remaining tasks

### PERF-MVP-075: prepared manager exists; product paths still repeat probe work

`ProjectAuthority::open_project` now opens a `ProjectManager` once, `OpenedProject` transfers it to `AssetManager::open_prepared_project`, and Editor document/catalog/watcher setup consumes the retained current snapshot. Locator lookup also uses the manager-owned index. The old three-manager reopen finding is no longer current.

The remaining duplication is above this module. A valid Welcome draft can be processed by the background probe, synchronously probed again on click, and then canonicalized/parsed again by open. Startup validates every recent project and subsequently opens the last one, so that manifest is processed twice. Replace the summary-only handoff with a generation ticket containing canonical identity and manifest fingerprint; promote it only after a mutation-stamp/TOCTOU check. Project save should compare the active generation without another canonical walk and schedule serialization/import/catalog/watcher work through Runtime11 budgets.

### PERF-MVP-100: bound persisted input before per-entry I/O

Normal `remember_recent_project` truncates to eight, but `decode_startup_session` first migrates raw JSON. Migration reserves `recent_projects.len()` and probes every legacy entry lacking a summary. A large old or malformed session can therefore cause unbounded allocation, canonical metadata walks and manifest reads before typed decode. Enforce session bytes/schema/entry limits and stable dedup at the raw boundary before any probe. Validate bounded rows against a manifest identity plus file-watcher generation in a background ticket; UI snapshots should only project last-good rows.

### PERF-MVP-559: source implementation present, dynamic gate missing

The current retained Welcome owner has a 50 ms trailing debounce, 250 ms maximum feedback delay, one pending plus one active generation, same-draft reuse, cancellation before I/O and between validation/probe phases, and a Background/Index job. Source tests include 1K and 1M draft bursts, latest-only behavior, delay bounds, cancellation and typed failure. Keep the task open until those tests run under the managed current source and counters prove queue entries/bytes/oldest age, filesystem calls and UI latency. The click path must also consume the accepted background generation instead of probing again.

### PERF-MVP-568: template transaction still multiplies bytes and parse work

`render_project_template` clones every embedded entry into a `Vec<u8>`, rewrites/parses the manifest and returns a summary. Editor ignores that summary while writing every entry synchronously, then loads and saves the manifest, publishes the directory, opens a manager and derives another summary. Preserve the tested staging/backup/rollback protocol, but carry shared static entry bytes and the rendered manifest artifact into a Runtime11 write ticket, create unique parent directories once, delete the post-write load/save, and reuse the prepared result.

## Acceptance plan

- Open/probe: paths `1/100/1K`, manifest `1KiB/64MiB`, unchanged/changed/replaced/link cases. Count canonical/link walks, stat/read/parse, probe tickets, manager opens, inventory scans, UI-thread I/O and p50/p95. Require at most one walk/parse/open/scan per accepted generation and zero stable snapshot I/O.
- Recent/session: persisted bytes `1KiB/1MiB/64MiB`, entries `0/8/1K/1M`, current/legacy/malformed. Count allocations before the cap, migrated/probed rows, reads/parses and startup latency/RSS. Oversized input must be rejected or reduced before per-entry filesystem work.
- Create: entries `1/1K/100K`, template bytes `1KiB/1GiB`, unique/shared parents and injected failures. Count cloned bytes, mkdir/write calls, manifest parse/serialize, UI-thread time and RSS. Require zero clone for unchanged static entries, one manifest parse/encode generation, no post-write load/save and unchanged rollback semantics.
- Save: assets `1/1K/100K`, dirty `0/1/100%`, watcher changes `0/1K`. Count root resolution, serialization, import/catalog refresh, watcher restart, queue bytes/age and UI latency. Stable save must not rescan/reparse on the caller thread.
- Run current-source managed project tests and F0 create/open/recent/restore/save plus F4 project switching. RenderDoc is not applicable to this CPU/filesystem slice; project-driven rendering is accepted by its render-owner plans.

## Reference check

- Godot `dev/godot/editor/project_manager/project_list.cpp` explicitly scans directories in `_scan_thread` to avoid blocking the window, then publishes in `_scan_finished`. Zircon should keep its stronger canonical/link checks but execute bounded project discovery/validation through the existing job system.
- Godot `ProjectList::load_project_data` loads one config for presentation and retains `modified_time_cache` for stable sorting. Zircon can use a stronger manifest fingerprint/file-watcher generation rather than mtime alone, while preserving the useful single-load and cached-projection shape.
- No current Unreal/Fyrox source path was used for a claim in this slice; no missing reference was substituted with memory.

## Static gates executed

- Read all current 18/18 Rust files and the listed production caller chains.
- `rustfmt --edition 2021 --check` passed for all 18 files.
- `git diff --check -- zircon_editor/src/core/project` passed. Existing tracked/untracked changes were not rewritten.
- Source inventory was 18 files, 1,690 physical lines and 23 inline tests at fingerprint `b9f52b7d6dc7b20b062138b48b68d0d71c24a8c2d08d4997a46befd958a3febd`.
- No managed Cargo, syscall/allocation/RSS scale run, WPR F0/F4 product trace or independent dynamic review ran. The module remains pending and `review.md` is unchanged.
