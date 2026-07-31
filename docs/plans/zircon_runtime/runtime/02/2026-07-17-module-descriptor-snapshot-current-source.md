---
related_code:
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
implementation_files:
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/entry/engine_entry.rs
  - docs/zircon_app/plugins.md
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-17-mvp-entry-static-review.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-17-module-descriptor-regeneration.md
tests:
  - zircon_app::plugins::tests::resolved_plugin_group_freezes_each_enabled_module_descriptor_once
  - zircon_app::plugins::tests::resolved_plugin_group_does_not_generate_disabled_module_descriptors
  - zircon_app::plugins::tests::resolved_plugin_group_preserves_the_nested_descriptor_snapshot
  - zircon_app::plugins::tests::resolved_plugin_group_does_not_regenerate_a_nested_descriptor_when_disabled
  - zircon_app::plugins::tests::builtin_plugin_groups_finish_in_descriptor_activation_order
  - cargo test -p zircon_app --lib resolved_plugin_group_ --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_app --test plugin_group_error_contract --locked --jobs 1 -- --nocapture --test-threads=1
doc_type: milestone-detail
---

# Runtime02 module descriptor snapshot current-source record

Plan: `docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`

Date: 2026-07-17

Status: `implemented_pending_managed_validation`

## Scope

This slice resolves PERF-MVP-004 at the app/runtime module-composition boundary. It freezes one descriptor snapshot for every enabled module represented by a `ResolvedPluginGroup`, then makes dependency sorting, built-in selection reporting, and built-in Core registration consume that same snapshot. It does not change runtime service activation, render/asset/editor gateways, Runtime11 tasks, Runtime12 input, or any plugin ABI.

## Architecture decision

- `zircon_app` remains the composition host and `zircon_runtime::CoreRuntime` remains the lifecycle owner.
- A resolved group owns parallel module and descriptor vectors with identical activation-order indices. The descriptor vector is exposed as a borrowed slice, preventing read paths from reconstructing descriptors.
- Direct enabled entries generate once during `try_finish`; entries disabled before that generation resolves generate zero times. A replaced entry discards only its old pending descriptor authority.
- `add_group` still resolves nested builders through `try_finish()?`, preserving immediate typed `PluginGroupError::ModuleOrder` propagation, but transfers the resolved pairs into the outer builder instead of regenerating descriptors. If the outer builder later disables an inherited entry, the nested validation remains generation 1 and the outer resolution does not invoke generation 2.
- Built-in diagnostics read the frozen descriptors and map owned clones into the report. Bootstrap reads the same slice and gives owned clones to Core registration. No process-global cache, compatibility shim, alternate report path, or guard weakening exists.

## Test-first evidence

The generation-count and disabled-entry tests were added before the production implementation. Against the previous source, the generation-count test has a deterministic RED trace: `sort_group_modules` invoked `descriptor()` once and each of two `ResolvedPluginGroup::module_descriptors()` calls invoked it again, producing count 3 versus expected 1. The direct-disabled test fixes the zero-call contract. Nested-group tests fix snapshot transfer and later outer-disable behavior without weakening the existing immediate typed-error integration test.

The shared managed Cargo lane was occupied during implementation. Per repository policy, no raw Cargo process was started. `rustfmt` and exact-path `git diff --check` pass. The failure-handoff validator reports 252 artifacts with 0 errors, and the coordinator failure audit reports no diagnostics. The plan-output audit reports one external Shader06 direct-record-limit violation and no Runtime02 violation; this slice did not modify that owner.

The former Windows reservation `ed67b4bee45f40d2b4e16f7ce379604e` bound exact3 fingerprint `74d5fd81bed89111b8a8ff9f64552f8ac11bf19a7e3b7630b7301c3d970a38b8`. It is absent from the current coordinator ledger and became permanently stale when Frameworks05 preference host wiring changed the shared `zircon_app/src/entry/engine_entry.rs` owner. Snapshot `903` established the combined exact15 descriptor/preference path set but is superseded by this record correction. Only a fresh snapshot and job created after the correction, guarded by full compile-input pre/post attestation, may provide acceptance evidence.

Independent read-only review first reported `Critical 0 / Important 1 / Minor 0` for an ambiguous nested-generation/outer-disable invariant. The immediate nested typed-error validation is intentionally retained; the new regression and documentation now distinguish the nested validated generation from the outer disabled generation. Follow-up review reports `Critical 0 / Important 0 / Minor 0`.

## Remaining gate

- Create and run a fresh combined snapshot gate; never recover or reuse reservation `ed67b4bee45f40d2b4e16f7ce379604e`.
- Run `plugin_group_error_contract` to confirm nested invalid dependency graphs remain typed errors.
- Review the final report/registration source equivalence and exact manifest.
- Do not rename the failure handoff to `fixed-*` or claim cold-start improvement until managed execution and any required trace evidence are recorded.
