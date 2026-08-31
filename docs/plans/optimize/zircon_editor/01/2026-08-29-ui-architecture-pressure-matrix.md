---
source_binding:
  head: 17b92691e1d67c3df09376a2ca599bf2e07a061d
  ui_dirty_path_count: 9864
source_artifacts:
  root: E:/zircon-profiles/ui-architecture-pressure-matrix-20260829
status: current_source_static_candidates_e6
product_timing: false
---

# UI architecture pressure matrix

## Purpose

This matrix reruns the repository's canonical deterministic pressure models and
then reconciles each result against current source. Model operation counts are
useful for rejecting an algorithmic shape, but they are not CPU, allocator,
RSS, GPU, or input-latency measurements. A large counterfactual ratio is not a
current bottleneck unless the current call chain still has the modeled shape.

## Ranked current-source decisions

| Priority | Current-source finding | Canonical pressure result | Required structural boundary | Evidence state |
| --- | --- | ---: | --- | --- |
| P0 | The old trailing 80 ms resize gate has been removed from current source. Size events now retain the latest presenter extent, merge one interactive frame request, configure that extent at `RedrawRequested`, commit the retained frame, then present normally. | The rejected 25-event debounce baseline permitted 1,040 ms / 63 frame budgets of geometry mismatch; the 2,000-event fixture permitted 8,076 ms / 485 budgets. The frame-cadence target bounds the deterministic model to 13.333/16 ms. | Keep one frame authority for extent, layout, hit geometry, and damage. The event boundary is now frame-bound; the remaining P0 is proving that the downstream metrics transaction retains semantic/pane/text/image structure and scales with affected geometry rather than the whole tree. | Static source contracts 21/21, Python compile, scoped rustfmt, obsolete-symbol guard, and diff check pass. Lower Rust, product latency/CPU/RSS/GPU, stale-geometry counters, and visual/hit parity remain open. |
| P0 | The WindowMetrics fast path still clones four dock `PaneData` payloads during scene conversion and clones the same four again while applying geometry. Two floating-window conversion sites also clone `active_pane`. | The source-proven zero-floating lower bound is eight semantic pane clones per frame. At 600 frames and an explicit 1 MiB payload estimate, the deterministic model reports 4,800 clones and 5,033,164,800 copied bytes; this is a capacity model, not allocator evidence. | Publish immutable semantic pane products by shared identity, publish geometry/hit products separately, and compose one coherent generation without materializing or cloning pane payloads on resize. | Current r3 source hashes match the bound report and pressure/source-drift contracts pass 3/3. Geometry-only types now name the intended responsibility, but host-contract fields remain owned `PaneData` and current source has no `Arc<PaneData>` carrier, so data ownership is not split. Production ownership and product validation remain open. |
| P0 | The layout-slot static candidate makes the flat serialized carrier private, routes callers through `UiTree`, owns one mutation-closed edge lookup, removes global repair/workspace scans, and patches child dependency membership exactly. Container/topology changes use an explicit parent-local rebuild. | At 10,000 children and 10,000 unrelated slots, the rejected full build is 100,030,000 modeled units versus 30,000 achieved. The rejected one-child dependency patch is 100,010,000 versus 1 achieved; a topology fallback visits 10,000 children and zero unrelated slots. | Complete physical parent-edge ownership, then prove all container parity and product behavior. | Model/source contracts 7/7, direct-field/obsolete-symbol guards, scoped rustfmt and diff checks pass. Lower Rust, production counters, managed parity and product timing remain open. |
| P1 | Ordered children are already retained by layout-order generation and reuse the same `Arc` on a stable parent. The remaining auto-layout defect begins afterward: every visited parent clears one scratch `TaffyTree`, recreates every visible child node plus the parent, solves, and reads every child frame. | Across the canonical wide, nested, independent-forest, and resize fixtures, a warm per-parent retained product avoids 1,366,000 Taffy node creations. The wide-parent case performs 1,000 retained-order lookups and zero sorts, then still creates 1,025,000 Taffy nodes. Required ancestor solves and child reads remain in the conservative target. | Retain one exact Taffy product per eligible parent under the existing layout owner; reconcile stable child `NodeId` values and exact styles, keep conservative ancestor solving first, and use typed local fallback. Do not introduce a second global tree authority. | Source-bound v2 model and current order/bridge guards pass 9/9. Production layout paths overlap shared changes, so M1 parity implementation, managed Rust, allocations, CPU/RSS, and product latency remain open. |
| P0 | Base and popup-projected hit grids now share finite geometry admission, a 128-cell per-axis/16,384-cell backing bound, a 4,096-cell per-entry threshold, adaptive cell-size coarsening, and checked allocation. The projected path no longer owns a duplicate cell mapper. | A 1,000,000×1,000,000 single-entry fixture would create 15,625×15,625 = 244,140,625 cells at the old fixed 64-pixel rule; the bounded current candidate preserves a 64×64 spatial partition with 4,096 memberships, a 59,604.64x cell-cardinality reduction without global 1×1 collapse. Separately, the existing paint-order cursor removes 49,995,000 legacy max-order visits for 10,000 sequential inserts. These are deterministic cardinalities, not timing or memory measurements. | Make the window/viewport the index-capacity authority, publish adaptive-coarsening/query-candidate counters, and add typed surface node/entry/membership/bytes budgets. Multiple genuinely full-surface overlapping entries can still produce O(entries) candidates and require an explicit authored-content budget. | Source-bound pressure and projected-helper contracts pass 11/11; scoped rustfmt/diff are green. Lower Rust, allocation/RSS counters, coarsened-cell candidate counts, and product resize/popup input p95 remain open. |
| P1 | Layout selection diagnostics were incrementally patched in the producer but copied as one flat Vec whenever a layout-domain frame was published. The selection carrier now uses the existing persistent 64-entry sequence; publication shares its root and one route replacement clones one leaf plus its directory path. | At 10,000 routes, 1,000 publications and one changed route per publication, modeled selection copies fall from 10,000,000 to 64,000, with 2,000 directory-node clones and 1,000 handle clones; reduction is 156.25x. | Keep diagnostics out of event queries, preserve JSON-array compatibility, and count leaf/directory clones in product resize capture before making latency claims. The public carrier change preserves borrowed iteration and single indexing, but owned/mutable/range Vec APIs require source migration. | Static/model contracts 11/11 pass; source-bound r4 receipt recorded. Rust snapshot/wire/COW regression is authored but not Cargo-executed. Product CPU/RSS/latency remains open. |
| P1 | Layout dirtiness also forced a complete focus-state clone and focus-path rebuild even when resize changed only geometry. Publication now reuses focus state and validates the retained route against the arranged-node index; focused-ID or parent-edge changes rebuild fail closed. | At 4,096 layout-only publications with an explicit 1,024-item focus-state payload and eight-node route, modeled clone work falls from 4,194,304 payload units to zero and is replaced by at most 32,768 indexed parent reads. The 128x ratio compares capacity units to visit units, not time. | Keep route validation O(depth), allocation-free, and outside clean event queries. Product resize capture must show zero focus state/path builds while reporting bounded validation visits; topology reparent must still advance the focus domain. | Static/model contracts 14/14 and adjacent frame/focus contracts 17/17 pass. Source-bound r2 receipt recorded; Rust resize/reparent regressions are authored but not Cargo-executed. Product CPU/RSS/latency remains open. |
| P0 | `RuntimeUiSurfaceSet` now rejects raw mouse motion in O(1), routes Keyboard/Text/IME and Navigation/Analog to retained owners, and queries a retained cell-to-Surface directory for captured/uncaptured pointer input. Resize-time queries map physical coordinates to the last published viewport and preserve the physical point in the forwarded `UiHitTestQuery`. Typed admission reserves reverse fanout/event-time rebuild for the cold unpublished state; non-finite pointer or degenerate viewport input is rejected before any Surface dispatch. Dirty publication now uses one retained cell-stamp array, recycles each Surface footprint buffer, and visits bounded cells without allocating a temporary vector per hit entry. | For 64 Surfaces and 100,000 pointer/focused/navigation/unrouted events, legacy fanout is 25.6 million dispatches and 25.2 million clones. The current modeled cutover is 400,000 dispatches, 100,000 necessary pointer fallthrough clones, and zero event-time rebuild probes. A warm 64-Surface patch additionally removes 64 occupancy and 64 footprint allocations, 32,640 transient boolean bytes, and 640,000 per-entry cell-vector allocations while retaining 2,040 stamp bytes. These are operation/allocation counts, not latency. | Keep rebuild-owned `RuntimeUiInputPublication` as the only global pointer authority; keep query-aware manager/pointer/Surface plumbing, affine resize mapping, direct capture/focus/navigation routing, retained patch scratch, and reason-specific invalid-admission counters. The existing immutable Surface vector makes index identity stable; require a separate stable ID only before dynamic mutation. Product p95/CPU/RSS evidence remains required. | Current source is bound by 10 critical sources and the v11/r13 artifact. Pressure and design suites pass 21/21; Python compile, scoped rustfmt, and diff checks are green. Lower Rust remains unexecuted by Cargo and product timing remains open. |
| P1 | Pane-button `Pressed` fallback damage is deliberately effect-agnostic: it unions the hit pane, two center-band frames and three status-bar frames, then requests a frame update. `Released` is already pointer-local. This does not affect stable hover, but an unknown-effect click can repaint nearly the complete workbench. | A source-bound 1920x1080 representative fixture covers 1,981,440 pixels, or 95.56% of the viewport. A typed action receipt preserving the button and a separately changed status bar represents 62,464 pixels in two bounded regions, or 3.01%; the deterministic area ratio is 31.72x. This is geometry, not GPU or latency timing. | Callback dispatch must publish typed affected-control/pane, status-text and active/sibling-pane receipts. Known actions emit bounded damage regions; missing or ambiguous receipts retain the current conservative fallback. Never shrink damage from action-name guesses or from the model alone. | Focused source/model contracts pass 5/5 and bind the exact fallback/caller hashes. The callback and native-pointer owners are externally dirty, so production behavior remains unchanged. Managed Rust, actual action distribution, draw work, GPU time and click-to-present p95 remain open. |
| P1 | Pointer/navigation dispatch now owns one route and lends it to handlers, but the first implementation still allocated a per-event visited `HashSet`. Both dispatchers now use a shared 16-node inline set and promote once only for deeper/more-divergent routes. | At 1,000,000 events, depth 1 navigation and depth 10 navigation/shared-ancestry pointer fixtures remove 1,000,000 visited-set heap allocations per dispatcher case while preserving identical visited inserts. A depth-10 four-candidate disjoint-ancestry upper bound (40 unique nodes) and depth 100 retain HashSet fallback. The prior route-copy model still removes up to 616 million pointer identity copies. | Keep one owned result route, borrow it through handler lifetime, stream candidates, use bounded inline membership for ordinary routes, and retain typed heap fallback rather than imposing quadratic deep-route scans. | Source-bound v3 artifact and candidate guards pass; combined route contracts 14/14, scoped rustfmt/diff green. Rust inline/promotion tests and product allocator/input p95 remain pending managed validation. |
| P1 | Inspector Runtime slot virtualization is bounded, but ordinary Editor presentation/value refresh still materializes the complete Inspector snapshot, copies the same nested property payload into the pane DTO, and rebuilds projection/surface ownership. Successful `WindowMetrics` refresh now reuses committed chrome/model/pane/presentation state and is excluded; only explicit metrics fallback re-enters this path. | With 10,000 properties, 1,000 stable presentation recomputes, 200 metrics fast-path hits, zero metrics fallbacks, and 1,000 single-field deltas, current source models 20,010,000 snapshot materializations plus 20,010,000 pane property copies. The retained target performs 11,000 property-record updates: 1,819.09x for snapshot materialization and 3,638.18x for combined two-stage work. | Selection/schema/value generations own shared logical properties and exact changed IDs. One retained Surface owns visible slots; metrics resize remains geometry-only, and a delta materializes only changed visible fields. Metrics fallback is typed and counted, never assumed for every resize. | Seven current Editor owners and Unreal/Fyrox/Slint reference anchors are source-bound by r2; focused model/contracts pass 9/9. Production owner is externally dirty; managed Rust and product allocation/CPU/RSS/p95 remain open. |
| P1 | Retained-host WindowMetrics recompute reconstructs `HostMenuPointerLayout`; any geometry difference makes `HostMenuPointerBridge::sync_shared` allocate a new `UiSurface`, dispatcher, and route-intent map and perform a full rebuild. Ordinary stable pointer moves do not take this path, but interactive resize does. | A 200-step fixture with seven buttons, three popup layers, 40 popup items, and three changed geometry nodes models 200 surface builds, 2,200 dispatcher registrations/route bindings/path strings, 8,000 popup projections and 9,600 node-domain visit units. Retained geometry publication removes topology/semantic work and uses 1,800 visit units, a 5.33x operation ratio. | Mirror the existing viewport-toolbar delta contract: topology/semantic changes rebuild; geometry-only WindowMetrics changes patch retained node frames and call `publish_authored_geometry`; stable layout publishes nothing. | Current menu full-rebuild and toolbar geometry reference paths are source-bound with Unreal `FSlateInvalidationRoot`; model contracts pass 6/6. Menu sources overlap external changes, so production and product timing remain open. |
| P1 | Render publication already owns persistent 64-command leaves, and the screen-space planner now consumes that exact identity instead of rebuilding a changed surface wholesale. Segment-retained image/text products remove most downstream batch reconstruction, but current image prepare still walks every surface segment and texture dependency on a stable resource generation; text returns early on a stable frame but recomposes frame-wide dependency/run indexes after a segment change. | Planner: a 64-surface/512-command/64-generation local-change model falls from 65,024 surface-level command visits to 36,800 leaf-level visits with 32,193 leaf hits. Image: 4,194,304 full batch visits fall to 1,536, but 262,144 unique texture dependency checks remain. Text: 4,194,304 batch visits fall to 4,608, but 8,388,608 active glyph and 32,768 font dependency checks remain. | Preserve command-leaf identity through all renderer consumers. A source-owned frame key returns before image prepare epochs and binding retention. Segment products own shared dependency/binding readiness so one changed surface segment visits only its delta. Resource-generation/backend-epoch changes use typed full fallback. Text delta composition must become persistent or locally patchable. | Planner leaf candidate and observability are static-complete: exact/all/local branches publish hit/count/rebuild, and evidence v2 separates leaf from surface conservation. Combined source/model suites pass 91/91. Managed Rust/product counters and timing remain open; image/text owners remain externally dirty. |

## Models that must not drive a current fix directly

| Model | Why it is not a current bottleneck claim | Use that remains valid |
| --- | --- | --- |
| Virtual-list slot materialization | The artifact reports `surface_materializer_wired=false`, while current Runtime source and tests already contain bounded physical-slot materialization. The 2,439.02x ratio describes the pre-wiring baseline. | Preserve the logical/physical separation and require product slot/node counts not to regress. Reconcile the model before quoting it as current. |
| SVG/GPU residency | The 79,688x parse and 1,204,705x upload-write ratios compare retained residency with a deliberately bad per-command reconstruction baseline. Current source already has tree, raster-product, atlas, device allocation, and bind caches. The remaining source-bound residual is different: the product always installs an unversioned external provider, disabling the WGPU generation fast path and producing 160,000 provider resolves plus registry locks in the default 10,000-present/16-source model. | Preserve zero stable parse/raster/upload, add a monotonic provider-product revision, and cache the complete prepare product by draw-list/provider/device generation. Unchanged generations must execute one revision check per present and zero per-source resolves. Historical traces remain diagnostics until current-source product counters and timing exist. |
| Surface-frame all-domain sharing | The 760,258,560 old clone-work baseline predates current domain handles, persistent render segments, and shared submissions. Current dirty renderer/surface source must be rebound before assigning the remaining work. | Retain the invariant that window-only updates clone no layout/hit/render payload and one render change clones only affected directory/segment paths. |
| Full-frame image/text prepare | The flat 4,194,304 batch-visit baseline is no longer the entire current path because segment caches exist. | The residual dependency checks remain actionable and are listed in P1 above. |

## Evidence inventory

All artifacts below were regenerated from the current workspace using default
canonical fixtures. Their hashes bind exact model output, not current product
timing.

| Artifact | SHA-256 |
| --- | --- |
| `runtime-ui-surface-input-publication-20260901-r13.json` | `0E09FD6F22F06833B2FBB7080E85C392F4AC42C592617EEDDA9DFBF1AE7264FD` |
| `ui-pane-button-fallback-damage-pressure-20260901-r1.json` | `5E71DE4FC4184F8A51B1D5085156707AE656177E4A4A28C95771F3B35C4AE4C2` |
| `runtime-ui-tree-hit-grid-admission-pressure-20260831-r3.json` | `77F70F264B2182CAA448F7F8FF8F37546C584E0281C72EACFE3768A32EB42F58` |
| `runtime-ui-layout-slot-index-pressure-20260831-current.json` | `C34588A044FC01D3DF86378E034BF1358E37E064EA8EF46788E6351E05525834` |
| `editor-window-resize-reflow-pressure-20260829-r2.json` | `CA7ED75C587695928681706A98FE931970A07F4ECE6C975FDD1C4E236A561510` |
| `ui-window-metrics-pane-clone-pressure-20260831-r3/ui-window-metrics-pane-clone-pressure.json` | `F0F089FF7F84ABF290DED99088C3AAF2EA807BD72E46A0A6B4FCC32ABBCE1017` |
| `editor-inspector-projection-pressure-20260831-current.json` | `AD3DD2799681C9478C122591D2EF7D5E238B2FB0F63AA882E73B6E9B6702C1E2` |
| `runtime-ui-image-prepare.json` | `B2B41DA04D35319BA238E5B7B868DE5717836682DA99AE8C171D1F453FAA6DA6` |
| `runtime-ui-text-prepare.json` | `EFBA2B5864B243B524F44F7A8993B33BD02EB188259CF79EBE9ABFF87FCDFB99` |
| `runtime-ui-virtual-list.json` | `65E7DD25FB9E1D85CA642FC38043A9594F0B3D20FDAFFD5E4694AA6ADC2279C1` |
| `editor-svg-gpu-residency-pressure-20260831-r16.json` | `DE6CDF732A3F9BDBD52644188C3447A0AB95B199203F38B0386828424A5EE276` |
| `runtime-ui-surface-frame-domain-sharing-pressure-20260831-r3.json` | `A37BF22AFEA6FD431EB97CF545A61DBA69A770A32172A9FF42952B64C16F5496` |
| `runtime-ui-render-dependency-product-pressure-20260831-current.json` | `AA127CF8A82294E7E1342ACB43B975AF2A2EC96F84C7C27A2F436E87606BEEE3` |
| `runtime-ui-render-dependency-product-memory-pressure-20260831-current.json` | `26FAD4BB145F94E94CED5A12A5EC9D3C81921F19443F061063CAADC452DF1E47` |
| `runtime-ui-command-leaf-plan-cache-pressure-20260831-r1.json` | `30BDA2A9F4478C00088A42848195D5238E0F31B40A0AC2CC7C876EC10D65864A` |
| `runtime-ui-taffy-parent-product-pressure-20260831-r2.json` | `03CF55E7C53BBA4FBFAD5F0CF53CB75D67950BD7FF9DA6C1FB7D456B81489694` |
| `editor-menu-pointer-resize-pressure-20260831-r1.json` | `26DFCE1CB5BBC9FDF9565B3A92E515FA8A38013B9F140DD657855121BEE61F0B` |
| `runtime-ui-dispatch-route-sharing-pressure-20260831-r4.json` | `B97ACA0D0A7F5AE561718E051955560990CAA628065D85DEC0B32CE655F5FBAA` |
| `ui-profile-preflight-20260831-r17.json` | `ABE052D947C9C598CC1A74ECDE4784B57DC8F847145A0B7F34C70155A5DF1C5A` |
| `runtime-ui-layout-report-aggregation-pressure-20260831-r4.json` | `9C1ACEFA05D01D4D0B3D7081BE464A8B0EDA24CB4BE1960F4ADE49AE846F1809` |
| `ui-render-dependency-memory-evidence-20260829-current-gap-r2.json` | `BFC9C2083825B9E6B6F1995D32D6A7A19CE587C11EB16F62099E2941EC2ED5DB` |

The layout model is now paired with `tools/runtime_ui_layout_edge_evidence.py` (12/12 focused
contracts). Historical replay is deliberately rejected with 20 missing counters and one missing
source manifest; rejection artifact
`E:\zircon-profiles\runtime-ui-layout-edge-evidence-20260829-historical-regression.json` has
SHA-256 `98C54729D6B460C50BA3CBAD079E636D58B8BB3B4623C57FFBF8484022374D12`.

The v3 layout pressure artifact is source-bound to HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784`
and four exact Zircon/Unreal files. It separates the achieved exact child-property patch from the
achieved `O(C_parent)` container/topology fallback; neither path depends on unrelated workspace slots.

Render dependency acceptance now has two explicit phases: stable-frame
`tools/ui_render_segment_evidence.py` and changed-segment
`tools/ui_render_dependency_delta_evidence.py`. Their v2 schemas no longer compare the legacy planner
`segment_cache_hit_count` with surface-segment counts: stable frames require command-leaf hit=count and
rebuild=0, while delta frames independently conserve image/text surface segments and planner command
leaves. Historical pressure data is
rejected because it lacks the required source-bound delta counters and manifest; rejection artifact
`E:\zircon-profiles\ui-render-dependency-delta-evidence-20260829-historical-regression.json` has
SHA-256 `8933909683F78C166A5404B2F1345D4A4F6CE836997E8E0A2548D5DF409035AD`.

The source-bound residual model adds the work omitted by the earlier flat-batch comparison. Across
4,096 frames it counts 1,048,576 current image dependency/binding lookups and 2,097,152 binding-map
retention visits. The target keeps four explicit resource-generation fallbacks yet reduces those to
1,152 and zero. For 32 one-segment text deltas, dependency entries fall from 65,536 to 1,024 and run
entries from 16,384 to 256. The focused model contracts pass 7/7; tool SHA-256 is
`2C11ABCA1505B9AFB3B0FB6949DB9AD1338FD7DF52DA4A39638964021AAB3333` and test SHA-256 is
`0DE659AA7B1CBB73E59D1B4B2A2BEA1E87D37D45ACCCE5B71864956980E469AC`.

The complementary retained-memory model makes the generation bound explicit instead of assuming cached
products are free. With 64 segments, three live generations and one changed segment per delta, it retains
1,769,472 bytes of base source payload, 55,296 bytes of live changed-segment versions and 26,352 bytes of
CPU metadata: 1,851,120 bytes total versus 5,308,416 bytes for three full payload copies. It avoids
3,483,648 bytes of payload duplication and remains independent of one million modeled presents. A
4,096-segment stress fixture uses 1,148,016 of the 8 MiB metadata budget. These are capacity-model bytes,
not RSS or GPU memory. Focused contracts pass 10/10; tool SHA-256 is
`E9F9DAA66F0E2C9895FBB378B0D394BFCCBBDCE463AEA35F6974B29CFF5D7C98` and test SHA-256 is
`5CF5397800478BF07C27CB2575D7F9A0D97C217467B971C4C9CE68A58F7961A4`.
The command-leaf planner, stable/delta evidence, render dependency and memory suites now pass 91/91
together. Current evidence-tool SHA-256 values are `F005A562A6974764C4A71B67A1647AABD283915E7DB5B8E1F7A8CBD4AA3CD9B5`
for stable v2 and `98A676E7C6F2BF270356FAEDE8DF6F45C03D271A7E55565F9D18F0D3A5666210`
for delta v2. This is static/model acceptance only; it does
not replace a managed Editor product run with process and GPU memory counters.

The current implementation slice reserves exact-size dispatch route traces from the iterator size
hint. This removes repeated `Vec` growth where every route step is retained, while unknown iterators
keep the previous lazy allocation behavior. Route ordering and diagnostics remain unchanged. Its
focused Rust tests are pending the managed Cargo lane and are intentionally not reported as
runtime-green here.

The menu-pointer resize model is source-bound to the current retained-host menu full rebuild, the
adjacent viewport-toolbar geometry-delta implementation, and Unreal's invalidation-root fast path.
Its default 200-step fixture reduces modeled node-domain visits from 9,600 to 1,800 while avoiding
200 topology rebuilds, 2,200 dispatcher registrations, 2,200 route-intent bindings/path strings and
8,000 popup-item projections during resize. The artifact is
`E:\zircon-profiles\editor-menu-pointer-resize-pressure-20260831-r1.json` with SHA-256
`26DFCE1CB5BBC9FDF9565B3A92E515FA8A38013B9F140DD657855121BEE61F0B` and source-manifest
SHA-256 `D537F377267377B4D31E3D219617C398949DFBBE72C0A9614B3A0990F31279BE`.
The six focused contracts pass; this remains an operation model, not resize latency evidence.

Dispatch-route evidence v3 separates the current worktree from its historical comparison baseline.
It proves the borrowed one-route contract and the 16-node inline visited-set wiring in the artifact,
while retaining an explicit HashSet promotion for disjoint/deep routes. The shared-ancestry depth-10
fixture removes one visited heap allocation per event; the disjoint-ancestry upper bound remains a
heap fallback instead of being hidden by an optimistic average. Artifact/source-manifest SHA-256 are
`CAB0C0E49AF20CF4E9C970418A906DCD39B1C1C8BC0EE30C9F1BB80225245B7D` and
`A2F50F98FC386D245ECFC4FBF2E0C1AB2570B6224233E6771109A383427A2C8C`.

The layout-report slice removes one temporary `BTreeMap` entry set per recompute, updates the
existing sorted reason vector in place, and replaces the frame-published flat selection Vec with the
existing 64-entry persistent sequence. The default 1,000-recompute/10,000-selection model avoids
7,992 transient reason-entry allocations and 9,936,000 selection clones while keeping the same
10,000,000 reason aggregation operations. The source-bound artifact is
`E:\zircon-profiles\runtime-ui-layout-report-aggregation-pressure-20260831-r4.json` with SHA-256
`9C1ACEFA05D01D4D0B3D7081BE464A8B0EDA24CB4BE1960F4ADE49AE846F1809` and source-manifest SHA-256
`656323793AB7CBAC92E89446AC926443D297319AE3350E555CC672C3A1CC8A4C`. This is a deterministic
capacity/copy-work model, not a timing or RSS claim. The current report clone
still copies the bounded fallback-reason Vec; the default model records 8,000
reason-entry copies and 1,000 small-vector allocations as explicit residuals.

The layout/focus publication slice removes `focus: layout` from the rebuild
marker and validates a retained focus route through the arranged-node index.
The source-bound artifact is
`E:\zircon-profiles\runtime-ui-surface-frame-domain-sharing-pressure-20260831-r3.json`
with SHA-256
`A37BF22AFEA6FD431EB97CF545A61DBA69A770A32172A9FF42952B64C16F5496`
and source-manifest SHA-256
`3A4ABD91903F58509C98FAC0313115F169126B09CEBEB07CA8002BA0238E4195`.
Its 4,194,304-to-32,768 default comparison is a clone-capacity versus indexed
parent-visit upper-bound model, not timing or RSS evidence.

Within that total, the product-memory gate passes 10/10 focused contracts. It reuses existing process
growth/quiescence and
64 MiB RHI image-pool limits, then requires three phase snapshots, generation/metadata bounds, binding
identity conservation, quiescent recovery and zero global/liveness/full-clone work. Current model
artifacts are deliberately rejected with 40 blockers because they contain no product phase counters or
interaction evidence and are not capture manifests. The missing production surface is precise:
`image_cache_cpu_resident_bytes` stops at `UiSurfacePresentStats`, while renderer generation, metadata,
source-payload and binding-product counters and the dedicated pressure action do not yet exist. The
capture manifest also omits the renderer image and text-segment-cache fingerprints required by the gate.
Total
driver GPU residency remains a later external measurement, not an inferred Rust byte count.

## Implementation order

1. Validate and harden the new frame-bound resize publication: prove matching
   surface/geometry generations, affected-only layout/hit work, and product latency.
2. Validate the tree-owned layout edge and exact-membership static candidate, then complete physical
   parent-edge ownership.
3. Validate segmented layout-report snapshot/wire parity in the managed lower
   lane and add leaf/directory clone counters to resize capture.
4. Land Runtime input publication and delete event-time Surface fanout/rebuild.
5. Publish typed pane-action damage receipts, retain the unknown-effect fallback, and route known
   button actions through bounded affected regions.
6. Convert retained-host menu resize from topology rebuild to exact authored-geometry publication,
   preserving the existing full fallback for menu/preset/topology changes.
7. Move Inspector logical property ownership above pane conversion and retain
   one bounded physical Surface.
8. Remove stable image/font dependency sweeps only after segment/product
   generation parity is measured.
9. Run current-source Windows product captures for 1/4/16/64 Surfaces, 200-step
   resize, 10,000 Inspector properties, stable hover, click, and SVG size-bucket
   revisit. Report CPU, allocation count/bytes, working/private set, input to
   damage/present p50/p95/p99, GPU time, upload bytes, and exact fallback reason.

No model closes a milestone. A milestone requires managed lower tests and a
source-bound product binary/profile. Current preflight r17 binds 276 critical
sources and HEAD `050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f` to the only current
managed E: target pool. That target contains no `zircon_editor.exe` or
`zircon_runtime.dll`, so it fails closed with exactly those two blockers.
WPR and xperf remain installed; the earlier `-RequireWpr` probe separately
proved that this token lacks system-profile privilege. No product timing was
inferred from tool availability or intermediate Cargo artifacts.
