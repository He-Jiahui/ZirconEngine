---
source_binding:
head: 050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f
  scene_conversion_sha256: C80312CE15AA306B4EDDCC42B59C408D022746C7F752190E81A83A52333B9DFE
  host_root_sha256: 3959845404109854816FB358D354BBED99D7829FC4FD9C887B71625D639AD34D
evidence:
pressure_artifact: E:/zircon-profiles/ui-window-metrics-pane-clone-pressure-20260831-r3/ui-window-metrics-pane-clone-pressure.json
pressure_artifact_sha256: F0F089FF7F84ABF290DED99088C3AAF2EA807BD72E46A0A6B4FCC32ABBCE1017
  source_manifest_sha256: 0173E3C7637EA68A36716D60279FFE89AA21FB4EFE48163AA2343F4658386133
product_preflight: E:/zircon-profiles/ui-profile-preflight-20260831-r17.json
product_preflight_sha256: ABE052D947C9C598CC1A74ECDE4784B57DC8F847145A0B7F34C70155A5DF1C5A
status: reviewed_design_ready_owner_reconciliation_required
product_timing: false
---

# WindowMetrics semantic-pane product plan

## Decision

WindowMetrics must be a geometry product, not a second owner of pane semantics.
The current geometry conversion clones four dock `PaneData` values and geometry
application clones those four values again. Floating-window conversion adds two
clone source sites per represented floating row. This makes a zero-floating
resize carry a source-proven lower bound of eight semantic pane clones per
published frame even when pane content did not change.

The fix is an ownership split, not a smaller clone loop. Publish stable semantic
pane products by shared identity, publish geometry and hit products separately,
and compose them by reference at paint/input consumption. A geometry-only frame
must preserve semantic product identity and must perform zero `PaneData` clones.

Do not solve this by placing the existing monolithic presentation behind one
`Arc` and cloning the whole value whenever geometry changes. That retains the
semantic/geometry invalidation coupling. Do not reconstruct a full presentation
on the paint or event hot path. Composition must remain a borrowed view or an
already-published generation product.

## Current-source evidence

The source binding above contains these fixed sites:

| Stage | Semantic clones per zero-floating geometry frame |
| --- | ---: |
| retained scene conversion | 4 dock `PaneData` clones |
| geometry presentation `apply_to` | 4 dock `PaneData` clones |
| total lower bound | 8 |

`scene_conversion.rs` also has two `candidate.active_pane.clone()` source sites.
Their runtime contribution depends on which floating-window conversion branches
execute; the fixed eight-clone dock count does not depend on that ambiguity.

The deterministic pressure tool is
`tools/ui-window-metrics-pane-clone-pressure.ps1`, with its focused Pester
contract under `tools/tests`. At 600 frames, zero floating windows and an
explicit 1 MiB payload estimate, current source models 4,800 semantic clones
and 5,033,164,800 copied bytes. The target is zero semantic clones. The byte
estimate is deliberately an input to a lower-bound model, not an allocator or
CPU measurement.

The focused pressure contracts pass 3/3. The tool now fails closed unless the
caller explicitly acknowledges changes to the expected 4 scene-conversion, 4
geometry-apply and 2 floating clone source sites. The canonical artifact was
regenerated against the current worktree and HEAD; its manifest and artifact
hashes are shown above.

## Reference-engine boundary

Unreal's invalidation root retains widget and cached-element state while dirty
widget lists identify the work for the current update. Its hit grid indexes
paint-space geometry when widgets are painted; stable cell membership updates
sort/user metadata without rebuilding unrelated widget state. Geometry change
does not require copying every widget's semantic payload into another scene.

Fyrox independently keeps measure/arrange validity and previous constraints on
UI nodes. Matching constraints return early, while invalidation marks only the
required layout stages. These references support the same ownership rule:
semantic state is persistent authority; geometry is a revisioned derivative.

Zircon's runtime surface already has persistent layout/hit sequences and
frame-owned hit authority. The Editor host projection must preserve that
separation instead of rejoining semantics and geometry in
`HostWindowPresentationData` on every resize.

## Required architecture

### 1. Shared semantic authority

Introduce a published semantic product for dock and floating panes. The exact
type name is owner-selected, but its contract is:

- each pane payload is immutable for a published semantic generation;
- dock and floating pane products are held through shared identity, normally an
  `Arc` at the semantic-product or pane-product boundary;
- property/selection/content changes create only the affected semantic products;
- a WindowMetrics update retains exactly the same semantic identities.

Avoid wrapping mutable pane models in locks for the render path. Build the next
immutable semantic generation and atomically publish it.

### 2. Geometry-only publication

Replace `HostWindowGeometryPresentationData`'s dependence on a complete
`HostWindowPresentationData` value with a geometry product containing only:

- root and workbench frames;
- dock, document, bottom-strip and floating-window rectangles;
- clip, visibility and z/order data required by paint and hit publication;
- a geometry generation and typed full-fallback reason.

The WindowMetrics fast path patches changed geometry rows and reuses all
unchanged pages/products. It never reads, clones or overwrites `PaneData`.

### 3. Published composed generation

Publish one generation handle that references semantic, geometry, interaction
and render products independently. Paint and native presentation adapters read a
borrowed composed view. Input reads the already-published geometry/hit authority.
Neither consumer materializes a monolithic owned presentation per event/frame.

A semantic or structural mutation may publish a new semantic product and a new
geometry product in one generation. A pure WindowMetrics mutation publishes a
new geometry product while retaining semantic identity. Generation publication
is the consistency barrier; readers must never combine products from different
generation manifests.

### 4. Typed fallback and counters

Only topology changes, pane insertion/removal/reparenting, malformed journals,
or explicit recovery may request a full host presentation rebuild. Record the
reason and affected row counts. Stable resize counters must expose:

- semantic pane clones and semantic products rebuilt;
- geometry rows visited, patched and fully rebuilt;
- hit entries patched and full-grid fallbacks;
- composed presentation materializations;
- retained semantic and geometry generations/bytes.

Missing counters are an invalid measurement, not zero.

## Complexity and memory contract

Let `R` be resize-dependent layout candidates, `G_changed` changed geometry
rows, `H_changed` changed hit entries, `P_changed` changed semantic pane
products, and `N` the full presentation size.

| State | Required work | Forbidden residual work |
| --- | --- | --- |
| stable pointer | `O(1 + cell candidates)` | layout, render-command or pane traversal |
| geometry-only resize | `O(R + G_changed + H_changed)` | `PaneData` clone/traversal or `O(N)` presentation rebuild |
| local semantic delta | `O(P_changed + dependent geometry/render rows)` | unrelated pane copies |
| typed structural fallback | `O(N)` with reason/counter | untyped full rebuild |

Retained memory is proportional to live semantic/geometry generations and their
changed persistent pages, not resize-frame count. Quiescence must release old
generations after presentation and GPU submission pins are gone.

## TDD and acceptance order

1. Add a lower RED contract with large pane payload sentinels. Across 600
   WindowMetrics updates, semantic `Arc` identities remain equal, semantic clone
   and rebuild counters remain zero, and geometry generations advance.
2. Split semantic and geometry publication without changing the event route,
   action ordering, pane content or native presentation output.
3. Convert paint, hit and native adapters to borrowed composed-generation views.
   Add one-row geometry delta, multi-floating-window, clip/z-order and topology
   fallback regressions.
4. Add a source guard forbidding `.pane.clone()` and `active_pane.clone()` in
   geometry conversion/application modules. A semantic rebuild module may clone
   a shared handle, never the payload value.
5. Run managed lower Rust tests, then current-source Editor product captures for
   stable hover, continuous window resize, scale-factor change and pane
   topology mutation. Require CPU, allocation count/bytes, RSS/private working
   set, geometry/hit visits, damage-to-submit and input-to-present p50/p95/p99.

Acceptance requires zero semantic pane clones/rebuilds and zero full-presentation
fallbacks during the stable resize phase, no regression in semantic action or
hit correctness, and product timing tied to the exact source manifest. The
source-bound pressure model alone does not satisfy this gate.

## Ownership and execution state

The relevant Editor product sources are externally dirty at this source
binding, including the geometry application/conversion and presentation data
owners. This plan does not absorb those edits. Production implementation needs
an exact-path owner transfer or a copy-complete reconciled owner candidate.

The current worktree exposes an incomplete product split rather than a shared
pane cutover. `HostWindowGeometryPresentationData` and the retained-pane scene
conversion entry points now name a geometry-only responsibility, but dock and
floating host-contract fields still own `PaneData`; the conversion and
`apply_to` paths therefore preserve semantics by cloning the payload. A
current-source search finds no `Arc<PaneData>` carrier. This is evidence that
the responsibility boundary has started moving while data ownership has not,
not authority to complete or revert the external migration from this session.
The runtime multi-Surface input owner is likewise externally dirty, so the two
highest-priority production edits remain ownership-sensitive rather than
abandoned.

The source-bound product preflight above inspected 276 critical UI source files
at HEAD `050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f`. It failed closed with exactly
two blockers: the managed profiling target contains neither
`zircon_editor.exe` nor `zircon_runtime.dll`. WPR, xperf and WPAExporter are
installed; r16 did not require privileged WPR CPU sampling as a readiness gate.
No product CPU, allocation, RSS, GPU or latency result can be claimed from this
state, and no Cargo command was run.
