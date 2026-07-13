---
related_code:
  - zircon_runtime/src/script
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger
plan_sources:
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
output_records:
  - docs/plans/zircon_runtime/runtime/13/2026-07-09-script-binding-and-reflection-output-records.md
status: owned_script_gates_accepted_animation_source_fixed_package_rerun_pending
---

# Runtime 13 Script Binding Current Gates

Date: 2026-07-11

The script-binding audit reports 19/19 source owners, 3/3 test roots, six fixed
host modules, 52 fixed functions, builtin/gameplay/macro callback counts
11/39/2, 11 capabilities, nine guard owners, no native ECS ABI references, no
oversized test owners, and `risks = []`.

The managed `script` snapshot was 347 passed / 9 failed. After repairing the
priority-review and script-record routing guards, a newly compiled default-
feature package reaches 354 passed / 2 failed. Those failures were the Vampire
state-machine artifact deserializer and active Render pipeline feature
descriptors. The Animation owner has since closed the typed fallback handoff in
`fixed-2026-07-11-animation-state-machine-infallible-conversion.md`: the
current/v3/v2/v1 decode chain uses explicit typed conversions, and its plugin
state-machine/StateKind focused gates pass. This removes the known source-level
cause, but no current Runtime `script` package rerun is claimed while all build
drives remain below the repository's safe-space threshold. Current-source route
validation passes 8/8 and the focused ledger validation passes 3/3. Runtime 13
owned script-binding gates are accepted, but the milestone remains
`in_progress` until the package rerun, the Render-owned descriptor gate, and the
prescribed full regression are green.
