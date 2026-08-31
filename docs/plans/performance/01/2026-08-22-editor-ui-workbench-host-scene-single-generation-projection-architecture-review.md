---
title: Editor workbench host scene single-generation projection performance review
date: 2026-08-22
module: zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection
priority: MVP-P0 editor host presentation
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate persistent windows, invalidation roots, and live tab reuse
---

# Goal

Publish one immutable host-scene projection per accepted shell/layout/content generation. Main and
native windows must consume shared scene and pane artifacts with independent apply cursors; a Full
host recompute must not project the same floating windows twice, and a narrow dock change must not
rebuild unrelated docks, chrome or panes.

## Reviewed source

- module files: `scene_projection.rs` and `scene_projection/dock_patch.rs`
- Rust files: 2/2
- lines: 900
- bytes: 34,508
- joined current source-bytes SHA256:
  `42a03de12ef71fc0b7680fab2db4aa29b67513588b8b3b6a11e0a1227762c173`
- joined pre-change source-bytes SHA256:
  `d6f67da52cfe8f7d05393cfd9fbd4cf4ae95f1273ded36607ad7019f4b111488`
- owning commit before review: `bee4c707b714738346b49bba15c59468b8bd9b39`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `scene_projection.rs` | 680 | `416d31347471eaa210823c9c9fc634cc63c0c89f60d75f733f25715c30048316` |
| `scene_projection/dock_patch.rs` | 220 | `4687311f0f11f9814aa022635686c82b9363f7fc746f73b75f5e4ebb78d12b26` |

Both files were read in full. The production call chain was followed through
`apply_presentation.rs`, `shell_content_presentation.rs`, `projection_cache.rs`, `host_data.rs`,
scene conversion and the chrome frame-query helpers. The related helpers were read only as needed;
their owning chrome module is not counted as fully reviewed by this record.

## Existing foundations to retain

Shell-content invalidation can already build only the changed left, right or bottom dock. Chrome
tabs and menu chrome have persistent projection caches, and several template projection caches
return shared `ModelRc` values on stable identities. Root menu popup surfaces are not instantiated
up front. These are correct incremental foundations.

## Structural findings

### P0: Full apply projects every floating window twice

`build_host_scene_data_with_cache` calls `floating_windows_with_pane_shell_layouts` for the main
scene. `apply_presentation` immediately calls `build_native_floating_surface_data`, which calls the
same function again with the same source, project overview, chrome snapshot and fixed 31 px header
metric. `surface_metrics_from_chrome_assets` currently ignores shell width, so these are identical
source transactions.

For F floating windows, one Full apply therefore performs 2F wide `FloatingWindowData` row clones,
2F header surface-ID formats, two header/tab node projections per window, and two active-pane shell
projections before conversion begins. The main and native conversion passes then materialize the
same floating host-contract rows again. M1 can safely remove the second source projection; the
second host-contract conversion remains a structural M2 issue.

### P0: Full scene assembly is monolithic rather than generation-owned

Every Full apply rebuilds page chrome orchestration, status nodes, resize/drag layers, four docks and
the floating layer. Each dock clones its wide `PaneData`, creates header/tab/rail nodes and runs the
active pane through a seven-stage kind-dispatch chain. Only menu chrome has a complete explicit
input/resource/text generation cache at this boundary. Stable pane, dock, status and floating
segments do not have source receipts, so unrelated invalidation still recreates their container
products.

The target is not a larger monolithic cache. It is a domain DAG whose immutable segments are keyed
by exact source and geometry generations, then assembled by shared handles.

### P0: chrome frame derivation repeatedly scans all projected nodes

The related `chrome_template_projection.rs` helpers create a formatted control ID and call
`control_frame` or `has_control_frame` for each tab. Both helpers scan all N template nodes and use
`row_data`, which clones each visited node. Page overflow, page tab frames, close frames and dock tab
frames therefore approach O(T*N) scans for T tabs, with repeated formatting and row-owner work.

This is not changed in this module because the complete chrome projection folder has not yet passed
its own file-by-file review. Its owner must publish a control-ID-to-frame index once per template
generation and derive all tab/header/overflow frames in one linear pass.

## M1 result

`build_native_floating_surface_data` now consumes the already projected `HostWindowSceneData`
floating layer. It clones the shared `ModelRc` handle and copies only native ID/bounds/header scalar
state. `apply_presentation` passes the main scene artifact instead of the pre-projection surface
source and chrome snapshots.

Static transaction count per Full apply changes from two floating source projections to one. For F
floating windows, this removes F wide window row clones, F additional active-pane clones, F header
surface-ID formats, F header/tab node builds and F pane shell projections. The main and native
source structures now share one model identity. Two host-contract conversions remain and are not
accepted by M1.

### P1: unused uncached scene entry preserves a misleading second path

`build_host_scene_data` constructs a fresh `HostChromeProjectionCache` and delegates to the cached
entry, but no current repository caller uses it. Production uses `build_host_scene_data_with_cache`.
The unused wrapper should be removed after integration-feature consumers are checked so a future
caller cannot accidentally discard persistent cache state every invocation.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`

Unreal tab invocation first reuses a live tab and only spawns when no reusable/restorable instance
exists (`TabManager.cpp:1711-1727`, `1766-1823`). Each `SWindow` is a persistent invalidation root
with a persistent hit-test grid (`SWindow.cpp:2077-2080`). Window paint calls the invalidation root
instead of reconstructing the window tree (`2120-2149`).

`FSlateInvalidationRoot` retains cached element data. It runs a slow path only when the root or
policy requires it; otherwise it updates the fast widget path, and does nothing when the update
list is empty (`SlateInvalidationRoot.cpp:356-424`). The transferable invariant is a persistent
per-window apply root fed by shared live model/widget ownership and exact invalidation. It is not
one full scene projection per consumer per refresh.

## Target architecture

1. Publish `{shell, layout, chrome, pane, floating}` generation receipts and immutable scene segment
   artifacts. A scene assembly clones shared handles; stable segments perform zero projection.
2. Publish one floating layout artifact per floating-window generation. Main overlay and native
   windows share its window/pane/header products while retaining separate window ID, position, DPI,
   damage and applied-generation state.
3. Convert shared layout artifacts to host-contract artifacts once. Main/native views reference the
   same immutable pane/header/table data rather than performing two complete conversions.
4. Keep the existing single-dock patch, but make each dock artifact generation-addressable. A pane
   content change replaces only the target pane segment; a resize changes only geometry-dependent
   header/frame products.
5. In the chrome owner, build one typed control-frame index per template generation. Derive all
   tab, close, header, subtitle, overflow and rail frames by indexed lookup or one linear traversal.
6. Remove the unused uncached scene entry and flat compatibility assembly after consumers move in a
   hard cutover.

Complexity targets:

- unchanged Full request: O(1) generation comparison and zero scene/pane/header projection;
- changed single dock/pane: O(changed pane + visible rows), unrelated segments zero;
- changed floating generation: O(F + changed pane rows) once, not once per consumer;
- main/native assembly: shared-owner clones only, no repeated full row conversion;
- chrome frame derivation: O(N + T), not O(T*N).

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| scene/segment builds by source generation | unchanged segment = 0 |
| floating source projection passes per Full apply | at most 1 |
| floating host-contract conversions/row clones | at most 1 shared artifact per generation |
| dock/pane/header/template builds | changed target only |
| control-frame node visits and ID formats | O(N + T) |
| main/native shared owners and copied bytes | one immutable segment owner, no full duplicate bytes |
| host full/scoped/native apply and OS calls | unchanged = 0 |

Matrix: fixed docks 0/1/4; tabs 0/1/16/1,000; template nodes 10/1,000/10,000; floating windows
0/1/16/1,000; pane rows 0/100/10,000; main/native/both; stable refreshes 1/1,000; resize, drag,
tab switch, pane content, status-only and render-only invalidations. Report median/p95/max main
thread CPU, allocations/refcount operations, node visits, rebuild counts, input-to-paint latency,
RSS and package energy on one source/executable fingerprint.

Use current editor profile scopes and WPR/ETW with all artifacts on D/E/F. RenderDoc is reserved for
current-source viewport/pane draw-call and pixel parity; it does not measure scene DTO clones.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add scene/segment/floating/conversion/frame-query counters and capture baseline. | source-bound WPR and allocation evidence |
| M1 | Reuse the main scene's projected floating model for native source data. | RED-to-GREEN contract and shared model identity |
| M2 | Publish generation-owned scene segments and one shared host-contract floating artifact. | stable/duplicate projection and conversion = 0 |
| M3 | Replace chrome repeated scans with a generation-owned frame index. | node visits O(N + T) |
| M4 | Delete uncached/flat compatibility assembly and enforce scoped domain patches. | one authority and no silent Full fallback |
| M5 | Run F0/F4 interaction, WPR/power and RenderDoc parity. | quantified before/after and product parity |

## Validation state

- Full module source review: passed, 2/2 files.
- Production apply/patch/conversion/cache call chain and Unreal reference functions: read.
- M1 source implementation: complete. Its RED-to-GREEN static performance contract is 2/2.
- M0 and M2-M5 implementation and dynamic acceptance: pending.
- Managed Cargo is pending while shared Cargo processes remain active.

The module remains in `pending.md` until M0-M5 pass on one fingerprint. M1 source reuse alone is not
end-to-end performance acceptance.
