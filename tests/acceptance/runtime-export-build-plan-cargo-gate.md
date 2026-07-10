---
related_code:
  - zircon_runtime/src/plugin/export_build_plan
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_feature_provider.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
output_records:
  - docs/plans/zircon_runtime/runtime/01/2026-07-09-tech-stack-and-dependency-governance-output-records.md
  - docs/plans/zircon_runtime/runtime/02/2026-07-09-core-spine-and-root-surface-output-records.md
---

# Runtime Export Build-Plan Cargo Gate Acceptance

Date: 2026-07-10

The default-feature locked runtime lib-test binary was produced by a successful `cargo test -p zircon_runtime --lib extensions --locked --jobs 1 --no-run` compile. Direct execution of its `export_build_plan` filter passed 67/67 with 7371 tests filtered out in 2.02 seconds.

This closes the focused export build-plan regression for Runtime 01 and Runtime 02. It does not close their remaining extensions, generated, core, full-lib, or downstream gates.
