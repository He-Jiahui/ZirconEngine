# Runtime Text font-resolution request receipt review

Date: 2026-08-27

Status: `bounded_candidate_decision_receipt_implemented / transient_completion_envelope_implemented /
generation_attempt_cost_retained / shaped_artifact_pollution_zero /
request_local_cache_lock_profile_implemented / structural_optimization_profile_gated /
static_checks_complete / managed_validation_pending / full_font_resolve_outcome_open`

## Structural review

Runtime Font 80 `RFF-P1-032` requires a typed `FontResolveOutcome` that can retain candidates, chosen
face, missing cluster, pending dependency, policy rejection, budget exhaustion, and fallback reason.
The current Runtime Text database is a synchronous process-shared snapshot. It cannot truthfully
publish collection generation, pending asset work, policy rejection, or backend capability.

Local Unreal `FontCacheCompositeFont.cpp::GetFontDataForCodepoint` retains sub/default/fallback
selection, while `SlateTextShaper.cpp` keeps face/loading state and a loading-face collection through
the shaping request. Zircon keeps its cluster-first policy, but adopts the same owner rule: resolution
work remains attached to the request until its session/operation owner consumes it.

Putting request diagnostics into public serde `ShapedGlyphRun` was rejected because it would make
one cache miss's work history resident and replay it on every cache hit. A process-global counter was
also rejected because concurrent request attribution would be false and the session owner would be
lost.

## Implemented boundary

- `TextFontResolutionReport` is a fixed 17-counter value. Together with attempt/restart counts,
  `TextShapingRequestDiagnostics` is 152 bytes on the current 64-bit target and has a 160-byte guard.
- The transient `TextShapingCompletion` and `GenerationTaggedShapedRun` carry diagnostics beside the
  glyph run. `ShapedGlyphRun`, its serde format, shaped-cache key, and resident-byte accounting do not
  reference the diagnostic types.
- The existing resolver loop records resolution/candidate cache hit-miss, actual
  `face_covers_codepoint` probes, primary rejection, complete and partial candidate visits, complete
  candidate rejection, and Primary/Fallback/Partial/LastResort/DepthLimit selection counts.
- Candidate compiler family-face filtering contributes its real coverage probes. Complete-coverage
  short circuit and partial ranking are counted in-place; no second coverage pass was added.
- A resolution-cache hit records the current hit and logical selection only. It does not replay the
  historical candidate visits or probes that built the cached result.
- Stable-generation shaping accumulates diagnostics across discarded attempts. The final Ready,
  terminal, or deferred outcome retains the request's attempt and restart costs.
- Session and parallel owners merge diagnostics only for actual shaped work. A shaped-run cache hit
  does not manufacture backend or resolution work.
- UI projection uses 35 stable session names and no raw text, face/family name, candidate ordinal,
  pointer, document identity, or dynamic label. The layout-resolve function emits 66 counters under a
  128-counter focused capacity; the broader extract+prewarm+layout integration capture uses 160 after
  adding fixed cache-lock and analysis-construction streams.

## Complexity and expected bottleneck evidence

The resolver still performs the same candidate construction and coverage calls. Added work is
constant-time saturating increments within existing loops and a fixed-value merge per shaping
attempt/session completion. Allocation complexity is unchanged; the completion envelope adds no heap
allocation and the cached glyph artifact is unchanged.

The new counters can distinguish primary fast path, resolution-cache hit, candidate-cache hit,
candidate traversal, partial-ranking work, missing diagnostics, and generation retry amplification.
They are measurement infrastructure, not evidence that a bottleneck has disappeared. Algorithm
changes remain forbidden until managed 31-sample cold/warm corpus data identifies the dominant term.

## Resolver/cache structural performance review

The current cache design places family candidates, compiled composites, fallback candidates,
resolutions, and line-metric envelopes behind one `Mutex<FallbackCacheState>`. Every hit mutates a
`HashMap` entry and two `BTreeMap` LRU positions, so a nominal resolution-cache hit is an exclusive
write critical section. A resolution miss then takes separate candidate/family lookup and insertion
critical sections. Cold `CompositeFontIndex::compile` also currently executes while that same state
lock is held.

The shaping owner first scans the complete text for primary-face coverage. If a missing character is
late in a long request, already accepted characters are revisited during grapheme itemization. That
may be cheaper than taking the cluster path for ordinary all-primary text, but it can amplify work for
mostly-primary mixed text. It is therefore a separate hypothesis from lock contention.

Local Unreal provides the structural reference, not a latency target. `FCachedCompositeFontData`
compiles subtypefaces and normalized priority/ordinary ranges when the composite entry is built;
`GetTypefaceForCodepoint` performs bounded checks plus binary search. `FSlateTextShaper` then performs
one ordered grapheme traversal, resolves a face/loading state, and merges adjacent equal-face sections.
This supports a future immutable collection snapshot and linear request itemizer. It does not by
itself prove that Zircon should copy Unreal's first-codepoint cluster policy or remove complete-cluster
coverage validation.

The next managed profile must distinguish these falsifiable hypotheses:

- **H1 shared-hit serialization:** warm repeated clusters show material lock wait or hold growth with
  worker count even when resolution hits approach 100%. Reject if wait is noise and CPU is dominated
  by key hashing/grapheme traversal.
- **H2 cold compile convoy:** composite miss p95/p99 lock hold is dominated by index compilation and
  blocks otherwise warm lookups. Reject if compile time is small and no concurrent waiter observes it.
- **H3 late primary rejection:** mostly-primary text with a miss at 1%, 50%, and 99% shows coverage
  probes materially above the cluster-only lower bound. Reject if the whole-text pre-scan remains a
  net win across the measured corpus.
- **H4 cache-layer duplication:** resolution miss plus candidate hit still spends material time in
  repeated BLAKE3 key construction or multiple LRU locks. Reject if coverage/backend work dominates.

No sharding, lock-free snapshot, request memo, pre-scan removal, or compiled decision graph is
implemented in this slice.

## Lock profiling infrastructure

All fallback-cache state access now passes through one `with_state` boundary. Test/profiling builds
measure lock acquire count, wait nanoseconds, and hold nanoseconds; ordinary non-profiling builds do
not call `Instant`. Global cumulative fields remain cache-owner benchmark statistics. Per-request
profiler publication does not subtract overlapping global snapshots: the cache owner aggregates the
same three values in request-scoped TLS only while one shaping itemization is active, then publishes
three fixed names once at completion. Concurrent workers therefore retain independent attribution and
do not take a profiler lock per cache access.

## Required profiling before resolver optimization

The managed Windows profiling run must use the repository's profiling capture plus a sampled CPU
trace. It must publish the exact build/features/font collection identity and run these lanes:

| Lane | Required scale |
|---|---|
| Primary whole-text fast path | 1, 100, and 10,000 grapheme clusters |
| Warm resolution-cache hit | 1, 100, and 10,000 repeated clusters |
| Resolution miss + candidate-cache hit | 1, 100, and 10,000 clusters |
| Cold candidate compiler | 1, 8, and 64 eligible families/candidates |
| Complete rejection + partial ranking | 1, 8, and 64 candidates; combining and emoji sequences |
| Last resort/missing diagnostic | unique and repeated missing clusters through the bounded log |
| Font generation churn | zero, one, and retry-budget-exhausting generation publications |

Each cold and warm lane requires 31 measured samples after an explicit warm-up policy. Record wall
p50/p95/p99, CPU samples by resolver/cache/coverage/backend stack, attempt/restart counts, every new
resolution counter, the now-available exact request-local cache lock acquire/wait/hold values,
allocations, peak RSS, and the repository's available Windows power/energy observation. Also record
output face/source ranges and glyph/layout
hashes so a faster but different decision is rejected.

The report must first fit observed cost against grapheme and candidate scale. A warm path that still
scales with historical candidate count, a cache hit that emits coverage probes, or lock wait that
dominates resolver CPU is a structural defect. Only then may a follow-up choose collection snapshot
partitioning, request-local memoization, compiled decision graph, or another owner change. Thresholds
must come from the measured Zircon baseline plus an executable same-corpus reference harness; no
Unreal/Godot/other-engine latency or power value may be invented from anecdotal experience.

## Static evidence

- Rust 2024 `rustfmt --check` passes for the scoped Rust owners.
- Scoped `git diff --check` reports only repository line-ending warnings.
- `ShapedGlyphRun` and shaped-cache memory accounting contain zero references to
  `TextFontResolutionReport` or `TextShapingRequestDiagnostics`.
- Production UI projection contains 35 unique `ui_text.session.*` names; configured focused capacity
  is 128 and the broader integration capture capacity is 160.
- Production owners remain below 800 lines; the UI profile root/session projection leaf are 694/193
  lines after the fixed mapping was split from the orchestration owner.
- Focused regressions exist for resolution-cache hit versus miss work, 160-byte fixed residency,
  generation-attempt accumulation, session merge, cache lock measurement/report-read exclusion, fixed
  lock profile names, and profile projection.

Managed Cargo, fault injection, concurrent generation stress, 31-sample p50/p95/p99, RSS/power,
WGPU rendering, and PNG evidence were not run. No dynamic performance or product-acceptance claim is
made.

## Remaining Runtime Font owner work

Runtime Font 80 M3/M5 must supply the full session-owned collection and capability result: stable
collection/content generation, bounded exact candidate/face trace when explicitly sampled, pending
asset dependency, policy rejection, budget exhaustion, backend/color/variation capability, and real
tofu handoff. Those states must extend or replace the synchronous subset without reintroducing a
process-global diagnostic owner or putting request history into the glyph cache artifact.
