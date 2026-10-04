---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-21-viewport-chrome-direct-settings.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/chrome_projection.rs
  - zircon_editor/src/ui/workbench/model/status_bar_model.rs
tests:
  - zircon_editor/src/ui/workbench/model/status_bar_viewport_chrome_text_tests.rs
  - tools/tests/test_editor862_viewport_chrome_direct_settings_performance_contract.py
---

# Editor862 - viewport chrome direct settings projection

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Viewport toolbar chrome patch | Read the authoritative scene viewport settings directly and derive only the shared grid/snap labels, avoiding complete Chrome snapshot and status-model construction while preserving damage/native-presenter behavior. | Intentional RED→GREEN contract `4/4`; lower grid/snap semantics and ignored `EDITOR862_VIEWPORT_CHROME_DIRECT_SETTINGS_BENCH_V1` marker are wired. Structural full Chrome/status builds change `1→0`; the combined related batch passes `43/43`, and the current-tree non-tooling loader passes `4121/4121` across `974` files. Managed Cargo/Release, allocator, and product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Scope boundary

Full shell/status presentation still uses `StatusBarModel::from_chrome`; both
paths now share one grid/snap label implementation. Viewport event routing and
tooling production are unchanged.

## Managed gate

No local Cargo command or coordinator poll was started. Keep this completion
entry pending until the owner-attributed Windows Release lane supplies current-
source compilation, lower marker, allocator, and viewport-chrome percentile
evidence.
