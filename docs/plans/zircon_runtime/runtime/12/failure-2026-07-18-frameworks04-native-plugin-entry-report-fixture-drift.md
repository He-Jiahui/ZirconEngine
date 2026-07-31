---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: frameworks04-native-plugin-entry-report-fixture-drift
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/zircon_runtime/frameworks/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
tests:
  - managed Windows job b683460bdf5045908517e328b85f962b / run c96ead9118594183a538a77ea88626ec
  - runtime_12_input_stack_mirror_docs_match_structure_audit_counts retry pending
---

# Runtime12 blocked by Frameworks04 native entry-report fixture drift

Runtime12 current-source mirror job `b683460bdf5045908517e328b85f962b` released with exit 101 and no live processes before executing its target test. The raw lib-test compile reported two E0063 errors in `native_plugin_live_host/tests.rs`: local `NativePluginEntryReport` fixtures did not initialize Frameworks04's new `missing_required_capabilities` and `denied_capabilities` fields.

The same compile also observed three Text01 import errors from a source state that preceded Text01 snapshot 497; the Text01 owner has since restored the `GlyphAtlasPageResidencyDecision` import and explicitly invalidated b683 as source-polluted. None of these errors is a Runtime12 mirror assertion result.

The lowest remaining repair is the Frameworks04 test consumer. Both local success fixtures must explicitly initialize the new capability outcome lists as empty. Production capability negotiation, ABI layout, thresholds, and Runtime12 input sources are out of scope.

Current state: `consumer_fix_applied_pending_fresh_managed_compile`. Runtime12 must not claim mirror red or green from b683.

## 2026-07-30 fresh managed retry evidence

- Framework04 r8 created source snapshot `1319` for this handoff and the two owned native-entry paths, then ran managed Windows job `e83f2aa0784d45cab6526effd572d7a2` / run `45760a71db784350a710e5b33d138fb4` with the canonical command:
  `cargo +1.94.1 test -p zircon_runtime --lib runtime_12_input_stack_mirror_docs_match_structure_audit_counts --locked --jobs 1 -- --nocapture --test-threads=1`.
- The job naturally released at `2026-07-30T06:26:14+08:00` with exit `101` and no live process IDs. The target Runtime12 test did not execute, so this retry is neither a Runtime12 red result nor Framework04 fixture acceptance.
- The current lib-test compile reached three Plugins01-owned errors before test execution: `runtime_profile/availability_projection.rs:262` uses non-const derived `PartialEq` inside `RuntimePluginAvailabilitySummary::category_count`, and `tests/plugin_extensions/profile_availability_projection.rs:358` / `:362` call `.first()` on the iterator returned by `RuntimePluginAvailabilityGeneration::entries(...)`.
- The lowest owner is already active as `plugins01-availability-generation-r4-20260730`, under [`runtime-profile-availability-rebuild`](../../../zircon_plugins/01/failure-2026-07-17-runtime-profile-availability-rebuild.md). Framework04 must not add an upper-layer workaround, weaken the compile boundary, or mark this handoff fixed until that owner returns a fresh source-valid compile boundary.
