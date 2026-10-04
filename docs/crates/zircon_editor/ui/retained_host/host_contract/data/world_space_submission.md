---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/node.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/scene.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/tests.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/node.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/scene.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-21 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - host_contract data world-space submission builder/model/test ownership scan
  - host_contract data world-space submission builder node/pane/scene ownership scan
  - scoped whitespace scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-21 attempted after nearby M3.S2 splits; timed out after 300 seconds before actionable editor diagnostics)
doc_type: module-detail
---

# World-Space UI Surface Submission

`data/world_space_submission.rs` is the retained-host data entry for projecting template-pane UI nodes into world-space UI surface submissions. It remains a structural module root: model shape lives in `world_space_submission/model.rs`, collection and sorting logic is rooted at `world_space_submission/builder.rs`, and private regression coverage lives in `world_space_submission/tests.rs`.

## Purpose

World-space UI submissions describe editor-authored template nodes that should be presented as world-space surfaces instead of ordinary screen-space chrome. The data records carry both viewport hit bounds and world transform facts: position, rotation, scale, physical width/height, pixels-per-meter, billboard/depth-test flags, render order, and camera target.

## Module Ownership

`world_space_submission/model.rs` owns `WorldSpaceUiSurfaceSubmission` plus the viewport hit-bound helper. It does not traverse panes or inspect template nodes.

`world_space_submission/builder.rs` is now the structural builder entry. `builder/node.rs` owns submission collection from a `ModelRc<TemplatePaneNodeData>`, filters nodes through `world_space_enabled`, derives missing world extents from arranged frame size and pixels-per-meter, skips non-positive projected extents, and performs per-surface stable render-order sorting. `builder/pane.rs` owns pane-family traversal and pane-surface id construction. `builder/scene.rs` owns aggregate dock/floating-window traversal plus final host-scene stable sorting.

`world_space_submission/tests.rs` owns the private helper coverage for candidate filtering, frame-to-world extent projection, stable ordering, and viewport hit-bound checks. The root file only wires these owners together and re-exports the public internal contract for existing `data::*` consumers.

## Behavior Model

Pane traversal covers the standard dock panes plus floating-window headers and active panes. Pane ids are appended to their surface ids when present so submissions from different pane families remain distinguishable after aggregation.

Sorting is deterministic. Per-surface collection sorts by `render_order`, then `node_id`, then `control_id`; host-scene aggregation also compares `surface_id` before node/control ids so multi-pane and floating-window submissions remain stable across frames.

Explicit `world_width` and `world_height` win when positive. If they are missing, the builder projects frame pixels into meters through `frame_extent / world_pixels_per_meter`. Non-positive explicit sizes, non-positive frame extents, or zero pixels-per-meter produce no submission instead of creating a degenerate world surface.

## Test Coverage

The module-local tests cover enabled-node filtering and ordering, projected extents when world size is omitted, and viewport hit-bound checks without requiring an RHI or presenter backend. During this implementation slice, validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an ownership scan confirming inline tests and traversal/model responsibilities no longer live in the root file, a scoped whitespace scan, and scoped `git diff --check`. The package-level editor Cargo check was attempted in the same M3.S2 pass and timed out after 300 seconds without actionable editor diagnostics; full Cargo tests remain deferred to the milestone testing stage per the user request.

The 2026-06-21 builder node/pane/scene split reduced `world_space_submission/builder.rs` from 146 lines to a 5-line structural entry. The new child owners are `builder/node.rs` at 81 lines, `builder/pane.rs` at 32 lines, and `builder/scene.rs` at 40 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a world-space submission builder node/pane/scene ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred to the milestone testing stage per the user's instruction.
