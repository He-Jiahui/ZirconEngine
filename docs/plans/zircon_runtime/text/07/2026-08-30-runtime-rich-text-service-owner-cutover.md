# Runtime rich-text parser and cache owner cutover

Date: 2026-08-30

Status: `roadmap_review_complete / current_source_consumer_map_complete /
unreal_owner_review_complete / failure_contract_written /
process_global_parser_cache_hard_cut_static_complete / managed_product_validation_pending`

## Scope

This cutover covers RRT-P1-013's production process-global parser/cache ownership. It moves the
bounded compiled-rich cache into `RichTextParser` and makes the retained Surface text session own the
default parser used by layout, measurement, prewarm, and render preparation. It does not yet add
project-qualified custom provider registration, unregister/revoke leases, targeted generation
retirement, or cancellable single-flight jobs; those remain RRT-P1-010, RRT-P1-014, and RRT-P1-016.

## Roadmap and interface classification

The repository roadmap maps `IService` to descriptor-level `EngineService`; it does not require every
concrete runtime object to become a global service singleton. `UiModule` has a real `EngineModule`
owner and non-stub driver/manager descriptors, but Runtime UI remains a convergence pressure point.
The full structure audit was launched and produced no result before its 120-second limit, so this
record does not call the whole crate converged. The targeted evidence is narrower:

- `UiSurface` already owns one retained `UiTextMeasureCache`;
- that cache owns one `SharedTextLayoutSession` and receives the runtime-selected font collection;
- all production rich compile/lookup consumers are below that session boundary;
- no production consumer reads the process-global compiled-rich report outside Surface profiling;
- `zircon_app` does not need text-specific bootstrap knowledge for this cutover.

The smallest convergence move is therefore a runtime object owned by the existing Surface/session,
not another `CoreRuntime` singleton, plugin, or app-level dependency.

## Current-source consumer map

The only production compilation paths are:

1. `ui/text/layout_engine.rs` before canonical rich layout;
2. `ui/text/resolved_layout.rs` before outcome publication;
3. `ui/text/measure_cache.rs` for measurement, retained Plain documents, and prewarm requests;
4. `ui/text/rich_text.rs` for render artifact preparation when no compiled owner is retained.

Every path already receives or is owned by `SharedTextLayoutSession`. The crate-level
`compile_rich_text`/`lookup_compiled_rich_text` bridge otherwise serves tests. The global cache report
and sampler serve Surface profiling and cache-unit tests only. This makes explicit owner injection a
closed internal migration rather than a new public compatibility layer.

## Current structural defect

`shared_builtin_parser()` and `shared_cache()` use independent process `OnceLock` owners. Documents
from unrelated runtime projects/sessions share residency, counters, LRU pressure, failure cells, and
parser lifetime. A Surface clear/shutdown cannot release its compiled-rich residency, and profiling
reports process-wide deltas as if they belonged to one Surface. Custom `RichTextParser` instances also
compile through the same cache and require numeric parser identity solely to avoid cross-owner key
collision.

This is an ownership and lifecycle defect. Removing a global mutex may reduce contention, but this
slice makes no latency, RSS, or power claim without a matched product profile. Per-owner 8 MiB is a
maximum admission bound, not eager allocation; multi-Surface aggregate quota remains open work.

## Unreal reference boundary

Local Unreal `URichTextBlock::RebuildWidget` creates parser/decorator state for the widget and gives
strong references to one `FRichTextLayoutMarshaller`; `SetDecorators` replaces that marshaller's
owner-local array. `FRichTextLayoutMarshaller::SetText` parses and builds runs for its retained layout.
There is no independent process-global compiled-rich cache that mixes unrelated widget/project
lifetimes.

Zircon's Surface session is broader than one widget because it intentionally shares shaping/layout
caches across one retained Surface. That is the correct MVP owner: it keeps one pipeline per UI
session while avoiding widget-level cache multiplication and process-level tenant mixing.

## Target ownership

```text
UiSurface
  -> UiTextMeasureCache
     -> SharedTextLayoutSession
        -> Arc<RichTextParser> (builtins + parser identity + budget)
           -> CompiledRichTextCacheOwner (bounded Mutex + single-flight cells)
```

Public custom `RichTextParser` objects own their own cache. Cloning a Surface/session retains the same
parser owner; clearing the session clears its compiled cache along with shaped/hard-line caches.
Production calls pass the session explicitly. Test-only corpus helpers may retain a cfg-gated default
parser, but no production static parser/cache/report remains.

## Required hard cut

1. Replace free global cache functions with `CompiledRichTextCacheOwner` methods.
2. Add the cache owner directly to `RichTextParser`; compile, lookup, report, and clear operate on that
   instance only.
3. Add an `Arc<RichTextParser>` to `SharedTextLayoutSession` and expose crate-private compile/lookup/
   report methods.
4. Change layout, resolved layout, measurement, prewarm, retained document, and render preparation to
   use their explicit session.
5. Make profiling sample the same Surface session report; remove the process-global report API.
6. Keep convenience parser/lookup bridges only under `cfg(test)` and update eviction tests to use one
   explicit session where owner continuity is the behavior under test.
7. Do not add a compatibility singleton, service locator, global Arc, or second cache.

## Validation gates

- failing static ownership contract before implementation;
- parser cache isolation across two Surface sessions;
- same-session pointer reuse and bounded eviction;
- session clear removes only that owner's residency;
- current Runtime Text static suite, workspace Rust 2021 targeted formatting, and scoped diff-check;
- managed Cargo and product WGPU/PNG remain required before milestone acceptance;
- matched multi-Surface allocation/RSS/contention/power profiling remains required before any
  performance claim.

## Implemented cutover

- `CompiledRichTextCacheOwner` now owns the bounded mutex, LRU/index, counters, and single-flight
  cells. The production `shared_cache`, free compile/lookup functions, and shared report API are gone.
- Every `RichTextParser` owns one cache and exposes compile/lookup/report/clear against that owner.
  The static built-in parser and convenience bridges remain only under `cfg(test)` for corpus tests.
- `SharedTextLayoutSession` retains `Arc<RichTextParser>`; cloned Surface sessions share that exact
  owner, while independently constructed Surface sessions do not share artifacts, counters, or
  clear/shutdown effects.
- Layout, resolved layout, measurement, rich prewarm, retained Plain-document parsing, and render
  artifact preparation all receive the Surface session explicitly. Profiling samples the report from
  that same session rather than a process-global counter stream.
- A post-cutover production re-export/caller audit found two compile-boundary leftovers: one-shot
  measurement still imported the test-only helper, and `ui::text` still re-exported it
  unconditionally. Both now use the explicit session provider or are cfg-gated; the ownership guard
  covers these boundaries.
- `layout_session.rs` remains the production orchestration boundary at 476 lines; its 479-line unit
  test responsibility moved to `layout_session/tests.rs` rather than growing a near-1,000-line root.

## Validation evidence and remaining gates

The pre-change ownership contract failed, then the combined Runtime Text static suite passed 36/36
after the hard cut. Source tracing finds no production `shared_cache`, shared report, or default
parser call, including the one-shot measurement and module re-export boundaries. The owner-local
Rust test covers same-session pointer reuse, independent-session cache
identity, and clear isolation, but it has not run because this slice deliberately bypassed the busy
managed Cargo queue. The new test module passes targeted Rust 2021 `rustfmt --check`, and the scoped
diff check passes. Whole-workspace formatting is not evidence: it currently stops on unrelated dirty
`zircon_editor` animation and Runtime graphics deformation sources.

This closes the process-global ownership part of RRT-P1-013 statically. It makes no latency,
allocation, RSS, contention, power, or Unreal-parity claim. RRT-P1-010 provider owner/lease/revoke
and registration admission, RRT-P1-014 cancellable single flight, and the remaining RRT-P1-016
snapshot retirement/revoke integration stay open. A later Windows validator request failed while
acquiring the managed Cargo lane (`cargo.acquire` post-response timeout); Cargo/rustc never started,
and the lane was not polled or bypassed. Real WGPU/PNG under `docs/tests/runtime/text` and matched
multi-Surface profiling also remain open.
