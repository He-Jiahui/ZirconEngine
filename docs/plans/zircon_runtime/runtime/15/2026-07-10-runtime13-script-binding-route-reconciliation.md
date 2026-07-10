# Runtime 15 / Runtime 13 script-binding route reconciliation

Date: 2026-07-10

Status: `runtime_15_runtime13_script_binding_route_archive_reconciliation_static_passed`

The `script_binding/split_layout.rs` guard now reads numbered Runtime13/15/Frameworks02 output records and the concrete `runtime_absorption_core_rows.rs` / `structure_route_maps/core_route_rows.rs` status children. It does not restore parent-plan detail mirrors or compatibility routes.

Verification:

- initial available-binary `script_binding`: 5 passed / 1 stale guard failed;
- current standalone script-binding suite: 3/3;
- Runtime13 `script::` behavior filter: 60/60;
- current SSR owner guard exposed by the broad `reflection` name filter: 1/1;
- scoped rustfmt: passed.

Two reflection-probe render behaviors remain external and are not waived by this structure record.
