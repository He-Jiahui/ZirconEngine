---
title: Editor59 Viewport Pointer Button Ownership
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: product_gate_pending
---

# Editor59 viewport pointer button ownership

## Defect and repaired owners

The current controller admitted Primary Down while Orbit or Pan already owned a drag. A current interaction extract allowed that press to overwrite navigation; a stale extract rejected the press, but Primary Up still removed the navigation drag through the unconditional `take()` fallback. Separately, retained Escape and window focus loss sent only the controller cancellation command, leaving Surface capture attached to the old gesture.

The lower shared cause belongs to Runtime `UiSurface`: a foreign button release used to end pointer capture. Runtime1009 owns its button-aware pressed/capture repair. Editor1019 consumes that contract and does not filter foreign edges, synthesize recapture, duplicate Runtime button metadata, or change the public focus DTO.

- `route_primary_pressed` rejects new primary admission while any drag is active. `handle_left_released` only terminates primary selection or handle dragging and restores Orbit/Pan unchanged.
- `SharedViewportPointerBridge::cancel_interaction` dispatches real Surface Cancel and then the existing controller terminal command, including when there is no Surface capture. The existing transaction owner restores gizmo previews. Both native Escape and window focus loss use this entry; focused Game input and the registered Stop shortcut retain their preceding keyboard routing.
- The canonical boundary is documented in [retained viewport](../../../../zircon_editor/ui/retained_host/app/viewport.md). No render packet construction, mesh copy, motion batching, or event reordering was added to pointer dispatch.

## Regression coverage

Ten tests were authored before the production repair; their managed red/green execution remains pending.

| Layer | Tests | Observable contract |
| --- | --- | --- |
| Controller | 4 in `scene_viewport_controller_handle_input/pointer_button_ownership_tests.rs` | Real Orbit/Pan Down survives foreign edges with current and stale extracts; camera changes continue after foreign Down and Up. Real primary selection and rendered-handle admission survive reciprocal navigation edges and terminate on primary release. |
| Shared retained bridge and real EventRuntimeHarness | 4 in `pointer_bridge/button_ownership.rs` | Owner Down, foreign Down/Up, out-of-viewport Move and owner Up preserve the original gesture. Outside camera and gizmo preview changes continue; owner Up stops outside delivery, commits one history record, and Undo restores the original transform. Cancel restores world/history and permits a new capture owner. |
| Native retained host | 2 in `app/tests/native_viewport_cancel.rs` | Actual Escape keyboard dispatch and actual window-focus callback terminate an active real pointer-driven gizmo preview, restore world/history, stop outside delivery, and admit a new navigation owner. |

The tests use existing render publication, picking, world, transaction and callback APIs. They do not fabricate controller drag state or prove capture solely with moves inside the viewport. Render packet reads are fixture publication outside the pointer handler.

## Evidence and acceptance

| Gate | Status |
| --- | --- |
| Scoped ownership | Eleven claimed paths; claim request `5b3565448a5d4427936a39762f6fafd5`. Existing canonical viewport doc and app test binding were checked outside sealed N/O manifests. Exact preimages/diffs are under `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-p-editor59-pointer-owner-*`. |
| Record authorization | Optimize maintenance request `37e24788126144eebd852e71654cc9ea`; Astra maintenance request `bdbffe385426421691c72f013cb6ad53`. |
| Static verification | Scoped Rust formatting, diff/structure checks and inverse byte-preservation evidence are stored with the owned snapshots. Foreign Play keyboard and app test-binding changes remain preserved. |
| Independent review and managed behavior | Independent source review passed against the final v2 manifest, including all ten tests and six exact preservation checks. Grouped Windows Runtime/Editor library validation remains pending. No direct Cargo, compilation monitoring, commit, push, or external notification was performed by this implementation agent. |
| ED59-G06 | Button ownership is implemented for the existing default-pointer bridge. Capture generation, native multi-device identity, stale-generation terminal disposition and the full G06 acceptance remain open. |
| ED59-G31/G32/G33 and product gates | The repair adds no full render packet/mesh payload construction to pointer input and changes no motion coalescing. Native product evidence and 100k/1M selectable, 1 kHz pointer, large-selection, 4/16 viewport allocation and p95/p99 measurements remain pending. No new fixed millisecond threshold or performance pass is claimed. |

The production change adds one drag admission check and one preserved release branch; cancellation uses the existing Surface lifecycle. Actual allocation and tail-latency results depend on the integrated Runtime and Editor path.
