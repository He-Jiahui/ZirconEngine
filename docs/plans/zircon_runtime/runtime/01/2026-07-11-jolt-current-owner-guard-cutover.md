# Runtime 01 Jolt current-owner guard cutover

Date: 2026-07-11

Status: `runtime_01_jolt_current_owner_guard_cutover_plan_status_48_passed`

## Problem

The executable dependency guard had already been renamed to `physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned`, but Runtime 01 plan-status and tech-stack inventories still requested the retired `...jolt_unavailable_and_plugin_owned` name. Aggregate readers concatenated historical numbered archives, allowing the stale current-owner assertion to pass from historical text.

The same Runtime 01 gate still required the retired “only executable V1 backend” wording even though Plugins 03 now owns a real optional Jolt backend behind `backend-jolt`.

## Hard cutover

- Current tech-stack and plan-status guards now require the feature-gated Jolt guard name.
- Runtime 01 plan, runtime index, architecture review, and Physics option documentation describe the plugin-owned optional Jolt backend while preserving feature-off typed Unavailable behavior and the no-silent-builtin-fallback invariant.
- The Runtime 01 behavior inventory now checks all six current anchors, including feature-on Jolt readiness and native stepping.
- Historical numbered archives remain unchanged as historical evidence; they are no longer accepted as the current-owner source for the retired guard.

During full plan-status regression, current Runtime 15 owner splits exposed missing architecture-review inventory entries and one parent-only row-data read. The review now lists all 710 current plan-status Rust support files in frontmatter/body, and the recent-static split-layout guard reads its folder-backed row-data child instead of expecting the child constant in the parent route.

## Evidence

- RED: exact `runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation` ran 1 test and failed because the current Physics option document no longer contained “only executable V1 backend”.
- GREEN: the same exact test passed 1/1 after the current-owner cutover.
- Complete standalone plan-status suite: 48 passed / 0 failed / 0 ignored / 0 filtered, 6.55 seconds.
- Final current-source dependency guard suite: 11 passed / 0 failed, including the renamed feature-gated Jolt guard.
- Final Runtime 01 mirror/route suite: 2 passed / 0 failed; current-owner scan reports 0 retired `...jolt_unavailable_and_plugin_owned` matches outside historical archives.
- Scoped Rust 2021 formatting check passed for the five changed guard files.
- Current architecture-review inventory calculation reports 0 missing frontmatter paths and 0 missing body paths across 710 plan-status Rust support files.

## Claim boundary

This closes current-owner and plan-status drift only. Runtime 01 remains `in_progress` until its package-level `tech_stack`, `extensions`, `text_shaper`, physics plugin, and `export_build_plan` Cargo gates run against a stable current-source window.
