---
related_code:
  - zircon_editor/src/ui/sample_grid
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/sample_grid.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/sample_grid.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sample_grid
plan_sources:
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
tests:
  - zircon_editor/src/ui/sample_grid/tests.rs
  - tools/tests/test_editor07_sample_grid_generation_contract.py
doc_type: module-detail
---

# Sample Grid Generation

`ui::sample_grid` owns the immutable domain projection shared by Blend Space and future sample-grid editors. Raw template attributes are normalized once, then converted into `SampleGridGeneration` before host painting. The retained host does not own a second raw tick or point model.

Each generation contains preformatted `SampleGridTick` slices and immutable `SampleGridPoint` slices. Its stable FNV-1a static token covers axis labels, normalized ranges, and tick values. Its dynamic token covers normalized ranges plus point position, label, and selection because range changes move every projected point. Selection and drag changes therefore cannot invalidate static surface/grid/text identity; axis-label and tick-only changes cannot invalidate point geometry; range changes invalidate both domains.

The host currently consumes these typed slices directly, eliminating per-paint tick formatting and parallel `ModelRc` traversal. A preformatted label is still copied into the current owned text paint command, and dashed lines/diamonds still expand to multiple quads. Generation-aware compiled command caching and bounded host batch primitives remain required before the command-amplification failure can close.
