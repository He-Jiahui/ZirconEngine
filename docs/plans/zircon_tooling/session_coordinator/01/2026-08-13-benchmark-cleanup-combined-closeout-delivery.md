---
record_kind: failure_closeout_delivery
status: accepted
lifecycle_keys_json: ["e:/git/zirconengine/docs/plans/zircon_plugins/01-plugin-architecture-core.md|e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|benchmark-validation-identity-injection", "e:/git/zirconengine/docs/plans/zircon_plugins/01-plugin-architecture-core.md|e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|host-api-abi-decode-target-cache-rmeta-missing", "e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|failure-closeout-proof-only-state-attribution-deadlock", "e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|cargo-run-terminal-projection-before-release", "e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|unmanaged-artifact-readonly-cleanup-wedge", "e:/git/zirconengine/docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md|e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|ephemeral-target-deleted-during-active-cargo", "e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|e:/git/zirconengine/docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md|finalize-readonly-git-index-lock-churn"]
delivery_paths_json: [".codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1", ".codex/skills/zircon-dev/scripts/validate-matrix.ps1", ".codex/skills/zircon-dev/validation/SKILL.md", ".codex/skills/zircon-dev/validation/manual-commands.md", "tools/session_coordinator/cleanup_deletion.py", "tools/session_coordinator/tests/test_git_index_lock.py", "tools/session_coordinator/windows_tree_delete.py"]
---

# Coordinator benchmark 与 cleanup failure 合并交付

本记录把 cleanup 删除事务与 Windows handle-bound tree deletion 的独立实现模块纳入
七个 failure 的原子 closeout，并补入受限 Git index-lock 恢复及其 Restart Manager
owner 核验。其余交付路径均由七个 fixed handoff 的
`related_code` 清单直接约束。
