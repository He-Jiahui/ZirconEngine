---
title: Editor workbench host data shared-generation and selection performance review
date: 2026-08-22
module: zircon_editor/src/ui/layouts/windows/workbench_host_window/{host_data.rs,shell_content_selection.rs,frame_rect.rs,mod.rs}
priority: MVP-P0 editor retained data ownership and pane selection
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate retained widget ownership and invalidation roots
---

# Goal

Replace deep-clonable workbench transfer DTO trees with immutable generation-owned shell, chrome,
dock, pane and floating artifacts. Stable presentation/apply work must clone shared owners only, and
pane selection must traverse each slot set once while preserving expanded-active, active and nonempty
fallback order.

## Reviewed source

- owner files: `host_data.rs`, `shell_content_selection.rs`, `frame_rect.rs` and module root `mod.rs`
- Rust files: 4/4
- current lines: 692
- current bytes: 22,398
- joined current source-bytes SHA256:
  `3461a1f01b6c1d1183e0a58678ddf9e9f15e91d0984aa11730a9b658e1ac6a06`
- joined pre-M1 source-bytes SHA256:
  `97d47c3719523b601138dcbfaf145cb236dd377ac57d10fb3641a51098c4b243`
- owning commit before review: `4d5f52aa2b76a3a877aabdd47b01a98dcdd59493`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `host_data.rs` | 565 | 17,326 | `d49c6aadb02af87e37cb696dd337a621091c7683e76568268cba0d894d54ee9e` |
| `shell_content_selection.rs` | 52 | 1,628 | `b102837e76d726b0f9cf56f8f0d455164e4a30b0b529ca50600241821a5683ed` |
| `frame_rect.rs` | 12 | 255 | `dba6fb19e726360914e9b788b930a85a1d8ab3b3512c3801f48584945eed187b` |
| `mod.rs` | 63 | 3,189 | `2824ad7e36358a6e090dd408879f7de2170097593a1dca1ced97ddf929f1f507` |

All four files were read in full. Clone/consumption paths were followed through shell presentation,
scene projection/dock patches, shell-content partial apply and scene-to-host conversion. Related
consumer modules are not counted as fully reviewed in this record.

## Existing foundations to retain

`ModelRc` and `SharedString` make many row/string owner clones reference-counted. `PaneContentSize`
and frame conversion are small value types. Selection returns borrowed stack/tab references and does
not allocate. The partial shell-content path can commit one changed dock. These are sound foundations,
but ordinary `Vec`, nested payload and top-level DTO ownership still cross the wrong boundaries.

## Structural findings

### P0: a pane is deeply cloned between model, scene and host generations

`PaneData` derives `Clone` and owns both flat `PaneNativeBodyData` and typed `PanePresentation`.
Inspector native data contains nested component/property `Vec`s; typed inspector, timeline, plugin,
export, animation and diagnostics payloads contain additional ordinary vectors and strings. Scene
projection clones each surface pane into a dock, and scene-to-host conversion clones the dock pane
again before consuming it. An active content generation can therefore be copied at multiple transfer
boundaries even after inactive payload construction is removed.

The correct unit is a shared immutable pane artifact with a generation receipt. Layout/scene must
attach a handle; host conversion must consume or borrow the same owner. `ModelRc` inside a deeply
cloned outer DTO is only a partial optimization.

### P0: one 565-line data file mixes every retained domain without receipts

`host_data.rs` contains tabs, floating windows, pane payloads, shell state, layout, menu chrome, page
chrome, status, resize, drag, every dock and native floating surfaces. Nearly every aggregate derives
`Clone`, but none declares a source generation or changed-domain receipt. This makes full clone and
full reconstruction the easiest API and prevents compile-time ownership of narrow invalidation.

Split by stable owners, not merely by file size: shell/chrome, panes, docks, floating and geometry
artifacts each need an immutable owner plus exact source/resource/text/geometry generations.

### P0: scene DTO duplicates source and host-contract shapes

The workbench layer first builds `HostWindowSceneData`, then retained-host conversion constructs a
second `host_contract::HostWindowSceneData`, mapping every menu, tab, frame, dock, pane and floating
row. Both trees coexist for an apply. The transfer shape is acting as a second retained scene rather
than a narrow projection receipt. Stable or one-dock updates must not rebuild two complete scene
representations.

### P1: side selection performs up to three slot/map traversals

`side_pane_selection` searches the same slot slice three times: expanded+active, then active, then
nonempty. Each pass repeats `tool_windows.get` and predicates. Current production slot sets are small,
so this is not the primary bottleneck, but the equivalent priority can be selected in one pass with
three borrowed candidates. It is a simple, independently safe M1 and prevents this helper from
scaling poorly if plugin-defined slot groups grow.

### P2: frame conversion and module exports are not hotspots

`frame_rect` copies four `f32`s. `mod.rs` only declares modules and re-exports crate-local contracts.
No runtime optimization is justified in either file. The large public-within-crate surface is a
boundary symptom; M2 should shrink it as artifact owners replace flat DTO types.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SWidgetSwitcher.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`

Unreal installs the foreground tab's existing content owner into its parent stack; it does not clone
a domain DTO tree (`SDockingTabWell.cpp:842-866`). `SWidgetSwitcher` retains all children while one
dynamic child has the active parent relationship, so switching changes ownership/visibility rather
than reconstructing all content. `FSlateInvalidationRoot` retains cached element data and chooses a
slow rebuild only for explicit invalidation; when the fast update list is empty it performs no widget
update (`SlateInvalidationRoot.cpp:356-424`).

The transferable invariant is persistent shared subtree owners plus exact dirty state. Zircon's
model-to-scene-to-host chain must not encode ownership as repeated deep-clonable transfer structs.

## Target architecture

1. Split `host_data` into shell/chrome, pane, dock/floating and geometry artifact modules with explicit
   generation receipts and shared immutable owners.
2. Make `PaneProjectionArtifact` typed and shared. Surface, scene, partial patch and host conversion
   pass the same handle; delete flat `PaneNativeBodyData` and duplicate typed payload ownership.
3. Make `HostWindowSceneArtifact` a composition of shared segment owners. Full assembly is handle
   cloning; one-dock changes replace one dock owner.
4. Converge workbench and retained-host scene shapes so conversion maps only the changed domain or
   disappears at the ownership boundary. Do not retain two complete value trees.
5. Attach exact model, content, template/resource/text and geometry generations to every artifact.
   Paint-only changes must not rebuild content.
6. Hard-cut deep `Clone` entry points after callers use shared artifacts. Keep value cloning only for
   small geometry/scalars and final foreign-ABI rows that require ownership.

Complexity targets:

- unchanged apply: O(1) receipt comparison, shared-owner clones only;
- one dock/pane change: O(changed artifact), unrelated segments zero;
- pane handoff across surface/scene/host: zero deep intermediate clones;
- full scene duplicate DTO bytes: zero;
- side selection: O(S) with one lookup per candidate slot.

## M1 result

`side_pane_selection` now uses one traversal. It retains the first nonempty stack and first stack with
an active ID, and returns immediately for the first noncollapsed active stack. After traversal it
selects active before nonempty, then retains the existing active-tab-row or first-row fallback. This
preserves the exact three-pass priority with at most S map lookups instead of 3S.

The source contract first failed with the measured three `slots.iter()` traversals, then passed after
the one-pass priority candidates were introduced. Current production slot groups are one or two
entries, so the absolute win is small; this is recorded as M1 rather than evidence that the P0 DTO
ownership problem is solved.

M1 does not alter DTO ownership. M2-M4 own the shared artifact hard cut and require dynamic clone/byte
counters before structural edits.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| `PaneData`/nested payload clone calls and bytes | intermediate = 0 |
| model/scene/host scene bytes alive together | one artifact authority |
| segment builds by generation | changed domain only |
| side-selection slot/map visits | at most S |
| stable apply allocations | 0 excluding final required ABI work |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: pane kinds and payload rows 0/1/1,000/10,000; panes/windows 0/1/16/1,000; slot groups
0/1/4/1,000 with collapsed/active/nonempty precedence; stable applies 1/1,000; full, shell-only,
one-dock, pane-content, geometry, resource and render-only changes. Capture clone calls/bytes, live
DTO bytes, allocations, map visits, segment builds, CPU, latency, RSS and package energy on one
source/executable fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is only for a
launchable current-source editor's GPU/pixel parity; it cannot validate DTO clone ownership.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add artifact build, deep-clone call/byte, live-tree byte and selection-visit counters; capture baseline. | source-bound scale/profile evidence |
| M1 | Collapse side-pane priority selection to one traversal. | RED-to-GREEN source contract and precedence behavior parity |
| M2 | Publish shared typed pane and dock/floating artifacts with exact receipts. | zero intermediate pane deep clones |
| M3 | Compose one shared scene artifact and converge duplicate host scene ownership. | stable zero build; one-domain replacement |
| M4 | Delete flat native-body, deep-Clone and duplicate transfer-tree paths. | one artifact authority |
| M5 | Run scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 4/4 Rust files.
- Production shell/scene/partial-patch/host conversion consumers and Unreal references: read.
- M1 source implementation: complete. Its static contract moved RED 1/1 to GREEN 1/1.
- Related pane/scene/chrome/shell source contracts: passed, 14/14.
- Changed Rust `rustfmt`, scoped diff check and plan-record audit self-test: passed.
- M0 and M2-M5 implementation and dynamic acceptance: pending.
- Managed Cargo remains unavailable because the current validation Session is terminal `archived`.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
