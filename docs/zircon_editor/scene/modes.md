---
related_code:
  - zircon_editor/src/scene/selection
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/viewport/controller/scene_viewport_state.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
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
topmost id for command evaluation. `SceneModeCtx` currently contains the shared
selection model and viewport settings. Transaction and gateway access will be
added when Editor05 M2 connects transform operations.

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

`SceneModeRegistration` binds one validated `ViewportToolModeDescriptor` to a
thread-safe `SceneModeFactory`. `SceneModeRegistry` is the runtime owner of
these registrations: duplicate typed ids are rejected at registration,
unknown ids are rejected at creation, and every created mode must report the
same `SceneModeId` as its descriptor. A mismatched plugin factory therefore
fails before the mode can enter a stack or consume viewport input.

The descriptor remains the metadata and capability-facing contract; the
factory is the executable contract. Neither a provider string nor descriptor
presence is treated as proof that a runtime mode instance exists. Production
host ingestion of extension registrations is still pending behind the current
Editor07 host ownership boundary.

## Current integration boundary

This implementation completes the selection-authority hard cut, independent
mode-stack contract, and descriptor-backed factory registry. The 28 production
consumers have migrated atomically and the parallel legacy field and helper
API are both absent. Fresh managed validation is still required before the
Editor07 failure is returned as fixed. Production host ingestion, built-in
Select/Transform registrations, and Navigation overlay-provider lifecycle
wiring remain open parts of Editor05 M1 and its other inbound handoffs.
