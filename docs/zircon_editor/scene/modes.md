---
related_code:
  - zircon_editor/src/scene/selection
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/handles/transform_handle_kind.rs
  - zircon_editor/src/scene/viewport/controller
plan_sources:
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-12-command-eval-scene-mode-selection-projection.md
  - docs/plans/zircon_editor/editor/07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md
tests:
  - zircon_editor/src/scene/selection/tests.rs
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/editing/viewport.rs
---

# Scene selection and mode contracts

Editor scene interaction owns two independent state families: world selection and
interaction modes. Neither state is serialized into runtime scene assets.

## Selection authority

`SelectionModel` is the only scene selection authority. It owns an ordered
`IndexSet<EntityId>`, a primary entity, and a generation for each of the Edit and
Play domains. A separate revision changes whenever either domain changes or the
active domain switches.

The model guarantees:

- duplicate entity ids are removed while insertion order is retained;
- a non-empty selection always has a primary entity inside the set;
- idempotent writes do not advance generation or revision;
- Edit and Play selections never overwrite one another;
- switching domains changes only the active projection, not either stored set.

`SceneViewportState` stores this model directly. The previous `selected:
Option<u64>` field and the controller's `selected_node` / `set_selected_node`
compatibility methods are deleted. Viewport internals, hierarchy, inspector,
snapshot, startup, binding dispatch, and Workbench state read
`active_primary()` from the model and use explicit active-domain operations
(`select_only_active`, `replace_active`, or `clear_active`) for mutation.

Non-selection edits leave the ordered set and primary unchanged. Deletion
filters entities that no longer exist while retaining surviving members; only
an empty result uses the command's fallback entity. PIE copies the Edit set
into the Play domain on entry, lets Play selection evolve independently, and
restores the complete pre-PIE dual-domain model on exit. History entries carry
ordered before/after selection snapshots only for selection-changing commands,
so ordinary property edits and gizmo commands cannot collapse a multi-selection
during undo or redo, while create/delete restore the entire prior set.

## Mode stack contract

`EditorSceneMode` defines lifecycle, input, update, and overlay hooks. A
`SceneModeStack` owns one base mode plus zero or more temporary overlay modes.
Input is offered from the top overlay downward and stops at the first
`InputOutcome::Consumed`; update and overlay extraction run bottom-up.

Mode ids are typed `SceneModeId` values. The stack rejects duplicate active ids,
calls `enter` exactly once on insertion, calls `exit` on pop, and exposes the
topmost id for command evaluation. `SceneModeCtx` exposes the shared selection
model, immutable viewport settings, and one inline input effect for the current
dispatch. Built-in modes may emit a pointer move, primary press, or primary
release effect; the controller applies only the effect from the mode that
consumed the input. Effects emitted by an overlay that returns `PassThrough`
are discarded, so an overlay cannot leak a partially handled interaction to
the base mode. Transform modes publish geometry-only preview requests; the
workbench applies those requests inside the Editor03 gizmo transaction lane,
records each accepted preview, and commits one command when the handle session
ends. The controller never writes the scene transform directly.

`SceneModeStack::project_command_eval_ctx` is the neutral command-state
projection. It preserves the caller's existing project, play, document,
capability, and history fields while replacing only `scene_mode` and
`selection_count` from the authoritative stack and active world-domain
selection. Production UI code must call this projection; inspector visibility
or toolbar state is not a valid substitute.

`ViewportOverlayBuilder` collects neutral `SceneGizmoOverlayExtract` values. It
is the common output boundary for built-in modes and the planned plugin overlay
provider registry; providers do not mutate viewport state directly.

## Mode factory registry

`SceneModeRegistration` binds one validated `SceneModeDescriptor` to a
thread-safe `SceneModeFactory`. `SceneModeRegistry` is the runtime owner of
these registrations: duplicate typed ids are rejected at registration,
unknown ids are rejected at creation, and every created mode must report the
same `SceneModeId` as its descriptor. A mismatched plugin factory therefore
fails before the mode can enter a stack or consume viewport input.

The descriptor remains the metadata and capability-facing contract; the
factory is the executable contract. Neither a provider string nor descriptor
presence is treated as proof that a runtime mode instance exists. Production
host ingestion prepares and validates a candidate registry before mutating the
workbench, then installs that prepared registry without invoking factories a
second time.

Registry-created modes retain their extension owner and execute factory/id,
enter/exit/update/input, and overlay callbacks through the editor plugin panic
boundary. A callback panic rolls back the current context or overlay-builder
increment, faults that mode instance, and cannot interrupt shutdown of the
remaining stack. Overlay-provider factories are prepared before host mutation;
provider extraction uses the same owner-scoped containment boundary.

## Activation and transform handles

`SceneModeActivation` is the command and UI transition value. `Select` and
`Transform(TransformHandleKind)` resolve to the built-in `scene.select` and
`scene.transform` descriptor ids; `Custom(SceneModeId)` uses the same registry
path for extension modes. A missing registry entry is an activation error, not
a controller panic, and a failed activation restores the prior transform-handle
configuration.

The mode stack is the sole owner of the current base mode. `SceneViewportSettings`
does not contain a current-mode or tool enum; it retains only the transform
handle kind used when the active base mode is `scene.transform`. Select and
custom modes therefore produce no transform handles. UI binding, editor events,
and the retained toolbar use `ActivateSceneMode`; the retired viewport-tool
enum and command protocol have no compatibility parser or controller fallback.

## Current integration boundary

This implementation has the selection-authority hard cut, the independent
mode-stack contract, descriptor-backed factory registry, and built-in
Select/Transform registrations. Production extension ingestion accepts only a
`SceneModeRegistration`, derives the descriptor projection from it, validates
the factory id atomically, and installs the executable factory into the
controller registry. Push/pop/update/shutdown lifecycle is connected through
the host and state lifetime.

`SceneModeActivation` is wired through the viewport controller, editor event
path, binding codec, and retained toolbar; custom ids remain intact in toolbar
projection. Primary pointer modifiers map to replace/extend/toggle selection,
and drag rectangles query the shared renderable interaction extract rather than
scanning the scene. Mode/provider gizmos are merged into that same immutable
extract, so render and pointer routing consume one overlay snapshot. Fresh
managed validation remains required before the Editor05 failure is returned as
fixed. Multi-selection transform pivots, Escape cancellation, and Editor03's
accepted transaction gate remain open work.
