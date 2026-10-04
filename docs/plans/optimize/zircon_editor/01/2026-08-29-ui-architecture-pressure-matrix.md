---
source_binding:
  head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
source_artifacts:
  root: E:/zircon-profiles/ui-architecture-pressure-matrix-20260829
status: current_source_static_candidates_e13
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
| P0 | WindowMetrics still clones eight semantic `PaneData` values. The first ownership repair removes the duplicate full-presentation `host_structure` Arc and uses the existing `structure_generation` scalar as semantic identity, so internal generation bookkeeping no longer forces COW; outstanding external generations still preserve snapshots through COW. The second repair makes UI Asset scoped refresh publish one validated `HostPanePresentationPatch` containing the pane payload, Surface frame, and popup hit index together, removing the stale-index O(N) pointer fallback and the generic all-pane hit-index rebuild. Generic non-scoped updaters still retain the full fallback. | At 600 geometry updates plus 1,000 one-pane semantic updates over 64 panes with 2,048 hit nodes each, the rejected dual-Arc baseline performs 64,000 whole-presentation pane clone operations; the scalar-generation candidate removes those internally forced clones and identity allocations in the no-outstanding-snapshot case. The current UI Asset scoped transaction performs 2,048,000 changed-pane index visits and zero unchanged-pane visits, while the generic path would perform 131,072,000 hit-node visits. For 100,000 pointer events and four popup candidates, the rejected frame-only scoped path performs 204,800,000 event-side node visits; the current publication performs 2,048,000 index-build visits plus 400,000 event-side candidate visits, 2,448,000 total and an 83.66x modeled reduction. These are operation counts, not allocator or timing evidence. | A stable pane location/id/kind owns one shared semantic pane product plus its Surface/frame/hit products. One typed host publication transaction validates exact old/new node identity and one shared model-occurrence directory, swaps changed pane handles, rebinds exact paint models, and publishes exact damage and generations atomically. Missing identity, topology/layout change, ambiguous old model, shared-model merge/split, or rebind mismatch uses a typed full fallback. | Twenty-two current/reference sources bind the model, including the canonical presentation paint-model enumerator, shared pane-node selector, transaction, state/window publication, shell-content model selection, Unreal invalidation/paint-space hit indexing, and Fyrox/Slint retained validity/cache contracts. Focused pressure/source tests pass 7/7; scoped Rust formatting and obsolete generic-updater/identity guards pass. Artifact `editor-pane-presentation-transaction-pressure-20260902-r8.json` SHA-256 `66FE92C0247E16BF67337F7F8C395D44C50A7530FC601408174D3E5550B1304A`, source-set SHA-256 `5FCFB33A20292B0A33ACC009307529897953A02670C480FCE0A6890469584655`. External componentized sources remain dirty; managed Rust and product timing remain open. |
| P0 | Scene-hierarchy fragments already carried exact logical rows and authored control IDs, but sparse publication cloned the complete host presentation and called `set_host_presentation`. Because the workbench node model identity changed, that path re-entered the global paint-model check and full workbench hit-index build. The current candidate builds one hierarchy pane patch plus one exact workbench row patch and validates both hit-index rebinds before a single COW commit. | At 65,536 sparse updates, 32,768 workbench nodes, 64 paint models, six hierarchy pane locations, and one changed logical/control row, the rejected path models 65,536 presentation snapshots, 65,536 full hit-index builds, and 2,147,483,648 workbench-node visits. The atomic candidate performs zero snapshots/full builds, 65,536 exact row validations, 393,216 pane validations, 4,194,304 paint-index handle clones, and 65,536 logical-row copies. The conservative modeled-work ratio is 456x; paint-model row indexing is excluded, so the old cost is understated. | The bridge remains the projection authority. Build the pane/workbench patches against the same current generation; validate all model identities and index membership before mutation; commit structure/geometry/hit generations once. Missing controls, divergent logical models, row-count/membership changes, or rebind failure invalidates bridge authority and uses the existing authoritative reflow. | Seven current/reference sources bind the contract, including Unreal invalidation-root/widget-proxy references. Static/source pressure suites pass 19/19 and scoped rustfmt/diff checks pass. Artifact `editor-hierarchy-sparse-publication-pressure-20260902-r2.json` SHA-256 `B68A46211848E1511BDE2A87A6063102E5C88BB3178D94AAB0565706F21F7934`, source-set SHA-256 `F35698FB115E3F3710B5FF95A68CE50BCA962E26AD5395CA5EB9DE26156ED466`. Lower Rust regressions are authored but not Cargo-executed; redraw damage and product latency remain open. |
| P2 | Close-prompt and asset-deletion overlays were native paint/pointer payloads but still used the generic presentation updater, materializing and checking the complete paint-model directory on every show, update, and clear. They now use two typed overlay publishers that preserve structure/geometry generations and regional redraw while retaining the existing workbench hit index. | At 10,000 overlay publications and 64 paint models, the rejected generic path materializes 10,000 temporary model directories and performs 640,000 identity visits. The typed path performs zero model-directory materializations and zero identity visits. Stable models trigger zero full hit-index builds in both paths. These are deterministic operation counts, not timing or allocation-byte evidence. | Overlay paint and native-pointer routing remain direct consumers of the presentation DTO and precede workbench hit-index dispatch. Keep overlay publication separate from template paint-model ownership; external generation snapshots still retain required presentation COW. | Seven current/reference sources are bound by overlay r1; focused model/source tests pass 5/5, scoped rustfmt/diff and no-generic-updater guards pass. Artifact `editor-overlay-presentation-pressure-20260902-r1/editor-overlay-presentation-pressure.json` SHA-256 `DE0EC96C8622835FAF34F59AC584ECBC967AD4B1669259F68F0E61BB68C10CCD`, source-set SHA-256 `870BA921C73A6964BA9560B5D2806FD18E178DC4C04CA4E5D67D7519EEBEFC19`. Managed Rust and product timing remain open. |
| P0 | The layout-slot static candidate makes the flat serialized carrier private, routes callers through `UiTree`, owns one mutation-closed edge lookup, removes global repair/workspace scans, and patches child dependency membership exactly. Container/topology changes use an explicit parent-local rebuild. | At 10,000 children and 10,000 unrelated slots, the rejected full build is 100,030,000 modeled units versus 30,000 achieved. The rejected one-child dependency patch is 100,010,000 versus 1 achieved; a topology fallback visits 10,000 children and zero unrelated slots. | Complete physical parent-edge ownership, then prove all container parity and product behavior. | Model/source contracts 7/7, direct-field/obsolete-symbol guards, scoped rustfmt and diff checks pass. Lower Rust, production counters, managed parity and product timing remain open. |
| P1 | Eligible auto-layout parents now retain their Taffy tree, exact available size, relative child frames, and last parent frame/clip. Stable K=1 changes and parent translation reuse the solve; resize/style/membership/order changes remain typed recomputes. | The source-bound v5 default fixture avoids 3,416,000 node creations. The unchanged K=1 case performs zero solves/child-layout reads and publishes 1,000 frames; translation performs zero solves/reads and publishes 1,024,000 synthesized frames; resize performs 12,000 solves and 192,000 reads. | Keep the retained product parent-owned and invalidate only on exact size/style/membership/order changes. Do not introduce a second global tree authority. | Source-bound pressure suite 12/12 plus scoped rustfmt/diff pass. Artifact `runtime-ui-taffy-parent-product-pressure-20260901-r6.json` SHA-256 `B69FB16AF4EF13EC4D64F4120CCBB43A1DE7051BCFAC94785F0004FE49287B02`. Managed Rust, allocation/CPU/RSS and product latency remain open. |
| P0 | Base and popup-projected hit grids now share finite geometry admission, a 128-cell per-axis/16,384-cell backing bound, a 4,096-cell per-entry threshold, adaptive cell-size coarsening, and checked allocation. The projected path no longer owns a duplicate cell mapper. | A 1,000,000×1,000,000 single-entry fixture would create 15,625×15,625 = 244,140,625 cells at the old fixed 64-pixel rule; the bounded current candidate preserves a 64×64 spatial partition with 4,096 memberships, a 59,604.64x cell-cardinality reduction without global 1×1 collapse. Separately, the existing paint-order cursor removes 49,995,000 legacy max-order visits for 10,000 sequential inserts. These are deterministic cardinalities, not timing or memory measurements. | Make the window/viewport the index-capacity authority, publish adaptive-coarsening/query-candidate counters, and add typed surface node/entry/membership/bytes budgets. Multiple genuinely full-surface overlapping entries can still produce O(entries) candidates and require an explicit authored-content budget. | Source-bound pressure and projected-helper contracts pass 11/11; scoped rustfmt/diff are green. Lower Rust, allocation/RSS counters, coarsened-cell candidate counts, and product resize/popup input p95 remain open. |
| P1 | Layout selection diagnostics were incrementally patched in the producer but copied as one flat Vec whenever a layout-domain frame was published. The selection carrier now uses the existing persistent 64-entry sequence; publication shares its root and one route replacement clones one leaf plus its directory path. | At 10,000 routes, 1,000 publications and one changed route per publication, modeled selection copies fall from 10,000,000 to 64,000, with 2,000 directory-node clones and 1,000 handle clones; reduction is 156.25x. | Keep diagnostics out of event queries, preserve JSON-array compatibility, and count leaf/directory clones in product resize capture before making latency claims. The public carrier change preserves borrowed iteration and single indexing, but owned/mutable/range Vec APIs require source migration. | Static/model contracts 11/11 pass; source-bound r4 receipt recorded. Rust snapshot/wire/COW regression is authored but not Cargo-executed. Product CPU/RSS/latency remains open. |
| P1 | Layout dirtiness also forced a complete focus-state clone and focus-path rebuild even when resize changed only geometry. Publication now reuses focus state and validates the retained route against the arranged-node index; focused-ID or parent-edge changes rebuild fail closed. | At 4,096 layout-only publications with an explicit 1,024-item focus-state payload and eight-node route, modeled clone work falls from 4,194,304 payload units to zero and is replaced by at most 32,768 indexed parent reads. The 128x ratio compares capacity units to visit units, not time. | Keep route validation O(depth), allocation-free, and outside clean event queries. Product resize capture must show zero focus state/path builds while reporting bounded validation visits; topology reparent must still advance the focus domain. | Static/model contracts 14/14 and adjacent frame/focus contracts 17/17 pass. Source-bound r2 receipt recorded; Rust resize/reparent regressions are authored but not Cargo-executed. Product CPU/RSS/latency remains open. |
| P0 | `RuntimeUiSurfaceSet` now rejects raw mouse motion in O(1), routes Keyboard/Text/IME and Navigation/Analog to retained owners, and queries a retained cell-to-Surface directory for captured/uncaptured pointer input. Resize-time queries map physical coordinates to the last published viewport and preserve the physical point in the forwarded `UiHitTestQuery`. Typed admission reserves reverse fanout/event-time rebuild for the cold unpublished state; non-finite pointer or degenerate viewport input is rejected before any Surface dispatch. Dirty publication now uses one retained cell-stamp array, recycles each Surface footprint buffer, and visits bounded cells without allocating a temporary vector per hit entry. | For 64 Surfaces and 100,000 pointer/focused/navigation/unrouted events, legacy fanout is 25.6 million dispatches and 25.2 million clones. The current modeled cutover is 400,000 dispatches, 100,000 necessary pointer fallthrough clones, and zero event-time rebuild probes. A warm 64-Surface patch additionally removes 64 occupancy and 64 footprint allocations, 32,640 transient boolean bytes, and 640,000 per-entry cell-vector allocations while retaining 2,040 stamp bytes. These are operation/allocation counts, not latency. | Keep rebuild-owned `RuntimeUiInputPublication` as the only global pointer authority; keep query-aware manager/pointer/Surface plumbing, affine resize mapping, direct capture/focus/navigation routing, retained patch scratch, and reason-specific invalid-admission counters. The existing immutable Surface vector makes index identity stable; require a separate stable ID only before dynamic mutation. Product p95/CPU/RSS evidence remains required. | Current source is bound by 10 critical sources and the v11/r13 artifact. Pressure and design suites pass 21/21; Python compile, scoped rustfmt, and diff checks are green. Lower Rust remains unexecuted by Cargo and product timing remains open. |
| P1 | Pane-button `Pressed` fallback damage is deliberately effect-agnostic: it unions the hit pane, two center-band frames and three status-bar frames, then requests a frame update. `Released` is already pointer-local. This does not affect stable hover, but an unknown-effect click can repaint nearly the complete workbench. | A source-bound 1920x1080 representative fixture covers 1,981,440 pixels, or 95.56% of the viewport. A typed action receipt preserving the button and a separately changed status bar represents 62,464 pixels in two bounded regions, or 3.01%; the deterministic area ratio is 31.72x. This is geometry, not GPU or latency timing. | Callback dispatch must publish typed affected-control/pane, status-text and active/sibling-pane receipts. Known actions emit bounded damage regions; missing or ambiguous receipts retain the current conservative fallback. Never shrink damage from action-name guesses or from the model alone. | Focused source/model contracts pass 5/5 and bind the exact fallback/caller hashes. The callback and native-pointer owners are externally dirty, so production behavior remains unchanged. Managed Rust, actual action distribution, draw work, GPU time and click-to-present p95 remain open. |
| P1 | Pointer/navigation dispatch now owns one route and lends it to handlers, but the first implementation still allocated a per-event visited `HashSet`. Both dispatchers now use a shared 16-node inline set and promote once only for deeper/more-divergent routes. | At 1,000,000 events, depth 1 navigation and depth 10 navigation/shared-ancestry pointer fixtures remove 1,000,000 visited-set heap allocations per dispatcher case while preserving identical visited inserts. A depth-10 four-candidate disjoint-ancestry upper bound (40 unique nodes) and depth 100 retain HashSet fallback. The prior route-copy model still removes up to 616 million pointer identity copies. | Keep one owned result route, borrow it through handler lifetime, stream candidates, use bounded inline membership for ordinary routes, and retain typed heap fallback rather than imposing quadratic deep-route scans. | Source-bound v3 artifact and candidate guards pass; combined route contracts 14/14, scoped rustfmt/diff green. Rust inline/promotion tests and product allocator/input p95 remain pending managed validation. |
| P1 | Inspector Runtime slot virtualization is bounded, but ordinary Editor presentation/value refresh still materializes the complete Inspector snapshot, copies the same nested property payload into the pane DTO, and rebuilds projection/surface ownership. Successful `WindowMetrics` refresh now reuses committed chrome/model/pane/presentation state and is excluded; only explicit metrics fallback re-enters this path. | With 10,000 properties, 1,000 stable presentation recomputes, 200 metrics fast-path hits, zero metrics fallbacks, and 1,000 single-field deltas, current source models 20,010,000 snapshot materializations plus 20,010,000 pane property copies. The retained target performs 11,000 property-record updates: 1,819.09x for snapshot materialization and 3,638.18x for combined two-stage work. | Selection/schema/value generations own shared logical properties and exact changed IDs. One retained Surface owns visible slots; metrics resize remains geometry-only, and a delta materializes only changed visible fields. Metrics fallback is typed and counted, never assumed for every resize. | Seven current Editor owners and Unreal/Fyrox/Slint reference anchors are source-bound by r2; focused model/contracts pass 9/9. Production owner is externally dirty; managed Rust and product allocation/CPU/RSS/p95 remain open. |
| P1 | The menu pointer bridge retains `UiSurface`, dispatcher, and routes. Hover-only sync exits through an O(1) state gate; geometry validates owned node/route identities and publishes exact changed frames; topology mismatch rebuilds. WindowMetrics now builds geometry only and shares menu trees, presets, and intrinsic popup widths by `Arc`. | The corrected five-domain 200-step fixture is 12,000 legacy versus 3,000 retained visit units (4.0x), with 200 Surface builds, 200 semantic layout builds, 2,200 dispatcher/route/path reconstructions, 8,000 popup text measurements and 8,000 popup item projections avoided. A 10,000 hover-only fixture avoids 10,000 Surface builds and 110,000 registrations. | Keep the shared semantic products immutable and the private Surface delta as the only hit-authority publication path; full semantic/topology changes remain typed rebuilds. | Current nine-source v3 binding and focused contracts pass 13/13; scoped rustfmt/diff pass. Managed Rust, Editor CPU/allocator/RSS and input-to-present timing remain open. |
| P1 | Render publication already owns persistent 64-command leaves, and the screen-space planner now consumes that exact identity instead of rebuilding a changed surface wholesale. Segment-retained image/text products remove most downstream batch reconstruction, but current image prepare still walks every surface segment and texture dependency on a stable resource generation; text returns early on a stable frame but recomposes frame-wide dependency/run indexes after a segment change. | Planner: a 64-surface/512-command/64-generation local-change model falls from 65,024 surface-level command visits to 36,800 leaf-level visits with 32,193 leaf hits. Image: 4,194,304 full batch visits fall to 1,536, but 262,144 unique texture dependency checks remain. Text: 4,194,304 batch visits fall to 4,608, but 8,388,608 active glyph and 32,768 font dependency checks remain. | Preserve command-leaf identity through all renderer consumers. A source-owned frame key returns before image prepare epochs and binding retention. Segment products own shared dependency/binding readiness so one changed surface segment visits only its delta. Resource-generation/backend-epoch changes use typed full fallback. Text delta composition must become persistent or locally patchable. | Planner leaf candidate and observability are static-complete: exact/all/local branches publish hit/count/rebuild, and evidence v2 separates leaf from surface conservation. Combined source/model suites pass 91/91. Managed Rust/product counters and timing remain open; image/text owners remain externally dirty. |

### Pane publication addendum

The r10 pane artifact supersedes the r8 receipt cited in the P0 pane row above.
It binds twenty-one current owners and six Unreal/Fyrox/Slint references, including
the viewport-toolbar SurfaceFrame projection. In a paired 1,000 stable/1,000
route-update fixture, the rejected generic publisher performs 128,000
paint-model identity visits and 32,000 floating-row clones. The current stable
path performs zero pane clones, presentation-generation increments, or
paint-model scans; route changes clone only affected panes. The native floating
window path now publishes metadata through a separate typed transaction instead
of retaining a generation snapshot across the generic updater. Its 1,000-update
fixture removes 1,000 forced aggregate COW clones and 64,000 paint-model identity
visits while preserving the workbench hit index. Bounds-only updates also avoid
1,000 structure-generation increments while retaining 1,000 geometry
increments; an equal repeat is a no-op.
Artifact
`editor-pane-presentation-transaction-pressure-20260902-r10/editor-pane-presentation-transaction-pressure.json`
has SHA-256
`48B358AC43552BDF41C99CE6BB510949065A474C33B7F149CEA969A55C938ABC`
and source-set SHA-256
`1A3A50074D45F411665212F0B4A3C463F333F7490D04688FF804C416971962AE`.
These are source-bound operation counts, not product timing evidence.

## Models that must not drive a current fix directly

| Model | Why it is not a current bottleneck claim | Use that remains valid |
| --- | --- | --- |
| Virtual-list slot materialization | The artifact reports `surface_materializer_wired=false`, while current Runtime source and tests already contain bounded physical-slot materialization. The 2,439.02x ratio describes the pre-wiring baseline. | Preserve the logical/physical separation and require product slot/node counts not to regress. Reconcile the model before quoting it as current. |
| SVG/GPU residency | Tree, raster-product, atlas, device allocation, and bind caches remain authoritative. The viewport provider now publishes a monotonic revision and WGPU caches the complete prepare product by draw-list/provider revision while unversioned custom providers keep conservative resolution. The source-bound 10,000-present/16-source model performs 10,000 revision checks, zero provider resolves/registry locks, and avoids 160,000 resolves; stable SVG reads/parses/raster/uploads remain zero. | Preserve the optional-provider fallback and prove actual Editor CPU/GPU/upload counters. Current-worktree artifact `editor-svg-gpu-residency-20260901-r7.json` SHA-256 `C142FC90C02F986559BD5EDD65967A81A2686054DD9F5BAB30642D75A61A6CDC`, source-set SHA-256 `3A11003DF2DF65A1B3410BE584CE0832D8A986A12E6E1AF019CC01FD4AC3F309`; four critical renderer sources are externally dirty, so product timing and commit-bound proof remain open. |
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
| `editor-pane-presentation-transaction-pressure-20260902-r10/editor-pane-presentation-transaction-pressure.json` | `48B358AC43552BDF41C99CE6BB510949065A474C33B7F149CEA969A55C938ABC` |
| `editor-hierarchy-sparse-publication-pressure-20260902-r2.json` | `B68A46211848E1511BDE2A87A6063102E5C88BB3178D94AAB0565706F21F7934` |
| `editor-overlay-presentation-pressure-20260902-r1.json` | `DE0EC96C8622835FAF34F59AC584ECBC967AD4B1669259F68F0E61BB68C10CCD` |
| `editor-inspector-projection-pressure-20260831-current.json` | `AD3DD2799681C9478C122591D2EF7D5E238B2FB0F63AA882E73B6E9B6702C1E2` |
| `runtime-ui-image-prepare.json` | `B2B41DA04D35319BA238E5B7B868DE5717836682DA99AE8C171D1F453FAA6DA6` |
| `runtime-ui-text-prepare.json` | `EFBA2B5864B243B524F44F7A8993B33BD02EB188259CF79EBE9ABFF87FCDFB99` |
| `runtime-ui-virtual-list.json` | `65E7DD25FB9E1D85CA642FC38043A9594F0B3D20FDAFFD5E4694AA6ADC2279C1` |
| `editor-svg-gpu-residency-20260901-r7.json` | `C142FC90C02F986559BD5EDD65967A81A2686054DD9F5BAB30642D75A61A6CDC` |
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

The layout model is now paired with `tools/analysis/performance/runtime/runtime_ui_layout_edge_evidence.py` (12/12 focused
contracts). Historical replay is deliberately rejected with 20 missing counters and one missing
source manifest; rejection artifact
`E:\zircon-profiles\runtime-ui-layout-edge-evidence-20260829-historical-regression.json` has
SHA-256 `98C54729D6B460C50BA3CBAD079E636D58B8BB3B4623C57FFBF8484022374D12`.

The v3 layout pressure artifact is source-bound to HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784`
and four exact Zircon/Unreal files. It separates the achieved exact child-property patch from the
achieved `O(C_parent)` container/topology fallback; neither path depends on unrelated workspace slots.

Render dependency acceptance now has two explicit phases: stable-frame
`tools/analysis/performance/ui/ui_render_segment_evidence.py` and changed-segment
`tools/analysis/performance/ui/ui_render_dependency_delta_evidence.py`. Their v2 schemas no longer compare the legacy planner
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

The retained shell-pointer v7 model is source-bound to the current menu Surface delta, shared
semantic layout product, viewport-toolbar geometry implementation, activity-rail geometry delta,
and Unreal's invalidation-root fast path. Its corrected five-domain 200-step menu fixture reduces
modeled node-domain visits from 12,000 to 3,000 while avoiding 200 topology rebuilds, 200 semantic
layout builds, 2,200 dispatcher/route/path reconstructions, 8,000 popup text measurements and
8,000 popup item projections. Its 10,000 hover-only fixture avoids 10,000 Surface builds and
110,000 registrations. Artifact
`E:\zircon-profiles\editor-shell-pointer-resize-pressure-20260901-r7.json` has SHA-256
`92B6B8BFC12B6FCFD042B695D36E1CDF9BFE66201108169238BE3B69F81D41DA` and source-manifest SHA-256
`2A536C9BD827FD9F97E8D60C5FE92A252B12E31477E6B49FAF77F3FEB2DA6369`.
The fixture's 22 focused Python/static contracts pass; this remains an operation model, not latency
evidence. Its activity-rail section additionally reduces 200 resize-time Surface rebuilds to zero,
avoids 1,200 dispatcher/route/path reconstructions, and reduces modeled node-domain visits from
7,000 to 5,000 (1.4x) for two visible strips and four buttons. It also avoids 200 semantic product
builds and 800 tab projections/identifier clones by sharing the left/right tab products.
Visibility/cardinality changes remain typed topology fallbacks.
The same WindowMetrics gate retains the geometry-free host-page, document-tab, and drawer-header
pointer receipts, avoiding 600 product builds and 2,600 item projections/identifier clones in the
default 200-step fixture while leaving non-WindowMetrics semantic recompute unchanged. The
welcome/recent pointer bridge uses its existing viewport-only sync on the same target, avoiding 200
recent-path clones and 1,600 recent-item projections in that fixture.

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

The pane presentation path has a broader ownership defect than the eight
WindowMetrics clones recorded by the earlier model. `HostContractState`
previously kept two `Arc<HostWindowPresentationData>` handles to one coherent
aggregate, so `Arc::make_mut` guaranteed an aggregate COW before ordinary
semantic changes. The first production candidate removes the bookkeeping Arc
entirely and uses the existing `structure_generation` scalar as semantic
identity. Semantic generation comparisons remain stable for unchanged geometry,
while the state owns only one presentation Arc and allocates no identity object
per update. An outstanding external generation still retains the presentation and
therefore still triggers required snapshot COW. The source-bound transaction
model at
`E:\zircon-profiles\editor-pane-presentation-transaction-pressure-20260902-r10\editor-pane-presentation-transaction-pressure.json`
(SHA-256 `48B358AC43552BDF41C99CE6BB510949065A474C33B7F149CEA969A55C938ABC`,
source-set SHA-256
`1A3A50074D45F411665212F0B4A3C463F333F7490D04688FF804C416971962AE`)
binds twenty-one current Zircon owners and six Unreal/Fyrox/Slint reference anchors.
For 64 panes with 2,048 nodes each and 1,000 one-pane semantic updates, the
rejected dual-Arc baseline performs 64,000 aggregate pane clone operations; the
current scalar-generation candidate removes those internally forced clones and
identity allocations when no external snapshot is outstanding. It does not yet
remove 4,800 geometry pane clones,
64,000 model identity visits, or 131,072,000 hit-node visits in generic
callers. The implemented UI Asset pane transaction performs 2,048,000
changed-node visits and zero unchanged-node visits, achieving the deterministic
64x shape for that scoped path while leaving generic callers to migrate. None of these
values are timing, allocator, RSS, or GPU measurements. The viewport-toolbar
projection now uses the same transaction: its paired 1,000 stable/1,000
route-update fixture removes 128,000 paint-model identity visits and 32,000
floating-row clones from the rejected generic path. Stable publication performs
zero pane clones, generation increments, and paint-model scans; route changes
clone only affected panes.
The native floating-window publisher separately removes the generation-snapshot
plus generic-updater COW shape. Its 1,000-update fixture removes 1,000 forced
aggregate COW clones and 64,000 paint-model identity visits while preserving
the workbench hit index. Bounds-only updates also avoid 1,000 semantic
structure-generation increments while retaining 1,000 geometry increments; an
equal repeat performs no commit.

The UI Asset scoped refresh also previously replaced its template nodes and
rebuilt only `body_surface_frame`, leaving `body_template_hit_index` bound to old
nodes or absent. The pointer route then failed the index identity check and
scanned all pane nodes for popup hit testing on every event. The current
candidate builds a validated `HostPanePresentationPatch`, publishes frame and
popup index from the same node identity, and rebinds only the changed paint
model. With 2,048 nodes, 1,000 scoped refreshes, 100,000 pointer events, and
four popup candidates, the rejected path performs 204,800,000 event-side
visits. The current model performs 2,048,000 publication-side index visits plus
400,000 event-side candidate visits, or 2,448,000 total, an 83.66x
operation-count reduction. Product pointer p95/p99
remains unmeasured.

The implementation boundary follows Unreal's reason-specific invalidation and
paint-time hit-grid update, Fyrox's separately retained measure/arrange/visual
validity, and Slint's generation-owned per-item rendering cache. Milestone 1a
has removed the internally duplicated aggregate Arc while preserving external
snapshot COW and semantic generation identity. Milestone 1b introduces a stable
`PanePresentationId` and pane presentation store so dock, document, and
floating locations share a semantic handle. Milestone 2 removes `PaneData` from geometry
publication and introduces one transaction that swaps changed handles, rebinds
exact old/new paint models, publishes exact damage, and advances related
generations atomically. Milestone 3 migrates UI Asset, Runtime Diagnostics, and
Inspector refresh through that transaction and removes the generic hot-path
closure, retaining a typed full fallback. Milestone 4 requires product counters
and Windows captures before accepting CPU, memory, GPU, or input-latency gains.

The Runtime Diagnostics refresh path had a duplicate authority: after the
common pane frame was published, the converter constructed another `UiSurface`
and rebuilt one node per diagnostic row solely to call `debug_surface_frame`.
The current source reads the published `body_surface_frame` directly, includes
non-dispatchable diagnostic labels in that frame with `UiInputPolicy::Ignore`,
and fails closed when publication is absent. Its single-pass reflector writer
also removes the temporary section/node vectors and flattened-string products.
The source-bound v2 artifact
`E:\zircon-profiles\runtime-diagnostics-surface-pressure-20260901\runtime-diagnostics-surface-pressure.json`
has SHA-256
`281034C6945CA64CD74B7AD183A831BA0C6572ABD280B08CE01C75C2F4A83901` and
source-manifest SHA-256
`5D597417E06C831DCDA9BB3ADEBE1BABA63B8BFC30AF20EC8D44EB05442A5A20`.
For 200 refreshes/256 rows it avoids 200 duplicate Surface builds, 51,400
tree/path materializations, 51,200 attribute-map products, 200 extra layout
rebuilds, 400 intermediate vectors, and 62,600 intermediate flattened strings.
These are deterministic operation counts, not timing or RSS evidence.

The published-frame repair does not yet cover active diagnostic document tabs or
floating windows. Their render-submission target remains `FullPresentation`,
whereas a unique visible drawer uses the existing `ShellContent` path. The
source-bound scope-pressure artifact
`E:/zircon-profiles/runtime-diagnostics-scope-pressure-20260902-r2/runtime-diagnostics-scope-pressure.json`
(SHA-256 `0B15C05728E8274979CBF5A2DC80711E4A6C97472F56F950391B295B25893C9A`,
source-manifest SHA-256 `32E3B3CC21E11DB19DE3E3C6431CAF16C7C81F9FE396D3C13F7B9297ECE1B483`)
models 200 refreshes and eight unrelated panes as 200 full refreshes, 1,600
unrelated-pane visits, and 200 global hit-index rebind boundaries. The planned
owner is a reusable pane-level publication target that builds one diagnostic
payload, patches one dock/document/floating row and requests one damage region;
unknown identity, changed topology/layout, or hit-index mismatch must retain a
typed full fallback. The artifact and its focused source contracts deliberately
carry `timing_claim=false` until managed product profiling is available.

Performance Timeline had an independent dead projection in the host contract:
paint, geometry profiling, and hit testing consume only `nodes`, while the
converter mapped all frame/span/hotspot/control rows and copied three summary
fields on every refresh. The current contract keeps source-side logical rows,
visible-row virtualization, and capture action nodes, and removes only the
unconsumed host DTOs. Artifact
`E:\zircon-profiles\performance-timeline-dead-projection-20260902-r2\performance-timeline-dead-projection.json`
has SHA-256
`0905CA761DFBF67B8312EEA60D7A673864368DBE34FD88829D3CBA2189F19F2E` and
source-manifest SHA-256
`A5057684488C9ADD96C55E787E6192B58034F8925D6030A64ECE07F5C9629B40`.
The default model avoids 153,600 dead row mappings, 800 dead control mappings,
and 600 dead summary/session/output clones. These are deterministic operation
counts, not timing or memory measurements.

## Implementation order

1. Validate and harden the new frame-bound resize publication: prove matching
   surface/geometry generations, affected-only layout/hit work, and product latency.
2. Validate the atomic sparse-hierarchy transaction, then continue moving Runtime
   Diagnostics, Inspector, and remaining pane publishers through typed local
   transactions; keep the global rebuild as a typed fallback for failed identity,
   paint-model rebind, or hit/damage parity.
3. Validate the tree-owned layout edge and exact-membership static candidate, then complete physical
   parent-edge ownership.
4. Validate segmented layout-report snapshot/wire parity in the managed lower
   lane and add leaf/directory clone counters to resize capture.
5. Land Runtime input publication and delete event-time Surface fanout/rebuild.
6. Publish typed pane-action damage receipts, retain the unknown-effect fallback, and route known
   button actions through bounded affected regions.
7. Convert retained-host menu resize from topology rebuild to exact authored-geometry publication,
   preserving the existing full fallback for menu/preset/topology changes.
8. Move Inspector logical property ownership above pane conversion and retain
   one bounded physical Surface.
9. Remove stable image/font dependency sweeps only after segment/product
   generation parity is measured.
10. Run current-source Windows product captures for 1/4/16/64 Surfaces, 200-step
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
