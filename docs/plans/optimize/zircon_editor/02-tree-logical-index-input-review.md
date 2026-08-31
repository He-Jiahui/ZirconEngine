# Tree Logical-Index Input Review

Status: `review_finding`

## Finding

Ordinary button hover, press, and focus reduction is an `O(1)` flag update. Tree pointer selection and rename are not: the Surface default-interaction path calls `tree_node_ids`, builds a borrowed ID `Vec` plus a deduplicating `HashSet` for all `N` logical nodes, and then performs `position`. A subsequent component `SelectOption` can repeat the same class of work through `ordered_node_ids + position`.

This leaves virtualized trees coupled to logical collection size. A click or double-click on one visible slot can still cost `O(N)` even when materialization and hit testing are bounded by the visible window.

## Target Authority

Use the existing Table item `row_index` metadata path as the repository precedent. Tree materialization and hit authority must publish a stable `(logical_index, tree_generation)` for each materialized item. Pointer routing consumes that index after checking generation and identity.

- Single selection, additive selection, and rename: `O(1)` logical lookup.
- Range selection: `O(Delta)` over the anchor-to-target interval.
- Full logical scan: generation/identity mismatch fallback only, with an explicit reason counter.
- The component reducer must not repeat a full logical scan after the Surface already validated the target.

## Acceptance

- Preserve duplicate-ID first-occurrence behavior.
- Cover virtual slot rebinding and template hot-reload generation changes.
- Stress first, middle, and last item clicks at 10,000 and 100,000 nodes.
- Include narrow and wide range-selection cases.
- Normal product clicks must report zero logical-node visits, zero temporary ID-vector entries, zero temporary dedup-index entries, and zero reducer rescan visits.
- Compare input-to-damage p50/p95/p99 and CPU/RSS before and after with current-source managed Editor artifacts.

## Pressure Budget

`tools/runtime_tree_logical_index_pressure.py` models 100,000 logical nodes, 1,000 single interactions, and 1,000 range interactions of width 10. With two current-style full logical passes per interaction, the model counts 400,000,000 logical-node visits, 400,000,000 temporary ID-vector entries, and 400,000,000 temporary dedup-index entries. The target authority budgets 10,000 range visits and zero full-tree temporary index entries; single selection and rename use the published index without logical-node visits. The resulting logical-visit ratio is 40,000x.

This is an algorithm budget, not a product measurement. Focused deterministic model tests pass 3/3. Artifact: `E:\zircon-profiles\runtime-tree-logical-index-pressure-20260828.json`, SHA-256 `CED16C7862A55F3FF5C9B0DFC1EF992D8F39D1AFE5951369D802A5C0685CBA9B`.

## Current Constraint

The Runtime tree interaction owner files currently contain shared, unattributed worktree changes. This review is read-only for those paths. No production implementation or Cargo result is claimed here.
