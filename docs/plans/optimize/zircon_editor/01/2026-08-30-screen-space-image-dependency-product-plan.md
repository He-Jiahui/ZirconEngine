---
source_binding:
  head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
  image_sha256: F1E0FD558DC9AC948163B976DBF46787FEC30C2F0319B1B15BF3F04DF2F99659
  plan_cache_sha256: 200179577E6F4A7FBE5EC88D2CDF497036ACEE3AD237750D0FC7D2D4C9D09968
  record_sha256: 8B52434C556746970A0A3B080FB2F82A70DB68BADB5C3D11FDC7E11AB498E2FB
evidence:
  pressure_artifact: E:/zircon-profiles/runtime-ui-render-dependency-product-pressure-20260831-current.json
  pressure_artifact_sha256: AA127CF8A82294E7E1342ACB43B975AF2A2EC96F84C7C27A2F436E87606BEEE3
  memory_artifact: E:/zircon-profiles/runtime-ui-render-dependency-product-memory-pressure-20260831-current.json
  memory_artifact_sha256: 26FAD4BB145F94E94CED5A12A5EC9D3C81921F19443F061063CAADC452DF1E47
status: reviewed_design_ready_owner_reconciliation_required
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
   `tools/ui_render_segment_evidence.py` and
   `tools/ui_render_dependency_delta_evidence.py`. Missing counters are errors,
   not zero.
5. Run managed lower Rust tests, then a current-source Editor product capture
   for stable hover, one-segment visual change, resize, SVG bucket revisit and
   device recovery. Record CPU, allocation count/bytes, RSS/private working set,
   GPU upload bytes, bind-group creation/cleanup, resource-generation churn and
   input-to-present p50/p95/p99.

The production `image.rs` and text segment cache are externally dirty at this
source binding. This record intentionally does not modify or absorb those
changes. Implementation starts only after their owner reconciles the current
candidate or transfers the exact paths. No Cargo validation was run for this
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
