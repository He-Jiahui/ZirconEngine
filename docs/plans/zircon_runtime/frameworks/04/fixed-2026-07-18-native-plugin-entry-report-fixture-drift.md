---
handoff_kind: fixed_return
status: validating
created_at: 2026-07-18
summary_slug: native-plugin-entry-report-fixture-drift
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/zircon_runtime/frameworks/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - managed Runtime12 mirror compile retry pending
---

# Frameworks04 native entry-report fixture drift fixed return

Frameworks04 added `missing_required_capabilities` and `denied_capabilities` to `NativePluginEntryReport`. Two native live-host success fixtures still constructed the old shape, so any current lib-test compile failed before the selected test could execute.

The repair initializes both fields with empty vectors in the two local success fixtures. This matches their descriptors, which request no capabilities, and preserves the production hard-cut contract. No default compatibility constructor, serde fallback, optional field, or production behavior change was added.

Status remains `validating` until a coordinator-managed Windows lib-test compile reaches and executes the Runtime12 target on the repaired current source.

