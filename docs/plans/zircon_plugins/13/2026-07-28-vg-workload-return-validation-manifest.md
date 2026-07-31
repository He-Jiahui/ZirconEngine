---
doc_type: milestone-validation-manifest
related_code:
  - zircon_runtime/tests/support/mod.rs
  - zircon_runtime/tests/virtual_geometry_support_descriptor_contract.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
tests:
  - virtual_geometry_support_descriptor_matches_the_plugin_compute_workload
  - virtual_geometry_debug_snapshot_contract
  - graphics::tests::plugin_feature_compile::gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph
---

# Virtual Geometry Workload Return Validation Manifest

Plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
Milestone: M5
Status: in_progress
Files: ["docs/plans/zircon_plugins/13/failure-2026-07-15-virtual-geometry-runtime-support-compute-workload-drift.md", "docs/plans/zircon_plugins/13/2026-07-28-vg-workload-return-validation-manifest.md", "zircon_runtime/tests/support/mod.rs", "zircon_runtime/tests/virtual_geometry_support_descriptor_contract.rs", "zircon_plugins/virtual_geometry/runtime/src/lib.rs"]

This manifest binds the current failure-return source set to a managed M5 validation run. It is not an acceptance record.
