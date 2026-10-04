---
title: Editor paint-text relative layout, glyph atlas and font publication protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-paint-text-relative-layout-glyph-atlas-font-publication-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/paint_text{.rs,/**,_tests.rs,_tests/**}`
- 37/37 Rust files source-reviewed. Exact Runtime artifact projection avoids duplicate shaping, but
  layout cache identity owns text and absolute x/y under a global mutex/clear-all policy; record-only
  still builds CPU-oriented glyph projections; fallback repeats shaping and contains `O(G*H)` mapping;
  font scan/I/O/parse is paint-lazy; raster cache is unbounded and globally locked per glyph. M1 has
  reduced run preference captures `G+1 -> 1`, host font captures `G -> 1` and same-format Swash copied
  bytes `B -> 0` (focused GREEN 3/3; owned contracts GREEN 85/85). M0/M2-M6 counters, relative artifact,
  linear fallback, font publication, bounded atlas and dynamic acceptance remain pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to MVP editor text layout/record/raster work. Record layout identity/owned bytes, artifact
and fallback passes, G/H operations, paint-thread font work, glyph residency/locks, CPU/RSS/latency,
WPR power and RenderDoc/pixel/text parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of absolute-position editor glyph layouts, parallel editor glyph DTOs, layout clear-all,
paint-thread font resolution and unbounded raw bitmap cache after Runtime artifacts/font publication/
atlas handles are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own prepared text artifact retention per presentation generation, translation-only paint reuse,
recording handle propagation and plugin-pane text damage rules.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own one relative prepared text contract shared by measurement, overflow, hit testing, recording and
paint, including borrowed text identity and dirty/generation semantics.

## `docs/plans/zircon_runtime/runtime/15-text-layout-shaping-and-font-resolution.md`

Own authoritative artifact coverage, linear cluster/glyph projection, off-paint prepared font
publication, deduplicated source bytes/faces and generation-safe fallback reasons.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own glyph atlas/resource handles, byte/page budgets, admission/upload/eviction/lock counters and
same-workload RenderDoc draw/batch/upload/GPU evidence.

## Acceptance handoff

The handoff requires 37/37 post-change fingerprints, focused and managed Rust behavior tests, the
text/glyph/origin/wrap/font/phase/backend matrix, same-executable WPR/power artifacts on D/E/F,
RenderDoc and pixel/text parity, milestone commit and quantified WeCom notification. Protected ledgers
remain unchanged until then.
