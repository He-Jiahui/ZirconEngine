---
title: Runtime81 Session-Owned Text Diagnostics
category: zircon_runtime
report_id: Runtime81-session-owned-text-diagnostics-2026-08-27
date: 2026-08-27
session_id: root-runtime-text-goal-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime81 Session-Owned Text Diagnostics

## Scope

This current-source review implements the non-validation portion of `RTS-P2-011`. It removes the
layout-fallback and shaping-failure process-global report mutexes, assigns their state to the actual
text layout session, preserves route provenance for whole-run and hybrid alternate-backend output,
and returns parallel-prewarm diagnostics to the retained session owner.

It does not change fallback admission policy, shaping output, cache admission, line breaking,
renderer routing, or any runtime budget. It does not claim a measured timing or power improvement.

## Baseline Review

The pre-change call graph contained two independent `OnceLock<Mutex<...>>` owners:

- `TextLayoutFallbackReport`, written from UI/layout failure publication;
- `TextShapingFailureReport`, written during direct-backend failure classification.

Their public getters had no workspace consumers. Neither report participated in retry, fallback, or
cache policy, but all sessions shared the same lock and last-failure slot. The parallel shaping batch
returned only aggregate ready/deferred/failed counts, so a retained session could not observe typed
failure or direct/alternate/hybrid route receipts produced by prewarm work.

## Reference Review

The local Unreal reference is
`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp`.
`FSlateFontCache` constructs and destroys its shaper/cache resources at the font-cache owner and
publishes cache statistics and flush controls from that owner. This supports owner-bound diagnostic
state. It does not support one process-global mutable failure report shared by unrelated text
sessions or documents.

## Implemented Ownership

`SharedTextLayoutSession` now owns `TextLayoutSessionDiagnostics` and resets it in `begin_frame`.
Standalone text operations already create one operation-local session, so they receive the same
bounded lifetime without adding a second global path.

The shaping layer owns a fixed `TextShapingDiagnosticsReport` value. Its route semantics are:

| Artifact/outcome | Route receipt |
|---|---|
| Ready run without a composition receipt | direct backend work |
| Ready run with an empty `alternate_ranges` receipt | whole-run alternate backend recovery |
| Ready run with non-empty `alternate_ranges` | hybrid direct/alternate composition |
| Final non-Ready outcome with a typed receipt | terminal run |

Whole-request Cosmic recovery and the rejected-hybrid-composition path now preserve the first direct
failure on the retained whole-run candidate. Ordinary direct runs keep `None`, so the extra heap
allocation remains restricted to alternate-backend output. Shaped-cache residency accounting already
includes the optional receipt.

`TextParallelShapeBatchReport` carries the same fixed shaping report. Worker completion records it at
the join/finish boundary, and `SharedTextLayoutSession::prewarm_horizontal_paragraphs` merges it after
the batch returns. Workers do not retain or lock the session, and the parallel module does not depend
on the layout-session diagnostics module.

## Cardinality and Cost Contract

The UI layout-resolve profile projects 14 fixed session names:

- six layout fallback/defer categories;
- three shaping failure/disposition counts;
- five direct/alternate/hybrid/recovered-range/terminal route counts.

The later capability/request-resolution follow-ups bring layout-resolve to 66 counters and 35 fixed
session names. The focused capture capacity is 128; the broader integration capture is 160 after the
fixed cache-lock and analysis-construction streams were added.
Reports retain fixed integers, stable enum-indexed failure counts, and at most the typed last receipt.
They retain no raw text, pointer, source label, document ID, dynamic backend name, or cause string.

Cache hits do not count as backend work. A completed cache-miss backend result is counted even when a
font-generation fence later prevents admission, because backend work already occurred.

Exact per-document drill-down remains open. `TextDocumentKey` exists only on part of the retained
plain-layout path; rich text, measurement, hit testing, and standalone shaping do not share one
document diagnostics owner. Publishing that key directly as a profiler label would violate the
low-cardinality contract. A future document dimension must use a separately bounded document owner.

## Complexity

- Session reset: `O(1)` fixed-value replacement.
- Ready-run classification: `O(1)` plus reading the already-retained alternate-range length.
- Failure recording: `O(1)` enum-indexed saturated counters.
- Parallel merge: `O(TextShapingFailureCode::COUNT)`, independent of source/glyph count.
- Cache-hit overhead: zero diagnostic writes.

No glyph, cluster, or source-text loop was added.

## Authored Regression Coverage

- diagnostics remain owner-local and reset at the frame boundary;
- whole-run alternate recovery has a distinct route receipt and stable failure code;
- parallel batches count direct backend work;
- prewarm batch diagnostics merge into the retained session owner;
- existing profile tests require the new fixed counters to be present.

## Evidence and Open Gates

Current static evidence:

- old global getter/record symbols: 0 matches;
- `OnceLock<Mutex<TextLayoutFallbackReport>>`: 0 matches;
- `OnceLock<Mutex<TextShapingFailureReport>>`: 0 matches;
- `layout_session.rs`: 787 lines, below the 800-line soft limit;
- scoped Rust 2024 rustfmt check: pass;
- scoped diff/conflict checks: pass.

Managed Cargo, fault injection, contention timing, 1/100/1k/10k 31-sample latency/allocation/RSS,
valid-sensor power, real WGPU framebuffer, and PNG evidence under `docs/tests/runtime/text` remain
pending. No screenshot was generated for this non-visual infrastructure slice.

Status:
`session_owned_diagnostics_implemented / process_global_report_mutexes_removed /
fixed_backend_route_projection_implemented / parallel_prewarm_merge_implemented /
document_drilldown_owner_open / static_checks_complete / managed_validation_pending`.
