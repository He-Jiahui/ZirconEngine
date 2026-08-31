---
related_code:
  - zircon_editor/src/ui/timeline/mod.rs
  - zircon_editor/src/ui/timeline/model.rs
  - zircon_editor/src/ui/timeline/ruler.rs
  - zircon_editor/src/ui/timeline/track_list.rs
  - zircon_editor/src/ui/timeline/keyframe_lane.rs
  - zircon_editor/src/ui/timeline/section_lane.rs
  - zircon_editor/src/ui/animation_editor/session/sequence.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
implementation_files:
  - zircon_editor/src/ui/timeline/mod.rs
  - zircon_editor/src/ui/timeline/model.rs
  - zircon_editor/src/ui/timeline/ruler.rs
  - zircon_editor/src/ui/timeline/track_list.rs
  - zircon_editor/src/ui/timeline/keyframe_lane.rs
  - zircon_editor/src/ui/timeline/section_lane.rs
  - zircon_editor/src/ui/timeline/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/ui/timeline/tests.rs
  - cargo test -p zircon_editor --lib ui::timeline::tests --locked --jobs 1
doc_type: module-detail
---

# Timeline Foundation

## Purpose

`zircon_editor::ui::timeline` is the renderer-neutral shared foundation for animation clips,
montages, and later time-based domain editors. It owns time ranges, ruler tick projection, snapping,
track-list projection, typed key/section selection, visible-key projection, and section-overlap
validation. It does not own an animation sequence, montage, playback session, runtime evaluation,
or persisted track data.

The module replaces the architectural role previously overloaded into the private single-track
`timeline_strip` path. It deliberately has no global cache, template-node dependency, or retained
host dependency. The old host implementation is not a compatibility API for this module; it is
scheduled for an owner-complete hard cut after the retained-host projection boundary is available.

## Domain Boundary

`TimelineModel` exposes the registered `TimelineEditorDescriptor`, registered track descriptors,
range, playhead, immutable track views, and the domain's reversible mutation protocol. Its associated
delta remains owned by the asset family. This prevents UI code from creating a second authority for
`AnimationSequenceAsset`, montage sections, or future sequence assets.

`TimelineTrackView` is a display projection that carries the domain's stable track identity,
`value_kind`, keys, and sections. `lane_kind_for_value` maps registered value kinds to common lane
renderers: curve, boolean, event, section, or ordinary keyframe. Unknown kinds remain keyframe
lanes so a missing visual specialization does not discard authored data.

## Time Interaction

`TimelineRange` normalizes invalid/reversed input and clamps interaction results. The ruler selects
1/2/5-based intervals from available width and label spacing without retaining a process-global tick
cache. `TimelineSnapSettings` resolves the nearest in-threshold grid position or authored boundary,
with a stable lower-time tie break; all results stay within the model range.

Timeline selection is a `BTreeSet` of typed key and section references, so multi-track selections
deduplicate deterministically without encoding identity into display strings. Keyframe lane data
borrows the domain view. A section policy explicitly chooses whether sections may overlap; when
forbidden, ranges use half-open semantics, therefore an end time equal to the next start time is
valid.

## Scope

This M2.1 foundation does not yet paint retained widgets, bind animation-session track mutations,
own a preview scene, or migrate the old template `timeline_strip` consumers. The hard cut must occur
as one retained-host migration: switch all consumers to the shared model, remove the old module and
its static cache, then run current-source interaction and performance validation. No adapter or
re-export from `timeline_strip` to this module is permitted.
