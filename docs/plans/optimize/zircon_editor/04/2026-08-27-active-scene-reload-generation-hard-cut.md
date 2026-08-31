---
title: Editor04 Active Scene Reload Generation Hard Cut
category: zircon_editor
report_id: Editor04-active-scene-reload-generation-hard-cut-2026-08-27
date: 2026-08-27
session_id: root-editor-architecture-goal-20260827
implementation_status: implementation_complete
validation_status: static_complete_managed_and_product_trace_pending
---

# Editor04 Active Scene Reload Generation Hard Cut

## Scope

This slice reviews and removes the project reopen and full source scan performed by the retained
editor host when an already-committed asset/resource event says that the active scene source
changed. The hard cut uses the lifecycle-owned active scene identity; the manifest default is only
the startup selection and cannot overwrite a secondary scene. It also closes the generation race
between off-lock scene preparation and authoring-world
installation. It does not redesign scene deserialization, watcher event folding, or Runtime import
transactions.

The owning product path is:

`RetainedEditorHost::refresh_project_assets -> apply_asset_refresh_plan -> request_active_scene_reload`.

## Current-Source Review

The refresh planner consumes Runtime `AssetChange` and `ResourceEvent` values after the active
project generation has already been updated. Despite that committed input, the retired
`reload_default_scene`
currently:

1. reads `ProjectInfo` from the active `AssetManager`;
2. calls `ProjectManager::open` for the same root;
3. calls `scan_and_import` for the complete project;
4. loads the manifest default scene from the newly opened manager, even when a secondary document
   is active;
5. installs a new authoring world.

The first three operations create a second project truth on the UI tick. Their cost is
`O(project files + import decisions + resource publication)` per matching event rather than
reusing the active generation before the unavoidable scene load. They can also race the
generation that produced the event.

The existing `AssetManager::current_project_snapshot` contract returns the prepared,
generation-bound `ProjectManager` retained by Runtime. `EditorUiHost::open_prepared_project`, path
resolution, and project save consume that contract, so scene reload should use the same owner.
Current-source field review shows that the resource registry, asset index, importer generation,
artifact residency, and catalog generation are shared by `Arc`; project paths/manifest/package
roots are small owned values. The shader dependency index is still copied, so snapshot acquisition
is not claimed to be strict `O(1)` and must eventually hard-cut to a shared project-generation
handle.

## Reference-Engine Evidence

Unreal's `IAssetRegistry` exposes `ScanModifiedAssetFiles` and `OnAssetUpdated`/
`OnAssetUpdatedOnDisk` rather than requiring a project-wide rescan for an individual changed
asset. `FAutoReimportManager` processes modifications through a time-limited editor tick state
machine. The applicable rule is to consume the committed asset generation/event and reload the
target package or scene; a UI refresh handler must not reopen the project database.

Relevant local sources:

- `dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h`
  (`ScanModifiedAssetFiles`, `OnAssetUpdated`, `OnAssetUpdatedOnDisk`)
- `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/AutoReimport/AutoReimportManager.cpp`
  (`ProcessModifications`, time-limited state-machine tick)
- `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp`
  (`FEditorFileUtils::LoadMap` as an explicit document transition, not an asset-event catalog scan)

## Baseline Evidence

Source-level operation counts for one matching active-scene event:

| Operation | Current | Target |
| --- | ---: | ---: |
| Project manager reopen | 1 | 0 |
| Full `scan_and_import` | 1 | 0 |
| Active generation snapshot | 0 | 1 |
| Scene deserialize/install | 1 | 1 |

PowerShell `Stopwatch` sampled 50 warm recursive enumerations after three warmups. This measures
only the filesystem-enumeration lower bound, not manifest parsing, sidecar reads, hashing, import
decisions, artifact work, resource preparation, or scene loading:

| Sample asset root | Files | Bytes | p50 | p95 | max |
| --- | ---: | ---: | ---: | ---: | ---: |
| `examples/vampire/assets` | 125 | 1,950,367 | 5.117 ms | 13.841 ms | 24.082 ms |
| `examples/woc/assets` | 111 | 29,941,344 | 11.640 ms | 45.518 ms | 52.600 ms |

Because a 60 Hz frame is 16.667 ms, the redundant enumeration alone already exceeds the frame
budget at the WOC p95 before any import or scene work. The structural operation count, rather than
these small sample timings, is the primary acceptance criterion.

Full editor profiling is currently blocked before `zircon_editor` by unrelated shared-tree
`zircon_runtime` hard-cut compile failures. No CPU, GPU, power, or shipping-project comparison is
claimed from the proxy sample.

## Implementation Plan

1. Add a Runtime-owned active-project generation snapshot containing the prepared
   `ProjectManager`, project root, and catalog sequence captured under the project-generation read
   gate.
2. Add a conditional commit operation that reacquires the generation read gate, rejects a stale
   token before invoking its closure, and retains the gate across the short editor-world commit.
   This prevents generation B from publishing between validation and replacement by generation A.
3. Hard-cut the retired default-scene reload to capture the lifecycle-owned active
   project/scene/document identity, load from the Runtime snapshot outside the gate, and replace
   the authoring world only through the lifecycle coordinator plus conditional generation commit.
4. When a same-project newer generation supersedes the prepared scene, enqueue one synthetic
   active-scene reload in the existing 32--250 ms asset-refresh accumulator. Project close or a
   different active root discards the obsolete result without retrying.
5. Add Runtime concurrency tests and an editor source contract. The contract rejects
   `ProjectManager::open`, `scan_and_import`, and an unconditional `replace_world` in the retained
   reload owner.

The token deliberately excludes `project_preparation_epoch`. That epoch identifies unpublished
candidate work and is required when a watcher candidate itself wants to publish. A scene reload is
derived from the already-published active generation, so a future preparation does not invalidate
it. Treating preparation start as active-generation supersession would either lose the requested
reload when that preparation fails or repeatedly deserialize the scene while a long project open
is still preparing. The generation read fence is retained through the terminal world replacement,
so a candidate cannot publish between the catalog-sequence check and the commit.

## Implementation

`RetainedEditorHost::request_active_scene_reload` now captures the lifecycle-owned active scene
identity and acquires one Runtime-owned project generation snapshot. It removes the secondary
`ProjectInfo` lookup, project reopen, full `scan_and_import`, and manifest-default target inference.
Scene loading/deserialization runs in a typed Editor job. The terminal lifecycle coordinator checks
the complete project/scene/document activation identity, including a monotonic lifecycle revision
that rejects A-to-B-to-A stale work, admits dirty state, and invokes
`commit_if_project_generation`, which checks project root and catalog sequence while retaining the
generation read fence through the short callback.

Independent review found and drove closure of the first pass's commit-time race: generation A can
no longer replace the editor world after generation B has published. A same-project catalog
supersession records a profile counter and submits one synthetic reload event through the existing
bounded refresh accumulator; project close, root replacement, or lifecycle identity replacement
discards the obsolete preparation. Dirty state is retained as a typed conflict instead of invoking
the old project-wide replacement. Runtime concurrency tests cover fence retention, stale-token
callback rejection, and the rule that an unpublished preparation epoch does not invalidate the
active generation.

The focused editor contract originally failed against the reopen/rescan implementation; the two
new active-identity/lifecycle cases also failed 2/2 before their implementation. The saved-conflict
recovery case then failed 1/1, and independent re-review produced a further 2/2 red case for
A-to-B-to-A plus unbounded admission retry. The later dirty Decision contract failed before the
command owner existed. The focused generation/competition contract now passes 13/13 and the current Editor10
static suite, including dirty-save ownership competition, passes 15/15. A production-directory scan
reports zero remaining `ProjectManager::open` or `scan_and_import` calls under
`ui/retained_host/app`. The runtime behavior tests are authored but cannot yet be executed through
managed Cargo. Historical isolated logs under
`E:/Git/ZirconEngine/.codex/validation-logs` report 79 core-minimal and 259 broader shared-tree
errors before the owned test binary. Current E-drive retries for Runtime `--lib --tests`, production
`--lib`, and `core-min --lib` instead reached 15, 7, and 4 minute limits; a shared-target production
retry reached 3 minutes. None produced a terminal result or captured Rust error, and timeout cleanup
left no matching Cargo/rustc process. Neither historical
failure nor current timeout is counted as an owned pass.
A subsequent E-drive shared-target `cargo check -p zircon_editor --lib --offline` reached a
terminal result after 161.3 seconds and failed in `zircon_runtime` with 61 current-source errors and
123 warnings before the editor owner compiled. This is a shared baseline failure, not an Editor10
pass or an active-scene reload diagnostic. A later isolated E-drive attempt remained in
`zircon_runtime` and timed out after 364.2 seconds without diagnostics; its exact process tree was
retired and it likewise did not compile the editor owner.

## Acceptance

- Static operation count is reopen `1 -> 0`, full scan `1 -> 0`, active generation snapshot
  `0 -> 1`, unconditional stale-world commit `1 -> 0`.
- Only an asset/resource event matching the lifecycle-owned active scene source, a bounded
  admission retry, an overlapping request, or a same-project superseded preparation requests a
  refresh. The manifest default is not consulted after startup activation. Asset accumulation and
  admission retry compose their deadlines through one owner, preserving the earlier wake when an
  empty asset refresh follows retry polling in the same tick.
- A controlled interleaving test proves that the conditional commit closure holds the Runtime
  generation read fence and that a stale token never invokes its closure.
- Focused static contracts, `rustfmt --check`, and scoped `git diff --check` pass. Runtime behavior
  tests remain pending until the shared compile baseline reaches the owned test binary.
- Final independent source review returned `READY` with no P1/P2 findings after rechecking the
  canonical SaveProject authority, exact lifecycle/history admission, toolkit save ownership,
  queued Save All continuation, and project/native close barriers.
- Managed Cargo and product F0/F4 traces remain mandatory before this optimization can be marked
  accepted. Product evidence must report event-to-commit wall time, UI blocked time,
  `ProjectManager` open/scan counts, scene load time, RSS, and power from the same scene workload.

## Remaining Work

This removes one architecturally invalid full scan. Runtime still needs to publish an
`Arc<ProjectManager>`-style immutable generation handle so snapshot acquisition cannot copy the
shader dependency index. Scene deserialization now executes in a typed Editor job. The
generation-bound Save/Discard/Keep Editing Decision surface is source-complete and statically
verified, and its final independent source review is `READY`; behavior execution remains pending.
Runtime-extension application and level allocation
still execute synchronously in the retained-host tick, and cancellation is not yet incremental
within the authority load. These are required, with large-scene p95 and
power evidence, before acceptance.
