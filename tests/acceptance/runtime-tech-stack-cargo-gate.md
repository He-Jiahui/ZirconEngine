---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack/split_layout.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
output_records:
  - docs/plans/zircon_runtime/runtime/01/2026-07-09-tech-stack-and-dependency-governance-output-records.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
---

# Runtime 01 Tech-Stack Cargo Gate Acceptance

Date: 2026-07-10

## Accepted scope

- The existing `hzb_msaa` bind-group-layout constructor is reachable from its sibling construct owner, so the full-feature runtime lib-test crate no longer fails before Runtime 01 tests with E0425.
- Runtime 01 concrete audit counts are read from the numbered Runtime 01 output archive. Its parent plan and the runtime index retain only current status and routing.
- Frameworks 02, Runtime 15, and both prioritized engine-code plans follow the same split: concrete status evidence is read from numbered archives/current child owners, while route-only parents are checked only for their routing contract.
- `mirror_docs.rs` remains below its 90-line budget (88 lines), and `split_layout.rs` remains below its 240-line budget (225 lines).

## Verification

| Gate | Result |
|---|---|
| Standalone focused tech-stack harness | 2 passed, 0 failed |
| `cargo test -p zircon_runtime --lib tech_stack --locked --jobs 1 -- --nocapture` | 14 passed, 0 failed, 7411 filtered out |
| Cargo test execution time | 28.14s after compile |
| Full command elapsed build time | 11m50s |
| Existing warning baseline | 492 warnings; no new error |
| Current-manifest Python regression | 2 passed, 0 failed |
| Direct Runtime 01 audit | no missing version/guard/Cargo anchors, no dependency violations, two Jolt slots, `risks = []` |

## Status decision

`runtime_01_tech_stack_locked_cargo_gate_passed` closes the exact locked `tech_stack` gate. Runtime 01 remains `in_progress` because the plan separately requires `text_shaper`, plugin physics, and `export_build_plan` validation. Runtime 15 also remains `in_progress`; this acceptance only closes the archive-source reconciliation exercised by the same Cargo filter.

`runtime_01_tech_stack_current_optional_backend_audit_passed` additionally closes the stale manifest-audit assumptions exposed by the later full structure audit. It does not broaden the focused Cargo result.

The same aggregate follow-up preserved the active Runtime Text owner's new surface leaves and synchronized Runtime 09 audit metadata rather than editing their production behavior. This is recorded separately as `runtime_09_text_surface_leaf_entry_map_audit_sync_static_passed`.

## Extensions gate follow-up

`cargo test -p zircon_runtime --lib extensions --locked --jobs 1 -- --nocapture` was also attempted against the current tree. Its command runner timed out after 1804.4 seconds while full-feature lib-test compilation was still active; a follow-up process check confirmed the owned Cargo/rustc subtree survived the runner timeout and continued consuming CPU. No test result or diagnostic is available yet, while several other owner sessions are compiling or linking separate runtime targets.

Status `runtime_01_extensions_locked_cargo_gate_runner_timed_out_build_continues_no_result` is intentionally neutral: the `extensions` gate remains pending and Runtime 01 remains `in_progress` until the continuing owned build yields executable evidence.

The surviving owned build later produced the executable, and the direct `extensions` filter ran 443 tests: 434 passed and 9 failed in 784.64s. The failures were stale ownership/path guards and stale catalog counts after embedded module descriptors became the single module-registration source. Current-source standalone guards pass 1/1 absorption, 2/2 animation/physics, 1/1 manager handles, and 11/11 tech-stack; a fresh runtime binary passes catalog merge 1/1 and feature catalog 7/7. The active Physics owner then advanced its registration to `physics.step` plus `physics.sync_to_scene`; the final source guard follows that two-anchor contract and passes 2/2.

Status `runtime_01_extensions_current_contract_focused_guards_passed_full_gate_rerun_pending` keeps the full 443-test gate pending until it is recompiled and rerun after the final Physics source advance.

The subsequent full structure audit exposed one audit-only hard-cutover drift: Runtime 01 still read the retired flat Physics `backend.rs`. The audit now reads `backend/mod.rs` plus the real `backend/selection.rs` policy owner. Regression tests pass 4/4 and the direct tech-stack audit reports no dependency-boundary violations or risks. Status: `runtime_01_tech_stack_folder_backed_physics_backend_audit_passed`.

## Physics Jolt feature audit reconciliation

The active Physics plugin manifest now declares `backend-jolt` as a dependency-backed optional feature, while the Runtime manifest retains one explicit passthrough feature. The Runtime 01 structure audit models that current 1+1 shape and verifies the optional `joltc-sys` declaration instead of requiring two empty feature slots.

Verification passed with the Runtime tech-stack Python suite at 4/4 and a direct audit result of two total visible feature slots, one Runtime passthrough slot, one Physics dependency-backed slot, optional dependency present, no dependency-boundary violations, and no risks. This is audit-contract acceptance only; it does not claim that the Jolt backend implementation or its behavior gates are complete. Status: `runtime_01_physics_jolt_dependency_feature_audit_passed`.

## Jolt executable-backend audit convergence

The later Physics M1-T3 implementation supersedes the historical constant-unavailable policy. Runtime 01 now requires feature-derived availability, concrete plugin-local Jolt backend owners, no `joltc-sys` in `zircon_runtime`, and both feature-off and feature-on behavior anchors.

Verification: Python Runtime tech-stack suite 4/4; standalone Rust dependency guard 11/11; standalone mirror guard 1/1; direct audit reports six behavior anchors, no missing anchors, no dependency-boundary violations, and `risks = []`.

Status: `runtime_01_jolt_feature_gated_plugin_owned_audit_static_passed`. Aggregate `extensions` and remaining Runtime 01 product gates stay pending.
