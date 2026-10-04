---
title: Editor scene-layer damage and paint-state performance review
date: 2026-08-22
module: zircon_editor retained-host paint_workbench_renderer scene_layers
priority: MVP-P0 workbench composition, floating panes and modal overlays
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate window paint context and invalidation root
---

# Goal

Make the workbench scene owner route exact layer damage before it prepares paint state, traverses
template ranges or emits overlay commands. Stable chrome, docks, floating panes and modal surfaces
must converge on generation-qualified retained layer ranges rather than reconstructing an immediate
scene in a fixed full-frame fan-out.

## Reviewed source

- Rust files: 10/10
- lines: 1,594
- bytes: 52,337
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `57d8ea7938a0983468f89289c7a8703f3adfefd719ee878b03965d353d06c4a7`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Scope: `paint_workbench_renderer/scene_layers.rs` and
`paint_workbench_renderer/scene_layers/**`, including production and inline/path tests.

Supporting behavior owners traced/read: paint override generation, floating-window paint and bounds,
menu popup paint and close-prompt paint. The earlier menu, dock-pane, template-node and native-pane
reviews remain the detailed owners of their leaf algorithms.

## Correct foundations to retain

1. Top/status chrome, dock regions, splitters and individual floating windows already use exact
   damage intersection before their expensive leaf paint.
2. The componentized extension workspace resolves a typed retained hit index and rejects damage
   outside its workspace before subtree construction and text-focus preparation.
3. Extension workspace traversal uses indexed subtree paint rows when the committed hit index matches;
   its legacy graph walk is a fallback, not the desired steady-state route.
4. Page overflow already consumes the shared visible-row range, so open-menu row paint is O(V), not
   O(N), after the menu milestone.
5. Root-template, menu and prompt layers first test their semantic visibility/open state.

## Structural findings

### P0: the scene root is an immediate fixed fan-out, not a retained invalidation owner

`draw_host_scene` and its componentized counterpart invoke every layer in fixed order on every paint.
Leaf functions recover visibility, damage, model range and state independently. This preserves visual
order but cannot reuse a stable chrome/dock/overlay command range or patch only the invalid layer.

M2-M4 introduce a scene paint plan keyed by layout/style/content/interaction generations. It owns
ordered layer bounds, retained command ranges and compact dirty masks. Paint visits only intersecting
dirty ranges; stable ranges are reused without re-running their Rust builders.

### P0: floating-layer state is prepared before proving that a window will paint

The scene wrapper materializes pane interaction, viewport-image and text-focus paint snapshots before
`docks::draw_floating_layer` checks an empty model or per-window damage. With zero floating windows or
damage outside all floating paint bounds, state preparations are `3 -> 0` in M1 by moving lazy
preparation behind the first accepted floating window. The window loop remains one pass and O(W);
the change must not add a preliminary scan.

### P0: componentized chrome repeats state preparation and range queries

Top chrome and status chrome call the same workbench-node painter separately and each prepares a text
focus snapshot. The two clips are semantically disjoint, but there is no owner damage route before
those preparations. M1 derives the two damage bits once and shares one text-focus snapshot across the
active clips: preparations `2 -> 1` when both paint and `2 -> 0` when neither paints. M3 replaces the
two spatial queries with generation-owned chrome command ranges; merging them into one broad clip is
rejected because it could repaint the center workspace and change composition.

### P0: page-overflow work continues for damage outside its popup

Once open, the page-overflow painter builds palette/metrics, emits popup primitives, walks visible
rows and prepares text even when the frame damage does not intersect the popup. Primitive clipping is
too late. M1 adds an exact popup damage gate before those operations: visible-row/text work `V -> 0`
for off-popup damage.

### P1: menu, prompt and root-template overlays do not share one typed layer route

The menu popup and close prompt have semantic early exits, but routing is distributed across leaf
functions and the prompt overlay intentionally covers the full frame. M2 publishes typed overlay
bounds/opacity/order in the scene plan. A visible full-screen modal remains a full-frame invalidation;
it must not be incorrectly reduced to the dialog bounds.

### P1: legacy componentized workspace discovery can rebuild a graph index in paint

Without a matching committed hit index, workspace discovery creates `HashMap`/`HashSet` structures and
walks parent chains. That path is useful for tests and transitional data, but not acceptable as a
steady-state fallback. M4 makes the indexed workspace descriptor mandatory for componentized paint and
keeps graph reconstruction outside the frame path.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`

`SWindow::PaintWindow` creates one `FPaintArgs` and `FSlateInvalidationContext`, supplies the culling
rectangle, and delegates the window to its invalidation root. `PaintInvalidationRoot` explicitly
chooses slow-path reconstruction or fast-path cached updates and reports whether widgets repainted.
Widget proxies carry persistent update flags/state for targeted repaint.

The transferable constraint is one window/scene paint context plus invalidation-owned retained ranges;
Zircon should not copy Unreal pointer ownership or layer-id repair mechanically. Exact Zircon damage,
text/image resources and current-backend captures remain the acceptance authority.

## Target architecture

1. Presentation generation publishes an ordered `HostScenePaintPlan`: typed layer id, bounds,
   generation, opacity/occlusion, retained command range and dirty reason.
2. One lazy paint context owns interaction, text-focus, viewport-image, theme/metrics and hit-index
   snapshots for all accepted layers in a frame.
3. Damage routing happens at the scene owner; leaves receive an already accepted layer/range and do not
   rediscover global state.
4. Top/status chrome and extension workspace consume disjoint indexed ranges from one workbench-node
   generation, with no broad double traversal.
5. Floating windows and overlays retain per-surface command ranges and exact shadow/modal bounds.
6. The renderer reuses stable command ranges and patches interaction/text/image instance data only for
   changed generations, preserving the current explicit back-to-front order.

## Instrumentation and acceptance

Matrix: workbench nodes `0/1/1k/10k/100k`, floating windows `0/1/16/128`, overlay
`none/menu/page-overflow/prompt/root-template`, damage `outside/one layer/cross-layer/full`, chrome
`top/status/both`, state `stable/hover/focus/image`, componentized `indexed/legacy-rejected`.

| Evidence | Acceptance |
| --- | --- |
| layer visits and damage rejects | only intersecting dirty layers visit builders |
| state preparations by kind | at most one lazy snapshot per accepted frame context |
| template spatial queries/visited rows | proportional to accepted layer ranges |
| floating logical/visited windows | one pass; zero state work when none accepted |
| command/range rebuilt and reused bytes | proportional to dirty generations |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/GPU and pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add layer/state/query/window/range rebuild-reuse counters and capture baseline. | attributable baseline |
| M1 | Gate componentized chrome/page overflow by damage; share chrome focus; lazily prepare floating state. | static counts above; parity contracts |
| M2 | Publish typed scene/overlay layer bounds, dirty reasons and one lazy paint context. | one owner route/context |
| M3 | Retain ordered chrome/dock/floating/overlay command ranges and patch compact state. | rebuild proportional to dirty ranges |
| M4 | Require indexed componentized workspace/chrome ranges; remove frame-path graph fallback. | no frame hash/parent reconstruction |
| M5 | Hard-cut immediate duplicate scene fan-out after all consumers migrate. | one composition authority |
| M6 | Run managed scale/input/WPR/power and RenderDoc/pixel/text parity matrix. | quantified accepted milestone |

## M1 implementation result

`docks/floating_windows.rs` now owns lazy pane paint-state preparation. Its existing single window
loop performs visibility/damage rejection first and initializes interaction, viewport-image and
text-focus snapshots only at the first accepted window. Later accepted windows reuse the same tuple.
There is no preliminary bounds scan and no per-window state generation.

The componentized chrome owner now resolves explicit top/status damage bits. It returns before text
focus or template-range work when neither clip intersects, and shares one text-focus snapshot when
both clips paint. Page overflow now rejects damage outside the resolved popup before theme, metrics,
visible-row and text work.

| Static frame-path work | Before | After | Change |
| --- | ---: | ---: | ---: |
| floating state preparations, `W=0` or no accepted damage | 3 | 0 | eliminated |
| floating state preparations, one or more accepted windows | 3 | 3 once | no per-window growth |
| floating window traversal | one O(W) pass | one O(W) pass | preserved |
| componentized chrome focus snapshots, both clips | 2 | 1 | -50% |
| componentized chrome focus/range work, neither clip | 2 snapshots + 2 queries | 0 | eliminated |
| page-overflow visible-row/text work, off-popup damage | V | 0 | eliminated |

These are source-path counts, not elapsed-time or power claims. M0 and M2-M6 remain necessary because
stable accepted layers still reconstruct immediate commands and top/status chrome still issue
separate indexed range queries.

Post-M1 direct owner scope:

- Rust files: 10/10
- lines: 1,629
- bytes: 53,202
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `2e7a45ca5d30e5d70fe833092e3b206c793497219758c1118f9b3f39fba85d61`
- unchanged direct owner files: 7 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `scene_layers/dock_layer.rs` | 129 | 4,330 | `03af1e718bd2c5584ab0f186f150551e5e650ede684743911b380327c5a63c07` |
| `scene_layers/overlay/componentized.rs` | 407 | 13,518 | `43f4ae3a827961cf4e3bdf2f7a2a63319da2c904947f733ea2f9a1c109b9106a` |
| `scene_layers/overlay/page_overflow.rs` | 294 | 10,228 | `8a48d948ab364015968b9d7abab9924317387ae3ad1323fc9c4f610c944261d5` |

Supporting floating owner:
`docks/floating_windows.rs`, 39 lines, 1,371 bytes, SHA256
`5dc1e0c071d9d2b986346d06ec1098975d80194dff043737105705c4f199f23c`.

Focused static contract:
`tools/tests/test_editor_scene_layer_damage_state_performance_contract.py`, 49 lines, 2,376 bytes,
SHA256 `2b7de4b6a6f85f8153f6943f415ce87953c173678aa48fb967852f1b9a88c4f8`.

## Validation state

- Full direct owner review: passed, 10/10 Rust files.
- Supporting state, floating bounds, menu and prompt behavior paths: traced/read.
- Relevant Unreal sources above: read and mapped to scene-context/invalidation constraints.
- M1 focused contract: RED 3/3 before implementation, GREEN 3/3 after implementation.
- Current owned editor performance-contract set: GREEN 74/74.
- Broad editor performance-contract set: 101/106 passed; its five failures are the unchanged known
  missing `component_showcase_state.rs`, missing `workbench_projection.rs`, missing `available_slots`,
  preview resize `.roots.clone()` and UI asset root dirty-helper `.roots.clone()` findings.
- `rustfmt --check` for the four changed Rust files and scoped `git diff --check`: passed.
- Existing floating-window bounds, componentized workspace/chrome and page-overflow Rust behavior tests
  remain present, but are not claimed passing until managed Cargo is executable.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

The module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
