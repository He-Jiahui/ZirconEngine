---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-05
summary_slug: validation-copy-zr-vm-external-source-pin
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-27-validation-copy-zr-vm-external-source-pin.md
---

# validation-copy-zr-vm-external-source-pin 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-05-validation-copy-zr-vm-external-source-pin.md](../../../zircon_editor/editor/08/fixed-2026-08-05-validation-copy-zr-vm-external-source-pin.md)
- 摘要：The sibling pin 503fb72163cd20ddf32a38f8a330083712f5d648 remains a valid commit while the sibling checkout HEAD has advanced, proving immutable pin semantics. Original Editor08 session editor08-keymap-signature-index-r1-20260727 is archived and snapshot 1122 has drift in key_chord.rs and keymap/tests.rs, so no historical Cargo replay was attempted; Editor08 must use a new current-source Session for product-level keymap/Cargo acceptance.
