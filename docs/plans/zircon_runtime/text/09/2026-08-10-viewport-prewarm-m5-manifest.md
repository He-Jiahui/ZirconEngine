Plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
Milestone: M5
Status: implementation_forward_repaired_static_second_review_complete_coordinator_atomic_staging_required_managed_validation_pending
Files: ["zircon_runtime/src/text/hard_line.rs", "zircon_runtime/src/text/mod.rs", "zircon_runtime/src/text/layout/mod.rs", "zircon_runtime/src/text/layout/rich_advance_index.rs", "zircon_runtime/src/text/layout/rich_advance_index/tests.rs", "zircon_runtime/src/ui/text/measure_cache.rs", "zircon_runtime/src/ui/text/resolved_layout.rs", "zircon_runtime/src/ui/text/layout_engine.rs", "zircon_runtime/src/ui/text/layout_engine/candidate_line.rs", "zircon_runtime/src/ui/text/layout_engine/wrapping.rs", "zircon_runtime/src/ui/text/layout_engine/wrapping/tests.rs", "zircon_runtime/src/ui/surface/render/text_prewarm.rs", "zircon_runtime/src/ui/surface/render/text_prewarm/tests/profile.rs", "zircon_runtime/src/ui/tests/text_pipeline/measure_cache.rs", "zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs", "zircon_runtime/src/graphics/tests/render_profiling/text_baseline.rs", "docs/plans/zircon_runtime/text/09/2026-08-09-text-pipeline-performance-architecture-and-profiling-plan.md", "docs/plans/zircon_runtime/text/09/2026-08-10-m0-observability-m4-manifest.md", "docs/plans/zircon_runtime/text/09/failure-2026-07-18-text-parallel-queue-budget-and-font-copy.md", ".codex/sessions/20260810-text09-m4-observability.md"]

# Text09 M5 Viewport And Canonical Prewarm Manifest

## Scope Delivered

Retained Plain/HorizontalTb/None/Clip owners defer source prewarm only when the exact shared
hard-line viewport query materializes a strict subset. A full viewport, including a capped single
line, remains prewarmed. Normal rich/vertical source uses visible hard-line requests with the base
style; inline rich source reuses coalesced resolved spans. Both routes now share canonical CRLF,
Unicode-separator, and shaping-cap boundaries with layout. Candidate lines assembled across markup
runs split again at the grapheme-safe shaping cap while preserving source and visual ranges.

2026-08-11 forward repair: the layout-cache route now asks that same hard-line query before
choosing an owner-document bypass. Only a strict Plain/HorizontalTb/None/Clip subset bypasses the
persistent cache; complete viewports and vertical text retain normal persistent cache reuse.
Partial geometry remains out of the persistent cache but is entered into same-frame dedup with the
retained canonical `Arc<str>`. The `uncached_document_resolves` profiler counter is now sampled
from that actual branch instead of request metadata.

## Fresh Testing Evidence

Focused regressions cover a 10,000-line sub-64-KiB viewport owner, complete-viewport retention,
rich/vertical source-cache hits, inline-rich resolved spans, canonical separators, and markup-joined
cap splits. Scoped Rustfmt and diff checks pass apart from existing CRLF notices. Managed Windows
Cargo, profiler capture, WGPU framebuffer proof, and the required PNG remain pending.

The forward repair adds cross-frame persistent-cache regressions for vertical and complete Plain
viewports, a same-frame partial viewport dedup regression, and a profile expectation that a
complete owner emits one cache miss rather than a false bypass. The source-only checks remain
`rustfmt --edition 2021 --check` and scoped `git diff --check`; no Cargo, profiler, WGPU, or PNG
result is claimed.

The same routing repair updates the forced-label profiler contract: complete label viewports emit
zero uncached-document resolves. The 1/100/1k document-key rows fit the persistent cache and
require settled hits; the 10k document-key row intentionally exceeds its 2,048-entry capacity and
requires deterministic misses. This keeps the M0 trace truthful without weakening the retained
partial-viewport contract.

2026-08-11 hot-path follow-up: a Plain owner with no canonical separator and no shaping-cap split
cannot select a strict viewport line window. That condition now returns before parsed-source
retention or hard-line-index construction, so normal one-line labels no longer churn the bounded
16-document/index owners merely to discover that their complete prewarm remains valid. For
multi-line and over-cap source, the existing exact shared query remains authoritative. A
persistent layout-cache lookup also now precedes that query, so an already-cached complete
viewport avoids a second index probe. Regressions cover CRLF, Unicode separators, the 64-KiB cap,
and a cross-frame complete viewport cache hit that performs no new hard-line probe.

## Review

The earlier source review confirmed the shared hard-line/run-cap invariant for inline rich layout
and prewarm, source/visual range preservation after cross-markup candidate splits, and fail-closed
PF-M5 viewport selection. The 2026-08-11 static forward review corrected the remaining route drift:
viewport metadata is not itself a bypass signal, partial viewport geometry retains same-frame dedup
without entering persistent cache, and the shared forced-label baseline now distinguishes
capacity-fitting persistent hits from 10k document-key capacity misses. Scoped format and diff
checks pass. The follow-up static review confirmed that the fast predicate is a conservative
subset of canonical hard-line segmentation and that the viewport key participates in the persistent
layout-cache key; managed validation remains the acceptance boundary.

The 2026-08-11 documentation convention audit also isolated the Text scope from unrelated
repository migration debt. The current full docs checker reports cross-domain missing-path records,
but it reports zero violations for this manifest, the Text09 performance plan, and the two priority
authorities `engine-code-structure-convention.md` and
`engine-code-review-findings-2026-06.md`. This is a scope attribution result, not a substitute for
the managed Cargo/WGPU gates.
