---
title: RG-A3 bounded RenderScene target selection repair
plan_id: astra-geometry-replay-20260905
parent_plan: astra-full-domain-20260905
owner: zircon_runtime::graphics::scene::render_scene::component_projector
session_id: astra-rg-a3-bounded-target-selection-20260927-01a0df17
status: implemented_pending_validation
---

## Scope

Targeted replay now merges ordered dependent ranges lazily. It opens one range per distinct
resource after the current stable key only when the pending revision map and resource scope
match the committed continuation. It returns at most 256 distinct primitive keys and reads
only enough memberships to identify a 257th distinct key. A changed revision or scope starts
at the beginning. The existing failed resolve/staging transaction and successful publication
cursor paths are unchanged. Resync still uses its bounded scene-key range.

For `R` distinct resources, a targeted selection reads no more than `R * 257` dependent
memberships, including duplicate memberships and lookahead. Its transient primitive-key
storage consists of at most `R` heap heads and 256 returned keys; it does not grow with the
number of dependents. The test-only counters record actual range reads and simultaneous
retained keys.

## Source evidence

- The pre-fix projector SHA-256 was
  `8fec75bab7c25f6fb0187394673ffb8afe577afe2aa8a9a4c3ff634b47df352b`.
  A narrow source guard confirmed targeted replay extended the entire dependent sets before
  truncation and did not increment the existing selection-visit counter. This establishes the
  regression's expected pre-fix failure without claiming an executed RED test.
- Added focused child tests for 10,000 shared-model dependents; overlapping model/mesh
  membership and repeated resource identities; failed resolve and staging retry; revision
  restart; and resource-scope restart. The tests assert sorted exact keys, truncation,
  nonzero and bounded actual iterator visits, retained-key peak, atomic publication, and
  continuation. They have not yet been run.
- `rustfmt --edition 2021 --check` and scoped whitespace/source guards were run after the
  implementation. Managed Cargo compilation, focused tests, and Windows Release performance
  evidence remain pending in the parent's coordinated batch. This record does not claim
  milestone acceptance.

## Changed paths

- `zircon_runtime/src/graphics/scene/render_scene/component_projector/projector.rs`
- `zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay.rs`
- `zircon_runtime/src/graphics/scene/resources/resource_streamer/geometry_replay/tests/bounded_target_selection.rs`

The source files already contained uncommitted shared-checkout work. This repair preserved that
content, and its exact bytes were transferred to the repair Session before editing.
