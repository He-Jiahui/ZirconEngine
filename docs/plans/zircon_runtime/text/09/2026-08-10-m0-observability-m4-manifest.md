Plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
Milestone: M4
Status: measurement_contract_forward_fix_implementation_complete_secondary_review_complete_coordinator_atomic_staging_required_managed_validation_pending
Files: ["zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs", "zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/state.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests/prepare_report.rs", "zircon_runtime/src/graphics/tests/render_profiling.rs", "zircon_runtime/src/graphics/tests/render_profiling/text_baseline.rs", "zircon_runtime/src/graphics/tests/render_profiling/text_baseline/localized_text_dirty.rs", "zircon_runtime/src/graphics/tests/render_profiling/text_baseline/layout_cache_pressure.rs", "zircon_runtime/src/text/native_bitmap_atlas.rs", "zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs", "zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache/residency.rs", "zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache/worker_requests.rs", "zircon_runtime/src/text/parallel/completion_queue.rs", "zircon_runtime/src/text/parallel/raster_pool.rs", "zircon_runtime/src/text/parallel/raster_pool/diagnostics.rs", "zircon_runtime/src/text/parallel/raster_pool/worker.rs", "zircon_runtime/src/text/parallel/shape_pool.rs", "zircon_runtime/src/text/parallel/shape_pool/tests.rs", "zircon_runtime/src/text/parallel/tests.rs", "zircon_runtime/src/text/render_state.rs", "zircon_runtime/src/ui/surface/render/extract.rs", "zircon_runtime/src/ui/surface/render/text_prewarm.rs", "zircon_runtime/src/ui/surface/render/text_prewarm/profile.rs", "zircon_runtime/src/ui/surface/render/text_prewarm/tests.rs", "zircon_runtime/src/ui/surface/render/text_prewarm/tests/profile.rs", "zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs", "zircon_runtime/src/ui/text/measure_cache.rs", "zircon_runtime/src/ui/text/resolved_layout.rs"]

# Text09 M4 Observability Completion Manifest

## Scope Delivered

The fixed M0 profiler stages now project the existing native atlas visible-instance and
draw-command reports alongside cache, worker, upload, and SDF counters. UI extraction projects
prewarm outcomes and the post-prewarm layout shaped-cache hit/miss/lookup/insert delta through a
folder-backed profiling owner. Empty prewarm batches publish the same seven fixed zero counters,
so stable-frame samples retain a complete stage trajectory. The projections add no recorder work
when profiling is disabled and preserve the existing renderer, cache, and atlas ownership
boundaries.

UI extraction also records the caller-thread `owner_prewarm_request_collection` and
`owner_prewarm_overlap_admission` spans before `render_command_collection`. The 8-owner profiling
regression requires each new span to be published exactly once on the calling frame, so a p95
trace can distinguish request projection and compatibility scanning from actual shaping without
adding a cache, worker, or renderer boundary.

The ignored 300-frame baseline applies the same exactness rule to every asserted span and counter:
each must publish exactly 300 indexed samples, with indices `0..299` present once each. Distinct
frame count alone is insufficient because a duplicate same-frame sample would otherwise conceal a
second profiling path.

Raster diagnostics include copied font bytes, request backpressure/cancellation, completion
drained/deferred/applied bytes, queue/running/completed/failed/peak/backlog totals, and strict
completion-byte admission rejections. A rejected bitmap is released immediately and represented by
a zero-byte typed failure completion so source-cache pending state clears and a later frame retries;
workers never wait while retaining an unbudgeted bitmap.

The frame drain accepts exactly one already-admitted completion larger than its per-frame byte
allowance only before applying any completion bytes, then records
`worker_completion_oversized_accepted_count`; all following completions still defer at the frame
budget. This prevents a multi-worker pool's larger global admission budget from permanently
blocking its deferred queue head while keeping the exception explicit in the M0 profiler data.

The native source-cache report now also snapshots the active face epoch's worker-font resident
bytes and backend-face entry count. The counters are maintained only when a font byte `Arc` enters
or leaves the epoch-owned map, then project through the raster upload report and fixed profiler
names. This is an observability repair, not a new cache cap or eviction policy: managed profiling
must establish whether the retained snapshot is a material long-session cost before M1 changes its
lifetime strategy.

The bounded shared compiled-rich-text cache is now sampled by the owning `UiTextMeasureCache` at
the end of UI extraction. Fixed counters expose frame deltas for hit, miss, parse, eviction,
admission bypass, and candidate probes, plus current resident entry and byte totals. The sampler
does not add a cache or parsing path, and the M0 baseline's one-surface rows keep this projection
to one producer per captured frame. A bounded recorder regression locks all eight names, values,
and caller-frame attachment.

## Fresh Source Regression Coverage

The feature-gated report regression asserts the native instance/draw projections. A `ui +
profiling` integration regression first settles explicit Native raster work through the real WGPU
UI path, then captures the same Native/SDF batch and requires fixed stage spans, positive native
instance/draw counters, and exactly one SDF batch. A focused profile regression now requires all
seven prewarm counters to exist with zero values for an empty batch. Scoped
`rustfmt --edition 2021 --check` and diff checks pass apart from existing CRLF notices. The current
extract, prewarm production, prewarm behavior-test, profiling, profiling-test, managed WGPU
baseline, localized-dirty child, and layout-cache pressure owners are 671, 269, 435, 396, 181,
748, 142, and 223 lines respectively, all below the project's 800-line guard.
The Windows-only ignored baseline encodes 1, 100, 1,000, and 10,000 visible static labels through
three named `UiSurface` and WGPU paths. `forced-full-rebuild` retains the explicit full
surface-projection rebuild workload and all Text CPU stages; it is not presented as a layout
recomputation proof. `retained-steady` uses `rebuild_dirty`, requires
zero layout/arrange/hit/render owner visits, and omits Text extract/prewarm/layout/shape samples
while still requiring positive native instances/draws and resolved GPU timestamps. The first
three label counts retain unique text; the 10k case retains 10,000 nodes/payloads while cycling
through 512 stable text identities, allowing the 1,024-entry shaped cache to converge without
removing large command/draw pressure. Every repetition has 60 warm-up and 300 measured frames,
bounded 65,536-sample retention, exact frame-index/span/counter coverage, stable raster/upload
zero-work gates, positive native instances/draws, and current plus resolved GPU profile export.
Forced frames require owner request collection and overlap admission alongside command collection;
retained-steady frames reject all three extract scopes.
Every resolved measured generation must also contain the exact `runtime-ui` / `ui.screen-space`
pass with its own GPU timestamp, and the resolved generation sequence must exactly match the 300
current measured profiles; a non-UI or displaced frame timestamp cannot satisfy the gate.

`localized-text-dirty` alternates only the first label between already-resident `L0000` and
`L0001` text, then calls the production `UiInvalidationReason::Text` / `rebuild_dirty` path. It
projects the existing rebuild report's layout/arranged/hit/render outer-node visits, render command
reuse/rebuild counts, and phase microseconds into the same frame-indexed profiler capture. Every
measured frame must remain at one-or-fewer affected layout/arranged/hit/render nodes and one-or-
fewer rebuilt command, so a fallback to complete extract is an explicit M0 failure rather than a
cache-hit-only result.

A separate ignored persistent layout-cache pressure baseline uses the same 1/100/1k/10k,
60/300, three-repetition matrix without document/viewport ownership. It requires settled hits and
zero misses through 1k, then exact capacity-bound misses at 10k while retaining 512 shaped-text
identities. The production owner bypass and synthetic cache-capacity workload are therefore both
visible rather than conflated.

The layout projection distinguishes persistent layout-cache hits/misses from Plain owner layouts
that actually materialize a strict partial hard-line window. The fixed
`ui_text.layout_resolve.uncached_document_resolves` counter is sampled from that resolve branch,
not from document/viewport metadata, so complete viewports and vertical text retain normal
persistent-cache reuse. Forced label frames require zero uncached document resolves; their
1/100/1k document keys require settled layout-cache hits, while the 10k row deliberately exceeds
the 2,048-entry layout cache and records deterministic capacity misses. Retained clean frames
require the counter to be absent. Managed Windows Cargo, profiler execution, and WGPU framebuffer
evidence remain pending coordinator validation.

## Coordinator Staging

The final static review found no remaining source P0/P2, but
`graphics/tests/render_profiling/text_baseline.rs` and this manifest are untracked while
`graphics/tests/render_profiling.rs` imports the baseline by path. The coordinator must stage
every entry in this manifest's `Files` list as one M4 integration candidate, including the parent
module, the baseline, and its child modules. A partial stage would turn the managed
`ui + profiling-chrome` test module into a missing-file compile failure; this is a staging
requirement, not a source fallback or a validation result.

## Review

The previous source review forward-fixed the atlas report visibility boundary. A fresh independent
review then followed the real empty-command call path through the default prewarm report, fixed
seven-counter projection, feature-disabled cfg, and bounded recorder test and found no P0/P1/P2.
The static-label baseline review forward-fixed the missing WGPU timing configuration and the
missing non-empty atlas-render gate. Its final form requires 300 resolved GPU generations with
timestamps and positive native instance/draw samples in every measured frame; current-source
review then forward-fixed the impossible 10k unique-text cache-thrash gate and found no remaining
P0/P1/P2 in that prior revision. The retained/forced measurement split and uncached-document
counter are a newer forward correction. Fresh review then found that the WGPU gate lacked an exact
UI-pass timestamp and that the production document bypass could not replace the original 10k
persistent-cache pressure contract. Those findings are forward-fixed in the current source,
including zero hit/miss assertions and a rich-owner non-bypass regression. The independent
secondary review found no P0/P1; its final P2 identified incomplete Rust 2021 import formatting in
the measurement-scope `extract.rs` and `render_profiling.rs` owners. That P2 is forward-fixed, and
the complete M0 measurement file set now passes `rustfmt --edition 2021 --check`. The mixed Native/SDF capture, empty-batch counter test,
strict worker admission tests, and ignored baseline are source regressions, not substitutes for
managed profiler evidence or the required real framebuffer proof.

The localized dirty measurement owner was added after that review. Its initial root retained the
default `ContentDriven` boundary, which would promote a leaf text invalidation to the whole label
tree. The baseline now explicitly uses `ParentDirected` on its `Free` root and has a non-WGPU
`UiSurface` regression that asserts single-node layout, arranged, hit, and render traversal after
a real `Text` invalidation. The fresh independent second review found no P0/P1/P2. This changes no
production invalidation policy; managed Cargo, profiler, and WGPU framebuffer evidence remain
pending.

The final M0 source review added active worker-font snapshot residency to the same profile chain.
Focused source-cache regressions prove same-epoch sharing, exact two-face byte accumulation and
entry count, and that face invalidation clears both values; the prepare-report projection test,
fixed profiler-counter mapping test, and mixed Native/SDF WGPU profiling regression require the
values to survive the public report boundary. The independent review found no P0/P1; its two P2
test gaps are forward-fixed here. Scoped Rustfmt and diff checks pass. Managed Cargo, profiler,
and framebuffer validation remain coordinator-owned and pending.

A final narrow review found that the static-label helpers accepted distinct-frame coverage without
rejecting duplicate samples. The helpers now require exactly one indexed span/counter sample for
each measured frame, and scoped Rustfmt plus diff checks pass. This closes the source-level
measurement-contract P2; it does not replace managed profiler, Cargo, WGPU, power, or framebuffer
evidence.

The subsequent M5 viewport repair changed the static-label cache route: complete label viewports
now use persistent layout entries instead of the old broad owner bypass. The forced-label contract
was forward-corrected in the shared baseline owner to require zero uncached-document resolves,
settled cache hits through 1k, and deterministic 10k document-key capacity misses. Its linked M5
manifest records the complete atomic staging set. Static format and diff checks pass; managed
Cargo, profiler, WGPU, power, and framebuffer evidence remain pending.
