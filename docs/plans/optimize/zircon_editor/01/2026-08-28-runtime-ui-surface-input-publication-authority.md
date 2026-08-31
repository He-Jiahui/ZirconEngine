# Runtime multi-Surface input publication authority

- Date: 2026-08-28
- Status: `static_candidate_e9_with_pointer_affine_navigation_cutover`
- Owner boundary: `zircon_runtime::dynamic_api::session::RuntimeUiSurfaceSet`
- Lower authority: immutable per-Surface `UiSurfaceFrame`
- Product consumer: `zircon_editor`, without a second hit-test tree
- Dynamic evidence: not yet available; no managed Cargo authorization was used

## Decision

The remaining input latency is structural. `RuntimeUiSurfaceSet` currently uses an input event as the trigger to probe and potentially rebuild every Surface. Reducing one clone at the final consumer does not change the `O(S)` algorithm, and caching inside an individual handler cannot repair it.

The target is one rebuild-owned `RuntimeUiInputPublication`:

- pointer capture directly selects one Surface;
- uncaptured pointer input queries a persistent global cell-to-Surface directory, then asks only the true overlapping per-Surface frame grids;
- keyboard, text, and IME directly select the published focused-text Surface;
- navigation and analog input directly select the published navigation Surface;
- raw `MouseMotion`, whose Surface route policy is `Unrouted`, is rejected once at the SurfaceSet boundary;
- clipboard, accessibility, timers, and popup lifecycle keep explicit target/owner routing;
- no input path may scan all Surfaces, scan trees/render commands, or run a lazy rebuild.

## Current-source evidence

`zircon_runtime/src/dynamic_api/session/runtime_ui.rs` retains two reverse fanout loops as typed fallback/oracle paths, but the common pointer/focus/navigation/raw-motion classes now bypass them:

- `dispatch_input` rejects raw `MouseMotion` before Surface work, routes Keyboard/Text/IME to one retained active focus owner, and routes Navigation/Analog to one published navigation owner;
- non-captured `dispatch_pointer_input` first queries `RuntimeUiInputPublication`, then visits only the topmost cell candidates; the reverse stack walk remains only for the typed cold `Unpublished` state, while invalid coordinates or degenerate viewports are rejected in O(1);
- `input_event_for_surface` moves the event only into Surface index 0 and clones it for the other `S-1` consumers.

This is a hot product path, not a dormant compatibility path:

- `dynamic_api/session/events.rs` sends raw `UiInputEvent::MouseMotion` through generic dispatch;
- `events/keyboard_ime.rs` sends Text, Keyboard, and four IME states through it;
- `events/gamepad.rs` sends Navigation and Analog through it;
- pointer cursor/button/wheel input uses the separate pointer fanout.

`ui/surface/input/mouse_motion.rs` always returns unhandled and labels the route `Unrouted`. The SurfaceSet now rejects this typed event once and records `ui.surface_set.input.unrouted_reject_count`, without rebuilding a Surface or allocating per-Surface diagnostics. The dynamic session still records the raw delta for gameplay input before this UI boundary.

There is also duplicate synchronization in the legacy fallback. `RuntimeUiSurface::rebuild_dirty` calls `synchronize_text_document_owners`, then `UiInputManager::dispatch_input_event` calls it again. Published focus, navigation, and pointer candidates no longer call `rebuild_dirty` in the event chain and therefore synchronize once per actual candidate; an invalid/unpublished pointer fallback still pays the legacy duplicate probe. The synchronization is generation-aware, so it is not necessarily a full tree scan on a stable frame, but the repeated call/state probe remains proportional to that remaining fallback.

Current stable worst-case complexity is therefore:

- Keyboard/Text/IME: one retained owner validation and one Surface dispatch, with no event-time rebuild probe;
- raw `MouseMotion`: `O(1)` rejection;
- Navigation/Analog: one retained owner eligibility check and one Surface dispatch, with no event-time rebuild probe;
- uncaptured pointer with a valid publication: `O(1 + C + sum(K_i))`, where `C` is the true Surface-cell candidate count and `K_i` is each selected Surface cell's candidate work;
- captured pointer: one Surface lookup and the same published/affine query, with no event-time rebuild probe;
- invalid, degenerate, or never-published pointer state: explicit legacy reverse fallback, still `O(S + sum(rebuild_i))`;
- dirty geometry becomes input-visible only at the next complete publication; an event does not rebuild it.

Calling lazy `surface_frame()` from input would not solve the problem. It would hide rebuild/publication work inside a getter while keeping the first event after a mutation unbounded.

## Reference-engine evidence

Unreal Slate is the primary reference.

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp::LocateWindowUnderMouse` first narrows the candidate by OS/top-level window order, including child windows before parents.
- `LocateWidgetInWindow` filters the plausible window and queries `Window->GetHittestGrid().GetBubblePath(...)`.
- pointer capture replaces the hit path for subsequent pointer events.
- keyboard, character, and analog entry points begin with the per-user focus path rather than scanning all windows.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp::AddWidget` indexes final paint-space geometry/render bounds during paint and updates cell membership only when it changes.
- `GetBubblePath` queries one cell and builds the selected widget path at event time; it does not reconstruct the grid.

Fyrox is a semantic comparison only. `dev/Fyrox/fyrox-ui/src/lib.rs` retains `picked_node`, `captured_node`, and `keyboard_focus_node`, and keyboard input directly targets the focus owner. Its recursive `pick_node` pointer algorithm remains `O(N)` and is explicitly rejected as Zircon's performance target.

Slint provides the same owner lesson in Rust. `dev/slint/internal/core/window.rs` retains a `focus_item`, starts key processing at it, walks its parent chain, and restores focus after popup close. Its retained mouse state is useful, but it does not replace Slate's cell-grid authority.

The adopted rule is therefore: Unreal's window/Surface candidate directory plus final-geometry per-Surface grid, with direct focus/capture ownership. Fyrox and Slint only confirm owner semantics.

## Ownership model

`RuntimeUiSurfaceSet` exclusively owns `RuntimeUiInputPublication`. `UiSurfaceFrame` remains the complete per-Surface immutable layout/render/hit/focus publication. Editor submits window input and consumes dispatch results; renderer consumes render products. Neither may build another node-level input index.

The publication needs:

- viewport and Surface-stack generation;
- a stable `RuntimeUiSurfaceId` independent of temporary vector position;
- stable-id to current Surface slot/frame mapping;
- a persistent chunked global cell directory containing compact Surface IDs in topmost-first order;
- each Surface's last published hit-test generation and occupied-cell footprint;
- active focused-text owner;
- active navigation/analog owner;
- the existing per-pointer capture owner;
- typed full-fallback reason when an incremental patch cannot prove correctness.

The global directory stores only Surface IDs. It must not duplicate `UiHitTestEntry`, route nodes, arranged nodes, or render commands. Once a Surface is selected, its published `UiSurfaceFrame.hit_grid` remains authoritative for frame, clip, pointer policy, route, popup affine projection, popup stack, and internal z/paint order.

Topmost order is a separate Surface-stack key. It must not be encoded by adding arbitrary local `z_index` values to a global stride. Transparent/unhandled fallthrough is preserved, but only Surfaces with eligible membership in the queried global cell are candidates.

## Routing matrix

| Input | Publication-time authority | Event-time work |
|---|---|---|
| Captured pointer move/up/cancel | pointer-id capture map | direct one-Surface frame query/dispatch |
| Uncaptured pointer down/move/up/wheel | global cell-to-Surface directory | one cell lookup, then topmost candidate fallthrough |
| Keyboard/Text/IME | active focused-text Surface | direct owner route; no cross-Surface fallback |
| Navigation/Analog | active navigation Surface | direct owner route; topmost eligible root is selected at publication when no focus exists |
| Raw `MouseMotion` | typed `Unrouted` classification | constant rejection plus session counter, no Surface result allocation |
| Clipboard/Accessibility | existing explicit Surface/node target | direct target |
| Timer/Popup lifecycle | owner encoded when queued/opened | direct owner; no expiry-time stack search |

Removing cross-Surface fallback for focus input is intentional. A focused event belongs to one active focus domain, as in Slate. If legacy broadcast behavior is genuinely required for another event type, it must be represented as an explicit `Broadcast` class, not inherited accidentally by Keyboard or IME.

## Publication and invalidation

The explicit rebuild/frame-publication boundary updates input publication before the next input batch:

- hit-test domain generation change removes the Surface's old footprint memberships, adds the new footprint, and replaces only affected persistent cell chunks;
- focus generation change reselects active focused-text and navigation owners without touching pointer cells;
- Surface add/remove/reorder, viewport/cell-grid change, or missing old footprint permits a typed `O(U+M)` full directory rebuild;
- every full fallback reports reason, visited Surface count, and visited membership count;
- input-induced dirty state queues the next publication/redraw and never rebuilds inside the same event or a lazy getter;
- text document owner synchronization runs at most once for the actual target Surface until it can itself be moved to a proven publication boundary.

Here `U` is global cell count and `M` is Surface-cell membership count. A local patch is proportional to old/new footprint plus affected cell membership; it must not scan all base entries just to validate an overlay invariant.

The current dynamic-session FFI exposes one `handle_event` call at a time and has no input-batch prepare callback. The implementation must therefore use the last completely published frame as the geometry authority, matching retained UI paint/hit semantics, rather than inventing an event-time synchronization barrier. Focus and capture ownership changes caused by a dispatch update the SurfaceSet publication in constant work after that dispatch; layout/render/hit mutations become visible when `tick_frame`/`current_ui_submission` publishes the next complete frame.

A viewport resize must not force a full Surface rebuild per native resize event. `RuntimeUiInputPublication` retains the source viewport of the currently presented resize snapshot. While the native window presents that snapshot at another size, pointer lookup applies one bounded two-axis affine map from current physical coordinates to the published source coordinates. The `UiHitTestQuery.point` remains the original physical point, while `virtual_pointer.current/previous` carry the mapped hit coordinates through route diagnostics. The lower `UiInputManager` boundary must forward this query to the existing query-aware `UiSurface` pointer dispatch without mutating the owned input event. Final resize reflow/current UI submission atomically replaces the frame, directory, and source viewport generation. Degenerate/non-finite axes use a typed reject or clamped fallback and a counter, never a tree scan. If the host later adds a true batch-preparation ABI, it may move publication earlier, but it must not restore lazy rebuild inside pointer or generic dispatch.

## Complexity and memory model

Target stable complexity:

- pointer: `O(1 + C + sum(K_i))`;
- focused input: `O(1 + route depth)`;
- unrouted raw motion: `O(1)`;
- local publication patch: proportional to changed Surface footprint and affected memberships;
- stack/viewport full fallback: `O(U+M)`.

`C` may equal `S` only when all Surfaces genuinely expose eligible content at the same point. The canonical Editor fixture should normally keep `C` at one or two.

`tools/runtime_ui_surface_input_publication_pressure.py` is a deterministic operation-count model, not product timing. With 64 Surfaces and 100,000 events in each pointer/focused/navigation/unrouted category, with two true pointer candidates per cell:

| Structural counter | Legacy all-event fanout | Current pointer/focus/navigation/raw cutover | Publication target |
|---|---:|---:|---:|
| Surface dispatches | 25,600,000 | 400,000 | 400,000 |
| Event payload clones | 25,200,000 | 100,000 | 100,000 |
| Event-path rebuild probes | 25,600,000 | 0 | 0 |
| Text-owner synchronization calls | 51,200,000 | 400,000 | 400,000 |
| Dirty node-scale work on first routed event | 640,000 | 0; shifted to pre-input publication | 0; shifted to pre-input publication |

For the four modeled categories, the raw-motion boundary, retained
Keyboard/Text/IME and Navigation/Analog owners, and pointer directory remove
25,200,000 modeled Surface dispatches, 25,100,000 event clones, and all
25,600,000 event-path rebuild probes from the legacy baseline.
Current-to-target is 1.0x and baseline-to-target is 64x. Cold unpublished
fallback, route depth, and product timing are excluded, so neither ratio is a
CPU speedup claim.

At 1920x1080 with 64px cells there are 510 cells. Even if all 64 Surfaces occupy every cell, compact `u32` cell memberships, reverse footprints, and the 2,040-byte retained visit-stamp scratch require an estimated 263,164 payload bytes, `O(U+M)`, without duplicating hit entries or routes. This excludes Arc/container/allocator overhead, so implementation acceptance must measure real capacity and RSS. A per-cell allocation-heavy structure or copied hit grids is unacceptable even if event CPU falls.

Artifact: `E:\zircon-profiles\runtime-ui-surface-input-publication-20260901-r13.json`

SHA-256: `0E09FD6F22F06833B2FBB7080E85C392F4AC42C592617EEDDA9DFBF1AE7264FD`

- schema: `zircon.runtime.ui_surface_input_publication_pressure.v11`
- bound revision: `f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`
- critical source-set SHA-256:
  `18FC9D1B746A54679AEB325B875C32E301E356DF294791FF426FA6937A090B13`
- critical sources: 10; dirty source entries: 10

For one warm publication with 64 dirty Surfaces, the previous footprint builder
allocated 64 cell-sized boolean occupancy arrays and 64 replacement footprint
buffers. The retained stamp scratch removes those allocations and 32,640 bytes
of transient boolean storage in the deterministic fixture; each Surface reuses
its existing footprint capacity when the new footprint fits. This is an
allocation-count model, not measured allocator latency.

The v11 model also binds the publication patch to the stamp-based uniqueness
invariant: the consumer-independent `footprint.sort_unstable()` pass was
removed. The default warm patch therefore removes 64 sort invocations and
32,640 sorted footprint items while retaining sorted per-cell Surface
candidates through the existing binary-search insertion path. This is a
deterministic work count, not a measured resize latency claim.

The same warm fixture previously allocated one temporary cell-index vector for
each of 640,000 visited hit entries. Publication now visits each bounded cell
through a stack-only callback, so the model reports 640,000 removed temporary
vector allocations and zero current per-entry cell vectors. The visited entry
and cell-membership work is unchanged.

The model now fails closed if the raw-motion or focused-owner path moves after
generic fanout, pointer publication moves after its legacy fallback, typed
invalid input rejection moves after that fallback, affine
physical/virtual mapping disappears, the query-aware manager/pointer/Surface
chain is bypassed, either fallback fanout disappears, or synchronization
ownership drifts. This prevents the current ratio from being reused after the
implementation shape changes.

## Test-first implementation plan

### M0: lower RED

Add `zircon_runtime` fixtures for:

- topmost hit and transparent fallthrough;
- no global-cell candidate produces zero Surface dispatches;
- capture directly routes follow-up events;
- popup projected geometry stays topmost and the old placeholder cannot hit;
- Keyboard/Text/IME directly target the active focus Surface;
- Navigation/Analog directly target the published navigation owner;
- pointer focus change updates the active focus Surface;
- Surface add/remove/reorder and viewport change invalidate the directory correctly;
- one hit-generation change patches only that Surface's memberships;
- missing footprint/reindex uses a typed full fallback;
- an event after dirty publication sees the new frame while its event-path rebuild counter remains zero;
- unpublished input retains an explicit cold compatibility fallback while non-finite pointer and degenerate viewport input produce zero Surface dispatch/rebuild;
- raw motion produces no Surface dispatch and no per-Surface diagnostic allocation.

Raw motion, focused/navigation-owner routing, pointer-cell publication, single-Surface
generation patching, affine resize lookup, physical-pointer preservation, typed
unpublished-versus-invalid admission, and
non-candidate dirty-Surface rejection now have lower Rust regressions plus
source-order contracts. They remain pending managed Cargo execution;
dynamic Surface mutation and product-level stress remain part of the full
cutover. The current `RuntimeUiSurfaceSet` builds its immutable Surface vector
once, so numeric index is a stable identity for the existing lifecycle. A
separate stable ID becomes mandatory before any add/remove/reorder API is
introduced; it is not an immediate event-path performance blocker.

The legacy route remains a test oracle during M1. Compare selected Surface sequence, reply, focus/capture/action requests, route diagnostics, and original physical virtual pointer.

### M1: publication authority

Keep the immutable generation snapshot, persistent cell directory, reverse footprints, retained patch scratch, and focus/navigation owners under the SurfaceSet publication boundary. Update them only from explicit SurfaceSet rebuild/render publication. Add a stable Surface ID before, and only before, introducing dynamic Surface add/remove/reorder. Do not add event call-site special cases.

### M2: hard cut

Dispatch by typed route class. Pointer uses capture/directory; focus classes use direct owners; unrouted input returns at the SurfaceSet boundary. Delete `input_event_for_surface` and all event-call-chain `rebuild_dirty` calls. Preserve only explicitly typed broadcast events if product semantics require them.

### M3: managed validation

Run lower focused Runtime tests first, then Runtime integration and Editor product-path regressions. Cargo must use official managed validation only. No Cargo was authorized or run for this record.

### M4: product stress and profile

For 1/4/16/64 Surfaces, run at least:

- 100,000 pointer moves;
- 10,000 click/wheel events;
- 10,000 Keyboard/Text/IME/Navigation events;
- 200 resize steps;
- candidate overlap `C=1/2/8/S`;
- stable, local-dirty, and dirty-first-input scenarios.

Collect CPU, allocator bytes/count, RSS, input-to-damage p50/p95/p99, input-to-present p50/p95/p99, event-path rebuild count, candidate count, and publication patch work. Model output cannot close this gate.

## Required instrumentation and budgets

Add counters:

- `ui.surface_set.input.event_count`;
- `ui.surface_set.input.directory_query_count`;
- `candidate_surface_count`;
- `dispatched_surface_count`;
- `event_clone_count`;
- `event_rebuild_count`;
- `text_owner_sync_count`;
- `capture_direct_route_count`;
- `focus_direct_route_count`;
- `unrouted_reject_count`;
- `tree_scan_count` and `render_command_scan_count`;
- `warm_path_allocation_count`;
- per-event `input_to_damage_us` and `input_to_present_us`;
- `publication_patch_surface_count`;
- `publication_cell_membership_count`;
- `publication_full_rebuild_count` and typed reason.

Acceptance budgets:

- stable event rebuild/tree/render scan count is zero;
- focus-class input dispatches to zero or one Surface exactly;
- canonical pointer candidate P95 is at most two;
- warm path allocates nothing except a necessary payload for true multi-candidate fallthrough;
- existing Editor input-to-damage P95 remains at most 1 ms;
- existing input-to-present P95 remains at most 9 ms;
- 1-to-64 Surface scaling regresses focused P95 by at most 5 percent;
- `C=1` pointer P95 regresses by at most 10 percent;
- real publication memory/capacity and RSS are reported, not inferred from payload estimates.

## Implemented dynamic evidence gate

`tools/runtime_ui_surface_input_evidence.py` now turns the routing and scaling
budgets into a fail-closed offline validator without changing the externally
owned production path. A run must identify exactly one of four route classes:
uncaptured pointer, captured pointer, focused input, or raw unrouted input.
Aggregate totals alone are insufficient. Every measured event must provide one
candidate, dispatch, clone, rebuild, text-owner sync, warm-allocation,
input-to-damage, and input-to-present sample. The validator then proves route
conservation per event, computes nearest-rank candidate and latency P95, rejects
event-time publication/tree/render scans, and enforces 1 ms input-to-damage and
9 ms input-to-present budgets.

The scaling validator requires focused and canonical `C=1` uncaptured-pointer
runs for 1/4/16/64 Surfaces. Every focused run must remain within 5 percent of
the one-Surface P95; every pointer run must remain within 10 percent. Missing
scale members are typed failures rather than omitted data. Source acceptance
requires measured-run manifests with valid fingerprints for `runtime_ui.rs`,
`frame_hit_test.rs`, and `event_routing.rs`; the analyzer is bound separately
by its own SHA-256 so it does not depend on capture-manifest self-inclusion.

The initial absent-analyzer suite produced 11 expected RED failures; the later
report-builder/tool-binding cases independently produced two RED errors, and
the candidate-bound/schema cases produced two more focused RED failures. The
complete focused suite is now 16/16 GREEN. Analyzer and test SHA-256 values are
`EC06A753B4CF694FA9E6200D94620C7357B1C118E0E329FD4885C149439270E7`
and `A28C93CF73B7603BE3EBA4BF954B2FE4E9A97BC5BCE1D8D305B9F2328AC9290E`.
Python compilation and scoped diff checks pass.

The historical product timeline
`20260811-201002-click-dock-patch-spaced/timeline.zrtrace.json` is correctly
rejected with 17 `missing_counter` blockers and one
`missing_source_manifest` blocker. The diagnostic artifact is
`E:/zircon-profiles/runtime-ui-surface-input-evidence-20260829-historical-regression.json`
with SHA-256
`31C30458127DCB764E4362B55171FB705F48BA082E943B82518A15ED141321A2`.
This proves that old aggregate evidence cannot false-green the target; it does
not prove the publication algorithm exists or meets product latency budgets.

## Current status

The source-bound pressure suite is 10/10 green, the design contract suite is
11/11 green, and the dynamic evidence validator remains 16/16 green (37/37
combined). Python compilation, Rust static formatting, and scoped
`git diff --check` are green. The lower Rust module now includes five focused
regressions, including footprint/stamp-capacity reuse, but Cargo execution is
deferred in this slice while current-source blockers are handled elsewhere.

`RuntimeUiSurfaceSet` now owns direct raw-motion rejection, focus/navigation
owners, the incremental pointer directory, affine resize queries, and typed
invalid-input admission. The publication patch path additionally retains one
cell-stamp array and recycles per-Surface footprint buffers. The cold
unpublished compatibility fanout and generic broadcast fallback remain by
explicit policy. Current-source Editor CPU/RSS/p50/p95/p99 evidence is still
missing, so this record does not claim the complete mouse or resize latency
milestone is product-accepted.
