---
related_code:
  - zircon_editor/src/ui/graph/mod.rs
  - zircon_editor/src/ui/graph/model.rs
  - zircon_editor/src/ui/graph/canvas.rs
  - zircon_editor/src/ui/graph/commands.rs
  - zircon_editor/src/ui/graph/node_widget.rs
  - zircon_editor/src/ui/graph/routing.rs
  - zircon_editor/src/ui/animation_editor/session/graph.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
implementation_files:
  - zircon_editor/src/ui/graph/mod.rs
  - zircon_editor/src/ui/graph/model.rs
  - zircon_editor/src/ui/graph/canvas.rs
  - zircon_editor/src/ui/graph/commands.rs
  - zircon_editor/src/ui/graph/node_widget.rs
  - zircon_editor/src/ui/graph/routing.rs
  - zircon_editor/src/ui/graph/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/ui/graph/tests.rs
  - cargo test -p zircon_editor --lib ui::graph::tests --locked --jobs 1
doc_type: module-detail
---

# Graph Foundation

## Purpose

`zircon_editor::ui::graph` provides the editor-only, renderer-neutral mechanics shared by graph
toolkits. It owns canvas coordinates, pan/zoom, marquee selection, drag-result projection, node
presentation, port compatibility defaults, required-input diagnostics, and deterministic route
geometry. It does not own graph asset data or mutate runtime state.

The initial consumer is the existing animation graph editor. State-machine and behavior-tree
toolkits consume the same boundary through their own `GraphModel` adapters. In particular, this
module does not mirror `AnimationGraphAsset`, create a second graph document, or add a compatibility
path for the former private graph implementation.

## Model Boundary

`GraphModel` exposes immutable node and edge views, the registered `GraphNodePaletteDescriptor`,
the domain's connection verdict, structure constraint, and its reversible mutation protocol. The
concrete delta type remains associated with the domain model: animation, state machine, and behavior
tree assets have different authoring payloads, so their data must not be erased into a shared UI
authority.

`default_connection_verdict` is available to domains whose ordinary rule is output-to-input
connection with equal `value_type`. It rejects missing nodes or pins, malformed directions,
duplicate edges, type mismatches, cycles for DAG/tree constraints, and a second incoming edge for a
tree input. A domain can return a richer `ConnectVerdict` without teaching the canvas about its
asset format.

Required input diagnostics are derived from graph views and edges. They intentionally do not modify
the graph or build presentation-only copies of a domain asset.

## Palette Provenance

Each `GraphNodePaletteDescriptor` carries a schema version and its effective owner. Direct
registrations use the explicit `editor.extension.direct` owner; plugin catalog materialization
overwrites that value with the package id before the descriptor enters the active catalog. An owner
must be non-empty and a schema version must be at least one, so catalog consumers can attribute a
palette without inferring ownership from a node name or UI route.

The active extension catalog is rebuilt from active plugin registrations for every manager
generation. Consequently an unloaded plugin's palettes disappear with the previous catalog
generation, rather than surviving in a UI-local cache. Per-node/pin migration, unknown-node
preservation, and semantic compiler compatibility are still separate M1 work; palette provenance
does not claim those capabilities.

## Canvas And Presentation

`GraphCanvasState` owns only view state. Zoom is clamped to `0.2..=4.0` and preserves the graph
coordinate below the pointer. Marquee selection is a deterministic `BTreeSet`; drags emit ordered
`GraphNodeMove` values in graph-local coordinates. A toolkit converts those moves into its own
transaction, keeping undo/redo and asset persistence in the domain authority.

`GraphNodePresentation` is a small retained-UI projection containing title, pin counts, attachment
labels, and selection state. Attachments model node-internal items such as behavior-tree decorators
and services without turning them into independent graph nodes.

`route_connection` has orthogonal and Bezier geometry policies. Route computation has no widget,
asset, global cache, or runtime dependency; retained rendering consumes its resulting points.

## Commands And Transactions

`aligned_node_moves` implements the six outer-bounds alignment modes used by mature graph editors.
It returns only changed graph-local positions, ordered by node id, and the graph domain converts the
result into its own delta.

`GraphModel::apply` and `GraphDeltaCommand<Model, Context, Target>` connect a domain delta to the
shared 03 `EditCommand` history. The command retains only its inverse delta between undo/redo, and
carries a stable document target so a multi-document context cannot apply a graph mutation to a
different asset. Domain mutation errors must be atomic; a mutation that has taken effect returns its
inverse in `GraphMutationEffect::Applied`.

`GraphClipboardModel` deliberately leaves subgraph serialization, node-id remapping, and paste
delta construction with the domain asset owner. The shared graph module supplies only selected-node
and paste-anchor vocabulary, preventing a second graph serialization authority in UI code.

## Scope

This module now covers M1.1 plus the reusable M1.2 command contracts. Binding a concrete
`GraphModel` adapter to animation/state-machine assets, retained-host painting, graph toolbar
registration, and concrete domain clipboard implementations remain subsequent Editor07 slices.
Those integrations must use this module rather than retaining or creating private graph-canvas
behavior.
