# One-shot text layout session current-source audit

Date: 2026-08-30

Status: `current_source_call_map_complete / retained_product_hot_path_confirmed /
no_structural_optimization_authorized / product_profile_trigger_pending`

## Question

Text11B P1-14 warned that convenience layout/measure APIs could construct a complete short-lived
session per call and bypass retained caches. This audit determines whether that remains a current
product hot path before any cache or ownership optimization is attempted.

## Current-source owner map

- `UiSurface` owns one `UiTextMeasureCache`, which owns one `SharedTextLayoutSession` and is used by
  retained layout, extraction, prewarm, artifact preparation, and input geometry.
- Dynamic Runtime HUD/menu fallback owns one `RuntimeUiExtractCache`. It receives the Core
  `FontCollectionService`, retains one `UiTextMeasureCache`, and reuses both the extract and text
  cache across frames. The `UiTextMeasureCache::default()` calls in `hud.rs` and `menu.rs` are inside
  test helpers only.
- `compute_layout_tree`, standalone render extraction, `layout_text`, public measure wrappers, and
  `shape_text_line` explicitly construct one operation-local owner. Repository call tracing finds
  these in tests, public compatibility exports, or the native product-framebuffer validation path;
  it does not find them inside the retained Surface frame loop.
- One-shot `DirectTextShapeRunProvider` captures one immutable font collection snapshot at operation
  start, so a multi-line request cannot mix generations even though it intentionally does not retain
  a cross-operation cache.

## Unreal reference

Unreal's `FSlateFontServices` owns long-lived game/render-thread font measure services, and each
`FSlateFontMeasure` owns per-font measurement caches. The relevant property is retained renderer/UI
ownership for product work, not making every utility call global. Zircon's retained Surface path now
has that property; its explicit one-shot adapters are a narrower compatibility boundary.

## Decision

There are zero confirmed repository-internal product hot-path calls that reconstruct a session once
per retained text owner or once per frame. No algorithm/cache change is authorized from the stale
P1-14 statement alone. In particular, do not add a TLS cache, process singleton, or hidden global
session to make one-shot microbenchmarks look warm.

If a downstream/product caller is later observed using these adapters repeatedly, capture first:

1. operation/session construction count and callsite category;
2. cold/warm p50/p95/p99 latency over plain/rich and Latin/CJK/RTL lanes;
3. allocation count/bytes, RSS, cache hit/miss/eviction, and backend call counts;
4. package power and matched retained-owner comparison when it is a real frame workload.

Only then migrate that caller to an explicit retained owner. This audit makes no latency, RSS, power,
or Unreal-equivalence claim and does not close managed Cargo/WGPU/PNG gates.
