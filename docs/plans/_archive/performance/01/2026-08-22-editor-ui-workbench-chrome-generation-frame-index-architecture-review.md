---
title: Editor workbench chrome generation-owned frame index performance review
date: 2026-08-22
module: zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection
priority: MVP-P0 editor chrome, tabs, docks and status
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate persistent tab wells and invalidation roots
---

# Goal

Publish one immutable chrome projection artifact per accepted template, resource, text, model and
geometry generation. The artifact must own projected nodes plus a typed control-frame index so page,
dock, menu and activity consumers never rediscover the same geometry with per-control full scans.
Stable chrome must perform zero projection, image loading, text measurement and frame indexing.

## Reviewed source

- module files: root plus `activity_rail`, `dock_header`, side dock header, `menu_chrome`,
  `page_tabs`, `status_bar` and module tests
- Rust files: 8/8
- current lines: 3,019
- current bytes: 104,248
- joined current source-bytes SHA256:
  `4571c6a561a467e94e775b4c6eb7831b4537fd73673e48fd197c26138a3e2652`
- joined pre-change source-bytes SHA256:
  `6edc9345208fe9484573c8bd2a37d7c8aa247019e23d770b3fd73bbed96cd287`
- owning commit before review: `bee4c707b714738346b49bba15c59468b8bd9b39`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `chrome_template_projection.rs` | 885 | `8572c0f535dd0a2d11f89f6326638cfe0e8626102ef3cc87ca9fd87e16af7f96` |
| `chrome_template_projection/activity_rail.rs` | 202 | `185f636432bfaa2abd368177c5087dfa4c6d4cb9bc6ee16455f6f73f575de04a` |
| `chrome_template_projection/dock_header.rs` | 196 | `9117515167accc02193990103a3ac57c4c48fb71ebf027177c9fdbc51ef8b667` |
| `chrome_template_projection/dock_header/side.rs` | 259 | `c195ba1f3b4a74706cf04c14aeff0513e0f7d5ea8a6576dd79855b5934d613e2` |
| `chrome_template_projection/menu_chrome.rs` | 365 | `255c1132224f5ead23946f08287b16cd83463dce22ee46eb67a50fb887e60b39` |
| `chrome_template_projection/page_tabs.rs` | 29 | `446a4af2ad13cab16214421bc41c6fde800bd5b903b9c97c1beed04f8368f16d` |
| `chrome_template_projection/status_bar.rs` | 44 | `d04a4d0048a986796d49c25844428b0db7a3c7754229108cc30877d036624609` |
| `chrome_template_projection/tests.rs` | 1,039 | `aed720bfa5f671a64a951e4bc4b031baf7f36f2236f0822620439f372448f925` |

All files were read in full. Production callers were followed through `scene_projection.rs` and
`scene_projection/dock_patch.rs`. `ModelRc::{row_data,get,iter}` ownership behavior and the upstream
host projection caches were checked. The test module is counted as one reviewed script; its size is
already above the repository modularization threshold, so new performance contracts must not extend
that monolith.

## Existing foundations to retain

Page chrome, activity rails, menus and side dock headers already have identity-based retained
projection caches. Stable `ModelRc` inputs can return shared node owners, side dock caching is bounded
to twelve entries, and tests assert shared model identity. Fallback layouts preserve readable tab
widths, active-tab visibility, close hit targets and runtime text metrics. These are useful retained
foundations; the problem is the post-projection rediscovery and incomplete generation contract.

## Structural findings

### P0: frame derivation scales as repeated tab-by-node scans

`control_frame` scans all N projected nodes for one control ID. It calls `row_data`, cloning every
visited `ViewTemplateNodeData`, then unions matching primitives. `control_frames` repeats this for
each activity/menu slot and `tab_frames` repeats it for tab and close controls. Page overflow repeats
another full scan for every tab through `has_control_frame`.

The page scene separately asks for bar, overflow, hidden-tab indices, project path and tab/close
frames. Dock scenes separately ask for header, subtitle and tab/close frames. With T tabs this is
approximately O(T*N), including wide node clones and formatted IDs on the editor main thread.

### P0: projected nodes and derived geometry have different ownership generations

The projection cache can reuse a stable node model, but frame DTOs are rebuilt by later callers.
There is no artifact that binds `{nodes, frame_index, template/resource/text/model/geometry receipt}`.
Consequently a shared node generation still pays repeated geometry discovery and output-model
materialization in each scene assembly. Adding another thread-local cache would hide, not fix, this
split authority and would make resource/text invalidation harder to prove.

### P1: tab composition clones rows and nodes before retained reuse

`tab_text_overrides`, fallback tab layout, active-row discovery, close filtering and tab state
composition repeatedly use `row_data` even when only borrowed fields are read. `tab_node_with_state`
fetches the same tab twice. Composition replaces each mutable node with a transformation of
`node.clone()`, copying the complete node before assigning it back. Icon-bearing nodes also format a
media path and call `load_preview_image` on every composition rebuild.

This work is bounded by O(N+T), but its constants are substantial because tab and node records contain
owned strings, patches and image handles. M1 should remove provably unnecessary row clones. Persistent
resource-generation icon resolution belongs to M2.

### P1: side-header cache lacks an explicit resource/text-metric receipt

The side dock cache key uses tab model identity and exact width/height bits. That correctly avoids
steady-state layout work, but it does not declare icon-resource or text-metric generation. The current
cache can therefore be fast while still having an incomplete correctness invalidation contract.
M2 must use an explicit generation receipt rather than growing more ad hoc key fields.

### P1: chrome segments do not share a common generation protocol

Menu chrome is retained at the host cache boundary, while status text rebuilds owned override strings
and patch maps on each full scene path. Activity rail, page and dock caches use different private key
shapes. This prevents one measurable rule such as “unchanged segment builds = 0” across the workbench
and allows status-only or render-only invalidation to recreate unrelated chrome products.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`

`SDockingTabWell` owns a persistent `TSlotlessChildren<SDockTab>` collection and mutates it only when
tabs open, insert or close (`40-96`). `OnArrangeChildren` traverses the live tab children once and
emits arranged geometry directly (`100-180`); it does not scan an unrelated flattened node table for
each tab ID. `TabManager` first reuses a live tab and spawns only when reuse/restoration is impossible
(`1711-1727`, `1766-1823`).

Each Unreal `SWindow` owns a persistent invalidation root and hit-test grid (`SWindow.cpp:2077-2080`).
Paint delegates to that invalidation root (`2120-2149`). `FSlateInvalidationRoot` keeps cached element
data and uses the fast widget path unless policy/root invalidation requires the slow path; an empty
update list performs no widget update (`SlateInvalidationRoot.cpp:356-424`). The transferable design
is persistent live chrome plus generation-owned arranged geometry, not repeated string lookup through
a reconstructed flat projection.

## Target architecture

1. Introduce `ChromeProjectionArtifact { nodes, frames, receipt }`. `frames` unions primitives once by
   control ID and exposes typed page, dock, menu and activity views without another node traversal.
2. Build the frame index in the same transaction as template projection. Its receipt includes exact
   template, resource, text-metric, model and geometry generations. No paint-only state participates.
3. Derive tab, close, overflow, bar, subtitle and activity frame models from one index traversal and
   borrowed tab rows. Stable consumers clone shared owners only.
4. Make page, dock, activity, menu and status caches implement one segment-generation protocol.
   Status-only changes replace only status; selection-only changes patch exact tab state.
5. Resolve chrome icons through a resource-generation cache and attach immutable image handles during
   composition. A stable icon must not format paths, decode, load or probe image size again.
6. Hard-cut over scene callers to artifacts, then delete raw-node frame-query entry points and private
   cache shapes that permit incomplete invalidation.

Complexity targets:

- changed chrome generation: O(N+T), one node traversal and one tab traversal;
- unchanged generation: O(1) receipt comparison, zero projection/index/image/text work;
- frame lookup: O(1) expected or O(log N) worst-case, with zero node clones;
- selection-only patch: O(changed tabs), not full template composition;
- scene assembly: shared artifact-owner clones only.

## M1 result

`ControlFrameIndex` now borrows control IDs from one node generation, unions duplicate primitives once
and preserves the old positive-width overflow predicate separately from valid hit geometry. Each
`control_frames`, `tab_frames` or overflow-hidden derivation creates one local index. Single-control
queries traverse borrowed nodes and no longer clone `ViewTemplateNodeData`.

For C controls, `control_frames` changes from C*N node visits and C*N wide node clones to N borrowed
visits plus C indexed lookups. For T closeable tab rows, `tab_frames` changes from 2T*N visits/clones
to N borrowed visits plus 2T lookups. Overflow discovery changes from T*N visits/clones to N borrowed
visits plus T lookups. The complete page post-projection query set is now a fixed five node traversals
plus indexed tab lookups instead of approximately `(3 + 3T)*N` cloning visits. Dock header/subtitle/tab
derivation is a fixed three traversals plus indexed tab lookups instead of approximately
`(2 + 2T)*N` cloning visits.

All production read-only `row_data` calls in the eight-file module were replaced by `get`/`iter`; one
`#[cfg(test)] model_nodes` ownership helper remains. Final `HostChromeTabData` still clones exactly one
`TabData` per owned output row. Tab composition uses `mem::take`, removing one complete node clone per
composed node. Menu gap discovery no longer allocates a temporary vector of all templates.

This is an intermediate O(k*N log N + T log N) implementation, where k is a small fixed caller-group
count. M2 must build and retain the index with projection to reach one O(N+T) transaction and zero
unchanged work.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| node visits per page/dock/activity frame generation | O(N), never T*N |
| cloned `ViewTemplateNodeData` during frame queries | 0 |
| cloned `TabData` | final owned output only |
| projection/index/icon/text builds by receipt | unchanged = 0 |
| control-ID formats and frame lookups | O(T), no per-lookup node scan |
| status/menu/page/dock/activity segment builds | changed segment only |
| main-thread CPU, allocations and input-to-paint latency | report median/p95/max |

Matrix: tabs 0/1/16/1,000; nodes 10/1,000/10,000; duplicate primitives 0/2/16; fixed and side
docks; menu 0/7/64; stable refreshes 1/1,000; selection, close, resize, resource reload, text-metric,
status-only and render-only invalidations. Capture node visits, frame-index builds, row/node clones,
allocations, main-thread CPU, input-to-paint latency, RSS and package energy on one source/executable
fingerprint.

Use managed Windows validation and WPR/ETW with all targets and traces on D/E/F. RenderDoc is only for
current-source GPU draw-call and pixel parity after a launchable editor exists; it cannot validate CPU
node scans or ownership churn.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add frame-query, node-visit, clone, projection, image and segment counters; capture baseline. | source-bound profile and scale curves |
| M1 | Add module-local frame index for multi-control queries and borrow read-only model rows. | RED-to-GREEN contract, behavior parity, static complexity bound |
| M2 | Publish generation-owned chrome artifacts and resource/text/model/geometry receipts. | unchanged work = 0; changed build O(N+T) |
| M3 | Convert selection/close/status changes to narrow immutable patches. | unrelated segment builds = 0 |
| M4 | Delete raw-node multi-query and incomplete private cache paths. | one authority, no compatibility scan path |
| M5 | Run scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full module source review: passed, 8/8 Rust files.
- Production scene callers, `ModelRc` ownership behavior and Unreal reference functions: read.
- M1 source implementation: complete. Its static performance contract moved RED 3/3 to GREEN 3/3.
- Full changed Rust set `rustfmt --check` and scoped diff check: passed.
- M0 and M2-M5 implementation and dynamic acceptance: pending.
- Managed Cargo did not start: the coordinator reports the current
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` Session as terminal `archived`. No raw Cargo
  bypass or fabricated Session identity was used.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint. Source review
or M1 static complexity improvement alone is not end-to-end performance acceptance.
