---
related_code:
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/handle_click.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  - tools/editor_viewport_toolbar_pointer_surface_reuse_pressure.py
  - tools/tests/test_editor_viewport_toolbar_pointer_surface_reuse.py
status: static_candidate_e1
product_timing: false
evidence_artifact: E:/zircon-profiles/editor-viewport-toolbar-pointer-surface-reuse-pressure-20260830-r1.json
evidence_sha256: 12C7EEEA2A3E096B141345AD78D9A16089FAFF9CE4960E2A8848F74F7CFA866C
source_revision: cc5cadbd597c3707954ebd6109fad0fd5643a152
source_set_sha256: 2C4A27788C3F9F95A053A1B51188023FA5D8CEFAB3FED3028A0C4E1BE28898DA
---

# Viewport Toolbar pointer Surface reuse

## Finding

The Viewport Toolbar pointer bridge already retains projected controls and skips
stable `UiSurfaceFrame` identities. Its last publication boundary still defeats
that retention: `rebuild_surface()` constructs a new `UiSurface`, pointer
dispatcher, route map, root, every surface node, and every control node, then
calls full `surface.rebuild()`.

This path is reached after a click supplies a previously unknown or changed
control frame, after a projected frame changes the control set, and after a
toolbar layout/topology change. A one-control frame correction therefore pays
the same reconstruction cost as a control add/remove or action-route change.

The clean implementation owner is
`viewport_toolbar_pointer/rebuild_surface.rs`. Its current callers and bridge
state files have external worktree changes, so this slice must remain confined
to that one production file plus new evidence files.

## Required authority

Node IDs already encode surface/control indices. A retained patch is valid only
when all of the following remain equal:

1. root ID and ordered surface-node IDs;
2. ordered control-node IDs per surface;
3. parent/child topology and total node count;
4. control action path and route ID for each stable node;
5. fixed input policy and z-order contract.

When those identities match, the existing `UiSurface`, dispatcher, and route
map remain authoritative. Only unequal explicit frames are written through
`UiTreeNodes::get_mut`. Because these frames are computed outside the Runtime
layout engine, they are published through `UiSurface::rebuild_authored_frames`;
using layout dirty plus `rebuild_dirty` would incorrectly reinterpret them as
layout-owned constraints.

Any mismatch is a typed structural fallback to the existing from-scratch
construction. The retained path must never reinterpret an action change as a
frame-only mutation.

This follows the Unreal Slate invalidation boundary reviewed under
`dev/UnrealEngine/Engine/Source/Runtime/SlateCore`: stable widget identity and
paint/hit products are retained, while structural invalidation rebuilds the
affected ownership boundary. The relevant lesson is product ownership, not API
surface imitation.

## Pressure model

The deterministic fixture uses 16 toolbar surfaces, 32 controls per surface,
1,000 frame-only updates, and 10 topology changes. There are 529 nodes and 512
routes in one full pointer Surface:

| work | current reconstruction | retained target |
| --- | ---: | ---: |
| Surface object reconstructions | 1,010 | 10 |
| authored-frame full pipeline rebuilds | 1,010 | 1,010 |
| full pipeline node-visit pressure | 534,290 | 534,290 |
| retained frame patches | 0 | 1,000 |
| node allocations | 534,290 | 5,290 |
| route materializations | 517,120 | 5,120 |
| dispatcher registrations | 517,120 | 5,120 |

The retained target still performs an identity/topology validation walk before
patching. This slice removes 1,000 Surface object reconstructions, 529,000 node
allocations, and 512,000 route materializations/dispatcher registrations. It
does **not** remove authored-frame arranged/hit/render full publication work,
does not claim constant-time lookup, and is not measured CPU latency.

The remaining structural requirement is a Runtime-owned authored-geometry
patch API that can update arranged geometry, hit cells, render geometry, and
the published `UiSurfaceFrame` from an exact changed-node set. The existing
`rebuild_dirty` geometry path is layout-owned and cannot safely serve this
external-frame bridge without changing frame authority.

## Implementation order

1. Completed: source-bound RED contract for retained patch versus typed full
   fallback.
2. Completed: split `rebuild_surface()` without changing externally dirty
   callers or bridge fields.
3. Completed: validate topology before the first mutation.
4. Completed: patch only unequal frames while retaining node, route, dispatcher,
   hover, and focus ownership.
5. Pending Runtime slice: add exact-node authored-geometry publication so frame
   patches stop paying full arranged/hit/render publication.
6. Pending managed validation: add lower Rust regressions for frame-only reuse
   and action/topology fallback when the Cargo lane is explicitly authorized.
7. Accept product behavior only with source-bound counters and toolbar-click
   input-to-present p50/p95/p99, allocation bytes, and layout/hit/render visit
   counts.

## Static verification

- `python -m unittest tools.tests.test_editor_viewport_toolbar_pointer_surface_reuse -v`:
  6/6 passed.
- Related toolbar generation, SVG/GPU, and renderer dependency evidence suite:
  92/92 passed.
- `python -m py_compile` for the pressure tool and its test: passed.
- `rustfmt --edition 2021 --check` for the production owner: passed.
- `git diff --check` for the four candidate paths: passed; Git emitted only the
  repository's existing LF/CRLF conversion warning.
- No Cargo command was run. Rust compile/product acceptance remains pending the
  managed validation lane.

## Non-goals

- no new Editor-side cache;
- no changes to pointer routing semantics or action ownership;
- no modification of externally dirty caller/bridge files;
- no claim that the deterministic model is product timing;
- no raw Cargo invocation.
