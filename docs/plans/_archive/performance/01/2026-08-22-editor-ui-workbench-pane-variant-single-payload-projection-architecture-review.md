---
title: Editor workbench pane variant single-payload projection performance review
date: 2026-08-22
module: zircon_editor/src/ui/layouts/windows/workbench_host_window/{pane_projection.rs,pane_presentation.rs,pane_payload.rs,floating_windows.rs}
priority: MVP-P0 editor panes, documents and floating windows
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate foreground-tab content ownership and widget switching
---

# Goal

Make one selected pane publish exactly one typed content payload per committed pane generation. A
Hierarchy pane must not clone inspector properties, timeline samples, plugin rows, export state, UI
assets and animation collections. Snapshot lookup, payload projection and host conversion must share
one immutable receipt so unchanged panes do no rebuild work and a changed pane scales with its own
content only.

## Reviewed source

- owner files: `pane_projection.rs`, `pane_presentation.rs`, `pane_payload.rs` and
  `floating_windows.rs`
- Rust files: 4/4
- current lines: 1,444
- current bytes: 49,653
- joined current source-bytes SHA256:
  `b2a57cfd3f10d549ec947d4494e108ede472a76287aea207d92adeca749ca4fe`
- joined pre-M1 source-bytes SHA256:
  `2a8103102f74bc4b71859befe2c456e4d7d44a9ab638794746126765e0165c1b`
- owning commit before review: `4d5f52aa2b76a3a877aabdd47b01a98dcdd59493`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `pane_projection.rs` | 865 | 31,151 | `b6af236ac6d615aff925f403875647a050aa6a894a66c4a940e712a4b28ffe22` |
| `pane_presentation.rs` | 160 | 4,924 | `f6d5379d642a9701eca3404fdf5cecf6d331d35b2bf4a0f4ea3eb80a84553a48` |
| `pane_payload.rs` | 250 | 7,604 | `47dd42b6763d4cc8c7bd05225302f6263b995eda5bbe66ef08765ba2fc856217` |
| `floating_windows.rs` | 169 | 5,974 | `6e2b60e51cadf38da766afce66192c0141cd1d7eeae2d27da19d6a58ac581d7d` |

All four files were read in full. Production callers were followed through `ShellPresentation`,
retained-host shell content, scene projection and host-contract pane conversion. The related flat
`PaneNativeBodyData` declaration and its variant consumers were inspected to prove inactive defaults
are valid, but `host_data.rs` is not counted as a fully reviewed file in this record.

## Existing foundations to retain

`PanePayload` is already a typed enum and `PanePayloadBuildContext` borrows source snapshots. Host
conversion already uses `pane_kind` or typed `PanePayload` to select one output branch. Scene
projection fills Assets, AssetBrowser and Project data only for their matching pane kinds. These are
the correct boundaries to preserve. The structural defect is the older flat native-body projection
that runs before those selectors.

## Structural findings

### P0: every pane constructs every native payload variant

`pane_from_tab_with_template_v2_data` unconditionally builds hierarchy rows from all scene entries,
inspector fields and nested plugin properties, console rows, timeline frames/spans/hotspots/controls,
module-plugin data, export data, generated-bottom data, the UI asset presentation and all animation
lists. Only after this work does host conversion select one kind.

For K selected main/floating panes, E scene entries, Q inspector/plugin properties, T timeline rows,
P plugin/export rows and A animation rows, one shell projection is approximately
O(K * (E+Q+T+P+A)). A Hierarchy pane therefore pays animation and timeline allocation; a Scene pane
pays all of them despite consuming none. This is main-thread ownership amplification, not a minor
iterator issue.

### P0: the selected animation and UI payloads are cloned before kind is known

Both optional presentations are cloned/defaulted before `build_pane_presentation`. The defaulted
animation object is passed as `Some`, then converted into six owned string/list models even for a
non-animation pane. The UI asset presentation is also cloned into every pane's native body. Default
construction hides the variant error rather than making it cheap.

### P0: snapshot discovery rescans the complete workbench for each selected pane

`find_tab_snapshot` walks every drawer, recursively visits every main-page workspace and then every
floating workspace for one instance ID. Side, bottom, document and each floating active pane call it
independently. With V tabs and F floating panes, shell lookup is O((4+F)*V). The workbench generation
owner must publish an instance-ID index once; consumers must not independently rediscover it.

### P1: typed payloads are flattened, cloned and reconstructed again

The newer `PanePayload` enum is converted to a flat `PaneNativeBodyData` whose fields hold every pane
shape. Timeline, generated-bottom, animation, module and export conversion then copies rows into host
contract DTOs again. The same logical payload therefore has enum, flat native-body and host forms
without a shared generation artifact. M1 can stop inactive construction; M2 must hard-cut to one typed
payload owner and make host/scene projections consume it directly.

### P1: floating-window chrome is recreated as formatted owned data

Every floating window maps all tabs to owned DTOs and formats the window target plus four edge target
group strings on each shell build. Its geometry parameter is unused. These values are stable for a
window identity and belong in a generation-owned floating-window artifact. M1 leaves this API intact
because the empty-template entry remains a real retained-host/test contract.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SWidgetSwitcher.h`

When foreground selection changes, Unreal updates one `ForegroundTabIndex`, emits the foreground
notification and calls `RefreshParentContent` (`SDockingTabWell.cpp:668-707`). That function reads
only the foreground tab and installs exactly its `GetContent()` plus optional left/right/title content
into the parent stack (`842-866`). `SDockingTabStack::SetNodeContent` replaces the one content slot
and optional slots (`449-459`). `SWidgetSwitcher` explicitly permits at most one visible child, stores
all children separately and exposes only one dynamic child to layout/prepass.

The transferable invariant is one persistent content owner per tab and one selected content subtree
per stack. Zircon should not manufacture all possible pane-domain DTOs when a foreground pane is
selected.

## Target architecture

1. Replace flat all-variant construction with one `PaneContentProjection` enum owned by the pane
   generation. Only the selected `ViewContentKind` builder runs.
2. Publish a `WorkbenchTabSnapshotIndex` once with the workbench snapshot. Drawer, page and floating
   lookup becomes O(1) expected or O(log V) worst-case from one O(V) build.
3. Bind `{pane identity, kind, content generation, template/resource/text/geometry receipt}` to one
   immutable `PaneProjectionArtifact`. Stable shell assembly clones its shared owner only.
4. Make scene node projection and host conversion consume the typed artifact directly. Delete the
   duplicate flat native-body copies and fallback reconstruction paths.
5. Publish floating-window identity/chrome/tab artifacts with exact window, tab and geometry
   generations. Do not reformat target groups or remap stable tabs.
6. Preserve narrow invalidation: hierarchy changes rebuild hierarchy only; timeline capture rebuilds
   timeline only; plugin refresh does not touch documents, animation or scene panes.

Complexity targets:

- changed selected pane: O(selected payload), independent of all inactive payload sizes;
- unchanged pane: O(1) receipt comparison, zero payload/list/row construction;
- all snapshot lookup in one shell generation: O(V), then indexed lookup;
- floating window identity/chrome: unchanged = zero formatting/mapping;
- host conversion: one selected typed payload, no duplicate full-form clone.

## M1 result

The current flat host ABI is retained, but `build_native_body` now constructs a default
`PaneNativeBodyData` and populates only the field matching `ViewContentKind`. The original optional
animation presentation is passed directly to the typed payload builder; animation and UI asset data
are cloned/converted only inside their matching branches. Hierarchy, inspector, console, timeline,
module plugins, build/export and generated-bottom likewise run only for their own kind. Assets,
AssetBrowser and Project remain default here and retain their existing kind-specific scene-projection
fill. `blank_pane` now uses the derived native-body default instead of manually constructing every
inactive field.

For non-native kinds such as Scene/Game/Welcome/Project, native payload builder calls fall from nine
domain builders/clones to zero. For a matching native kind they fall to one. The animation path no
longer constructs and converts a default animation presentation for every unrelated pane. The source
contract moved RED 3/3 to GREEN 3/3 and guards the selected builder, kind-confined heavy builders and
absence of unconditional animation/UI clones.

This removes inactive O(E+Q+T+P+A) allocation immediately without pretending to solve duplicated
typed/flat ownership. Compatibility entry points remain because repository call analysis finds live
production/test users. M2 owns the typed hard cut and snapshot index.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| payload builder calls by kind | selected kind = 1; inactive kinds = 0 |
| scene/inspector/timeline/plugin/animation rows cloned | selected payload only |
| tab snapshot entries visited | one O(V) index build, then indexed lookup |
| pane/list/model allocations | stable = 0; changed = selected payload only |
| shell/floating projection builds | matching generation only |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: every `ViewContentKind`; panes 0/1/4/1,000; tabs 0/1/16/1,000; scene entries,
inspector properties, timeline rows, plugins/export rows and animation rows 0/1/1,000/10,000;
floating windows 0/1/16/1,000; stable refreshes 1/1,000; selection, resize, tab switch, content-only,
resource, text and geometry invalidations. Capture builder counts, rows/bytes cloned, allocations,
main-thread CPU, latency, RSS and package energy on one source/executable fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for
current-source GPU draw/pixel parity after the editor can launch; it does not prove removal of CPU DTO
construction or snapshot scans.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add per-kind builder, row/byte, lookup, allocation and shell-generation counters; capture baseline. | source-bound scale/profile evidence |
| M1 | Build only the selected native payload and remove unconditional animation/UI cloning. | RED-to-GREEN contract and pane behavior parity |
| M2 | Publish typed pane artifacts and one tab-snapshot index; hard-cut flat duplicate ownership. | selected-only O(payload), indexed lookup, stable zero work |
| M3 | Publish generation-owned floating-window chrome/tab artifacts. | unchanged mapping/formatting = 0 |
| M4 | Delete fallback reconstruction and flat all-variant compatibility paths. | one typed payload authority |
| M5 | Run scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 4/4 Rust files.
- Production pane/shell/scene/host consumers and Unreal reference functions: read.
- M1 source implementation: complete. Its static contract moved RED 3/3 to GREEN 3/3.
- Related pane/scene/chrome/shell source contracts: passed, 13/13.
- Changed Rust `rustfmt`, scoped diff check and plan-record audit self-test: passed.
- M0 and M2-M5 implementation and dynamic acceptance: pending.
- Managed Cargo remains unavailable because the current validation Session is terminal `archived`.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint. Source
review or the M1 inactive-payload reduction alone is not end-to-end performance acceptance.
