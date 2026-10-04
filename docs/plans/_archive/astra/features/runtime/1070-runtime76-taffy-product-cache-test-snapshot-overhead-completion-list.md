---
doc_type: feature-completion-list
status: source_applied_managed_validation_pending
runtime: Runtime76
related_plan:
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
proposed_target: docs/plans/astra/features/runtime/1070-runtime76-taffy-product-cache-test-snapshot-overhead-completion-list.md
---

# Runtime1070: Taffy product-cache snapshot overhead

| Item | Candidate change | Evidence or gate | State |
|---|---|---|---|
| Production snapshot overhead | Compile the last-update snapshot map and its write/prune path only for tests; clear the previous test snapshot at update-error and discard boundaries. | The independently reviewed source repair is applied after exact preimage checks, live claims and attribution. The raw source preimage and candidate hashes are in the accompanying JSON manifest. | source_applied_managed_validation_pending |
| Update-error regression | Add an observable cache regression proving a failed exact update drops the product and does not expose the preceding test snapshot. | The regression is in the candidate source; managed execution is pending. | source_applied_managed_validation_pending |
| Runtime76 RUL-P1-019 | Keep the full nested Taffy graph requirement open. This bounded snapshot repair does not replace the per-parent shallow products with one nested graph. | docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md | open |
| Runtime package validation | Run the normal managed `zircon_runtime` library check with `cfg(test)` off, plus grouped library tests with `cfg(test)` on after exact source admission. | No Cargo or test run is included in this candidate. | pending |
| Release performance qualification | Measure the admitted layout workload against a product-grounded baseline; do not infer performance acceptance from cache counters or unit tests. Release library unit/ignored benchmarks compile with `cfg(test)` and retain this instrumentation. | No release product samples or performance claim are included. | pending |

## Caller and regression references

The production route is `pass/arrange.rs::try_arrange_taffy_owned_children` → `pass/taffy_arrange.rs::try_arrange_taffy_owned_children_with_scratch` → `taffy_bridge/compute.rs::compute_taffy_child_frames` → `TaffyParentProductCache::compute_child_frames` in `taffy_bridge/product_cache.rs`. The cache is owned and retained/discarded through `pass/slot.rs`; bridge errors continue through the arranger's existing fallback branch.

Existing semantic coverage includes `taffy_parent_product_root_resize_reuses_all_child_contracts` and `taffy_exact_unchanged_contract_reuses_solve_and_publishes_only_required_child` in `pass/arrange/tests.rs`. The candidate adds `failed_exact_update_discards_product_and_previous_snapshot` directly beside the cache implementation. None of these tests substitutes for product-grounded Release performance evidence.

## Candidate scope

Only product_cache.rs changes. compute.rs, taffy_bridge/mod.rs, pass/slot.rs, pass/mod.rs, pass/engine.rs, and pass/taffy_arrange.rs are unchanged required companions for the reviewed declarations and caller closure. They retain the frozen working-tree bytes; do not include them in an apply patch for this repair.
