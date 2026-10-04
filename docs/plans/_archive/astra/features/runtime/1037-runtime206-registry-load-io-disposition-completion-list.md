---
doc_type: feature-completion-list
status: candidate_static_complete_managed_validation_pending
runtime: Runtime206
related_plan:
  - docs/plans/optimize/zircon_runtime/206/2026-09-29-registry-load-io-disposition.md
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
---

# Runtime1037: registry load I/O disposition

| Item | Implemented | Evidence | State |
|---|---|---|---|
| Missing file | Rebuild only when the registry read reports `NotFound`. | `persistence.rs`; focused missing-file regression. | candidate_static_complete_managed_validation_pending |
| Existing unreadable path | Return the registry I/O error without scanning sources or writing a replacement. | `unreadable_registry_does_not_rebuild_from_project_sources` uses a directory at the registry path; old behavior reports the invalid asset root. | candidate_static_complete_managed_validation_pending |
| `ASSETREG-P1-015` | Remove the I/O-to-rebuild branch and a separate existence probe. | Source and tests above; decode/version/duplicate disposition remains separate. | partial |

- [x] Preserve the preexisting import-order change in the shared source file.
- [x] Add tests for unreadable and missing registry paths.
- [x] Route non-`NotFound` I/O failures back to the caller before rebuild.
- [x] Record static evidence and the unmeasured performance claim boundary.
- [ ] Run the focused regressions and Runtime package check in one grouped managed validation batch.
- [ ] Measure Release open latency and establish an acceptance target before a performance pass.
- [ ] Resolve corrupt/version/duplicate migration disposition and durable recovery separately.
