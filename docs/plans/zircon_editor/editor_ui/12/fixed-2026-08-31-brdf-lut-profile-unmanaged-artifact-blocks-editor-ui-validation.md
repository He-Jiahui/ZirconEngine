---
handoff_kind: fixed
status: fixed
created_at: 2026-08-31
summary_slug: brdf-lut-profile-unmanaged-artifact-blocks-editor-ui-validation
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_runtime/shader/06
plan_link_mode: child_record_only
related_code:
  - tools/build-editor.ps1
tests:
  - ".\\tools\\zircon-session.ps1 artifact audit"
  - ".\\tools\\build-editor.ps1 -Ephemeral"
resolved_at: 2026-08-31
---

# Shader 06: unmanaged BRDF LUT profile artifact blocks Editor UI validation

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Editor build and native WGPU visual acceptance
- 修复责任计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 交接原因：失败发生在 Cargo 启动前，唯一被报告的 unmanaged 路径属于 Shader06 BRDF LUT profiling 产物；UI12 不拥有该目录或其生命周期。

## 失败现象与复现证据

2026-08-31 执行受管产品构建：

```powershell
.\tools\build-editor.ps1 -Ephemeral
```

协调器以 `unmanaged_artifacts_detected` 拒绝请求，`cleanupReservations` 为空，并报告：

```text
E:\ZirconBuilds\shader06-brdf-lut-profile
```

构建没有进入 Cargo/rustc，因此本次没有 Editor 当前源码编译诊断、产品 EXE 或 WGPU 截图。UI12 没有删除、移动、认领或豁免该路径，也不会用旧二进制替代产品验收。

## 最低共享层根因

Shader06 owner 需要核对该目录对应的 profiling job、artifact reservation 与发布/清理终态。若产物仍需保留，应通过协调器登记为合法持久产物；若生命周期已结束，应由原 owner 按受管流程清理并释放。不得由 UI12 直接删除外部产物，也不得放宽全局 unmanaged audit。

## 架构修复验收

- `artifact audit` 不再把 `E:\ZirconBuilds\shader06-brdf-lut-profile` 报告为 unmanaged。
- 不引入路径白名单、扫描豁免或 UI12 专用绕过。
- 随后的 `build-editor.ps1 -Ephemeral` 能越过 artifact governance 并真正启动受管 Cargo；源码编译与产品视觉结果仍由 UI12 独立验收。

## 禁止临时方案

- 不得由 UI12 删除、移动、重命名或认领 Shader06 产物。
- 不得把该路径加入全局忽略名单，也不得降低 unmanaged artifact 审计强度。
- 不得使用旧 Editor EXE、HTML mock 或非产品截图替代当前源码的 WGPU 视觉验收。

## 修复结果与回传

- 根因：The completed Shader06 BRDF LUT profile directory remained outside a live coordinator artifact reservation and was therefore reported as unmanaged.
- 架构修复：The expired profile output now has the required terminal state: E:\ZirconBuilds\shader06-brdf-lut-profile is absent, with no whitelist, scan exemption, ownership fabrication, or UI12 cleanup.
- 验证：Coordinator artifact audit request cb4006545a3247a7af6ee2287d2055a6 returned unmanaged=[]; exact-path and E:\ZirconBuilds inventory checks confirm the directory is absent. build-editor was not rerun because the shared Cargo CPU lane remains reserved; no compile or visual result is claimed.
- 回传：Artifact governance is clean and Editor UI12 may resume its managed build/WGPU gate when the Cargo lane is available.
