# Surface input font-collection owner record (2026-08-30)

## Scope

Affected plans: Text03, Text08, Text09, Text index, optimize 11B, engine code structure convention,
and the June code-review findings. This record is the writable canonical status handoff while the
large Text03/Text09/index/11B files are held by another writer and reject atomic patch replacement.

## Finding

Runtime retained surfaces already shape and lay out through their Core-owned
`FontCollectionService`. When a resolved glyph artifact is missing, however, caret, selection, IME
composition rectangles, and pointer hit testing rebuilt simple LTR source metrics with the
process-global `DirectTextShapeRunProvider`. One input query could therefore combine layout from one
collection with face metrics and fallback from another.

## Unreal reference

`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontMeasure.cpp:48-54` creates
`FSlateFontMeasure` with a concrete `FSlateFontCache` and retains it. The corresponding member is
declared in `Public/Fonts/FontMeasure.h:239`; measurement and `FindCharacterIndexAtOffset` use that
same owner. Zircon input geometry must preserve the same owner relationship after a Surface selects
its collection.

## Completed implementation

- Added `FontCollectionTextShapeRunProvider`, which shapes through an immutable
  `FontCollectionSnapshot` and reports that snapshot's full collection revision.
- Replaced the global provider inside `SourceLineGeometry` with the collection-bound provider.
- Added explicit collection geometry/hit-test entrypoints. Existing Editor/standalone compatibility
  entrypoints acquire the declared process-owner snapshot explicitly.
- Exposed the retained measure session snapshot through `UiTextMeasureCache`.
- Captured one Surface snapshot per IME context refresh and shared it across caret/composition
  geometry; captured one Surface snapshot per pointer hit-test query.
- Added a generation fence because neutral `UiResolvedTextLayout` has no collection revision. Source
  metrics are reshaped only when the Surface's observed layout generation equals the captured snapshot
  generation; otherwise geometry fails closed to the published artifact/glyph advances until layout
  recomputation.
- Added an independent-collection Rust regression requiring shaped face handles to retain the
  injected collection id and revision.

The shaping, line-break, cluster, caret, selection, and hit-test algorithms are unchanged. The added
work is one Arc-backed snapshot lease per input query/context refresh, not per glyph; the old direct
provider already acquired a shared snapshot before shaping.

## Evidence and status

- Static ownership suite: 19/19 passed.
- Rustfmt check: passed for all touched Rust files.
- Scoped diff check: passed, with line-ending warnings only.
- Rust behavior test: written, not run because the existing Cargo/E-drive validation blocker is being
  bypassed in favor of non-validation work.
- Real IME/pointer input, Cargo, WGPU/PNG, 31-sample latency/allocation, RSS, package power, and matched
  Unreal workload: pending.
- Screenshot: not produced; this slice changes ownership and has no independently valid product frame.

Status:
`surface_input_geometry_collection_bound_static_implemented /
process_global_source_metric_recovery_removed /
stale_layout_current_snapshot_mixing_rejected /
managed_product_validation_pending`.
