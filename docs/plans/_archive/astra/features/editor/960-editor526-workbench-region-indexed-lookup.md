---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor526-workbench-region-indexed-lookup.md
implementation_files:
  - zircon_editor/src/ui/workbench/autolayout/workbench_skeleton.rs
tests:
  - zircon_editor/src/ui/workbench/autolayout/workbench_skeleton.rs
---

# Editor960 Editor526 workbench-region indexed lookup

Canonical `EditorRegion` queries now check the enum's expected slot before
falling back to the legacy linear search for reordered payloads. The focused
regression keeps serialized-order permutations resolvable. Marker
`EDITOR526_WORKBENCH_REGION_INDEXED_LOOKUP_BENCH_V1` models 65,536 canonical
lookups and the 229,372-to-65,536 candidate-check reduction. Managed Editor
Release validation remains pending.
