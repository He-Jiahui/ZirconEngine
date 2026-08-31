# One-shot text provider snapshot owner record (2026-08-30)

## Scope

Affected plans: Text09, Text index, optimize 11B, engine code structure convention, and the June
code-review findings. This record tracks the compatibility-provider generation-boundary repair after
Surface input geometry was bound to its Core-owned collection.

## Finding

`DirectTextShapeRunProvider` was a zero-state compatibility object. Its horizontal and vertical
requests delegated to the process-global shape wrapper, which acquired a current font snapshot for
each request. A multi-line standalone/editor measure or source-range operation could therefore mix
two font generations when a publication happened between requests.

## Unreal reference

`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontMeasure.cpp:48-54` creates
`FSlateFontMeasure` with a concrete `FSlateFontCache` and stores it in the owner. The corresponding
`FontCache` member is declared in `Public/Fonts/FontMeasure.h:239`; measurement and character-index
queries use that same cache. Zircon's one-shot compatibility provider now follows that operation
ownership rule.

## Completed implementation

- `DirectTextShapeRunProvider` now captures one immutable `FontCollectionSnapshot` at construction
  and reports the captured revision.
- Horizontal and vertical requests use
  `shape_text_with_diagnostics_in_font_collection` with the captured snapshot, so publication cannot
  switch the collection within one operation.
- Process-owner wrappers remain available for Editor/standalone compatibility; retained Runtime
  surfaces continue using `SharedTextLayoutSession` or the explicit collection-bound provider.
- Existing test helper construction was migrated from a unit literal to `Default`; a regression
  publishes a new generation after provider construction and requires returned glyph face handles to
  retain the original collection identity.

No shaping algorithm, line-break policy, cache capacity, glyph loop, or renderer behavior changed.

## Evidence and status

- Static source contract suite: 20/20 passed.
- Python compile check: passed.
- Targeted Rustfmt (`skip_children=true`): passed for all touched Rust files.
- Scoped `git diff --check`: passed, with existing LF/CRLF conversion warnings only.
- Rust publication-mid-operation behavior test: written, not run because the managed Cargo/E-drive
  validation blocker is being bypassed in favor of non-validation work.
- Real WGPU/PNG, IME/pointer input, 31-sample latency/allocation, RSS, package power, and matched
  Unreal workload: pending.
- Screenshot: not produced; this owner-boundary slice has no independently valid product frame.

Status:
`one_shot_provider_snapshot_bound_static_implemented /
cross_generation_metric_mix_removed /
managed_product_validation_pending`.
