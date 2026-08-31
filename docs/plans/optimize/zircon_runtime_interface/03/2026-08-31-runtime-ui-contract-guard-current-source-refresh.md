---
record_kind: optimization_validation
status: static_passed_managed_cargo_pending
created_at: 2026-08-31
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
source_snapshot_id: 2525
managed_validation_ticket: cd1d6cff000c42d18bf7d4a3abb4be84
managed_validation_submit_request: 0ffcd0e158c34e8b91dcf6e2b918bff7
related_tests:
  - tools/tests/test_runtime_ui_arranged_visibility_index_performance_contract.py
  - tools/tests/test_runtime_ui_gpu_image_cache_performance_contract.py
  - tools/tests/test_runtime_ui_paint_element_scratch_performance_contract.py
  - tools/tests/test_runtime_ui_pointer_component_state_owner_structure.py
  - tools/tests/test_runtime_ui_render_prewarm_scan_contract.py
  - tools/tests/test_runtime_ui_surface_pipeline_profile_contract.py
  - tools/tests/test_runtime_ui_table_module_structure.py
  - tools/tests/test_runtime_ui_transient_paint_metadata_contract.py
---

# Runtime UI current-source performance guard refresh

## Scope

Eight static contracts still searched pre-split parent modules or pre-identity cache types. The
production implementations were already present in their current child owners, so this change
refreshes exact source routing without changing Runtime or Runtime Interface behavior:

- arranged visibility queries are counted across `extract.rs`, `owner_text_prewarm.rs`, and
  `popup_anchor.rs` while the retired ancestor walk remains forbidden in all three;
- image preparation validates management identity, readiness identity, and prepare epoch rather
  than the removed `Arc<ResourceManagementGeneration>` representation;
- transient paint scratch and metadata assertions follow `paint_projection.rs` and
  `text_batches.rs`, retaining one reusable vector and text-only generation;
- owner-text prewarm, surface rebuild profiling, and pointer state guards follow their current
  child modules while preserving single-scan, four-exit profiling, and narrow visibility checks;
- the table mutation child keeps a bounded 400-line budget after the method family expanded.

The Runtime09 surface inventory was also synchronized exactly from 26 to 44 current entries. Its
remaining legacy/source-scan drift is not normalized here and is tracked by
`failure-2026-08-31-runtime-ui-architecture-audit-baseline-drift.md`.

## Static validation

- Focused repaired contracts: 35/35 passed.
- Batched Runtime Interface, Runtime UI and Editor asset-palette contracts: 443/444 passed.
- The sole remaining failure is the canonical Runtime09/Tooling13 architecture-audit Failure above.
- Python bytecode compilation for the eight tests and audit module: passed.
- Scoped `git diff --check`: passed.
- Current-source managed Windows ticket `cd1d6cff000c42d18bf7d4a3abb4be84` is queued for
  `cargo +1.94.1 test -p zircon_runtime_interface --locked --release --jobs 1` with source
  snapshot `2525`; no terminal result is claimed yet.

## Re-attested performance data

These are deterministic algorithm-pressure models, not product CPU/GPU/frame timings:

| Contract | Retired path | Current path | Delta |
| --- | ---: | ---: | ---: |
| Arranged visibility combined work | 25,870,467,072 ancestor visits | 202,375,168 indexed/rebuild work units | 127.834196x reduction |
| Image registry resolution | 2,415,919,104 record visits | 2,359,296 record visits | 1024x reduction |
| Retained image segment prepare | 4,194,304 batch visits | 1,536 batch visits | 2730.666667x reduction |
| Owner-text prewarm draw-order work | 300,000,000 visits | 200,000,000 visits | 100,000,000 removed; 1.5x reduction |
| Transient paint stable JSON generation | 32,768 calls | 8,192 calls | 24,576 removed; 75% reduction |
| Transient paint debug-label formatting | 32,768 calls | 0 calls | 100% removed |

No commit, push, or WeCom performance message is authorized by this record alone. Those actions
require the relevant managed validation and coordinator integration receipts.
