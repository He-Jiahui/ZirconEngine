---
status: source_complete_validation_pending
created_at: 2026-08-30
implementation_status: ticket-owned-view-instance-layout-descriptor-retirement-source-complete-static-verified
managed_validation_status: blocked_unmanaged_artifacts_detected
related_code:
  - zircon_editor/src/ui/workbench/view/mod.rs
  - zircon_editor/src/ui/workbench/view/view_registry_descriptor_mutation.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration/ticketed_command_revoke.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/AnimationEditor/Private/AnimationEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Blutility/Private/EditorUtilitySubsystem.cpp
---

# Editor08 ticket-owned view instance and descriptor retirement

## Current-source architecture review

The contribution Store already retains the exact `ContributionTicket` for each view descriptor and
exposes `views_for_ticket`. The workbench keeps separate live instance, layout, focus, floating
window, document toolkit, and window-registry state. Before this slice, revoke removed Store and
command entries but left those UI objects live, so a revoked extension view could remain visible or
be reopened from the `ViewRegistry`. The host's existing `close_view` path is the authoritative
cleanup transaction: it detaches every layout host, repairs focus, closes document toolkits,
removes animation/UI-asset sessions, and synchronizes native window state.

Unreal's editor module shutdown treats live tab closure and tab-spawner unregistration as separate
operations. Zircon follows the same ordering while keeping the Store ticket as the sole owner
identity: close live instances first, then unregister descriptors, then publish the remaining Store
and command projection.

## Implemented source slice

1. `ViewRegistry::unregister_view` now refuses descriptor removal while any instance references it
   and rejects missing descriptors. This makes descriptor lifetime fail closed at the registry
   boundary.
2. `EditorUiHost::retire_extension_views` derives matching instance IDs from the current session,
   preflights all document close leases, closes each instance through the existing lifecycle, and
   unregisters the exact descriptor set. No parallel owner map is introduced.
3. Ticket revoke snapshots view IDs from the Store before cloning/revoking, performs view retirement
   before runtime consumer and Store/router publication, and preserves the existing scene/overlay
   cleanup order.
4. The ticketed command-router regression opens one plugin view, revokes its owner, and proves the
   instance and descriptor disappear while the remaining plugin descriptor stays available.

The close preflight prevents a busy document toolkit from partially changing layout state. All
changes are serialized by the existing plugin registration gate. Retirement cost is linear in open
view instances plus the revoked descriptor count and is confined to plugin lifecycle operations;
the frame/layout hot path is unchanged.

## Remaining gate

Managed Windows Cargo validation and independent review are pending. Native serialized command
execution still intentionally fails closed without an explicit executor; typed native provenance,
callback lease quiescence, and full executable command dispatch remain separate Editor08 work.

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-30 | `architecture-reviewed / implementation-starting` | 复核 Store ticket、EditorManager session/layout/window registry、document toolkit close lease 与 Unreal tab-spawner shutdown 对称关系，确定“先关闭实例，再注销 descriptor，再发布 Store/router”顺序。 |
| 2026-08-30 | `source-complete / static-verified / managed-validation-request-submitted` | 新增 `ViewRegistry::unregister_view` fail-closed API；EditorManager 按 ticket descriptor 集合关闭 live instances 并清理布局、焦点、document toolkit、浮动窗口和窗口注册；ticketed revoke 回归覆盖已打开插件 view 的 instance/descriptor 消失及剩余 owner 保留。定向 `rustfmt` 与 `git diff --check` 通过；E 盘 `validate-matrix.ps1` 聚焦测试已提交，未声称终态。 |
| 2026-08-30 | `source-complete / static-verified / managed-validation-blocked` | 受管 `validate-matrix.ps1` 重试被协调器拒绝：目标目录 `E:\ZirconBuilds\editor08-validation-20260830` 仍登记为未注册清理保留项（`unmanaged_artifacts_detected`）。未删除或绕过该保留项，Failure artifact 保持冻结；源码与定向 rustfmt 结果仍有效，待协调器释放后再提交受管验证。 |
