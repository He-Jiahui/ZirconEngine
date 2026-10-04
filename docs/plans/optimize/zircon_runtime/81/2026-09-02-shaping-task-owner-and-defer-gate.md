# Runtime81 shaping task owner and typed defer gate

Date: 2026-09-02

Status: `architecture_review_complete / source_fingerprint_recorded /
profile_plan_complete / implementation_not_started / managed_profile_pending`

Owners: `RTS-P0-001`, `RTS-P1-015`, `RTS-P1-016`

## 1. Decision

Do not split a shaping request at the 64 KiB work threshold. Do not return an
unrecoverable `Deferred` result from the synchronous UI layout path. Do not add
another worker owner beside Core's `EngineTaskGraph`.

The first structural correction is execution ownership, not a shaping
algorithm rewrite:

1. `TextRuntimeContext` receives a Core-owned text task scope and compute
   execution handle during TextModule construction.
2. A deferred shaping work unit owns the complete immutable source, style,
   direction, language, source range, vertical mode, font collection identity,
   and font generation. The byte threshold classifies admission only.
3. The work unit publishes through a typed ticket with pending, ready,
   cancelled, generation-stale, and failed terminal states. Only a ready value
   for the admitted collection and generation may enter `ShapedRunCache`.
4. UI invalidation and retry must be connected before an on-demand cache miss
   is allowed to defer. Until then, synchronous layout remains the compatibility
   path and only prewarm may use background completion.

This preserves the current MVP rendering contract while creating the lifecycle
needed to remove caller blocking later.

## 2. Current-source evidence

### 2.1 Zircon execution path

- `SharedTextLayoutSession::resolve_or_shape_outcome` performs cache lookup and
  calls the canonical backend synchronously on every miss.
- `shape_paragraphs_with_cache_in_font_collection` deduplicates complete owned
  paragraph requests, but `parallel_for` joins before returning; its
  `caller_wait_nanos` receipt proves the UI caller still waits.
- `prewarm_render_command_text` obtains
  `TaskPools::process_default().compute()` at the call site. This worker owner is
  not the TextModule/Core task lifecycle and therefore cannot provide scoped
  cancellation, shutdown census, or per-runtime attribution.
- `TaskPool::spawn` is fire-and-forget. `TaskGraphScope::submit` is the existing
  Runtime contract that adds bounded admission, a `TaskHandle`, cooperative
  cancellation, terminal observation, and shutdown drain ownership.
- `TextRuntimeContext` already owns font, Unicode snapshot, session identity,
  and draining state. It currently owns no task scope or work registry.

### 2.2 Unreal reference boundary

- `FSlateTextShaper` shapes a source range while receiving the complete source
  pointer; text outside the requested range remains available as shaping
  context.
- HarfBuzz work is itemized by direction, font face, script, and grapheme-aware
  boundaries. An arbitrary byte threshold is not a script-run or cluster
  boundary.
- `FShapedTextCache` keys retained shaped sequences by range and shaping
  context, rejects dirty sequences, and tries to reuse a whole-run shape before
  reshaping a smaller range.
- `FTextLayout` separates stable line models from lazily generated line views.
  Font generation changes dirty layout and shaping caches, after which normal
  update materializes the new view.

Zircon should adopt the stable owner, range/context retention, dirty
publication, and lazy materialization structure. Unreal's implementation does
not justify Zircon's 64 KiB value, TaskGraph API, or cancellation policy.

## 3. Frozen inputs

The implementation must recheck these SHA-256 values before editing because
several paths are concurrently modified in the shared worktree.

| File | Lines | SHA-256 |
|---|---:|---|
| `zircon_runtime/src/text/shaping/work_budget.rs` | 128 | `eac70ec0d8b0dd38aee8b64de040977abf440ba1fe0a7d25de56ada7524c2129` |
| `zircon_runtime/src/text/layout_session.rs` | 604 | `fbc660b59ff78bb8663a7ffa5faa6856463879739aa5aec97f1eb44cf5664662` |
| `zircon_runtime/src/text/parallel/shape_pool.rs` | 532 | `b97c3608d3a3ef98bee8a2ce1f6eed26ce6b290b0f8ca5c1c2e42217fecdee36` |
| `zircon_runtime/src/text/context.rs` | 643 | `59f24e1ca5694717e637d8e7b6d03fdd2a7cdfe23cd4f4534cee61b2001cee36` |
| `zircon_runtime/src/text/module.rs` | 221 | `b8af88c7c284a1d5522e7310dbed79314b15c5afade792aa34d4fc12de79d4e7` |
| `zircon_runtime/src/ui/surface/render/text_prewarm.rs` | 246 | `783f9e0e19617ba353a6e8dc2b9e7dd945f402d1d699ab9794a09c5967977e87` |
| `zircon_runtime/src/core/runtime/tasks/pool.rs` | 465 | `9887dea0d089bfa1b55c94267d6cc210a851e8403cb1eede29cdef4413ae401b` |
| `zircon_runtime/src/core/runtime/tasks/task_graph/scope.rs` | 461 | `cac234ab1d0a82dc24077cbd2c9ba932dc4a6340cb3735c94a36eeef15dccec7` |
| `dev/UnrealEngine/.../SlateTextShaper.cpp` | 1104 | `b229d91857c1acdd9fd4625e7b850f1f9b1566bfb8d23bab0b04dddf138298ec` |
| `dev/UnrealEngine/.../FontCache.h` | 1139 | `52324695a8187753db247e9be7d7586c3313121d7b050205e42d89c0e95e7898` |
| `dev/UnrealEngine/.../ShapedTextCache.cpp` | 279 | `8a1e0163dbd309e5ffc8596883f319c238845db9b76f82928f47800ce258a712` |
| `dev/UnrealEngine/.../TextLayout.cpp` | 3853 | `19d3e344fbb3a8c73891d91bc1bb434e68b9a6a1bca0b066129b9724199f8a36` |

`context.rs` is currently untracked and `module.rs`, `layout_session.rs`,
`measure_cache.rs`, plus the TaskGraph tree have external worktree changes.
Those blobs must not be overwritten or treated as clean baseline content.

## 4. Bottleneck hypotheses and required profile

No structural optimization is authorized from static inspection alone. Run the
managed profiling build on the E: drive after the current Cargo owner admits the
validation job.

### 4.1 Workloads

Collect cold and warm runs for 1, 100, 1k, and 10k grapheme inputs in each
category:

- Latin direct success;
- Arabic RTL with joining and ligatures;
- Indic combining/cluster text;
- emoji ZWJ, variation selector, modifier, and regional-indicator sequences;
- mixed BiDi and paired brackets;
- direct-to-whole-alternate and hybrid fallback;
- vertical `Upright`, `Sideways`, and `TransformOrRotate`;
- cache hit, unique cache miss, batch duplicate, and generation-stale retry.

Use one worker and the production worker budget. Record 31 measured samples
after warm-up for each qualified case.

### 4.2 Measurements

- caller wall time: p50, p95, p99, minimum, median, maximum;
- backend CPU time and shape call count;
- queue wait, worker execution, explicit wait, and cancellation latency;
- input bytes, glyphs, graphemes, logical segments, and output clusters;
- source-owner materialization bytes and unique owner bytes;
- shaped-cache lookup, insert, eviction, and generation rejection counts;
- allocation count, allocated bytes, peak retained bytes, and RSS;
- process CPU package/energy sample available from the approved Windows power
  harness, with identical idle correction and workload duration;
- glyph/layout artifact digest parity between synchronous and scheduled paths.

### 4.3 Decision gates

- If caller wait is not a dominant p95/p99 contributor, do not cut over
  on-demand shaping; retain only owner/lifecycle cleanup.
- If source copying dominates, design document snapshot plus range leases before
  background scheduling. Do not combine this with a glyph SoA migration.
- If backend CPU dominates and work is parallelizable only across independent
  paragraphs, schedule whole semantic paragraphs. Do not byte-slice one
  paragraph.
- If queue wait or context switching removes the gain below 8 requests, keep an
  evidence-derived inline crossover. The current value 8 is not accepted
  without the new data.
- The scheduled path must match synchronous glyph, cluster, source-map, layout,
  caret, selection, and artifact digests for every correctness workload.

No Unreal timing or power comparison may be claimed until both engines use the
same fonts, text, locale, scale, viewport, warm/cold state, and sample method.

## 5. Implementation milestones

### M0 - Execution owner injection

Status: `not_started / external_blob_reconciliation_required`

- TextModule declares the TasksModule dependency and resolves Core's TaskGraph.
- Text runtime creates a `TaskGraphScope` named for the text subsystem and
  retains it for context lifetime.
- Production prewarm no longer calls `TaskPools::process_default()`.
- Context drain closes text admission; Core shutdown owns worker termination.
- Standalone compatibility constructors remain explicit and are excluded from
  the production owner claim.

### M1 - Typed complete-request ticket

Status: `not_started / depends_on_M0`

- Add a bounded pending registry keyed by the exact shaped-cache lookup
  identity plus exact source verification.
- A ticket owns a complete `Arc<str>` request envelope, retained-byte lease,
  `TaskHandle`, and one terminal result slot.
- Cancellation is cooperative between semantic stages; it never publishes a
  partial glyph run as ready.
- Duplicate pending requests share one work record without merging different
  collection or generation identities.

### M2 - Prewarm scheduling and publication

Status: `not_started / depends_on_M1_and_profile_gate`

- Schedule eligible unique prewarm misses without making the renderer or cache
  owner cross-thread mutable.
- Drain ready results on the owning UI/session thread under item and byte
  budgets; insert only generation-qualified ready runs.
- Publish pending, ready, cancelled, stale, failed, queue-wait, execution, drain,
  and retained-byte receipts.
- Preserve the current synchronous on-demand miss path.

### M3 - On-demand defer and UI invalidation

Status: `not_started / depends_on_M2_and_product_invalidation`

- Add an explicit typed pending result distinct from font-generation retry.
- Connect terminal completion to retained surface dirtying and next-frame
  layout retry before enabling defer.
- Prove that no clean-frame cache can retain a failure/zero layout after the
  ticket becomes ready.
- Remove synchronous oversized fallback only after visual and performance
  qualification.

## 6. Acceptance

The P0 remains open until all of the following are true:

- long Arabic, Indic, emoji, ligature, mixed-BiDi, and vertical inputs retain
  complete shaping context and exact cluster/source maps;
- duplicate pending work is single-flight and generation/collection isolated;
- cancellation, drop, context drain, runtime shutdown, panic, and stale
  generation reach typed terminal states without leaked source bytes;
- pending completion dirties and successfully rebuilds the owning retained UI;
- 31-sample results show the targeted caller-wait bottleneck was reduced and no
  significant CPU, allocation, RSS, or power regression was introduced;
- real WGPU rendering matches the synchronous baseline and the PNG evidence is
  stored below `docs/tests/runtime/text`, never in `target`;
- managed Cargo/test validation is accepted, then the milestone is committed
  and its quantified result is sent to WeCom.

Current completion receipt: architecture review, source freeze, Unreal mapping,
bottleneck hypotheses, profile matrix, staged implementation design, and
acceptance conditions are complete. Runtime code, managed profile, Cargo,
WGPU/PNG, power comparison, commit, and WeCom notification are pending.
