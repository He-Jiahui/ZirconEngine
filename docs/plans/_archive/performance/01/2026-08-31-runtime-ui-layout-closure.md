---
related_code:
  - zircon_runtime/src/ui/layout
  - zircon_runtime/src/ui/surface/surface/rebuild
  - zircon_runtime/src/ui/surface/virtual_list_materialization.rs
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/02/failure-2026-07-18-runtime-ui-incremental-layout-still-full-tree.md
  - docs/plans/zircon_editor/editor_ui/02/failure-2026-07-18-runtime-ui-slot-lookup-and-taffy-tree-rebuild.md
  - docs/plans/zircon_editor/editor_ui/02/failure-2026-07-18-runtime-ui-virtualization-full-child-scan.md
write_scope: []
status: pending
---

# Runtime UI layout closure

This is a current-source static revalidation of constraints, container arrange,
incremental layout, slot indexing, Taffy bridging, responsive layout and list
virtualization. It remains pending because current Cargo fails in broad foreign
integration work and no F4 product profile is available. No Rust source was
changed.

## Scope and source state

- `zircon_runtime/src/ui/layout/**`: 33 Rust files, 9,672 physical lines, 8,924
  nonempty lines, 324,559 bytes, 60 tests, two ignored manual performance tests
  and two include sites. Sorted raw-content SHA256:
  `6f54376e6a162f82f9ca47b691e52b70dfc14be55b350a71e26b2a0d4cf77aea`.
- The scope is broad foreign modified/added work. Isolated rustfmt passes 18 of
  33 files; the other 15 fail current import or assertion formatting. Scoped
  diff-check reports line-ending warnings only. Existing work was preserved.
- The E-drive `zircon_runtime` check fails with 214 non-focused integration
  errors. No focused Rust test executed. The two ignored tests measure a
  constraint final-sum short circuit and single-pass material metadata lookup;
  they are synthetic release wall-clock evidence, not product acceptance.

## Closed findings and positive work

- The former global slot scan is closed in current source. `UiLayoutSlotIndex`
  serves parent/child lookup and repairs cached edges when the tree changes.
- The former blanket incremental-layout false claim is closed. Surface rebuild
  has local arranged geometry/input, hit-grid, render-cache and navigation
  patches, with explicit full-rebuild fallback when the patch contract fails.
  Dirty-set admission is capped before choosing the incremental route.
- The former first-party materialized virtual-list full-child scan is closed.
  The surface publishes an indexed materialized projection, and fixed-extent
  arrange operates on materialized child slots with O(1) item offsets.
- Responsive candidates and thresholds are retained. Root size changes inside
  the same threshold band can avoid responsive mutation. Linear, masonry and
  measurement scratch vectors are pooled and retain capacity.
- Constraint and material metrics have focused semantic tests, and the layout
  tests cover slot parity, incremental patches, virtualization, responsive
  thresholds, container families and failure fallbacks.

These current facts supersede the broad July statements that every incremental
layout remained a full-tree rebuild, every slot lookup scanned all slots, or the
materialized fixed-list path iterated all logical children. The historical
failure records remain useful provenance, but those claims must not be carried
forward as current defects.

## Retained findings

1. **Taffy nodes are reconstructed per affected container (P0/P1).** A pooled
   `TaffyLayoutBridgeScratch` retains vector and tree capacity, but each
   `begin_children` clears the `TaffyTree`, creates one leaf per visible child
   and creates a new parent before compute. Nested container recursion can hold
   one high-water tree per nesting depth in the thread-local pool. This avoids
   allocator churn relative to a fresh local tree but is not a retained surface
   layout graph. Stable/locally changed Taffy subtrees should upsert exact node
   styles/edges and remove retired nodes in one generation-owned tree.
2. **Layout-engine diagnostics are unconditional work (P0/P1).** Every arranged
   container pushes a `UiLayoutEngineSelection`. Finishing constructs the
   sequence and derives fallback counts; incremental publication builds route
   maps and replacement rows. Persistent sequence sharing reduces accepted
   frame copying, but there is no `Disabled/Counters/Sampled/Full` gate at the
   producer. Default rendering should record no per-container diagnostic rows;
   counters mode should use dense fixed IDs without labels or route objects.
3. **Generic scroll virtualization remains subtree-linear (P1).** If no
   materialized projection exists, `ScrollableBox` validates/iterates every
   child, advances the cursor and recursively clears every offscreen subtree.
   The first-party materialized fixed-list path is positive, but the public
   generic container still permits O(N plus offscreen descendants) work. A
   virtualized route needs a provider/index contract; otherwise it must be
   explicitly classified as a finite non-virtual container with a count budget.
4. **Responsive dirty work retains owned metadata (P1).** Candidate refresh
   copies component strings and selected TOML attributes into responsive
   definitions. Full `for_tree` scans are now initialization/full-refresh work,
   not stable-frame work, but dirty storms still lack one node/attribute/byte
   proposal and can produce broad candidate generations before layout.
5. **Scratch retention is not aggregate-accounted (P1).** Thread-local linear,
   masonry, measure and Taffy pools preserve high-water capacities. This is
   desirable for stable reuse, but aggregate bytes by worker/thread and nesting
   depth are absent from UI residency and diagnostics. A transient extreme
   layout can retain workspace capacity indefinitely.
6. **Incremental products still have split generation authority (P1).** Layout,
   arranged tree, hit grid, render cache, navigation and engine-selection
   reports are patched through related but separate structures. The current
   fallback behavior is safe, yet a failed downstream patch can repeat or
   discard work without one candidate `UiSurfaceLayoutGeneration` receipt.
   Publish the whole accepted projection atomically after every child patch is
   validated.

## Architecture handoff

1. Compile one immutable `UiSurfaceLayoutGeneration` containing topology,
   styles, slots, responsive state, Taffy node identity, arranged geometry and
   exact downstream patch identities. Dirty work builds one checked candidate;
   successful publication advances every dependent generation atomically.
2. Retain one Taffy graph per surface generation. Upsert changed nodes/styles
   and edges, remove exact retired nodes and compute only admitted roots. Keep
   pooled temporary measurement vectors, but account current/high-water bytes
   per worker and bound depth/count before mutation.
3. Make virtualized containers consume a typed item provider and materialized
   window generation. Logical item count, visible/overscan slots, extent/index
   bytes and subtree work are pre-admitted. The generic child-vector fallback is
   finite-container behavior and cannot silently claim virtualization.
4. Compile responsive definitions once from template/style generations. Root
   width selects a dense threshold band; dirty updates replace only affected
   definitions under node/attribute/byte budgets.
5. Diagnostics `Disabled/Counters/Sampled/Full` gates engine-selection and
   patch reports at production. Disabled emits no rows, maps or labels;
   Counters uses dense engine/fallback IDs; Full borrows compiled labels.
6. Attach exact visit, allocation, retained-workspace and fallback facts to the
   accepted layout generation, then measure F4 with current-source Cargo and
   product UI before choosing an algorithmic micro-optimization.

## Evidence and acceptance gates

Bevy's `UiSurface` retains one entity-to-Taffy map and one `TaffyTree`, updates
or creates exact nodes in `upsert_node`, computes on the retained graph and
removes exact retired entities. This supports a persistent surface Taffy owner;
it does not prescribe Zircon ECS or layout ABI. Unreal Slate's retained
invalidation and lazy text-view path similarly support generation-qualified
dirty work rather than unconditional full projection.

The selected repository source-contract and pressure suites pass 115/115 tests
across document index, text infrastructure, incremental layout, edge evidence,
order authority, rebuild budgets, report aggregation, slot indexing,
materialized projection, Taffy parent work and surface materialization. These
models validate current ownership shapes, not product latency or power.

M0 adds RED counters for stable/dirty Taffy node creates, diagnostic rows,
generic versus materialized list visits, responsive copies, scratch high-water
bytes and patch rollback. M1-M3 establish the surface/Taffy/virtual-provider
generations. M4-M6 gate diagnostics and collect current F4 evidence.

Acceptance covers nodes/containers/slots 0/1/64/1K/10K/cap+1; depth
1/8/64/cap+1; linear/grid/masonry/Taffy/free/overlay; exact stable, style-only,
geometry-only, topology and root-threshold changes; generic/materialized lists
with 0/1/100/10K/1M logical rows; patch success/failure/rebase; diagnostics
Disabled/Counters/Full; and 1/2/N workers. Report proposal/measure/arrange/patch
latency, tree/node/slot/subtree visits, Taffy creates/upserts/removes, Vec/map/
String allocations and bytes, scratch current/high-water residency, fallback
reasons and accepted/rebased/fault generations.

Hard gates: current source builds; stable retained Taffy nodes are not rebuilt;
dirty work is affected-generation proportional; materialized virtualization
does not visit logical offscreen children; generic fallback is count-bounded;
failed patches publish no mixed surface state; Disabled diagnostics own zero
rows/labels/maps; workspace residency is bounded and observable; diagnostics
match actual work. No benchmark artifact or micro-fix is warranted before these
ownership corrections.
