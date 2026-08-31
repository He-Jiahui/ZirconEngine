---
title: Editor10 Active Scene Reload Async Preparation
category: zircon_editor
report_id: Editor10-active-scene-reload-async-preparation-2026-08-27
date: 2026-08-27
session_id: root-editor-architecture-goal-20260827
implementation_status: source_implemented_open_followups
validation_status: focused_static_complete_managed_and_product_trace_pending
---

# Editor10 Active Scene Reload Async Preparation

## Scope

The active-generation hard cut removes the invalid project reopen and full source scan from an
active-scene refresh. The remaining retained-host path still performs artifact I/O, scene
deserialization, reference resolution, and `World` construction synchronously inside the UI tick:

`refresh_project_assets -> apply_asset_refresh_plan -> request_active_scene_reload`.

This slice moves project-authority scene open into the existing `EditorJobSystem`, keeps one typed
pending ticket in the retained host, and performs only the terminal authoring-world preparation and
generation-fenced replacement on the UI thread. It does not claim that `create_level` is cheap or
thread-safe; that boundary remains on the main thread until profiling proves it can move.

## Current-Source Review

- `Scene::load_scene_from_uri` loads the scene artifact and constructs every entity/component,
  resolving mesh, physics, animation, prefab, script, camera, and resource references as it walks
  the scene. Its work scales with scene entities and referenced data.
- `ProjectAuthority::open_scene` already owns project-root containment, linked-component rejection,
  scene URI validation, source identity, and scene loading. An editor job must call this owner rather
  than invoke `Scene::load_scene_from_uri` directly.
- `EditorJobSystem` is the established bounded owner for editor background work. `EditorJob` output
  is `Send`, `JobTicket::try_take` is nonblocking, `JobPriority::Interactive` prevents a direct user
  refresh from being treated as background maintenance, and cancellation is available by `JobId`.
- The retained host already polls model-import and other typed tickets before asset refresh. A scene
  load should use the same lifecycle and must not introduce a raw thread or callback that mutates UI.
- A second matching active-scene event while a load is pending must set one coalesced retry bit.
  Project close cancels and drops the local ticket only after the project close authority succeeds;
  a completed stale job can only install through the Runtime generation fence and document
  lifecycle route gate.

## Reference-Engine Evidence

Unreal exposes `LoadPackageAsync` with a completion delegate and priority in
`CoreUObject/Public/UObject/UObjectGlobals.h`. `AssetViewUtils.cpp` queues package request IDs before
an explicit wait/reset phase. The applicable architecture rule is asynchronous package preparation
followed by an explicit terminal transition; it is not permission to mutate editor state from a
loader worker.

Local references:

- `dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/UObjectGlobals.h`
  (`LoadPackageAsync`, completion delegate, package priority)
- `dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetViewUtils.cpp`
  (`PendingPackageRequestIds`, `LoadPackageAsync`, explicit `FlushAsyncLoading` boundary)

## Baseline Evidence

The repository examples bound only input scale; they are not product timing evidence:

| Scene | Bytes | Entity tables |
| --- | ---: | ---: |
| `examples/woc/assets/scenes/bootstrap.scene.toml` | 320 | 1 |
| `examples/vampire/assets/scenes/main.scene.toml` | 88,241 | 110 |
| `examples/woc/assets/scenes/eastbrook_mvp.scene.toml` | 115,884 | 268 |

Before this slice, UI-thread scene-load operations per reload were one artifact load, one complete
deserialize/world build, one `create_level`, and one world replacement. Target after this slice is
zero UI-thread artifact loads and zero UI-thread scene deserializations. `create_level` and terminal
replacement remain one each and require separate wall-time traces.

## Implementation Plan

1. Add a headless-safe `ProjectSceneLoadJob` and typed ticket under `core/project`; the job calls
   `ProjectAuthority::open_scene`, checks cancellation before and after the non-interruptible load,
   and uses the shared Index category with interactive priority.
2. Replace synchronous retained-host reload with `request_active_scene_reload`: capture the exact
   active project/scene/document identity, acquire one active project generation, submit the typed
   job, and retain token/identity/ticket in one pending slot.
3. Coalesce every overlapping request into one retry bit. Poll completion before asset refresh,
   prepare the authoring seed on the UI thread, and install only through
   `commit_if_project_generation`.
4. Serialize terminal installation with open/create/project-close routes, fail closed when the
   document is dirty, and preserve project settings plus the active document binding. Publish a
   prepared Runtime level only inside that terminal installation so rejection cannot orphan a
   registered level.
5. On successful commit, invalidate render and presentation. On same-project supersession,
   an overlapping request, enqueue one bounded synthetic refresh. Admission pressure uses a
   separate identity/generation-keyed 64/128/256 ms retry state and becomes terminal after three
   retries. On a
   successful close, cancel and drop the local ticket without waiting for scene I/O.
6. Add Runtime-independent static contracts and Rust behavior tests where the current shared Cargo
   baseline permits. Capture product event-to-worker, worker-load, UI-prepare, and commit timing
   before accepting the optimization.

## Acceptance

- Retained asset refresh contains no `Scene::load_scene_from_uri` call.
- At most one active-scene load ticket is retained per host; overlapping requests collapse to one
  retry and do not allocate unbounded jobs.
- ProjectAuthority remains the only scene-open validation/load owner.
- A stale project generation or superseded project/scene/document activation revision never
  replaces the authoring world, including an A-to-B-to-A route sequence.
- Dirty local edits enter a typed conflict state instead of being discarded; successful refresh
  preserves project settings and the active document identity while resetting scene history,
  selection, and viewport state.
- A rejected terminal install leaves no newly registered Runtime level behind.
- Project close is not blocked by scene I/O and drops the pending UI ownership.
- Focused contracts and formatting pass. Managed Cargo and product F0/F4 timing/power evidence are
  required before this report can become accepted.

## Implementation

`ProjectAuthority::submit_scene_open` now submits `ProjectSceneLoadJob` through the shared Index
category at interactive priority. The job returns a typed `ProjectSceneDocument`, checks
cancellation before and after the non-interruptible authority load, and never receives a UI handle.
The retained host keeps one `PendingActiveSceneReload`; overlapping events set one retry bit rather
than creating another job. Admission-limit failures retain an identity/generation-keyed state with
64/128/256 ms backoff. The fourth rejection becomes terminal for that exact identity/generation;
same-generation watcher events coalesce without restarting the loop, while a new activation or
Runtime generation resets it. A successful project-close authority commit then cancels and drops
the ticket/admission state without waiting; a rejected close leaves them intact. Asset accumulation
and admission retry share one retained-host maintenance wake owner: each refresh publishes the
earliest of the accumulator and retry deadlines, so the empty refresh later in the same tick cannot
erase a retry scheduled by the reload poll.

On completion, Runtime first classifies the project generation without retaining the read fence
through `create_level`. Already stale work is discarded before main-thread preparation. Current work
is prepared and then passed through `SceneDocumentReloadCoordinator`, which serializes the exact
`ActiveSceneDocumentIdentity` check with open/create/project-close routes. The identity includes a
monotonic activation revision, so an old A load cannot commit after A-to-B-to-A. Dirty state produces a
typed retained conflict. The host uses the dedicated `reload_active_scene_world` transition, so it
does not clear the active document binding or reload project settings. Runtime level creation now
returns an unregistered `PreparedLevel`; publication occurs immediately before the gateway swap and
rolls back on failure. The terminal Runtime generation fence remains held through that installation.

The first activity-identity/lifecycle contracts failed 2/2 before implementation, the saved
conflict recovery contract failed 1/1, and independent re-review then drove a 2/2 red case for
A-to-B-to-A plus unbounded admission retry. The dirty Decision contract then failed because the
conflict owner and command surface did not exist. The focused generation/competition contract passes 13/13 and the
complete Editor10 static suite, including dirty-save ownership competition, passes 15/15. Rust behavior
tests have been authored for the typed scene-load job, generation precheck,
lifecycle supersession/dirty rejection, staged Runtime-level publication, and shared maintenance
deadline composition. Their execution remains pending. Current E-drive isolated attempts for
Runtime `--lib --tests`, production `--lib`,
and `core-min --lib` reached their 15, 7, and 4 minute limits; a shared-target production retry then
reached its 3 minute limit. None produced a terminal result or captured Rust error, and timeout
cleanup left no matching Cargo/rustc process. These attempts are not counted as
passing or failing the owned behavior tests. A later E-drive shared-target
`cargo check -p zircon_editor --lib --offline` reached another terminal result after 161.3 seconds but
failed in `zircon_runtime` with 61 shared current-source errors and 123 warnings before compiling
the editor owner; it likewise provides no owned behavior result. A subsequent isolated E-drive
attempt remained in `zircon_runtime` and timed out after 364.2 seconds without diagnostics; its exact
process tree was retired, and it also provides no editor-owned result.

The retained conflict now owns one generation-bound Decision with Save, Discard, and Keep Editing.
Save uses the canonical `SaveProject` command and Global-history save token; it does not route the
lifecycle document id through the toolkit dirty-save batch. Keep Editing suppresses the exact watcher generation.
Discard is an explicit terminal dirty policy; it skips admission but does not clear history until
the lifecycle identity and Runtime generation fences admit `reload_active_scene_world`. An evicted
Decision ticket is republished rather than guessed. See
`2026-08-27-active-scene-reload-dirty-conflict-decision.md`.
The final independent source review returned `READY` with no P1/P2 findings; this is source-review
evidence only and does not replace the pending managed behavior or product trace gates.

## Remaining Work

The Runtime project snapshot still clones the shader dependency index. World runtime-extension
application, level allocation, and authoring seed construction remain on the UI thread because
their service access and thread-safety contract have not been proven. Job cancellation is checked
only around the current non-interruptible authority load. The retained conflict command surface is
implemented, but its Rust behavior and product interaction lanes have not reached the editor test
binary because of the shared Runtime compile baseline. Product
traces must determine whether terminal preparation, world replacement, or downstream
hierarchy/render rebuild is now the dominant frame stall.
