# Editor Template Transform Owned-Row Clone

- Date: 2026-07-17
- Scope: `paint_template_nodes/template_node_pipeline/{draw.rs,transform.rs}` and `template_node_pipeline_tests/transform.rs`
- Acceptance state: `direct_fix_implemented_dynamic_pending`
- Plan item: `PERF-MVP-218`

## Finding

`ModelRc::row_data` returns an owned `TemplatePaneNodeData`. `draw_template_nodes_with_transform` then called `source_node.clone()` before handing the DTO to a transform. Any transformed path therefore copied the node's owned Strings, nested model handles, and other fields twice per visited row even though neither branch needed to retain the first owned value.

## Direct fix

The transform dispatch now uses an explicit `match`: a present transform consumes the owned row, while the identity branch moves the same row into its result. Existing behavioral tests cover moved, clipped, suppressed, no-transform, source-model immutability, and pixel parity. A focused source guard rejects reintroduction of `source_node.clone()`.

## Dynamic acceptance still required

- Run the current-source template pipeline and editor performance suites.
- Measure 1, 1,000, and 10,000 transformed nodes with clone bytes, allocations, and CPU p50/p95/p99.
- Preserve extension-workspace, activity/browser projector, clip, suppression, ordering, and pixel behavior before closing the item.
