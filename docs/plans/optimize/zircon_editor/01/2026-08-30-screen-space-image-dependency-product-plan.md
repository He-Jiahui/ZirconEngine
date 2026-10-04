---
source_binding:
  head: 5798051603e7f7f565538125c9aba96d5beabae2
  dependency_product_source_set_sha256: A0A23EE5FFC4BB9AD23D81F85ED07238F66E879DB5669E10CAAE5181CEF961A9
  dependency_product_model_sha256: E2394D749F1F17CF90C58AA3C40C9556FC63688E5731DBFB39EFFBAD4343F38B
evidence:
  pressure_artifact: E:/zircon-profiles/runtime-ui-render-dependency-product-pressure-20260901-r5.json
  pressure_artifact_sha256: 1E878135A2D4FACDF7D568A1B9CA886AF70806E002C4C3583442792B7051F0C2
  memory_artifact: E:/zircon-profiles/runtime-ui-render-dependency-product-memory-pressure-20260831-current.json
  memory_artifact_sha256: 26FAD4BB145F94E94CED5A12A5EC9D3C81921F19443F061063CAADC452DF1E47
status: implementation_static_green_managed_and_product_validation_pending
product_timing: false
---

# Screen-space image dependency-product plan

## Decision

The remaining image prepare cost is a lifetime and publication problem, not an
SVG parser problem and not a reason to add another Editor cache. Current
`ScreenSpaceUiImageSystem::prepare` retains geometry by segment identity, but a
stable frame still allocates a prepare epoch, visits every segment dependency,
looks up every GPU texture and bind group, and scans the binding cache for
retention. Its stable complexity is therefore `O(D + B)`, where `D` is the
number of visible unique image dependencies and `B` is the binding-cache entry
count.

The product renderer already owns the correct constant-time frame identity.
`ScreenSpaceUiPlanCache` returns the same `Arc<PreparedScreenSpaceUi>` for an
exact stable submission, and `ScreenSpaceUiRenderer` already uses that identity
to return early from vertex preparation. Image preparation must consume that
frame product identity instead of rediscovering stability from the segment
slice.

Do not implement this plan by hashing or comparing every segment on a stable
frame. That changes the constant but preserves `O(S)` work. Do not keep the
current epoch sweep and merely increase its idle window. That preserves both
the frame-wide scan and the false requirement that unchanged segments be
touched to keep their bindings alive.

## Current-source SVG cache adjudication

The 2026-08-31 current-source review confirms that Editor SVG work is already
split across three real cache layers:

- `visual_assets/svg/cache.rs` retains up to 1,024 parsed `Arc<usvg::Tree>`
  values. A unique normalized path alias hits memory before metadata or file
  reads; targeted asset events invalidate only content-changed paths.
- `visual_assets/loading/cache.rs` retains up to 4,096 raster results under a
  64 MiB budget. Its key includes semantic asset identity, raster target and
  tint, and cached pixels share `Arc<[u8]>`; a warm hit skips candidate-path
  construction and filesystem probing.
- `ScreenSpaceUiImageBindingCache` reuses a bind group by stable
  `Arc<GpuTextureResource>` identity, while `ScreenSpaceUiImagePrepareTextureCache`
  memoizes requested-to-resolved texture IDs for the current resource-management
  generation.

Consequently, a high SVG load counter must first be separated into tree-cache
miss, raster-cache miss, texture-resource replacement and bind-group creation.
Treating all four as a parser miss would hide the actual invalidation source.
The cache key dimensions are not the current structural defect.

The remaining stable-frame defect is visible after these hits: `prepare`
still visits every render segment, `refresh_segment_dependencies` visits every
retained texture dependency and `retain_prepare_epoch` scans the binding map.
Those visits exist solely to rediscover and renew products that already have
stable identities. This is the `O(S + D + B)` work eliminated by the product
authority and change journal below.

Asset refresh is event-driven rather than frame-driven. Exact resource paths
perform targeted fingerprint invalidation; only a lagged resource event stream
requests a reconciliation scan, and sprite-atlas source changes request a full
visual-cache clear. Product capture must therefore record these refresh reasons
beside SVG cache counters so a real event storm is not mistaken for cache-key
failure.

The separate Runtime `UiIconAtlasBuilder` is not currently a product-path
explanation for Editor stalls: current-source references are its own module and
tests only. Its 512-entry parsed-document cache returns a cloned document and a
builder call still deduplicates, sorts and lays out the complete request set.
That prototype needs a retained atlas product before product adoption, but it
must not be mixed into the existing Editor raster/GPU cache diagnosis.

## Direct reference evidence

Unreal is the primary lifetime reference:

- `SlateCore/Public/Rendering/SlateResourceHandle.h` defines a shared handle
  that can be safely cached and becomes invalid when its resource is destroyed.
- `SlateCore/Private/Rendering/ShaderResourceManager.cpp` reuses the brush's
  existing handle when its proxy is unchanged; it does not renew every visible
  brush through a frame epoch.
- `SlateRHIRenderer/Private/SlateRHIResourceManager.h/.cpp` keeps dynamic
  texture/material resources in persistent maps, creates on miss, reuses free
  resources, and performs explicit or GC-driven cleanup outside draw-element
  traversal.
- `FSlateRHIResourceManager::GetVectorResource` delegates to the persistent
  vector graphics cache, so SVG raster lookup and GPU resource lifetime are
  separate from per-frame geometry traversal.

Fyrox is a secondary generation reference. Its
`fyrox-impl/src/renderer/cache/texture.rs` indexes a process renderer cache by
stable texture cache identity and updates GPU data or sampler state only when
their modification counters change. Fyrox UI rendering still performs more
per-command work than the Zircon target, so it is not the complexity model; it
only confirms that resource revision, not frame visitation, is the invalidation
authority.

These sources support one common rule: immutable draw products hold usable
resource handles, while a resource manager owns creation, revision and cleanup.
Visibility traversal is not a lease-renewal protocol.

## Required architecture

### 1. Frame product authority

Pass `&Arc<PreparedScreenSpaceUi>` (or an equivalent published frame-product
identity) into image preparation. Retain a weak identity plus these typed
inputs:

- resource-management generation identity;
- viewport identity if it is not already guaranteed by the prepared product;
- backend/device epoch;
- explicit force-full-upload state.

An exact hit returns before segment iteration, texture lookup, bind-group
lookup, cache cleanup, vertex hashing and upload preparation. Backend recovery
must create a new epoch or a new image system; pointer identity alone is not a
device-loss contract.

### 2. Published segment change journal

`ScreenSpaceUiPlanCache` already knows which segment entries were reused and
which were rebuilt. Publish that fact with the new prepared frame rather than
making each renderer consumer compare all `Arc<PlannedScreenSpaceUi>` leaves.
The journal must distinguish:

- replacement/update at an index;
- appended leaves;
- truncated leaves;
- full fallback and its typed reason.

A one-segment replacement visits one image segment. A stable frame visits zero.
Insertions or order changes may conservatively publish a changed suffix if that
is what the planner actually rebuilt; they must not be mislabeled as a local
replacement.

### 3. Segment-owned binding product

Each retained image segment product must strongly own the bind groups and GPU
texture identities used by its draws. The binding cache is a discovery index,
not the lifetime authority. A safe first implementation is:

- cache entries own a shared binding product;
- segment dependencies clone the shared product on lookup;
- stable and unchanged segments retain it without an epoch touch;
- cleanup runs on insertion/explicit pressure and may remove only entries not
  pinned by a segment or in-flight submission;
- active bindings above the nominal discoverability limit remain valid and are
  reported, never removed before render.

A weak discovery map is also valid if upgrading it cannot create duplicate live
bindings and the segment/in-flight products remain the sole strong owners. A
raw integer handle into an evictable map is not valid unless the slot itself has
generation and pin ownership.

This lifetime is separate from the device-wide texture-allocation ledger. The
ledger accounts physical texture bytes; the segment product accounts the
bind-group and texture-reference lifetime required to encode the retained draw.

### 4. Typed fallback

Only these conditions may perform a full segment/dependency rebuild:

- resource-management generation change;
- backend/device epoch change or recovery;
- viewport change when geometry projection changes;
- explicit force-full-upload;
- malformed or unavailable change journal.

Each fallback records a reason, segment count and dependency count. An ordinary
one-segment delta or stable frame must never be reported as a full fallback.

### 5. Submission lifetime

Prepared segment products must remain pinned through command encoding and the
corresponding queue submission. Replacing the current frame may make an old
product undiscoverable, but must not release its resources while an in-flight
submission still references them. Reuse the existing upload/present transaction
boundary; do not add per-draw cloning as a substitute.

## Complexity and memory contract

Let `S` be segment count, `D_changed` dependencies in changed segments,
`D_all` all visible dependencies, and `B` binding-cache entries.

| State | Required CPU work | Forbidden residual work |
| --- | --- | --- |
| stable | `O(1)` frame-key check | `O(S)`, `O(D_all)`, or `O(B)` scan |
| local delta | `O(J + D_changed)` where `J` is the published journal size | unrelated segment/dependency walk |
| typed full fallback | `O(S + D_all)` | hidden fallback without reason/counters |
| pressure cleanup | bounded by configured cache pressure | cleanup on every stable frame |

Retained metadata must remain proportional to live segment products and unique
binding products, not frame count. Replaced generations are released after the
last frame/submission pin. Product validation must report retained generation
count, segment metadata bytes, binding-product count, cleanup visits, and
quiescent recovery.

The current-source deterministic fixture contains 4,096 frames, 64 segments,
four dependencies per segment, 32 one-segment deltas and four resource-
generation fallbacks. It models the following residual reduction:

| Operation | Current source | Target |
| --- | ---: | ---: |
| image segment visits | 262,144 | 288 |
| dependency/binding lookups | 1,048,576 | 1,152 |
| binding retention entry visits | 2,097,152 | 0 |

The 910.22x ratio is an operation-count model, not a timing claim. The target
includes all 256 segment visits and 1,024 dependency visits from the four typed
full fallbacks; it does not hide them in the delta result.

## TDD and acceptance order

1. Add a lower source contract proving image preparation receives the prepared
   frame identity and a published change journal. First demonstrate RED against
   the current segment-slice-only API.
2. Add pure lifetime tests for a binding product: stable retention without
   touches, changed-segment replacement, pressure cleanup of unpinned entries,
   protection of active/in-flight entries, and backend-epoch replacement.
3. Add lower prepare tests with counters:
   - stable frame: zero segment visits, dependency checks, binding lookups and
     retention scans;
   - one-segment delta: exact journal conservation and work equal to that
     segment's dependencies;
   - resource generation/backend recovery/viewport/forced upload: exact typed
     full fallback;
   - truncation and insertion: no stale draw or binding survives.
4. Run the existing fail-closed evidence analyzers
   `tools/analysis/performance/ui/ui_render_segment_evidence.py` and
   `tools/analysis/performance/ui/ui_render_dependency_delta_evidence.py`. Missing counters are errors,
   not zero.
5. Run managed lower Rust tests, then a current-source Editor product capture
   for stable hover, one-segment visual change, resize, SVG bucket revisit and
   device recovery. Record CPU, allocation count/bytes, RSS/private working set,
   GPU upload bytes, bind-group creation/cleanup, resource-generation churn and
   input-to-present p50/p95/p99.

At the 2026-08-31 source binding, production `image.rs` and the text segment
cache were externally dirty. This record intentionally did not modify or absorb
those changes. Shared integration later reconciled that ownership boundary. The
2026-09-01 artifacts below bind the exact current candidate contents, including
dirty-path metadata and per-file hashes. No Cargo validation was run for this
record.

## Current-source static revalidation (2026-08-31)

The dependency-product, retained-memory, delta-evidence, memory-evidence and
screen-space plan-cache suites pass 47/47 against current HEAD
`14c89f9776bed828cc85e05e4b9914b3f8d1e784`. Python bytecode compilation for
the five evidence/model tools also passes. The two production authorities
remain externally dirty and are bound by exact hashes:

- `image.rs`:
  `F1E0FD558DC9AC948163B976DBF46787FEC30C2F0319B1B15BF3F04DF2F99659`;
- `text/segment_cache.rs`:
  `D03393EB5A9CB914DC1B1E4A6DC09055BBA9ADBDF718371A6106E2C686478ED7`.

The current CPU operation-count artifact is
`E:/zircon-profiles/runtime-ui-render-dependency-product-pressure-20260831-current.json`
with SHA-256
`AA127CF8A82294E7E1342ACB43B975AF2A2EC96F84C7C27A2F436E87606BEEE3`.
For 4,096 frames, 64 segments, four image dependencies per segment, 32 local
deltas and four typed resource-generation fallbacks, current image preparation
still models 262,144 segment visits, 1,048,576 dependency/binding lookups and
2,097,152 binding-retention visits. The target publication performs 288
segment visits, 1,152 dependency checks and zero retention scans. The 910.22x
ratio is an operation-count reduction, not measured speedup.

The retained-memory artifact is
`E:/zircon-profiles/runtime-ui-render-dependency-product-memory-pressure-20260831-current.json`
with SHA-256
`26FAD4BB145F94E94CED5A12A5EC9D3C81921F19443F061063CAADC452DF1E47`.
With three retained generations and one changed segment per delta generation,
the target retains one 1,769,472-byte source payload, 55,296 bytes of changed
payload, and 26,352 bytes of metadata for 1,851,120 modeled bytes total. The
metadata remains below the explicit 8 MiB budget and independent of one million
present calls. A rejected full-generation clone would retain 5,308,416 payload
bytes, duplicating 3,483,648 bytes.

This revalidation completes the implementation-ready baseline and its
fail-closed acceptance gates. It does not change the production status:
stable-frame `O(1)` dependency publication is not implemented, and managed
Rust tests plus current-source CPU/allocation/RSS/GPU/input-to-present evidence
remain required.

## Current-source model correction (2026-09-01)

The v1 operation model correctly captured image `O(S + D + B)` work, but it
under-counted stable text preparation. Before `prepare_frame_product` can reuse
its retained product, `ScreenSpaceUiTextSystem::prepare` calls
`refresh_font_dependencies`. That method walks every segment, rebuilds the
ordered dependency entries, clears the active dependency vector and set, and
recomposes the complete active font set. The subsequent `frame_matches` still
compares every segment identity on a stable frame. A changed final segment also
forces a full identity walk before the frame dependency and run indexes are
rebuilt.

The corrected current-source artifact is
`E:/zircon-profiles/runtime-ui-render-dependency-product-pressure-20260901-r2.json`
with SHA-256
`ECA0C9A784194504985C16B184F26E36C58E7BA61BF9FC8C061C0CE03BB2B38B`.
It binds nine Zircon authorities plus four direct Unreal/Fyrox reference files
under source-set SHA-256
`E3802746D13E1592C0C764DCE330710130CA30F391C0FD146487150D2D6B6350`.
The bound production paths are clean at HEAD
`5798051603e7f7f565538125c9aba96d5beabae2`.

For the same 4,096-frame, 64-segment fixture, v2 additionally reports:

| Text operation | Current source | Target product |
| --- | ---: | ---: |
| stable segment identity checks | 259,840 | zero segment scans; 4,096 `O(1)` frame-key checks |
| tail-delta segment identity checks | 2,048 | journal-sized checks |
| font dependency segment visits | 262,144 | 288 |
| font dependency entry visits | 524,288 | 576 |
| delta glyph dependency entry visits | 65,536 | 1,024 |
| delta run entry visits | 16,384 | 256 |

The target requires the plan cache to publish one authoritative frame identity
and exact segment change journal. Image and text consumers must validate their
cached base generation before applying a journal; a skipped or failed consumer
cannot apply a delta against a different base. Font/resource readiness remains a
separate typed generation input. The target does not suppress polling or retry
for unresolved assets, upload failures, or atlas recovery. Those paths require
explicit counters and may perform bounded revalidation until ready.

This is still a deterministic operation-count model, not product timing. The
next implementation slice is the plan-cache publication contract and the image
binding lifetime product. Text downstream atlas work needs a separate audit
before any whole-text `O(1)` claim.

## Implementation status (2026-09-01)

The first production slice now implements the authority described above:

- `ScreenSpaceUiPlanCache` publishes a monotonic frame generation and an exact
  segment change journal. Vertex, image and font-dependency consumers validate
  the journal base generation before applying a local patch and otherwise take
  an explicit full rebuild path.
- Image segments strongly own an `Arc<ScreenSpaceUiImageBindingProduct>` that
  pins both the GPU texture and bind group. The discovery map evicts only
  unpinned entries under insertion pressure; the old per-frame retention sweep
  and raw integer binding handle are removed.
- `UiTextureDependencyCache` reuses the persistent 64-command leaves published
  by `UiRenderFrameExtract`. An exact submission is `O(1)`; a local visual
  change scans segment identities but visits commands only in replaced leaves.
- `UiTexturePrepareReceipt` retains prepared rows across frames. Added/removed
  dependencies are patched from the dependency journal, Ready rows are not
  polled per frame, upload/generation failures retry each frame, and pending or
  failed loads retry on readiness-generation change. A management-generation
  change remains an explicit full dependency fallback.
- Image invalidation consumes the receipt's binding-product generation instead
  of unrelated global readiness churn. Failed frame submission rollback clears
  the receipt after revoking texture publication, so an aborted upload cannot
  survive as a false Ready cache hit.
- Text stable-frame identity and active font dependency composition now consume
  the same generation/journal authority. Local text changes replace only their
  segment products, update glyph-key reference counts, and patch an aggregate
  tree instead of rebuilding frame glyph-fanout and run-span indexes that had no
  render consumer.
- A changed render segment now compares only its exact text preparation inputs.
  The collision-free comparison includes route/generation, geometry, glyph
  artifact identity, paint, language, shaping, decoration and clip state. A
  distinct plan whose text inputs are unchanged updates the retained plan `Arc`
  without rebuilding the text segment product.
- If every changed segment is text-stable, the cache advances only the prepared
  source generation and reuses the existing text frame-product `Arc` and text
  product generation. SDF atlas, CPU preparation and compiled-frame admission
  therefore retain their `O(1)` generation hit instead of rediscovering the
  same text from the complete frame.

The v7 deterministic fixture reports these implemented operation counts across
4,096 frames, 64 render segments, 1,024 UI commands, 32 one-leaf deltas and
four resource-generation fallbacks. Four local deltas are conservatively
classified as text-affecting; the remaining 28 are text-stable UI changes:

| Operation | Rejected pre-product path | Current implementation |
| --- | ---: | ---: |
| UI command dependency discovery | 4,194,304 | 512 |
| UI dependency prepare visits | 1,048,576 | 1,152 |
| image segment visits | 262,144 | 288 |
| image dependency/binding checks | 1,048,576 | 1,152 |
| image binding retention-map visits | 2,097,152 | 0 |
| text delta glyph dependency entries | 65,536 | 128 |
| text delta run entries | 16,384 | 32 |
| text frame segment `Arc` clones | 2,048 | 256 |
| downstream atlas/CPU/compiled-frame text entries | 196,608 | 24,576 |

A local-delta frame still compares retained leaf identities; the fixture records
2,048 such comparisons. It does not claim a journal-sized submission discovery
path that current source does not yet provide. Text admission performs 1,024
exact input comparisons, restricted to changed segments. The 28 text-stable
deltas avoid 1,792 segment `Arc` clones and 172,032 modeled downstream text-entry
visits.

True text changes remain an explicit residual: they publish a new frame product,
clone the 64 segment-product `Arc` values and cause the atlas/CPU/compiled-frame
consumers to validate or rebuild from the complete text frame. Converting those
consumers to a shared text-segment journal is the next text-specific milestone;
this slice intentionally does not claim it. The figures are deterministic
operation counts, not CPU or input-to-present timing.

Static validation currently consists of eight source-bound Python tests,
Python bytecode compilation, `rustfmt --check` on the changed Rust paths and
`git diff --check`. Cargo, managed lower tests and Editor product capture have
not run under the current validation gate, so this milestone is not accepted as
a measured responsiveness fix yet.

## Text downstream invalidation review (2026-09-01)

### Structural root cause

The remaining true-text-change cost is not one slow glyph loop. One
`ScreenSpaceUiTextFrameProductGeneration` currently acts as the invalidation
identity for four different products:

1. resolved/shaped text owned by a render segment;
2. SDF glyph residency and run-to-slot mapping;
3. CPU SDF run metrics plus native decoration metrics;
4. viewport-specific SDF vertices, materials and draw ranges.

These products do not have the same dependency set. A color, frame, clip or
effect change must recompile draw geometry, but it must not invalidate glyph
residency. A text or shaping change may alter glyph keys and CPU metrics. A
font revision invalidates every dependent product. A viewport change affects
compiled geometry but does not by itself require a new atlas slot. Treating all
four as one frame generation therefore turns a one-segment text delta into
three complete frame walks and can also rebuild products whose inputs did not
change.

Current source confirms the coupling:

- `sdf_atlas.rs` flattens every SDF segment, snapshots every text batch,
  recollects every glyph key and reconstructs every run when the frame
  generation changes;
- `text/sdf_cpu_frame.rs` compares or prepares every SDF and native text and
  replaces both complete output vectors;
- `sdf_render/compiled_frame.rs` uses the same generation as a whole-frame
  admission key, after which `sdf_render.rs` clears and rebuilds all vertices,
  text ranges, materials and draws;
- `text.rs` still passes every native segment to bitmap-atlas preparation on a
  true text-product change.

The lower SDF bake is already retained. `SdfFontBakeCache::prepared_atlas`
returns before glyph-cache lookup when atlas size and exact slot placements are
unchanged. The next repair must preserve that authority and avoid rebuilding
the higher-level slot/run plan; it must not add a second glyph bitmap cache.

### Reference-engine adjudication

Unreal Slate is the primary reference. `FShapedTextCache` retains shaped
sequences under an exact run/range/font/context key and derives reusable
subsequences. `FSlateFontCache::GetShapedGlyphFontAtlasData` and
`GetSdfGlyphFontAtlasData` first use a glyph-owned cached atlas handle, then a
process font-cache glyph key, and allocate only on a miss. Adding or editing one
text run does not repack every resident glyph. Atlas texture updates are a
separate cache-maintenance phase.

Slint is the secondary invalidation reference. Its `TextLayoutCache` is an
`ItemCache<Vec<TextParagraph>>`; the property tracker reevaluates only the dirty
item, while a scale-factor change explicitly clears the relevant cache class.
Fyrox retains `FormattedText` line and glyph buffers at the widget boundary and
keeps measure/arrange validity separate. Fyrox still rebuilds a widget's glyph
vector during arrange, so it is evidence for ownership and invalidation scope,
not the target renderer complexity.

The transferable rule is two-dimensional retention: text/run products are
owned by stable UI leaves, and glyph residency is owned by a persistent glyph
key cache. A frame generation may describe publication order, but it must not
be the only dependency identity for every downstream product.

### Target products and journal

`ScreenSpaceUiTextFrameProduct` remains the immutable publication consumed by
the renderer, but it must publish a typed
`ScreenSpaceUiTextFrameChangeJournal` containing:

- base and current text-product generations;
- exact changed segment indices;
- appended/truncated segment counts when order remains representable;
- a typed full-rebuild reason for font revision, viewport contract,
  unavailable/malformed journal, fallback mutation or explicit recovery.

Every downstream cache validates its retained base generation before applying
the journal. No consumer may infer a local patch by comparing every segment.
The journal is immutable and shared with the frame product, so one consumer
cannot consume or advance it for another.

Each consumer owns a distinct retained product:

- **atlas segment product:** exact SDF glyph-key set and ordered run-key rows;
- **CPU segment product:** SDF run preparations and native decoration metrics;
- **compiled segment product:** viewport-specific decorations, glyph vertices,
  material rows and draw ranges.

The atlas owner additionally retains glyph-key reference counts and stable
slot placement. Removal only decrements residency demand; it does not repack a
page. A new glyph allocates into retained free capacity or a new page. Page
pressure/eviction is a typed fallback and may relocate only the selected page;
it must never masquerade as a one-segment patch.

### Complexity contract

Let `J` be changed text segments, `G_J` their glyph-key entries, `R_J` their
text runs and `V_J` their compiled vertices. Let `G`, `R`, `V` be frame totals.

| State | Required work | Forbidden work |
| --- | --- | --- |
| stable frame | `O(1)` generation checks | segment, glyph, run or vertex scan |
| local paint/geometry text delta | `O(J + R_J + V_J)` | atlas-key or unrelated segment scan |
| local shaping delta with resident glyphs | `O(J + G_J + R_J + V_J)` | global slot rebuild or unrelated CPU run preparation |
| new glyph with retained page capacity | `O(J + G_J + R_J + V_J)` plus bounded allocation | atlas-wide repack |
| font revision / malformed journal / pressure relocation | typed `O(G + R + V)` fallback | silent full rebuild |

Run-count or vertex-count changes require an offset-directory update. They may
move a changed suffix in the first implementation, but must report the moved
entry/byte count. They are not permitted to recompute unrelated shaping, glyph
keys or CPU metrics. The final compiled representation should use segment-owned
vertex ranges and partial buffer writes so same-cardinality local changes write
only `V_J` bytes.

### Milestone order

- **T0 evidence and plan:** bind current Zircon and reference sources, extend
  the deterministic pressure model to distinguish paint-only, resident-glyph,
  new-glyph and typed fallback deltas, and reject timing claims.
- **T1 publication:** add the immutable text frame journal and lower tests for
  exact base/current generation, changed indices, append/truncate and typed
  fallback. Existing stable-frame `Arc` reuse remains unchanged.
- **T2 atlas product:** retain per-segment key/run products, key reference
  counts, stable slot placement and run directories. Prove a resident-glyph
  one-segment edit performs no global key collection, slot rebuild or SDF bake.
- **T3 CPU product:** retain CPU outputs per segment and patch only journal
  entries. Cardinality changes update the output directory without recomputing
  unrelated runs.
- **T4 compiled product:** retain segment vertex/material/draw products and use
  partial buffer writes for same-capacity deltas. Atlas page relocation or
  viewport/device recovery remains typed full compilation.
- **T5 product acceptance:** run managed lower tests and a current-source
  Editor capture for stable hover, button state, resize, one-label paint edit,
  one-label content edit, new glyph, font reload and device recovery. Capture
  CPU/allocation/RSS, glyph-key/run/vertex visits, atlas allocation and upload
  bytes, partial buffer writes, and input-to-present p50/p95/p99.

No T1-T4 source slice is accepted from static checks alone. The current Cargo
lane restriction remains in force; T0 may progress with source guards and
deterministic models, while managed Rust and product timing wait for explicit
authorization.

## 状态与产出记录

| Milestone | Status | Date | Evidence |
| --- | --- | --- | --- |
| T0 evidence and plan | Static GREEN; dynamic acceptance pending | 2026-09-01 | `E:\zircon-profiles\runtime-ui-render-dependency-product-pressure-20260901-r8.json`; schema v9; artifact SHA-256 `76CF8BD949D6966573532C9149AE282A4A50689BAE22611E0C4255E99EEB3A74`; source-set SHA-256 `585C9667CC2F96DAB7B4D4DB07798C4C4B81D624979C7D6C904E639720BC14D1`; source binding is current HEAD `9963f8eb72e2d725d2536eb50b393b30387a1ffa`; deterministic model tests 8/8. Default scenario records current 24,576 downstream consumer-entry visits versus the target partition of 96 atlas glyph-key, 24 atlas-run, 32 CPU-run and 128 compiled glyph/vertex visits. These are operation counts, not timing claims. |
| T1 publication | Static GREEN; managed Rust validation pending | 2026-09-01 | `ScreenSpaceUiTextFrameChangeJournal` is published with base/current generations, exact changed segment indices, append/truncate topology and typed full-rebuild reasons. Lower journal tests and source guards are present; no Cargo run. |
| T2 atlas product | Static GREEN; managed Rust validation pending | 2026-09-01 | `SdfAtlasSegmentProductIndex` retains per-segment key/run products, reference counts, stable slots and local run patches. Resident glyph changes avoid global key collection and slot repacking in the static contract; pressure/relocation remains typed fallback. No Cargo run. |
| T3 CPU product | Static GREEN; managed Rust validation pending | 2026-09-01 | `ScreenSpaceUiSdfCpuFrameIndex` retains SDF/native metric products and patches journal entries without a full segment scan. Current implementation has a typed full fallback for incompatible topology; no Cargo run. |
| T4 compiled product | Static GREEN; managed Rust validation pending | 2026-09-01 | `SdfCompiledTextSegmentIndex` owns segment ranges and applies same-cardinality replacements; vertex and aligned material buffers receive coalesced partial writes. Static source guards cover the no-all-segment-scan path and lower helpers cover unchanged-segment preservation and pre-mutation cardinality rejection. Structural cardinality changes still use typed full compiled fallback, and per-segment material/draw ownership may reduce cross-segment batching; both are T5 measurement gates. No Cargo run. |

### Current implementation limits

The journal currently reports that a changed text segment changed, but does not
carry a consumer-specific semantic subtype. Therefore the static candidate still
visits 128 atlas glyph keys for the default four text deltas (paint, resident
glyph and new glyph partitions combined), while the target model is 96. This is
an honest residual, not a claim that paint-only changes are already atlas-free.

The compiled product deliberately accepts only same-cardinality local patches.
If a text edit changes glyph, decoration or draw cardinality, the renderer
returns to the explicit full compiled path; atlas and CPU products remain
separately retained where their journals permit it. A follow-up suffix-directory
slice may move only affected output ranges, but it must report moved bytes before
being accepted.

The segment-owned draw plan preserves deterministic ownership and local material
ranges, but can prevent adjacent equal materials from being merged across
segments. T5 must compare draw count and input-to-present latency against the
pre-slice renderer before this representation is treated as a net performance
win.

All current evidence is static: scoped `rustfmt --check`, `git diff --check`,
Python source-bound tests (8/8), and bytecode compilation. Cargo, managed lower
tests, and Editor product capture remain pending explicit lane authorization.
