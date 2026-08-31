---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/async_loader.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/icon_atlas.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/retained_host/ui_perf/counter_catalog.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache/resource.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry/allocation_ledger.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs
  - zircon_runtime/crates/zr_rhi/src/ui_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/viewport_product_registry.rs
  - tools/editor_svg_gpu_residency_pressure.py
  - tools/tests/test_editor_svg_gpu_residency_design_contract.py
  - tools/tests/test_editor_svg_gpu_residency_pressure.py
  - tools/ui-profile-counter-evidence.ps1
  - tools/ui_svg_cache_evidence.py
  - tools/tests/test_ui_svg_cache_evidence.py
source_binding:
reviewed_head: 050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f
  reviewed_dirty_path_count: 15241
status: gpu_device_ledger_counter_and_fail_closed_evidence_gate_static_validated_managed_rust_and_product_pending
latest_source_binding_artifact:
  path: E:/zircon-profiles/editor-svg-gpu-residency-pressure-20260831-r16.json
  sha256: DE6CDF732A3F9BDBD52644188C3447A0AB95B199203F38B0386828424A5EE276
---

# SVG CPU product and GPU residency acceptance

## Finding

The current implementation already has five distinct retained layers:

1. `SvgTreeCache` retains `Arc<usvg::Tree>` by normalized source path and source
   fingerprint.
2. The visual pixel cache retains raster products by logical asset, physical
   target size, tint, and content identity.
3. The editor icon atlas retains immutable RGBA pages and redirects commands to
   stable page identities.
4. The device-scoped shared image registry retains GPU texture allocations by
   resource key and generation.
5. Each surface retains texture handles and bind groups by the same identity,
   and reports upload, shared-resolve, and resident-byte counters.

That structure does not by itself prove that the product path is stable. The
existing idle-hover gate requires SVG tree hits, zero tree misses, and zero GPU
uploads, but SVG parse and raster work are only profile scopes. They are not
scenario-prefixed counters and are therefore invisible to the gate.

A historical trace demonstrates the false-green shape. It is not current-source
acceptance evidence:

- artifact:
  `E:/zircon-profiles/runtime09-svg-m7/shell-content-presentation-patch/20260811-201002-click-dock-patch-spaced/timeline.zrtrace.json`
- SHA-256:
  `955D689B187B7D45AA5E152CAEC74C941E5670997EFF3A165C57F3268FDC8AC7`
- `visual_assets_render_svg_parse`: 32 spans, 342,607 us total
- `visual_assets_render_svg_raster`: 51 spans, 2,392 us total
- `visual_assets_svg_tree_cache_lookup`: 51 spans, 273 us total
- idle-hover GPU evidence: 9 image prepare cache hits, 0 prepare command
  visits, 0 upload writes, and 0 cache-key allocations

Thus zero GPU uploads does not imply zero SVG parse/raster work. One plausible
mechanism is that repeated CPU rasterization produces an already-resident
resource identity, so the GPU layer correctly avoids another upload. Asset and
target identities are absent from the historical spans, so this remains a
hypothesis rather than a root-cause claim.

## Required authority

The accepted path must keep one product identity across all layers:

```text
source generation
  -> parsed Arc<usvg::Tree>
  -> (source generation, physical size bucket, tint, raster policy)
  -> shared Arc<[u8]> raster product + stable resource key
  -> immutable atlas page identity (for packed editor icons)
  -> (resource key, resource generation)
  -> device-scoped GPU allocation identity
  -> surface-local bind product referencing that allocation
```

Stable paint may look up these products, but it must not read the SVG file,
parse the tree, rasterize pixels, hash a fresh pixel buffer, allocate a new GPU
key, or upload bytes. Source invalidation must advance only affected source
generations. Window resize may create one raster/GPU product for each newly
entered physical-size bucket; revisiting a bucket must reuse the existing
product. A full cache clear is not an acceptable response to one changed SVG.

## Instrumentation contract

Add scenario-prefixed counters at the existing authority boundaries, not in a
parallel cache:

| counter | meaning |
| --- | --- |
| `svg_parse_count` / `svg_parse_bytes` | file parse work after tree-cache miss |
| `svg_raster_count` / `svg_raster_pixels` | CPU vector raster work after pixel-product miss |
| `svg_raster_product_hit_count` | retained raster product reused without pixel materialization |
| `svg_raster_product_miss_count` | a new physical-size/tint product was required |
| `svg_raster_unique_bucket_count` | distinct product keys admitted in the measured phase |

The counters must use `record_current_ui_perf_counter`, so startup warmup is not
attributed to measured `idle_hover`, `window_resize`, or `click`. The evidence
artifact must bind the exact source revision, dirty overlay paths, scenario,
warmup count, measured run ordinal, and relevant source hashes.

## Acceptance matrix

| scenario | required result |
| --- | --- |
| stable hover after warmup | tree miss, parse, raster, raster pixels, pixel-product miss, GPU upload/shared upload, GPU key allocation, prepare source visit, and external-provider resolve are all 0; tree/product/prepare hits are positive and exactly one provider-revision check is allowed per present |
| click with unchanged icon state | same zero-work constraints as stable hover |
| resize across new physical buckets | parse remains 0; raster misses, raster count, and GPU uploads are no greater than distinct new buckets; duplicate sizes do no work |
| resize returning to a prior bucket | parse, raster, pixel-product miss, key allocation, and GPU upload are all 0 |
| one SVG source change | exactly the affected source generation is invalidated; unrelated tree/raster/GPU products remain resident |
| sequential disjoint multi-window working sets | device-wide unique allocation bytes remain within the configured device budget; evicted registry entries cannot stay alive outside the device ledger solely through surface caches; shared and surface counters identify double counting |

Any missing required counter is a typed evidence failure, not numeric zero. A
current-source profile must also report CPU p50/p95/p99, input-to-present p95,
GPU time, upload bytes, and RSS delta. The historical trace above may only be
used as a regression fixture for the analyzer.

## Implementation order

1. Land the scenario counters in the existing `UiPerfCounter` catalog and the
   existing parse/raster authority once those externally modified files return
   to their owner.
2. Extend the current visual-cache gate to fail closed on missing parse/raster
   counters and enforce the matrix above.
3. Capture current-source stable-hover and resize evidence before changing the
   cache algorithm.
4. If tree misses remain, repair source-generation/path canonicalization. If
   tree hits coexist with raster misses, repair raster product key/publication.
   If CPU products hit but uploads recur, repair GPU resource generation or
   residency. Do not add a fourth cache.
5. Add one device-scoped allocation identity/byte ledger and make surface bind
   products explicit pins. Registry eviction is complete only after all surface
   and in-flight pins release; per-surface logical budgets must not masquerade
   as unique GPU allocation bytes.
6. Give the external image provider a monotonic product revision. Cache the
   complete prepare product by draw-list, provider, and device generations;
   unchanged generations must return before the per-source resolve loop.
7. Run the same product and multi-window scenarios after the repair and accept
   only source-bound measured evidence.

The involved production and capture paths had external worktree changes during
this review, so this slice deliberately records the contract without editing
those owners or starting Cargo.

## Implemented fail-closed analyzer

`tools/ui_svg_cache_evidence.py` now enforces the stable-interaction subset of
the matrix without changing the externally owned capture or production paths.
It aggregates repeated samples for one scenario, requires all retained-hit and
zero-work counters to be present, rejects non-integral or non-finite values,
and validates the measured-run source manifest against every relevant owner.
Capture-wide parse/raster spans are reported only as diagnostics because they
cannot prove which scenario performed the work.

The focused contract first failed 9 tests while the analyzer/report API and
per-owner fingerprint validation were absent and now passes 9/9. The analyzer
and test SHA-256 values are
`A378AB5406FAF5CEC9709F310A8A38F9CC3AB998DF217A58E261B3D3433BF874`
and `B12B9CDB6CB6D2FA513D22860B5AE81A88E5FB91A8AEBA4F706FEE0F2F969860`.
Replaying the historical trace returns 13 `missing_counter` blockers plus one
`missing_source_manifest` blocker while preserving the 32 parse spans and 51
raster spans as diagnostic-only observations. The rejection artifact is
`E:/zircon-profiles/ui-svg-cache-evidence-20260829-historical-regression.json`
with SHA-256
`9E54EDEC6E1A9BBF328E1FD508627405E3815786B5A671EF7B2BA150CA3249EA`.
This closes the evidence false-green, not the production cache performance.

## Async invalidation repair (2026-08-30)

The retained lookup and GPU residency layers were not the only structural gap.
The reviewed background visual-asset loader had same-key pending deduplication,
but targeted cache invalidation advanced one global
`VISUAL_ASSET_CACHE_EPOCH` before it knew whether a resident product was
affected. Before the repair:

- `loading/cache.rs:82-92` advances the epoch for every non-empty path list and
  only then resolves aliases, compares content fingerprints, and removes
  affected base keys.
- `loading/cache.rs:96-104` advances the same epoch for every reconcile pass.
- `loading/async_loader.rs:227-235` captures that global epoch in every request.
- `loading/async_loader.rs:287-300` rejects the completed product whenever any
  later path event advanced the epoch, even if the event was for an unrelated
  source or the affected file content was unchanged.
- The stale completion releases the pending key and wakes the host. The next
  paint still misses and schedules the same raster product again.

The resulting failure loop is:

```text
SVG product miss -> background parse/raster
  -> unrelated or content-unchanged path event advances global epoch
  -> completed product is discarded as stale
  -> completion wake requests paint
  -> same SVG product misses and is enqueued again
```

This was a concrete mechanism capable of producing repeated SVG rasterization
while the GPU cache remained correct and reported zero repeated uploads. The
current candidate removes that amplification mechanism. Product evidence is
still required to determine how much of the observed interaction stall it
caused because the current capture does not publish scenario-prefixed
parse/raster counts or the invalidating source identity.

Unreal Slate provides the relevant lifecycle precedent. Its vector cache key
is `(BrushName, PixelSize)` (`SlateVectorGraphicsCache.h:43-68`); a paint-time
miss inserts one proxy and one pending request
(`SlateVectorGraphicsCache.cpp:45-70`). `UpdateCache` rasterizes pending SVGs
in parallel and publishes them into retained atlases
(`SlateVectorGraphicsCache.cpp:78-174`). It clears the resource map only after
an explicit deferred flush request (`:177-220`). The important precedent is
not a particular size-bucket policy. It is that unrelated resource activity
does not invalidate every pending vector product.

### Implemented ownership split

The candidate keeps stale protection without adding another pixel cache. Its
invalidation authority is split into two levels:

1. The global cache epoch now advances only for a true full pixel-cache clear.
   Loader replacement remains protected by the existing binding epoch.
2. Each logical visual base key has a monotonic source generation. Candidate
   paths are normalized through the existing alias index and registered on the
   worker before filesystem probing, so the UI event path does not hash SVG
   files when it schedules a miss.
3. Targeted refresh first resolves affected sources and compares content
   fingerprints. Only byte-changing, appearing, or disappearing sources
   advance their generations and invalidate dependent products.
4. Completion publication requires the same binding epoch, global-clear epoch,
   and captured logical-source generation. Unrelated and content-unchanged
   events preserve the product.
5. `source_base_keys` is the reverse alias-to-logical-product authority.
   `pending_base_loads` keeps that relation alive while variants are in flight;
   its key includes the clear epoch and source generation, so a stale
   completion after cache clear cannot release a new task. Completion,
   cancellation, and stale binding paths release only their own token. A
   changed source therefore stales only dependent logical products, while
   lagged event streams reconcile tracked fingerprints without a global
   cutover.

This protocol preserves race correctness: a product rendered from bytes that
changed during the job cannot publish, while unrelated work is no longer
discarded. Targeted invalidation is `O(A + D * S + K log E)`, where `A` is the
changed path's alias count, `D` its dependent logical products, `S` the
candidate sources per product, and `K` the resident variants removed. Async
publication checks one logical generation rather than scanning the cache.

### Source-bound pressure evidence

`tools/editor_svg_gpu_residency_pressure.py` now binds nine current-source
authorities and fails closed if their source-generation, async completion,
refresh, SVG raster, atlas, shared-allocation, surface-reference, or Surface
construction contracts change. It explicitly rejects a targeted/reconcile
function that advances the global clear epoch and an async loader that captures
that epoch. The pressure tests cover the historical global-epoch amplification,
the current source-generation authority, the no-event baseline, invalid inputs,
current-source hashing, a changed-source guard, and multi-Surface allocation
reachability (8/8). The design-contract and evidence suites bring the focused
static total to 23/23.

Artifact:

- path:
  `E:/zircon-profiles/editor-svg-gpu-residency-pressure-20260830-r11.json`
- SHA-256:
  `6B1070F9D17222A33083456755F526E23EE8527F76078B0C6906E4BF5A31D563`
- bound revision:
  `cc5cadbd597c3707954ebd6109fad0fd5643a152`
- critical source-set SHA-256:
  `E04DE986015E6D433DD430FC72D47F01BCA269D9182AAFA87B9DDAFA94D61CB3`
- critical sources dirty: `true`

The default deterministic pressure case uses 256 in-flight SVG products, 100
unrelated path events, 100 content-unchanged path events, and one real source
change affecting four products. It assumes each event lands before one
in-flight wave publishes and that a discarded wave is repainted and
rescheduled:

| protocol | stale completions | raster attempts |
| --- | ---: | ---: |
| historical removed global epoch | 51,456 | 51,712 |
| current source-generation contract | 4 | 260 |
| avoided by current candidate | 51,452 | 51,452 |

This is an algorithm-pressure model, not product timing. It excludes job
latency, event coalescing, actual source counts, CPU/GPU time, allocator and
lock contention, RSS, and the probability of the modeled interleaving. Product
acceptance therefore still requires scenario-prefixed `svg_parse_count`,
`svg_raster_count`, `visual_asset_async_enqueued_count`,
`visual_asset_async_completed_count`, and
`visual_asset_async_stale_discard_count`, together with invalidating-source
top-N evidence and CPU/input-to-present/RSS/GPU percentiles.

Static candidate validation on 2026-08-30:

- the source-generation design contract was observed RED against the old
  global-epoch implementation, then GREEN after the repair;
- design, pressure, and fail-closed evidence suites pass 23/23;
- exact-file Rust formatting, Python compilation, and `git diff --check` pass;
- lower-layer Rust regressions are present for unrelated changed sources,
  unchanged events, pending generation eligibility, targeted resident
  invalidation, missing-source appearance, lagged reconciliation, and stale
  completion after cache clear;
- managed Rust compilation and product-path profiling remain pending. No raw
  Cargo command was used for this candidate.

### Current-source multi-Surface residency finding

The device-scoped shared image registry has a 64 MiB map budget, but a registry
eviction does not necessarily release the GPU texture. Every surface has its
own 64 MiB `WgpuUiImageCache`; an external image becomes a local
`WgpuUiImageResource` whose texture handle and bind group keep the device
allocation alive after the registry drops its map entry. The local resource
correctly retains zero CPU RGBA bytes, but its `resident_bytes` still describes
a logical reference budget rather than a new physical allocation.

The default source-bound pressure case uses 16 surfaces, each sequentially
admitting a disjoint full-budget working set. The statically reachable result
is:

| quantity | bytes |
| --- | ---: |
| shared registry reports | 67,108,864 (64 MiB) |
| all surface caches report | 1,073,741,824 (1 GiB) |
| unique GPU allocations retained by surfaces | 1,073,741,824 (1 GiB) |
| naive shared + surface counter sum | 1,140,850,688 (1.0625 GiB) |
| known double count in naive sum | 67,108,864 (64 MiB) |

Thus the 64 MiB shared registry ceiling is not a device-wide physical GPU
budget, and summing current counters is not a valid substitute: it double
counts current shared products while omitting the allocation identity needed
to distinguish registry-owned from surface-pinned evicted textures. This is a
static reachability scenario, not measured driver residency.

The repair must keep one device allocation identity and byte ledger
authoritative until all surface and in-flight pins release. Surface caches may
retain layout-specific bind products, but those references must remain visible
to the device budget. Required product evidence adds unique allocation bytes,
pinned-but-registry-evicted bytes, surface pin counts, eviction completion, and
multi-window churn RSS/GPU percentiles. Reducing the numeric limit alone cannot
repair this ownership gap.

The fail-closed stable-product analyzer now treats async stale discard as work,
not as an optional diagnostic. Its source manifest also requires the visual
pixel cache, async loader, and asset-refresh classifier in addition to the
tree/raster/GPU/capture owners. The focused analyzer suite passes 10/10.
Replaying the historical trace with this contract yields 14
`missing_counter` blockers plus `missing_source_manifest`; one missing counter
is explicitly
`ui.idle_hover.visual_asset_async_stale_discard_count`. The rejection artifact
is
`E:/zircon-profiles/ui-svg-cache-evidence-20260830-historical-epoch-gate.json`
with SHA-256
`547ADC02E1483330DAA99BB5495D1EC6CEA53C2AE6D959F93E2CD72EBB7627B8`.
This remains a rejection fixture, not evidence that the historical executable
did or did not experience stale completion churn.

### Device allocation ledger candidate (2026-08-30)

The multi-Surface ownership gap above is now repaired in the current static
candidate. The registry contains one device-scoped allocation ledger, and each
allocation record directly owns its `wgpu::Texture`. Registry lookup, Surface
bind products, the prepared frame dependency set, and submitted GPU work hold
typed pins to that same record. Dropping a registry entry therefore removes
discoverability but cannot remove the allocation from the physical byte budget
until the final Surface or GPU-completion pin releases it.

The Surface image cache no longer has a CPU-image fallback that creates a
second presenter-local texture. CPU pixels are admitted only through the
device registry; a device-budget rejection invalidates the stale local bind
product and remains retryable. Surface resources retain only the bind group,
logical size/identity, and an optional shared allocation pin. Runtime-renderer
external images remain owned by their existing RHI producer and are not copied
into a Surface texture.

Submission lifetime is explicit. A full or damage present takes an O(1) clone
of its prepared allocation-set identity before queue submission, then hands the
guard to `Queue::on_submitted_work_done`. Retained projection copies do not
sample image resources and take no image pin. Rebuilding the dependency set is
O(I) only when image preparation already visits the `I` compiled resource
identities; an unchanged prepared generation reuses it in O(1).

Normal registry and Surface admission now read maintained entry/byte counters
in O(1). Only an actual capacity violation enumerates the bounded registry or
Surface cache (at most 256 entries) and sorts LRU candidates in O(R log R).
Pinned registry evictions reduce lookup membership but not unique physical
bytes. If no immediately releasable candidate can satisfy the 64 MiB device
budget, admission rejects instead of escaping into a per-Surface allocation.

The design follows the reference-engine ownership boundary recorded during
review: Unreal Slate separates map removal from RHI release and flushes render
work before destructive resource teardown; Fyrox keeps texture allocation
cache authority at renderer scope rather than per window. Zircon's typed pins
provide the same lifetime separation without an event-path scan or synchronous
GPU wait.

The presenter now reports these distinct gauges/counters through
`UiSurfacePresentStats`:

- `image_device_allocation_count`
- `image_device_allocation_bytes`
- `image_registry_evicted_pinned_bytes`
- `image_surface_pin_count`
- `image_in_flight_present_pin_count`
- `image_eviction_completion_count`

Existing `image_shared_resident_bytes` remains registry discoverability, while
`image_cache_resident_bytes` remains a per-Surface logical reference measure.
They must not be summed as physical GPU memory. The six device-ledger fields now
flow through the existing Editor presenter batch into scenario-prefixed
`ui.<scenario>.gpu_image_*` counters. The batch retains exact capacity ownership:
52 counters are unconditional, with the existing optional timestamp counter and
two optional GPU-time counters bringing the full batch to 55. This removes the
counter-wiring blocker for a later multi-window product capture without adding a
second stats path or any per-counter string formatting on the present hot path.

The SVG/GPU evidence analyzer is now schema v2 and consumes those six series
with their actual semantics instead of adding them as event totals. Allocation
bytes/count and pin values are gauges reported as peak/final; eviction
completion is a cumulative series whose samples must be monotonic. The gate
fails closed when any series is missing or misaligned, physical bytes exceed
the source contract's 64 MiB device budget, evicted-pinned bytes exceed total
allocation bytes, or pins exist without an allocation. An explicit
`--require-quiescent` capture additionally requires final evicted-pinned bytes
and in-flight present pins to reach zero. Its source manifest now requires all
14 CPU cache, Editor counter, RHI ledger, submission-lifetime, and capture-tool
owners, so a counter-only overlay cannot be accepted against an older runtime.

The v4 deterministic pressure model preserves the old reachable state as a
regression baseline and applies the new admission contract:

| 16-Surface disjoint full-budget scenario | bytes |
| --- | ---: |
| pre-ledger reachable unique allocations | 1,073,741,824 (1 GiB) |
| device-ledger unique allocations | 67,108,864 (64 MiB) |
| explicitly rejected later working sets | 1,006,632,960 (960 MiB) |
| physical budget overshoot | 0 |

Source-bound artifact:

- path:
  `E:/zircon-profiles/editor-svg-gpu-residency-pressure-20260830-r13.json`
- SHA-256:
  `5C68053DADC12871B4963CC29707BCB01AA24517243324C068678B88DCD6AD07`
- schema: `zircon.editor.svg_gpu_residency_pressure.v4`
- bound revision: `399f2318150ae4fa0df3a2543133b03b80099288`
- critical source-set SHA-256:
  `17033A878C3B02A5BDA9F9210BF295455FDBF66A53AEF6F00D8C95B29F49DC0C`
- critical sources: 15; dirty source entries: 15. The manifest now binds the
  Runtime ledger fields and all three Editor counter-consumer owners together.

The lower design contract was observed RED before the ledger module existed,
and the multi-Surface pressure contract was observed RED while the model still
described the old 1 GiB current behavior. The design, pressure, and fail-closed
evidence suites now pass 27/27. Exact-file Rust formatting, Python compilation,
and tracked-path `git diff --check` pass. Managed Rust compilation, lower GPU
tests, Editor product profiling, and multi-window churn RSS/GPU percentiles
remain required before this milestone can be accepted.
No raw Cargo command was used for this candidate.

## Current-source static revalidation (2026-08-31)

The pressure model was regenerated against current HEAD
`050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f` and the exact 20-file dirty
overlay identified by the model. Its source-set SHA-256 is
`8B1DCC8F5264CF21735AEBE92AD953A3E647C53BB5634E3BFED81D2D49E8926B`, so
the source-generation invalidation and device-ledger guards still match the
reviewed candidate rather than a historical checkout.

The current artifact is
`E:/zircon-profiles/editor-svg-gpu-residency-pressure-20260831-r16.json`
with SHA-256
`DE6CDF732A3F9BDBD52644188C3447A0AB95B199203F38B0386828424A5EE276`.
The deterministic default still reports:

- unrelated and content-unchanged path events cause zero stale SVG-product
  discards under the current source-generation contract;
- one changed source invalidates four dependent products, reducing the modeled
  raster attempts from the removed global-epoch baseline's 51,712 to 260;
- 16 disjoint Surface working sets retain at most 64 MiB of unique device
  allocation, reject 960 MiB that cannot fit, and report zero physical-budget
  overshoot;
- stable materialization work is zero. The 20,480,000 image-command visits are
  a conceptual upper bound retained for comparison with the removed
  reconstruction baseline, not the current WGPU prepare-loop count;
- the product framework installs an external viewport-product provider on every
  UI Surface. Because that provider has no readiness/product revision, the
  generation fast path is disabled and the default 10,000-present, 16-source
  scenario executes 160,000 provider resolves and 160,000 registry lock
  acquisitions. This is retained dependency-query work, not repeated SVG file
  loading, parsing, rasterization, or texture upload.

The 20-file manifest includes the device submission owners plus the product
provider construction and registry-resolution owners. A UI Surface now
pins the prepared allocation set into a native submission packet; the render
device transfers those pins with the packet ticket, registers them after
`queue.submit`, and drops them only when the shared completion callback retires
the completed tickets. The Surface no longer owns a parallel
`on_submitted_work_done` callback. This keeps device allocation lifetime on the
same submission timeline as other GPU work and preserves the original
device-ledger invariant after the owner migration.

The remaining structural repair is not to bypass the provider. The provider
must publish a monotonic revision for its complete immutable product snapshot;
the image cache then retains the prepared allocation/bind set under
`(draw-list generation, provider revision, device generation)`. An unchanged
tuple performs one O(1) revision check per present and returns before the source
loop; a changed provider revision invalidates external dependencies and permits
the existing CPU fallback/readiness behavior to run. In the default model this
reduces provider resolves from 160,000 to 0 while retaining 10,000 revision
checks, changing stable dependency work from `O(P * R)` to `O(P)`. This follows
the Unreal Slate boundary where stable resource proxies are owned and versioned
by the resource manager rather than rediscovered by each draw operation.

The focused fail-closed evidence, invalidation pressure, and device-ledger
design suites pass 28/28, including the packet/submission/retirement ordering
guards and the external-provider fast-path pressure contract. Python bytecode
compilation also passes. This validates the static
ownership and evidence contracts only. Managed Rust tests, current-source
stable-hover/resize traces, CPU/RSS/GPU percentiles, scenario-prefixed
parse/raster counters, and multi-window eviction churn remain required;
therefore the SVG/GPU milestone is still a static candidate and not
product-accepted.
