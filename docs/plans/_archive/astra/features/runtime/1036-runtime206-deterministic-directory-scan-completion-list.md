---
doc_type: feature-completion-list
status: candidate_static_complete_managed_validation_pending
runtime: Runtime206
related_plan:
  - docs/plans/optimize/zircon_runtime/206/2026-09-29-deterministic-directory-scan.md
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
---

# Runtime1036: deterministic metadata directory scan

| Item | Implemented | Evidence | State |
|---|---|---|---|
| Sibling scan order | Buffer and sort immediate directory paths before validation and recursion, preserving lexical depth-first order within each asset root. | `zircon_runtime/src/asset/registry/rebuild.rs`; preimage `2f68e5a8`, candidate `8ee80cd7`. | candidate_static_complete_managed_validation_pending |
| Traversal regression | Check nested `.zmeta` order from the filesystem and from a known unordered input list; exclude non-sidecars. | `metadata_scan_collects_siblings_in_lexical_depth_first_order` in `rebuild/deterministic_scan_tests.rs`; candidate `c55c3c08`. | candidate_static_complete_managed_validation_pending |
| Runtime206 `ASSETREG-P1-021` | Remove `read_dir` sibling-order dependence from project metadata collection. | Source and test above; grouped managed Cargo and Release-scale scan profile remain pending. | partial |

- [x] Preserve the archived Session's existing capacity and test-module edits.
- [x] Add a focused filesystem traversal regression and sort before recursion.
- [x] Record source preimage, candidate hashes, cost, and acceptance limits.
- [x] Claim and attribute the initial four-path candidate; preserve its receipt.
- [x] Reattribute the strengthened source, test, and final record bytes to this Session.
- [ ] Run the focused regression and Runtime package check in one grouped managed validation batch.
- [ ] Measure scan latency and allocated bytes at the agreed product scale before a performance acceptance claim.
- [ ] Resolve separate root precedence and path-equivalence policies under `ASSETREG-P1-022` and `ASSETREG-P1-023`.
